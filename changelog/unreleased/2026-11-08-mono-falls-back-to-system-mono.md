- **`Family::Mono` falls back to the platform's monospaced font.** When the mono slot holds no
  monospaced face (the app named none, so the slot is the sans, or named a family it didn't
  register), `Family::Mono`, `TextStyle::mono()` and mono rich spans now shape with
  `Family::SystemMono`'s families (SF Mono, Cascadia Mono or Consolas, fontconfig's
  `monospace`) instead of the proportional sans. An app whose mono slot is a registered
  monospaced face keeps it. Code in apps that never set a mono family changes width.
