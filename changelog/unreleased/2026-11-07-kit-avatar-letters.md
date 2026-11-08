- **Kit `Avatar` letters, tooltip and pictures.** `Avatar::letter_size(pt)` and
  `letter_weight(w)` set the initials' type in place of 0.4 of the side at 600;
  `single_initial()` shows the name's first letter only; `help(text)` gives it a tooltip. An
  avatar whose picture already has pixels when it is built no longer paints the name colour and
  initials under it, so a picture with transparent parts doesn't show them through (an opaque
  picture looks the same; the element tree loses the hidden initials text under it).
