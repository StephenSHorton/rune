- **Dashed strokes, and dashes that flow.** `Shape::dashed(dash, gap)` draws a shape's strokes as
  dashes (round caps with a dash of 0 draw dots), `Shape::dash_offset(points)` slides the
  pattern along them, and `El::dash_flow(speed)` marches it on forever at `speed` points a
  second, as a node graph's animated edges do. The renderer moves it on the shared clock, so the
  view doesn't rebuild, and Reduce Motion holds the dashes still. Dashes are cut on the CPU into
  ordinary stroke segments, so they draw in the same pass and need no shader change. New
  `LoopKind::March` and `LoopSpec::march` in `rune-motion` carry the flow; a `match` on
  `LoopKind` needs an arm for it.
