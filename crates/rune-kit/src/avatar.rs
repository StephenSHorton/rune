//! Avatar (shadcn `ui/avatar`): a person's picture, or their initials on a colour of their own
//! while there is none, with an optional status dot.

use rune::{Color, El, Pixels, SurfaceFit, TextStyle, el, text};

use crate::Kit;

/// An avatar's outline: a circle, or a square with the kit's rounded corners.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum AvatarShape {
    #[default]
    Round,
    Rounded,
}

/// `Avatar::new("Ada Lovelace").image(&s.photo).size(32.0).status(k.c.success.icon).build(&k)`.
///
/// It shows the initials ("AL") on a colour picked from the name, the same on every build, and
/// the picture over them once it has pixels, so a picture still loading shows the initials.
/// Assistive technology reads it as an image named for the person.
pub struct Avatar {
    name: String,
    initials: Option<String>,
    image: Option<Pixels>,
    size: f32,
    shape: AvatarShape,
    fill: Option<Color>,
    ink: Option<Color>,
    status: Option<Color>,
}

impl Avatar {
    pub fn new(name: impl Into<String>) -> Avatar {
        Avatar {
            name: name.into(),
            initials: None,
            image: None,
            size: 32.0,
            shape: AvatarShape::Round,
            fill: None,
            ink: None,
            status: None,
        }
    }
    /// The picture, filling the avatar (cropped to its shape).
    pub fn image(mut self, p: &Pixels) -> Self {
        self.image = Some(p.clone());
        self
    }
    /// The side in points, 32 by default.
    pub fn size(mut self, s: f32) -> Self {
        self.size = s.max(0.0);
        self
    }
    pub fn shape(mut self, s: AvatarShape) -> Self {
        self.shape = s;
        self
    }
    /// The letters shown without a picture, in place of the name's initials.
    pub fn initials(mut self, s: impl Into<String>) -> Self {
        self.initials = Some(s.into());
        self
    }
    /// The colour behind the initials, in place of the one picked from the name.
    pub fn fill(mut self, c: Color) -> Self {
        self.fill = Some(c);
        self
    }
    /// The initials' colour, white by default.
    pub fn ink(mut self, c: Color) -> Self {
        self.ink = Some(c);
        self
    }
    /// A presence dot at the bottom right, in `c` (online, away, busy), ringed in the page colour.
    pub fn status(mut self, c: Color) -> Self {
        self.status = Some(c);
        self
    }

    pub fn build(self, k: &Kit) -> El {
        let side = self.size;
        let radius = match self.shape {
            AvatarShape::Round => side / 2.0,
            AvatarShape::Rounded => k.radius.md.min(side / 4.0),
        };
        let letters = self.initials.clone().unwrap_or_else(|| initials(&self.name));
        let font = TextStyle { size: (side * 0.4).round().max(6.0), weight: 600.0, line_height: 0.0, ..k.ty.ui_medium };
        let mut face = el()
            .key("face")
            .size(side, side)
            .radius(radius)
            .fill(self.fill.unwrap_or_else(|| name_color(&self.name)))
            .center()
            .a11y_hidden()
            .child(text(letters).key("initials").font(font).color(self.ink.unwrap_or(Color::WHITE)));
        if let Some(p) = &self.image {
            face = face.child(
                el().key("image").absolute(0.0, 0.0).size(side, side).radius(radius).pixels_fit(p, SurfaceFit::Fill),
            );
        }
        let mut e = el().role("img").label(self.name).size(side, side).child(face);
        if let Some(c) = self.status {
            let dot = (side * 0.3).round().max(6.0);
            let ring = (side / 16.0).clamp(1.5, 3.0);
            e = e.child(
                el().key("status")
                    .absolute(side - dot + ring / 2.0, side - dot + ring / 2.0)
                    .size(dot, dot)
                    .radius(dot / 2.0)
                    .fill(c)
                    .border(ring, k.c.bg_base)
                    .a11y_hidden(),
            );
        }
        e
    }
}

/// The first letters of the name's first and last words, upper-cased ("Ada Lovelace" → "AL",
/// "plato" → "P").
pub(crate) fn initials(name: &str) -> String {
    let words: Vec<&str> = name.split_whitespace().filter(|w| w.chars().any(char::is_alphanumeric)).collect();
    let first = |w: &str| w.chars().find(|c| c.is_alphanumeric());
    let mut out = String::new();
    let picks = match words.as_slice() {
        [] => vec![],
        [one] => vec![*one],
        [a, .., b] => vec![*a, *b],
    };
    for w in picks {
        if let Some(c) = first(w) {
            out.extend(c.to_uppercase());
        }
    }
    out
}

/// A colour for `name`, the same every time: one of the hue circle's steps at a lightness white
/// initials read on.
pub(crate) fn name_color(name: &str) -> Color {
    // FNV-1a: stable across runs and platforms, unlike the std hasher.
    let mut h: u32 = 0x811c_9dc5;
    for b in name.trim().to_lowercase().bytes() {
        h = (h ^ u32::from(b)).wrapping_mul(0x0100_0193);
    }
    let hue = (h % 24) as f32 * 15.0;
    rune::color::oklch(0.56, 0.12, hue)
}

#[cfg(test)]
#[path = "avatar_tests.rs"]
mod tests;
