- **Kit `Badge` chips.** `Badge::pill()` rounds the ends fully; `height(h)` and `padding(px)`
  replace the 16 (18) point height and 4 (6) point side padding; `border_width(w)` replaces the
  0.5 point border (0 drops it); `max_width(w)` cuts a longer text short with an ellipsis and
  shows the whole of it as a tooltip; `plain()` makes it plain text instead of a status, for
  chips whose changes shouldn't be announced. Badges that set none of these are unchanged.
