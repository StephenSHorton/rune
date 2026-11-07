package dev.rune

import android.text.Editable
import android.text.InputType
import android.text.Selection
import android.view.KeyEvent
import android.view.View
import android.view.inputmethod.BaseInputConnection
import android.view.inputmethod.EditorInfo
import android.view.inputmethod.ExtractedText
import android.view.inputmethod.ExtractedTextRequest
import android.view.inputmethod.InputConnection
import android.view.inputmethod.InputMethodManager
import org.json.JSONObject

/**
 * The view the keyboard talks to. It draws nothing (Rune draws the field); it holds an
 * [Editable] with the focused field's text around the caret, lets the keyboard edit it the way
 * it edits an EditText (composing, autocorrect, swipe), and after each edit, or each batch of
 * them, tells Rune what changed: `preedit` for the word being composed, `edit` for the rest.
 * Rune answers with the field as it now is ([sync]); until it has taken every edit sent, the
 * Editable stays the truth.
 */
class RuneInputView(private val activity: RuneActivity) : View(activity) {
    private val imm = activity.getSystemService(InputMethodManager::class.java)
    private val editable: Editable = Editable.Factory.getInstance().newEditable("")

    /** The focused field's path in Rune; empty when no field has focus. */
    private var key = ""
    /** UTF-8 byte offset in Rune's text where [value] starts. */
    private var base = 0L
    /** The text window as Rune has it once it takes the edits sent: never the composing word. */
    private var value = ""
    private var composing = false
    /** The last preedit sent, so a repeat (BaseInputConnection batches inside its own calls) isn't. */
    private var preedit = ""
    private var anchor = 0
    private var focus = 0
    private var seq = 0L
    private var shown = false
    private var batch = 0
    /** Selection and composing span last told to the keyboard. */
    private var told = intArrayOf(-1, -1, -1, -1)

    init {
        isFocusable = true
        isFocusableInTouchMode = true
    }

    override fun onCheckIsTextEditor() = true

    /** Back from the background: the window's focus hid the keyboard; bring it back. */
    override fun onWindowFocusChanged(hasWindowFocus: Boolean) {
        super.onWindowFocusChanged(hasWindowFocus)
        if (hasWindowFocus && shown) {
            requestFocus()
            imm.showSoftInput(this, 0)
        }
    }

    override fun onCreateInputConnection(out: EditorInfo): InputConnection {
        out.inputType = InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_FLAG_AUTO_CORRECT or
            InputType.TYPE_TEXT_FLAG_CAP_SENTENCES
        out.imeOptions = EditorInfo.IME_ACTION_DONE or EditorInfo.IME_FLAG_NO_FULLSCREEN or
            EditorInfo.IME_FLAG_NO_EXTRACT_UI
        out.initialSelStart = Selection.getSelectionStart(editable)
        out.initialSelEnd = Selection.getSelectionEnd(editable)
        val connection = Connection()
        out.initialCapsMode = connection.getCursorCapsMode(out.inputType)
        told = intArrayOf(out.initialSelStart, out.initialSelEnd, -1, -1)
        return connection
    }

    /** Rune's side, on the UI thread: `{key, text, base, anchor, focus, show, reshow, applied}`. */
    fun sync(json: String) {
        val o = JSONObject(json)
        val newKey = o.getString("key")
        val text = o.getString("text")
        val newBase = o.getLong("base")
        val a = o.getInt("anchor").coerceIn(0, text.length)
        val f = o.getInt("focus").coerceIn(0, text.length)
        if (newKey != key) {
            key = newKey
            replace(text, newBase, a, f)
        } else if (o.getLong("applied") >= seq) {
            // Rune has every edit sent: what it says differs from ours only when it changed
            // the field itself (a tap moved the caret, the app rewrote the text), or dropped one.
            if (text != value || newBase != base) {
                replace(text, newBase, a, f)
            } else if (!composing && (a != anchor || f != focus)) {
                anchor = a; focus = f
                Selection.setSelection(editable, a, f)
                report()
            }
        }
        val show = o.getBoolean("show")
        // `reshow`: a tap on Rune while a field has focus; Back may have hidden the keyboard.
        if (show && (!shown || o.optBoolean("reshow"))) {
            requestFocus()
            imm.showSoftInput(this, 0)
        } else if (!show && shown) {
            imm.hideSoftInputFromWindow(windowToken, 0)
        }
        shown = show
    }

