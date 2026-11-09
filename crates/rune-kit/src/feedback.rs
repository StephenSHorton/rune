//! Spinner, Loader, Progress, ProgressCircle and Skeleton (`@opencode/ui` components/spinner,
//! feedback/loader, components/progress, feedback/progress-circle; shadcn `ui/skeleton`).
//!
//! Renderer-owned loops cool through presence on removal; the spinner's run on the wall clock,
//! so every spinner on screen keeps one phase.

use rune::{Color, El, Icon, col, el, icon, row, spacer, text};

use crate::Kit;

mod radar;

/// Soft Radar, the activity spinner (`Spinner`, `spinner-radar.ts`): a 5×5 grid of 2-unit dots
/// on a 3-unit pitch in a 16-unit box, whose lit head sweeps clockwise around each square ring
/// every 1.8 s, leaving a fading trail, with the centre dot steady at 0.64. Unlit dots rest at
/// 0.2. Every spinner shares the wall clock's phase, so all of them on screen sweep together, and
/// under reduced motion each dot holds the pose a fifth of the way into its loop, as Desktop's
/// do. 18 points by default; Desktop's `ActivityRing` is 14 and its session progress indicator 16,
/// both in `icon_muted`.
pub struct Spinner {
    size: f32,
    color: Option<Color>,
}

impl Spinner {
    pub fn new() -> Spinner {
        Spinner { size: 18.0, color: None }
    }
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }
    /// The dots' ink (CSS `currentColor`); `std_text` by default.
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }
    pub fn build(self, k: &Kit) -> El {
        let ink = self.color.unwrap_or(k.c.std_text);
        let u = self.size / 16.0;
        let mut e = el().role("progressbar").label("Loading").size(self.size, self.size);
        for (i, d) in radar::dots().iter().enumerate() {
            let side = d.size * u;
            let dot = el().key(format!("d{i}")).absolute(d.x * u, d.y * u).size(side, side).radius(side * 0.24).fill(ink).a11y_hidden();
            e = e.child(match d.cycle {
                Some(c) => dot.cycle(c),
                None => dot.opacity(d.still),
            });
        }
        e
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

static RING: Icon = Icon {
    name: "kit-loader-ring",
    view: 16.0,
    body: r#"<circle cx="8" cy="8" r="6" stroke="currentColor" stroke-width="2" fill="none"/>"#,
};
/// A third of the ring, from 12 o'clock clockwise, round caps.
static ARC: Icon = Icon {
    name: "kit-loader-arc",
    view: 16.0,
    body: r#"<path d="M8 2A6 6 0 0 1 13.258 10.89" stroke="currentColor" stroke-width="2" stroke-linecap="round" fill="none"/>"#,
};

/// The v2 loader, for loading that isn't AI activity (`Loader`): a 2-unit ring at 0.2 with a
/// third of it, round-capped from 12 o'clock, turning once every 900 ms; it stops at 12 o'clock
/// under reduced motion. 16 points by default, in `icon_muted`.
pub struct Loader {
    size: f32,
    color: Option<Color>,
}

impl Loader {
    pub fn new() -> Loader {
        Loader { size: 16.0, color: None }
    }
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }
    /// The ring's ink (CSS `currentColor`); `icon_muted` by default.
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }
    pub fn build(self, k: &Kit) -> El {
        let ink = self.color.unwrap_or(k.c.icon_muted);
        el().role("progressbar")
            .label("Loading")
            .size(self.size, self.size)
            .child(icon(&RING, self.size).absolute(0.0, 0.0).color(ink).opacity(0.2))
            .child(icon(&ARC, self.size).absolute(0.0, 0.0).color(ink).looping(rune_motion::LoopSpec::rotation(0.9)))
    }
}

impl Default for Loader {
    fn default() -> Self {
        Self::new()
    }
}

/// A progress indicator's value for assistive technology: the fraction done, read as a percentage.
fn report(e: El, value: Option<f32>) -> El {
    match value {
        Some(v) => {
            let v = v.clamp(0.0, 1.0);
            e.range_value((f64::from(v) * 1e4).round() / 1e4, 0.0, 1.0).a11y_value(format!("{}%", (v * 100.0).round()))
        }
        None => e,
    }
}

