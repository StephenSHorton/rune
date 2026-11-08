//! ColorSwatch and ColorPicker: a colour chip, and a grid of preset chips with a hex field and
//! an optional hue slider.

use std::rc::Rc;

use rune::{Color, El, TextEdit, Tx, col, row};

use crate::{Kit, Slider, TextInput};

mod swatch;

pub use swatch::ColorSwatch;

/// `#rrggbb` for `c`, ignoring alpha, as the hex field shows it.
pub(crate) fn hex6(c: Color) -> String {
    let mut h = c.hex();
    h.truncate(7);
    h
}

/// A [`ColorPicker`]'s hex field, kept in the app's state.
#[derive(Clone, Debug, Default)]
pub struct ColorPickerState {
    pub hex: TextEdit,
    /// The colour the hue slider turns, as picked from a swatch or typed: turning keeps its
    /// lightness and chroma even where a hue clips them.
    toned: Option<Color>,
}

impl ColorPickerState {
    /// A field showing `c`.
    pub fn new(c: Color) -> ColorPickerState {
        ColorPickerState { hex: TextEdit::new(hex6(c)), toned: Some(c) }
    }
    /// Show `c` in the field, as a pick from outside the picker would.
    pub fn set(&mut self, c: Color) {
        self.hex.replace_text(hex6(c));
        self.toned = Some(c);
    }
}

type Access<S> = Rc<dyn for<'a> Fn(&'a mut S) -> &'a mut ColorPickerState>;
type Change<S> = Rc<dyn Fn(&mut S, Color, &mut Tx)>;

/// The twelve presets a picker shows by default: the hue circle in even steps, at one
/// lightness and chroma.
pub(crate) fn default_presets() -> Vec<Color> {
    (0..12).map(|i| rune::color::oklch(0.66, 0.16, 15.0 + i as f32 * 30.0)).collect()
}

/// A colour picker:
/// `ColorPicker::new("tag", &s.picker, |s: &mut App| &mut s.picker).value(s.tag).on_change(|s: &mut App, c, _| s.tag = c).build(&k)`.
///
/// A grid of preset swatches (the chosen one ringed), a hex field (`#rgb` or `#rrggbb`, with
/// or without the `#`) that reports a colour once what is typed parses, and with
/// [`Self::hue`] a slider that turns the colour's hue at its lightness and chroma. A swatch
/// or hue change writes the new hex into the field.
pub struct ColorPicker<S> {
    key: String,
    state: ColorPickerState,
    access: Access<S>,
    value: Option<Color>,
    presets: Vec<Color>,
    columns: usize,
    swatch: f32,
    hue: bool,
    change: Option<Change<S>>,
}

