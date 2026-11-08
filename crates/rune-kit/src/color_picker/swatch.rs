//! ColorSwatch: one colour chip.

use rune::{Color, El, Tx, el};

use super::hex6;
use crate::{Acts, Kit};

/// A colour chip: `ColorSwatch::new(c).selected(s.tag == c).on_press(..).build(&k)`.
///
/// With a press handler it reads as a radio button, checked when selected, named by its
/// [`Self::label`] or else its hex; without one, as an image of the colour.
pub struct ColorSwatch {
    color: Color,
    size: f32,
    selected: bool,
    round: bool,
    label: Option<String>,
    acts: Acts,
    pressable: bool,
}

impl ColorSwatch {
    pub fn new(color: Color) -> ColorSwatch {
        ColorSwatch {
            color,
            size: 20.0,
            selected: false,
            round: false,
            label: None,
            acts: Acts::default(),
            pressable: false,
        }
    }
    /// The side in points, 20 by default.
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }
    /// The chosen one: a ring in the text colour around it.
    pub fn selected(mut self, on: bool) -> Self {
        self.selected = on;
        self
    }
    /// A circle instead of a rounded square.
    pub fn round(mut self) -> Self {
        self.round = true;
        self
    }
    /// The accessible name ("Blue"); its hex by default.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }
    pub fn on_press<S: 'static>(mut self, f: impl Fn(&mut S) + 'static) -> Self {
        self.acts.press(f);
        self.pressable = true;
        self
    }
    pub fn on_press_tx<S: 'static>(mut self, f: impl Fn(&mut S, &mut Tx) + 'static) -> Self {
        self.acts.press_tx(f);
        self.pressable = true;
        self
    }
    pub fn build(self, k: &Kit) -> El {
        let side = self.size;
        let radius = if self.round { side / 2.0 } else { k.radius.sm.min(side / 4.0) };
        let mut e = el()
            .size(side, side)
            .radius(radius)
            .fill(self.color)
            // A hairline so a swatch the page's own colour still shows its edge.
            .border(1.0, k.c.border_base)
            .label(self.label.unwrap_or_else(|| hex6(self.color)));
        if self.selected {
            e = e.outline(2.0, 2.0, k.c.text_base);
        }
        if !self.pressable {
            return e.role("img");
        }
        e = e.role("radio").checked(self.selected).focusable(true).focus_ring(2.0, 2.0, k.c.border_focus);
        self.acts.apply(e, true)
    }
}