/// A linear progress bar with an optional label and value label (`Progress`). Without a value
/// it is indeterminate: a quarter-width segment sweeps across the track, wrapping from the left
/// (held mid-track under reduced motion).
pub struct Progress {
    label: Option<String>,
    value: Option<f32>,
    show_value: bool,
    height: f32,
    color: Option<Color>,
    track: Option<Color>,
    radius: f32,
    label_hidden: bool,
}

impl Progress {
    /// `value` in 0..=1; `None` is indeterminate.
    pub fn new(value: Option<f32>) -> Progress {
        Progress {
            label: None,
            value,
            show_value: false,
            height: 8.0,
            color: None,
            track: None,
            radius: 999.0,
            label_hidden: false,
        }
    }
    /// An indeterminate bar: work under way with no measure of how far along it is.
    pub fn indeterminate() -> Progress {
        Progress::new(None)
    }
    /// The bar's thickness in points, 8 by default; a thin 2–4 point bar drops the track's border.
    pub fn height(mut self, h: f32) -> Self {
        self.height = h.max(1.0);
        self
    }
    /// The filled part's colour, `border_active` by default (a status colour, an accent).
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }
    /// The track's colour behind the fill, `surface_base` by default.
    pub fn track(mut self, c: Color) -> Self {
        self.track = Some(c);
        self
    }
    /// The corner radius of the track and the filled part, round ends by default; 0 draws
    /// square ends, the indeterminate sweep's segment too.
    pub fn radius(mut self, r: f32) -> Self {
        self.radius = r.max(0.0);
        self
    }
    /// What is progressing ("Upload"), shown above the bar and read by assistive technology.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }
    /// Keep the [`Self::label`] for assistive technology only, with no header above the bar
    /// (unless [`Self::show_value`] asks for one).
    pub fn label_hidden(mut self) -> Self {
        self.label_hidden = true;
        self
    }
    pub fn show_value(mut self) -> Self {
        self.show_value = true;
        self
    }
    pub fn build(self, k: &Kit) -> El {
        let c = &k.c;
        let mut root = col().role("progressbar").w_full().gap(4.0);
        if let Some(l) = &self.label {
            root = root.label(l.clone());
        }
        let shown = self.label.as_ref().filter(|_| !self.label_hidden);
        if shown.is_some() || self.show_value {
            let mut header = row().key("header").gap(8.0).center_y();
            if let Some(l) = shown {
                header = header.child(text(l.clone()).key("label").font(k.ty.text_small).color(c.std_text));
            }
            header = header.child(spacer());
            if let (true, Some(v)) = (self.show_value, self.value) {
                header = header.child(
                    text(format!("{}%", (v * 100.0).round())).key("value").font(k.ty.text_small).color(c.text_weak),
                );
            }
            root = root.child(header.a11y_hidden());
        }
        root = report(root, self.value);
        let mut track = row()
            .key("track")
            .w_full()
            .h(self.height)
            .radius(self.radius)
            .fill(self.track.unwrap_or(c.surface_base))
            .clip_soft(0.0);
        if self.height > 4.0 {
            track = track.border(1.0, c.border_weak);
        }
        let fill = el().key("fill").h_full().radius(self.radius).fill(self.color.unwrap_or(c.border_active));
        track = track.child(match self.value {
            Some(v) => fill.w_frac(v.clamp(0.0, 1.0)),
            // A quarter-width segment crosses the whole track, then wraps from the left.
            None => fill.w_frac(0.25).looping(rune_motion::LoopSpec::slide(1.4, [-1.0, 0.0], [4.0, 0.0])),
        });
        root.child(track.key("track").a11y_hidden())
    }
}