    /** Takes Rune's text as ours and starts the keyboard over on it. */
    private fun replace(text: String, newBase: Long, a: Int, f: Int) {
        value = text; base = newBase; anchor = a; focus = f; composing = false; preedit = ""
        BaseInputConnection.removeComposingSpans(editable)
        editable.replace(0, editable.length, text)
        Selection.setSelection(editable, a, f)
        told = intArrayOf(-1, -1, -1, -1)
        imm.restartInput(this)
    }

    /** After the keyboard edited [editable]: tell Rune, then the keyboard, what it now is. */
    private fun changed() {
        if (key.isEmpty()) {
            // No Rune field has the keyboard: nothing to edit.
            if (editable.isNotEmpty()) editable.clear()
            return
        }
        val text = editable.toString()
        val s = Selection.getSelectionStart(editable).coerceIn(0, text.length)
        val e = Selection.getSelectionEnd(editable).coerceIn(0, text.length)
        var cs = BaseInputConnection.getComposingSpanStart(editable)
        var ce = BaseInputConnection.getComposingSpanEnd(editable)
        if (cs > ce) cs = ce.also { ce = cs }
        if (cs in 0 until ce) {
            val word = text.substring(cs, ce)
            val tail = text.length - ce
            val end = value.length - tail
            if (end < cs || !value.regionMatches(0, text, 0, cs) || !value.regionMatches(end, text, ce, tail)) {
                // Text outside the word changed too: commit that first, then compose on it.
                val committed = text.substring(0, cs) + text.substring(ce)
                edit(committed, cs, cs)
                send("preedit", cs, cs, word)
            } else {
                send("preedit", cs, end, word)
            }
            composing = true
        } else if (composing || text != value) {
            edit(text, s, e)
            composing = false
        } else if (s != anchor || e != focus) {
            anchor = s; focus = e
            send(JSONObject().put("kind", "selection").put("anchor", s).put("focus", e))
        }
        report()
    }

    /** Sends the change from [value] to [next] as one replacement, and expects [next] after it. */
    private fun edit(next: String, a: Int, f: Int) {
        var start = 0
        while (start < value.length && start < next.length && value[start] == next[start]) start++
        if (start > 0 && Character.isLowSurrogate(value.getOrElse(start) { ' ' }) &&
            Character.isHighSurrogate(value[start - 1])) start--
        var end = value.length
        var nextEnd = next.length
        while (end > start && nextEnd > start && value[end - 1] == next[nextEnd - 1]) { end--; nextEnd-- }
        if (end < value.length && Character.isLowSurrogate(value[end]) && end > 0 &&
            Character.isHighSurrogate(value[end - 1])) { end++; nextEnd++ }
        anchor = a; focus = f
        send(JSONObject().put("kind", "edit").put("start", start).put("end", end)
            .put("text", next.substring(start, nextEnd)).put("anchor", a).put("focus", f))
        value = next
    }

    private fun send(kind: String, start: Int, end: Int, text: String) {
        val id = "$start $end $value\u0000$text"
        if (kind == "preedit" && id == preedit) return
        preedit = if (kind == "preedit") id else ""
        send(JSONObject().put("kind", kind).put("start", start).put("end", end).put("text", text))
    }

    private fun send(event: JSONObject) {
        if (event.getString("kind") != "preedit") preedit = ""
        seq++
        event.put("seq", seq).put("key", key).put("base", base).put("expected", value)
        activity.runeInput(event.toString())
    }

