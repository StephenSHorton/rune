- **Tray check items.** `TrayItem::check(id, label, checked)` shows the platform's own check mark
  beside an item on Windows (`MF_CHECKED`) and Linux (a dbusmenu `checkmark` toggle). A pick
  still sends `TrayEvent::Menu(id)`; the app flips its state and calls `Tray::set_menu`.
  `TrayItem::is_checked` reads it back. `TrayItem` gains a `Check` variant.