impl<S: 'static> ColorPicker<S> {
    pub fn new(
        key: impl Into<String>,
        state: &ColorPickerState,
        access: impl for<'a> Fn(&'a mut S) -> &'a mut ColorPickerState + 'static,
    ) -> ColorPicker<S> {
        ColorPicker {
            key: key.into(),
            state: state.clone(),
            access: Rc::new(access),
            value: None,
            presets: default_presets(),
            columns: 6,
            swatch: 24.0,
            hue: false,
            change: None,
        }
    }
    /// The current colour (ringed when it is a preset).
    pub fn value(mut self, c: impl Into<Option<Color>>) -> Self {
        self.value = c.into();
        self
    }
    /// The swatches, in order, in place of the default twelve (the hue circle in 30° steps).
    pub fn presets(mut self, colors: &[Color]) -> Self {
        self.presets = colors.to_vec();
        self
    }
    /// Swatches per row, 6 by default.
    pub fn columns(mut self, n: usize) -> Self {
        self.columns = n.max(1);
        self
    }
    /// A swatch's side in points, 24 by default.
    pub fn swatch_size(mut self, s: f32) -> Self {
        self.swatch = s;
        self
    }
    /// Show a hue slider under the field.
    pub fn hue(mut self, on: bool) -> Self {
        self.hue = on;
        self
    }
    pub fn on_change(mut self, f: impl Fn(&mut S, Color, &mut Tx) + 'static) -> Self {
        self.change = Some(Rc::new(f));
        self
    }

    pub fn build(self, k: &Kit) -> El {
        let gap = 6.0;
        let n = self.columns;
        let width = n as f32 * self.swatch + (n - 1) as f32 * gap;
        let current = self.value.map(hex6);
        let pick = |c: Color| {
            let (access, change) = (self.access.clone(), self.change.clone());
            move |s: &mut S, tx: &mut Tx| {
                access(s).set(c);
                if let Some(f) = &change {
                    f(s, c, tx);
                }
            }
        };
        let mut grid = col().gap(gap).key(format!("{}-swatches", self.key)).role("radiogroup").label("Presets");
        for (r, chunk) in self.presets.chunks(n).enumerate() {
            let mut line = row().gap(gap);
            for (i, &c) in chunk.iter().enumerate() {
                let sw = ColorSwatch::new(c)
                    .size(self.swatch)
                    .selected(current.as_deref() == Some(hex6(c).as_str()))
                    .on_press_tx(pick(c))
                    .build(k)
                    .key(format!("swatch{}", r * n + i));
                line = line.child(sw);
            }
            grid = grid.child(line);
        }
        let typed = self.state.hex.text.trim().to_owned();
        let invalid = !typed.is_empty() && parse_hex(&typed).is_none();
        let (access, change) = (self.access.clone(), self.change.clone());
        let field = TextInput::new(&self.state.hex)
            .width(Some(width - self.swatch - 8.0))
            .placeholder(current.clone().unwrap_or_else(|| "#000000".into()))
            .label("Hex colour")
            .invalid(invalid)
            .on_edit(move |s: &mut S| &mut access(s).hex)
            .on_change({
                let access = self.access.clone();
                move |s: &mut S, tx: &mut Tx| {
                    let Some(c) = parse_hex(&access(s).hex.text) else { return };
                    access(s).toned = Some(c);
                    if let Some(f) = &change {
                        f(s, c, tx);
                    }
                }
            })
            .build(k)
            .key(format!("{}-hex", self.key));
        let preview = ColorSwatch::new(self.value.unwrap_or(Color::TRANSPARENT)).size(self.swatch).build(k).key("preview");
        let mut e = col().gap(10.0).key(self.key.clone()).w(width).child(grid).child(row().gap(8.0).center_y().child(preview).child(field));
        if self.hue {
            let base = self.state.toned.or(self.value).unwrap_or(k.c.bg_accent);
            let set_hue = pick_hue(self.access.clone(), self.change.clone(), base);
            let shown = self.value.and_then(hue_of).or_else(|| hue_of(base)).unwrap_or(0.0);
            e = e.child(
                Slider::new(shown)
                    .range(0.0, 360.0)
                    .step(1.0)
                    .width(width)
                    .label("Hue")
                    .value_text(|h| format!("{h:.0}°"))
                    .on_change(set_hue)
                    .build(k)
                    .key(format!("{}-hue", self.key)),
            );
        }
        e
    }
}

fn pick_hue<S: 'static>(access: Access<S>, change: Option<Change<S>>, base: Color) -> impl Fn(&mut S, f32, &mut Tx) {
    move |s: &mut S, h: f32, tx: &mut Tx| {
        let c = with_hue(base, h);
        let st = access(s);
        st.hex.replace_text(hex6(c));
        st.toned.get_or_insert(base);
        if let Some(f) = &change {
            f(s, c, tx);
        }
    }
}

/// `#rgb` or `#rrggbb`, the `#` optional and surrounding space ignored.
pub(crate) fn parse_hex(s: &str) -> Option<Color> {
    let s = s.trim();
    let digits = s.strip_prefix('#').unwrap_or(s);
    if !matches!(digits.len(), 3 | 6) || !digits.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    rune::color::hex(&format!("#{digits}"))
}

/// The colour's OKLCH hue in degrees, `None` for a grey.
pub(crate) fn hue_of(c: Color) -> Option<f32> {
    let [_, a, b, _] = c.to_oklab();
    (a.hypot(b) > 0.02).then(|| b.atan2(a).to_degrees().rem_euclid(360.0))
}

/// `c` turned to hue `h` at its own lightness and chroma, clipped into sRGB (a vivid colour can
/// lose some chroma at a hue sRGB can't show it at); a grey takes a middle chroma.
pub(crate) fn with_hue(c: Color, h: f32) -> Color {
    let [l, a, b, alpha] = c.to_oklab();
    match a.hypot(b) {
        chroma if chroma > 0.02 => rune::color::oklcha(l, chroma, h, alpha),
        _ => rune::color::oklcha(l.clamp(0.45, 0.75), 0.15, h, alpha),
    }
}

#[cfg(test)]
mod tests;
