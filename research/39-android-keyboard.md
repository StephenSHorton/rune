# Android: the on-screen keyboard (prototype)

Rune on Android is a prototype. This note covers what it took to make a Rune text field accept
Gboard: composing, autocorrect and swipe typing. It also covers what the work shows about Rune
on Android, and what accessibility, the clipboard and links would need.

## What was built

- **`crates/rune/android/kotlin/dev/rune/`**: two Kotlin classes, about 300 lines together.
  - `RuneActivity` is NativeActivity plus one view. Rune still draws through the native window
    and still gets touches from the native input queue.
  - `RuneInputView` is the view the keyboard talks to. Its `onCreateInputConnection` returns a
    `BaseInputConnection` over an `Editable` holding the focused field's text. That text is a
    window of at most 512 UTF-16 units on each side of the caret.
- **`crates/rune/src/platform/android/ime.rs`**: the Rust side. It has one JNI export
  (`RuneActivity.runeInput`) and one call back into Kotlin (`RuneActivity.runeSync`). The
  messages are the ones the web shell's hidden textarea already exchanges with
  `Runtime::web_edit` and `Mirror`, so the editing model, the stale-context checks and the
  `on_change` callbacks are the web's, unchanged.
- **`examples/android-text`**: a Rune text field in a small Gradle project
  (AGP 8.11, Kotlin 2.0). `scripts/android-apk.sh android-text --install` builds the Rust
  library with cargo, has Gradle package it next to the Kotlin host, then installs and starts
  the APK. `examples/android-hello` still builds without Gradle or Java.

How a keystroke travels:

1. Gboard calls the InputConnection on the UI thread.
2. The Editable changes. After each call, or each batch of calls, the view diffs the Editable
   against the text Rune last confirmed.
3. It sends one of:
   - `edit`: a UTF-16 replacement plus the new selection;
   - `preedit`: the composing word, plus the range of committed text it replaces;
   - `key`: Enter.
4. The native call only queues the message and wakes the event loop.
5. The loop applies the message with `web_edit` and draws. After the frame it calls `runeSync`
   with the field's text window, the selection, whether the keyboard should be up, and the
   last edit it took.
6. The view ignores a sync older than its last edit. A newer one is the truth: if the text
   differs, the view replaces its own and restarts the keyboard. If only the selection moved
   (a tap on the field), it reports the new selection to the keyboard with `updateSelection`.

## What was checked on the emulator

Checked on API 35, Google APIs arm64, with Gboard:

- **Typing:** taps arrive one character at a time, sentence caps are on, and suggestions show.
- **Autocorrect:** "helo" + space becomes "Hello ". Picking a suggestion replaces the word.
  Gboard also split "helohelo" into "helo helo". A tap inside a word moves Rune's caret, and
  Gboard then offers replacements for that word.
- **Swipe:** one glide over h-e-l-o gives "Hello". Backspace right after a swiped word deletes
  the whole word, as Gboard does in an EditText.
- **Composing:**
  - Gboard's Japanese keyboard composes romaji into kana. Rune draws the composition underlined
    as its preedit, and the committed text stays empty until a candidate (日本) is picked.
  - Gboard's English layouts on this image never compose. They commit letter by letter in a
    stock `EditText` and in Settings' search as well, so this is the keyboard, not the bridge.
- **Enter and Done:** both submit the field. The keyboard hides when the field loses focus and
  comes up when a field gains it.
- **Coming back to the keyboard:** after Back dismisses it, a tap on Rune while the field
  still has focus brings it back. After the app goes to the background and returns, it is up
  again.
- **Volume:** about 800 hardware-key characters (`adb shell input text`, which goes to the
  native input queue rather than through the keyboard) each ended in a sync to Kotlin. No JNI
  errors, and the text grew past the 512-unit window. jni 0.22 pushes a local frame on each
  `attach_current_thread`, so a sync's strings don't pile up.
- **Without the host:** `examples/android-hello` (plain NativeActivity) runs as before. The
  first `runeSync` fails there and Rune stops calling it.

Stills: `demo/android/keyboard-*.png`.

## What it shows about Rune on Android

- **Rune's text model was ready.** The web shell had already had to drive a field from a
  platform editor that owns its own text. The same protocol carried Android's InputConnection
  without changing the runtime; only a `cfg` was widened. iOS's `UITextInput` would likely fit
  the same way.
- **NativeActivity is the limit, not Rune.**
  - Its window owns the surface and the input queue, so ordinary views neither draw nor get
    touches. The host view exists only to hold the keyboard's connection.
  - The window doesn't resize for the keyboard (`adjustNothing`): a field in the bottom half of
    the screen ends up under it. Rune would need the IME inset (`WindowInsets.ime()`) as a
    safe-area inset and would scroll the focused field into view itself.
- **winit gives Android nothing for text.**
  - On NativeActivity winit 0.30 sends no `Ime` events, and `set_ime_cursor_area` does nothing
    there.
  - Its `set_ime_allowed` shows the keyboard against NativeActivity's own view, which isn't the
    one being served, and hides it again on the next reset. Rune skips both calls on Android.
- **Fonts:** Rune's fallback search had no Android font folder, so Japanese drew as boxes.
  This change adds `/system/fonts` (NotoSansCJK and the other scripts' Noto files).
- **Gaps left in this bridge:**
  - Password fields don't open the keyboard. They would need `TYPE_TEXT_VARIATION_PASSWORD`
    and the web's `secure` message.
  - There is no `CursorAnchorInfo`, so keyboards that float a candidate window don't know where
    the caret is. Rune's `ime_area` has the rect.
  - The Editable holds only the ±512-unit window, so a jump of the caret further than that
    restarts the keyboard.
  - Hardware keys go through winit, not the keyboard's connection. Each one changes Rune's
    text behind the Editable, so the host restarts the keyboard on the next sync. That's
    correct, but it costs Gboard its state for each key.
  - Samsung's keyboard batches its edits differently from Gboard (see
    `05-text-ime-a11y-layout.md`) and is untested.

## Out of scope: what each would take

- **Accessibility.** The host view would return an `AccessibilityNodeProvider` built from
  Rune's accesskit tree. accesskit has no Android adapter in this tree, so that means either
  one in Kotlin (nodes, bounds, actions, focus) fed over the same JNI pipe, or one in Rust over
  JNI. The tree, its roles and its actions already exist for the other platforms. The same view
  is where TalkBack's focus and its actions (click, scroll, set text) would come in.
- **Clipboard.** `ClipboardManager` through JNI on the UI thread: `setPrimaryClip` for copy,
  `getPrimaryClip` for paste, behind Rune's existing `clipboard::Client`. The keyboard's own
  paste key already works, because it commits text through the InputConnection. What's
  missing is Rune's copy and cut, and an app's paste. Android 13 and later shows its own toast
  on copy.
- **Links.** `Intent(ACTION_VIEW, uri)` from the activity. It is one JNI call, on the
  existing `run_on_java_main_thread` path, behind Rune's open-URL hook. Custom tabs would add
  the androidx browser library.
