- **Kit `ColorPicker::on_commit` and a hue track.** `on_commit(f)` hears settled picks only: a
  swatch press, a hue drag released or a hue key step, and a typed hex on Return or when the
  field loses focus, where `on_change` hears every step. The hue slider's track now shows the
  hues it turns through, at the colour's lightness and chroma, in place of the plain track and
  accent fill; `Slider::track_gradient(stops)` paints any slider's track that way.
