package dev.rune

import android.app.NativeActivity
import android.content.pm.PackageManager
import android.os.Bundle
import android.view.ViewGroup

/**
 * A NativeActivity that can take the on-screen keyboard: Rune draws and gets touches as under
 * NativeActivity, and a [RuneInputView] holds the keyboard's connection. Name it as the
 * activity in AndroidManifest.xml (with `android.app.lib_name` as for NativeActivity), or
 * subclass it.
 *
 * Rust's side is `crates/rune/src/platform/android/ime.rs`, which documents the messages.
 */
open class RuneActivity : NativeActivity() {
    private lateinit var input: RuneInputView

    override fun onCreate(savedInstanceState: Bundle?) {
        // NativeActivity opens the library without registering it with the VM, so its JNI
        // exports (`runeInput`) resolve only once it is loaded here too. Same file, same handle.
        val info = packageManager.getActivityInfo(componentName, PackageManager.GET_META_DATA)
        System.loadLibrary(info.metaData?.getString("android.app.lib_name") ?: "main")
        super.onCreate(savedInstanceState)
        input = RuneInputView(this)
        // Beside NativeActivity's own content view, not in place of it: that one reports the
        // content rect Rune lays out in.
        addContentView(input, ViewGroup.LayoutParams(1, 1))
    }

    /** Rust, on android_main's thread: the focused field changed. */
    @Suppress("unused")
    fun runeSync(json: String) {
        runOnUiThread { input.sync(json) }
    }

    /** To Rust, from the UI thread: an edit from the keyboard. It only queues. */
    external fun runeInput(json: String)
}
