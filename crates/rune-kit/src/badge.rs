//! Badge / tag (`@opencode/ui` data-display/badge) and Divider (`layout/divider`).

use rune::{El, TextStyle, el, row, text};

use crate::Kit;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BadgeVariant {
    #[default]
    Neutral,
    Accent,
    Subtle,
    /// A state badge in the kit's [`StateColors`](crate::StateColors): the role's soft fill,
    /// its text colour and its border ("Done", "Passing").
    Success,
    /// See [`BadgeVariant::Success`] ("Stale", "Waiting").
    Warning,
    /// See [`BadgeVariant::Success`] ("Failed", "Blocked").
    Danger,
    /// See [`BadgeVariant::Success`] ("New", "Beta").
    Info,
}

/// A small label in a tinted box: `Badge::new("Done").variant(BadgeVariant::Success).build(&k)`.
///
/// Assistive technology reads it as a status, announcing a change to its text politely, unless
/// [`Self::plain`] makes it plain text.
pub struct Badge {
    label: String,
    variant: BadgeVariant,
    standard: bool,
    high_contrast: bool,
    pill: bool,
    height: Option<f32>,
    padding: Option<f32>,
    border: Option<f32>,
    max_width: Option<f32>,
    plain: bool,
}

impl Badge {
    pub fn new(label: impl Into<String>) -> Badge {
        Badge {
            label: label.into(),
            variant: BadgeVariant::Neutral,
            standard: false,
            high_contrast: false,
            pill: false,
            height: None,
            padding: None,
            border: None,
            max_width: None,
            plain: false,
        }
    }
    pub fn variant(mut self, v: BadgeVariant) -> Self {
        self.variant = v;
        self
    }
    /// The older, larger tag (`appearance="standard"`).
    pub fn standard(mut self) -> Self {
        self.standard = true;
        self
    }
    /// A stronger border (`data-high-contrast`).
    pub fn high_contrast(mut self) -> Self {
        self.high_contrast = true;
        self
    }
    /// Fully rounded ends (a chip), in place of the small corner radius.
    pub fn pill(mut self) -> Self {
        self.pill = true;
        self
    }
    /// The height in points, in place of 16 (18 for [`Self::standard`]).
    pub fn height(mut self, h: f32) -> Self {
        self.height = Some(h.max(1.0));
        self
    }
    /// The padding left and right of the text in points, in place of 4 (6 for
    /// [`Self::standard`]).
    pub fn padding(mut self, px: f32) -> Self {
        self.padding = Some(px.max(0.0));
        self
    }
    /// The border's width in points, in place of 0.5; 0 drops it. Variants with no border
    /// (`Accent`, `Subtle`) stay without one.
    pub fn border_width(mut self, w: f32) -> Self {
        self.border = Some(w.max(0.0));
        self
    }
    /// At most `w` points wide: a longer text is cut short with an ellipsis, and the whole of
    /// it shows as a tooltip.
    pub fn max_width(mut self, w: f32) -> Self {
        self.max_width = Some(w.max(0.0));
        self
    }
    /// Read as plain text rather than a status: for chips in a list or card, where every
    /// change announced would be noise.
    pub fn plain(mut self) -> Self {
        self.plain = true;
        self
    }

    pub fn build(self, k: &Kit) -> El {
        let c = &k.c;
        let (fill, ink, border) = match self.variant {
            BadgeVariant::Neutral if self.standard => (c.surface_raised_base, c.std_text, Some(c.border_weak)),
            BadgeVariant::Neutral => {
                (c.layer_02, c.text_muted, Some(if self.high_contrast { c.border_strong } else { c.border_base }))
            }
            BadgeVariant::Accent => (c.bg_accent, c.text_contrast, None),
            BadgeVariant::Subtle => (c.layer_03, c.text_muted, None),
            BadgeVariant::Success => (c.success.bg, c.success.fg, Some(c.success.border)),
            BadgeVariant::Warning => (c.warning.bg, c.warning.fg, Some(c.warning.border)),
            BadgeVariant::Danger => (c.danger.bg, c.danger.fg, Some(c.danger.border)),
            BadgeVariant::Info => (c.info.bg, c.info.fg, Some(c.info.border)),
        };
        let (h, px, radius, font) = if self.standard {
            (18.0, 6.0, k.radius.xs, k.ty.text_small_medium)
        } else {
            (16.0, 4.0, 2.0, TextStyle { line_height: 11.0, ..k.ty.small_medium })
        };
        let h = self.height.unwrap_or(h);
        let px = self.padding.unwrap_or(px);
        let radius = if self.pill { h / 2.0 } else { radius };
        let mut e = row().center().gap(4.0).h(h).px(px).radius(radius).fill(fill);
        let mut label = text(self.label.clone()).font(font).color(ink);
        if let Some(w) = self.max_width {
            e = e.max_w(w).shrink();
            label = label.truncate().shrink().help_when_truncated(self.label.clone());
        }
        e = if self.plain {
            e.child(label)
        } else {
            // A status: assistive technology announces a change to its label, politely.
            e.role("status").label(self.label).child(label.a11y_hidden())
        };
        match (border, self.border.unwrap_or(0.5)) {
            (Some(b), w) if w > 0.0 => e.border(w, b),
            _ => e,
        }
    }
}

/// A 1 point rule (`Divider`), horizontal unless `vertical`.
pub struct Divider {
    vertical: bool,
}

impl Divider {
    pub fn horizontal() -> Divider {
        Divider { vertical: false }
    }
    pub fn vertical() -> Divider {
        Divider { vertical: true }
    }
    pub fn build(self, k: &Kit) -> El {
        let e = el().role("separator").fill(k.c.border_base);
        if self.vertical { e.w(1.0).h_full() } else { e.h(1.0).w_full() }
    }
}
