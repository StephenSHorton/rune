- **Android: typing with the on-screen keyboard (prototype).** An app whose activity is
  `dev.rune.RuneActivity` (Kotlin, in `crates/rune/android`) instead of NativeActivity gets
  Gboard's composing, autocorrect, suggestions and swipe typing in Rune text fields: a host
  view holds the keyboard's connection and passes its edits to Rune over JNI with the web
  shell's edit protocol. Winit's IME show/hide calls are skipped on Android. Text falls back to
  the system's fonts in `/system/fonts`, so CJK renders. `examples/android-text` is a Gradle
  project around it, and `scripts/android-apk.sh android-text` builds it. See
  `research/39-android-keyboard.md`.