    private fun key(name: String) {
        seq++
        activity.runeInput(JSONObject().put("seq", seq).put("kind", "key").put("key", name).toString())
    }

    /** The keyboard tracks the caret and the composing word through this, as with EditText. */
    private fun report() {
        val a = Selection.getSelectionStart(editable)
        val b = Selection.getSelectionEnd(editable)
        val now = intArrayOf(minOf(a, b), maxOf(a, b),
            BaseInputConnection.getComposingSpanStart(editable), BaseInputConnection.getComposingSpanEnd(editable))
        if (now.contentEquals(told)) return
        told = now
        imm.updateSelection(this, now[0], now[1], now[2], now[3])
    }

    private inner class Connection : BaseInputConnection(this@RuneInputView, true) {
        override fun getEditable(): Editable = this@RuneInputView.editable

        override fun beginBatchEdit(): Boolean { batch++; return true }

        override fun endBatchEdit(): Boolean {
            if (batch > 0) batch--
            if (batch == 0) changed()
            return batch > 0
        }

        private fun after(result: Boolean): Boolean {
            if (batch == 0) changed()
            return result
        }

        override fun commitText(text: CharSequence?, cursor: Int) = after(super.commitText(text, cursor))
        override fun setComposingText(text: CharSequence?, cursor: Int) = after(super.setComposingText(text, cursor))
        override fun setComposingRegion(start: Int, end: Int) = after(super.setComposingRegion(start, end))
        override fun finishComposingText() = after(super.finishComposingText())
        override fun deleteSurroundingText(before: Int, after: Int) = after(super.deleteSurroundingText(before, after))
        override fun deleteSurroundingTextInCodePoints(before: Int, after: Int) =
            after(super.deleteSurroundingTextInCodePoints(before, after))
        override fun replaceText(start: Int, end: Int, text: CharSequence, cursor: Int,
            attribute: android.view.inputmethod.TextAttribute?) =
            after(super.replaceText(start, end, text, cursor, attribute))
        override fun setSelection(start: Int, end: Int) = after(super.setSelection(start, end))

        override fun getExtractedText(request: ExtractedTextRequest?, flags: Int): ExtractedText {
            val e = this@RuneInputView.editable
            return ExtractedText().apply {
                text = e.toString()
                startOffset = 0
                partialStartOffset = -1
                partialEndOffset = -1
                selectionStart = Selection.getSelectionStart(e)
                selectionEnd = Selection.getSelectionEnd(e)
            }
        }

        override fun performEditorAction(action: Int): Boolean {
            finishComposingText()
            key("Enter")
            return true
        }

        /** Delete keys edit the Editable like any other edit; Enter goes to Rune as a key. Not
         * to the native input queue, or Rune would see them twice. */
        override fun sendKeyEvent(event: KeyEvent): Boolean {
            when (event.keyCode) {
                KeyEvent.KEYCODE_DEL, KeyEvent.KEYCODE_FORWARD_DEL -> {
                    if (event.action == KeyEvent.ACTION_DOWN) delete(event.keyCode == KeyEvent.KEYCODE_DEL)
                    return true
                }
                KeyEvent.KEYCODE_ENTER, KeyEvent.KEYCODE_NUMPAD_ENTER -> {
                    if (event.action == KeyEvent.ACTION_DOWN) performEditorAction(EditorInfo.IME_ACTION_DONE)
                    return true
                }
            }
            return super.sendKeyEvent(event)
        }

        private fun delete(backward: Boolean) {
            val e = this@RuneInputView.editable
            val a = Selection.getSelectionStart(e)
            val b = Selection.getSelectionEnd(e)
            beginBatchEdit()
            removeComposingSpans(e)
            if (a != b) {
                e.delete(minOf(a, b), maxOf(a, b))
            } else if (backward && a > 0) {
                e.delete(Character.offsetByCodePoints(e, a, -1), a)
            } else if (!backward && a < e.length) {
                e.delete(a, Character.offsetByCodePoints(e, a, 1))
            }
            endBatchEdit()
        }
    }
}
