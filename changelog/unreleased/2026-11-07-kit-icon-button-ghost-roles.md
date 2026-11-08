- **Kit `IconButton` glyph size and ghost role colours.** `IconButton::glyph_size(pt)` sets the
  icon's side in place of the control size's (16 for Regular), keeping the button's size.
  `IconButtonVariant::GhostDanger` and `GhostSuccess` are ghost buttons (no fill at rest, the
  kit's hover and pressed fills) with the danger or success role's icon colour. An app that
  matches `IconButtonVariant` exhaustively needs arms for the two new variants.
