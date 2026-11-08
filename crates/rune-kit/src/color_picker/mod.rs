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
    /// A colour typed into the field and reported to `on_change` but not yet to `on_commit`:
    /// Return or the field losing focus commits it.
    pending: Option<Color>,
}

impl ColorPickerState {
    /// A field showing `c`.
    pub fn new(c: Color) -> ColorPickerState {
        ColorPickerState { hex: TextEdit::new(hex6(c)), toned: Some(c), pending: None }
    }
    /// Show `c` in the field, as a pick from outside the picker would.
    pub fn set(&mut self, c: Color) {
        self.hex.replace_text(hex6(c));
        self.toned = Some(c);
        self.pending = None;
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
/// [`Self::hue`] a slider that turns the colour's hue at its lightness and chroma, its track
/// painted with the hues it turns through. A swatch or hue change writes the new hex into the
/// field. [`Self::on_change`] hears every step; [`Self::on_commit`] only settled picks.
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
    commit: Option<Change<S>>,
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
            commit: None,
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
    /// The colour moves: a swatch pick, every hue drag step or key, and every keystroke that
    /// leaves a hex that parses.
    pub fn on_change(mut self, f: impl Fn(&mut S, Color, &mut Tx) + 'static) -> Self {
        self.change = Some(Rc::new(f));
        self
    }
    /// The colour settles: a swatch pick, a hue drag released (not cancelled) or a hue key
    /// step, and a typed hex on Return or when the field loses focus (the last one that
    /// parsed). For saving once rather than on every step.
    pub fn on_commit(mut self, f: impl Fn(&mut S, Color, &mut Tx) + 'static) -> Self {
        self.commit = Some(Rc::new(f));
        self
    }

    pub fn build(self, k: &Kit) -> El {
        let gap = 6.0;
        let n = self.columns;
        let width = n as f32 * self.swatch + (n - 1) as f32 * gap;
        let current = self.value.map(hex6);
        let pick = |c: Color| {
            let (access, change, commit) = (self.access.clone(), self.change.clone(), self.commit.clone());
            move |s: &mut S, tx: &mut Tx| {
                access(s).set(c);
                for f in [&change, &commit].into_iter().flatten() {
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
        let mut field = TextInput::new(&self.state.hex)
            .width(Some(width - self.swatch - 8.0))
            .placeholder(current.clone().unwrap_or_else(|| "#000000".into()))
            .label("Hex colour")
            .invalid(invalid)
            .on_edit(move |s: &mut S| &mut access(s).hex)
            .on_change({
                let (access, track) = (self.access.clone(), self.commit.is_some());
                move |s: &mut S, tx: &mut Tx| {
                    let Some(c) = parse_hex(&access(s).hex.text) else { return };
                    access(s).toned = Some(c);
                    if track {
                        access(s).pending = Some(c);
                    }
                    if let Some(f) = &change {
                        f(s, c, tx);
                    }
                }
            });
        // Only with a commit to make: otherwise Return still presses a dialog's default button.
        if self.commit.is_some() {
            field = field.on_submit(commit_typed(self.access.clone(), self.commit.clone()));
        }
        let field = field.build(k).key(format!("{}-hex", self.key));
        let preview =
            ColorSwatch::new(self.value.unwrap_or(Color::TRANSPARENT)).size(self.swatch).build(k).key("preview");
        let mut e = col()
            .gap(10.0)
            .key(self.key.clone())
            .w(width)
            .child(grid)
            .child(row().gap(8.0).center_y().child(preview).child(field));
        // The field lost focus with a typed colour not yet committed: this probe, mounted for
        // that build only, commits it on its first layout and goes again.
        if self.commit.is_some() && self.state.pending.is_some() && !self.state.hex.focused {
            let typed = commit_typed(self.access.clone(), self.commit.clone());
            e = e.child(
                rune::el()
                    .key(format!("{}-blur-commit", self.key))
                    .absolute(0.0, 0.0)
                    .size(0.0, 0.0)
                    .a11y_hidden()
                    .on_layout_tx(move |s: &mut S, _, tx: &mut Tx| typed(s, tx)),
            );
        }
        if self.hue {
            let base = self.state.toned.or(self.value).unwrap_or(k.c.bg_accent);
            let set_hue = pick_hue(self.access.clone(), self.change.clone(), base);
            let shown = self.value.and_then(hue_of).or_else(|| hue_of(base)).unwrap_or(0.0);
            // Seven 60° steps from red around to red, at the colour's lightness and chroma.
            let stops: Vec<(f32, Color)> = (0..7).map(|i| (i as f32 / 6.0, with_hue(base, i as f32 * 60.0))).collect();
            let mut slider = Slider::new(shown)
                .range(0.0, 360.0)
                .step(1.0)
                .width(width)
                .label("Hue")
                .value_text(|h| format!("{h:.0}°"))
                .track_gradient(&stops)
                .on_change(set_hue);
            if self.commit.is_some() {
                slider = slider.on_commit(pick_hue(self.access.clone(), self.commit.clone(), base));
            }
            e = e.child(slider.build(k).key(format!("{}-hue", self.key)));
        }
        e
    }
}

/// Commits the colour typed into the field, if one is waiting.
fn commit_typed<S: 'static>(access: Access<S>, commit: Option<Change<S>>) -> impl Fn(&mut S, &mut Tx) {
    move |s: &mut S, tx: &mut Tx| {
        let Some(c) = access(s).pending.take() else { return };
        if let Some(f) = &commit {
            f(s, c, tx);
        }
    }
}

/// Reports hue `h` on `base` to `report` (the change or the commit), writing its hex into the
/// field.
fn pick_hue<S: 'static>(access: Access<S>, report: Option<Change<S>>, base: Color) -> impl Fn(&mut S, f32, &mut Tx) {
    move |s: &mut S, h: f32, tx: &mut Tx| {
        let c = with_hue(base, h);
        let st = access(s);
        st.hex.replace_text(hex6(c));
        st.toned.get_or_insert(base);
        st.pending = None;
        if let Some(f) = &report {
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