/// A placeholder for content still loading (`Skeleton`): a muted block whose opacity pulses
/// between full and half every 1.6 s, holding full under reduced motion. Full width and 12
/// points tall by default; size it to the text or picture it stands in for. Assistive
/// technology skips it unless it is given a [`Self::label`].
///
/// `col().gap(8.0).child(Skeleton::new().width(120.0).build(&k)).child(Skeleton::new().build(&k))`
pub struct Skeleton {
    width: Option<f32>,
    height: f32,
    radius: Option<f32>,
    label: Option<String>,
}

impl Skeleton {
    pub fn new() -> Skeleton {
        Skeleton { width: None, height: 12.0, radius: None, label: None }
    }
    /// A circle `d` points across (an avatar's place).
    pub fn circle(d: f32) -> Skeleton {
        Skeleton { width: Some(d), height: d, radius: Some(d / 2.0), label: None }
    }
    /// The width in points; the container's full width by default.
    pub fn width(mut self, w: f32) -> Self {
        self.width = Some(w);
        self
    }
    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }
    /// The corner radius, the kit's small radius by default.
    pub fn radius(mut self, r: f32) -> Self {
        self.radius = Some(r);
        self
    }
    /// Report it to assistive technology as a busy indicator named `l` ("Loading messages").
    /// Give one to the first block of a group, not every block.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }
    pub fn build(self, k: &Kit) -> El {
        // Reduced motion holds the loop's first pose, `low`: full opacity here.
        let e = el()
            .h(self.height)
            .radius(self.radius.unwrap_or(k.radius.sm))
            .fill(k.c.layer_03)
            .looping(rune_motion::LoopSpec::opacity(1.6, 0.0, 1.0, 0.5));
        let e = match self.width {
            Some(w) => e.w(w),
            None => e.w_full(),
        };
        match self.label {
            Some(l) => e.role("progressbar").label(l),
            None => e.a11y_hidden(),
        }
    }
}

impl Default for Skeleton {
    fn default() -> Self {
        Self::new()
    }
}

/// A small ring that fills clockwise (`ProgressCircle`): `value` in 0..=1.
pub struct ProgressCircle {
    value: f32,
    size: f32,
    label: Option<String>,
}

impl ProgressCircle {
    pub fn new(value: f32) -> ProgressCircle {
        ProgressCircle { value, size: 16.0, label: None }
    }
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }
    /// The accessible name: what is progressing ("Upload").
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }
    pub fn build(self, k: &Kit) -> El {
        let ink = k.c.icon_muted;
        let e = report(el().role("progressbar"), Some(self.value));
        let e = match self.label {
            Some(l) => e.label(l),
            None => e,
        };
        e.size(self.size, self.size)
            .child(icon(&RING, self.size).absolute(0.0, 0.0).color(ink).opacity(0.2))
            .child(icon(arc_icon(self.value), self.size).absolute(0.0, 0.0).color(ink))
    }
}

/// Arcs for progress values, quantised to 2% (icons are static; one leak per step at most).
fn arc_icon(v: f32) -> &'static Icon {
    use std::sync::Mutex;
    static CACHE: Mutex<Vec<Option<&'static Icon>>> = Mutex::new(Vec::new());
    let step = (v.clamp(0.0, 1.0) * 50.0).round() as usize;
    let mut cache = CACHE.lock().unwrap_or_else(|e| e.into_inner());
    if cache.len() < 51 {
        cache.resize(51, None);
    }
    if let Some(i) = cache[step] {
        return i;
    }
    let a = step as f32 / 50.0 * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
    let (x, y) = (8.0 + 6.0 * a.cos(), 8.0 + 6.0 * a.sin());
    let large = if step > 25 { 1 } else { 0 };
    let body: &'static str = if step == 0 {
        ""
    } else if step == 50 {
        r#"<circle cx="8" cy="8" r="6" stroke="currentColor" stroke-width="2" fill="none"/>"#
    } else {
        Box::leak(
            format!(
                r#"<path d="M8 2A6 6 0 {large} 1 {x:.3} {y:.3}" stroke="currentColor" stroke-width="2" fill="none"/>"#
            )
            .into_boxed_str(),
        )
    };
    let i: &'static Icon = Box::leak(Box::new(Icon { name: "kit-progress-arc", view: 16.0, body }));
    cache[step] = Some(i);
    i
}
