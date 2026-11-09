//! Slider (shadcn `ui/slider`, AppKit's `NSSlider`): a horizontal track with a knob that drags
//! or steps a value in a range.

use std::any::Any;
use std::rc::Rc;

use rune::{Color, Drag, El, Phase, Tx, el};

use crate::{Kit, State, shadows};

/// What the handlers need from one build: the range, the step and the knob's size.
#[derive(Clone, Copy, Debug)]
struct Geometry {
    value: f32,
    min: f32,
    max: f32,
    step: f32,
    knob: f32,
}

impl Geometry {
    /// `v` clamped into the range and snapped to the step, counted from the minimum.
    fn snap(self, v: f32) -> f32 {
        let v = v.clamp(self.min, self.max);
        if self.step > 0.0 {
            let n = ((v - self.min) / self.step).round();
            (self.min + n * self.step).clamp(self.min, self.max)
        } else {
            v
        }
    }
    /// Where `v` sits along the track, 0 at the minimum and 1 at the maximum.
    fn frac(self, v: f32) -> f32 {
        let span = self.max - self.min;
        if span > 0.0 { ((v - self.min) / span).clamp(0.0, 1.0) } else { 0.0 }
    }
    /// The value under the knob's centre at `x` on a track `w` wide.
    fn at(self, x: f32, w: f32) -> f32 {
        let travel = (w - self.knob).max(1.0);
        let t = ((x - self.knob / 2.0) / travel).clamp(0.0, 1.0);
        self.snap(self.min + t * (self.max - self.min))
    }
    /// The knob's centre for `v` on a track `w` wide.
    fn knob_x(self, v: f32, w: f32) -> f32 {
        self.knob / 2.0 + self.frac(v) * (w - self.knob).max(0.0)
    }
    /// One keyboard step: the step, or a hundredth of the range for a continuous slider.
    fn key_step(self) -> f32 {
        if self.step > 0.0 { self.step } else { (self.max - self.min) / 100.0 }
    }
}

/// A callback with the app's state type erased, so `on_change` and `on_commit` share one wiring.
type Erased = Rc<dyn Fn(&mut dyn Any, f32, &mut Tx)>;
type Wire = Rc<dyn Fn(El, Geometry, Option<Erased>, Option<Erased>) -> El>;

fn erase<S: 'static>(f: impl Fn(&mut S, f32, &mut Tx) + 'static) -> Erased {
    Rc::new(move |s: &mut dyn Any, v, tx: &mut Tx| {
        if let Some(s) = s.downcast_mut::<S>() {
            f(s, v, tx)
        }
    })
}

fn wiring<S: 'static>() -> Wire {
    Rc::new(|e, g, change, commit| wire::<S>(e, g, change, commit))
}

/// A horizontal slider:
/// `Slider::new(s.volume).range(0.0, 1.0).step(0.05).label("Volume").on_change(|s: &mut App, v, _| s.volume = v).build(&k)`.
///
/// Dragging the knob moves the value; pressing the track jumps there. While focused, Left and
/// Down step it down, Right and Up up, Page Down and Page Up by ten steps, Home and End to the
/// ends. Every value it reports is clamped to the range and snapped to the step. Assistive
/// technology reads it as a slider with its value and can step it.
pub struct Slider {
    geo: Geometry,
    width: f32,
    label: Option<String>,
    value_text: Option<Box<dyn Fn(f32) -> String>>,
    state: State,
    wire: Option<Wire>,
    change: Option<Erased>,
    commit: Option<Erased>,
    gradient: Option<Vec<(f32, Color)>>,
}

impl Slider {
    /// A slider at `value`, in 0..=1 with no step unless [`Self::range`] and [`Self::step`]
    /// say otherwise.
    pub fn new(value: f32) -> Slider {
        Slider {
            geo: Geometry { value, min: 0.0, max: 1.0, step: 0.0, knob: 16.0 },
            width: 160.0,
            label: None,
            value_text: None,
            state: State::Rest,
            wire: None,
            change: None,
            commit: None,
            gradient: None,
        }
    }
    /// The range, `min` at the left end and `max` at the right (0..=1 by default).
    pub fn range(mut self, min: f32, max: f32) -> Self {
        (self.geo.min, self.geo.max) = if min <= max { (min, max) } else { (max, min) };
        self
    }
    /// The values it takes, counted from the minimum: `0.05` for twentieths. 0 (the default) is
    /// continuous, and the keys then move a hundredth of the range.
    pub fn step(mut self, step: f32) -> Self {
        self.geo.step = if step.is_finite() { step.max(0.0) } else { 0.0 };
        self
    }
    /// The track's width in points, 160 by default.
    pub fn width(mut self, w: f32) -> Self {
        self.width = w.max(self.geo.knob);
        self
    }
    /// The accessible name: what the slider sets ("Volume").
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = Some(l.into());
        self
    }
    /// How assistive technology reads the value ("40%", "1.5×"). By default a 0..=1 slider
    /// reads as a percentage and any other as its number.
    pub fn value_text(mut self, f: impl Fn(f32) -> String + 'static) -> Self {
        self.value_text = Some(Box::new(f));
        self
    }
    /// Paint the track left to right with these `(position, colour)` stops (positions in
    /// 0..=1, at most 8), in place of the plain track and the accent fill up to the knob: a hue
    /// strip, an opacity ramp.
    pub fn track_gradient(mut self, stops: &[(f32, Color)]) -> Self {
        self.gradient = Some(stops.to_vec());
        self
    }
    pub fn state(mut self, s: State) -> Self {
        self.state = s;
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        if d {
            self.state = State::Disabled;
        }
        self
    }
    /// The value moves: every drag step, track press and key, with the new value (already
    /// clamped and snapped). A cancelled drag reports the value it started from.
    pub fn on_change<S: 'static>(mut self, f: impl Fn(&mut S, f32, &mut Tx) + 'static) -> Self {
        self.change = Some(erase(f));
        self.wire = Some(wiring::<S>());
        self
    }
    /// The value settles: a drag released (not cancelled) or a key step, with the final value.
    /// For saving a setting once rather than on every move.
    pub fn on_commit<S: 'static>(mut self, f: impl Fn(&mut S, f32, &mut Tx) + 'static) -> Self {
        self.commit = Some(erase(f));
        self.wire = Some(wiring::<S>());
        self
    }

    pub fn build(self, k: &Kit) -> El {
        let c = &k.c;
        let g = Geometry { value: Geometry::snap(self.geo, self.geo.value), ..self.geo };
        let (w, h, bar, knob) = (self.width, 20.0, 4.0, g.knob);
        let x = g.knob_x(g.value, w);
        let disabled = self.state == State::Disabled;
        let text = match &self.value_text {
            Some(f) => f(g.value),
            None if g.min == 0.0 && g.max == 1.0 => format!("{}%", (g.value * 100.0).round()),
            None => format_number(g.value, g.step),
        };
        let mut e = el()
            .role("slider")
            .range_value(f64::from(g.value), f64::from(g.min), f64::from(g.max))
            .range_step(f64::from(g.key_step()))
            .a11y_value(text)
            .size(w, h)
            .disabled(disabled);
        let track = el().key("track").size(w, bar).absolute(0.0, (h - bar) / 2.0).radius(bar / 2.0).fill(c.layer_04);
        e = match &self.gradient {
            Some(stops) => e.child(track.linear_gradient(std::f32::consts::FRAC_PI_2, stops).a11y_hidden()),
            None => e.child(track.a11y_hidden()).child(
                el().key("range")
                    .size(x, bar)
                    .absolute(0.0, (h - bar) / 2.0)
                    .radius(bar / 2.0)
                    .fill(c.bg_accent)
                    .a11y_hidden(),
            ),
        };
        let mut thumb = el()
            .key("knob")
            .size(knob, knob)
            .absolute(x - knob / 2.0, (h - knob) / 2.0)
            .radius(knob / 2.0)
            .fill(c.knob)
            .a11y_hidden();
        thumb = shadows(k, thumb, &k.elevation.knob);
        if self.state == State::Focus {
            thumb = thumb.outline(2.0, 1.0, c.border_focus);
        }
        e = e.child(thumb.focus_ring(2.0, 1.0, c.border_focus));
        if let Some(l) = self.label {
            e = e.label(l);
        }
        if disabled {
            return e.layer_opacity(0.5);
        }
        e = e.focusable(true);
        match self.wire {
            Some(w) => w(e, g, self.change, self.commit),
            None => e,
        }
    }
}

/// Attaches keys and drag: moves report to `change`, settled values to `commit`, and a key
/// step to both.
fn wire<S: 'static>(e: El, g: Geometry, change: Option<Erased>, commit: Option<Erased>) -> El {
    let report = move |s: &mut S, v: f32, tx: &mut Tx, moved: bool, settled: bool| {
        let pairs = [(moved, &change), (settled, &commit)];
        for (on, f) in pairs {
            if let (true, Some(f)) = (on, f) {
                f(s as &mut dyn Any, v, tx);
            }
        }
    };
    let report = Rc::new(report);
    let r = report.clone();
    let step =
        move |s: &mut S, steps: i32, tx: &mut Tx| r(s, g.snap(g.value + steps as f32 * g.key_step()), tx, true, true);
    let r = report.clone();
    let limit = move |s: &mut S, max: bool, tx: &mut Tx| r(s, if max { g.max } else { g.min }, tx, true, true);
    // The drag keeps the pointer's offset from the knob's centre when it grabbed the knob, so the
    // knob doesn't jump under it; a press on the track lands the knob's centre there.
    // `d.data` keeps the value the gesture started from: later builds have moved `g.value`.
    let at = move |d: &Drag| {
        let start = g.knob_x(d.data, d.w);
        let grab = if (d.start_x - start).abs() <= g.knob / 2.0 + 2.0 { d.start_x - start } else { 0.0 };
        g.at(d.x - grab, d.w)
    };
    e.on_step(step).on_step_limit(limit).on_drag(move |s: &mut S, d: &mut Drag, tx: &mut Tx| match d.phase {
        Phase::Down | Phase::Move => {
            if d.phase == Phase::Down {
                d.data = g.value;
            }
            report(s, at(d), tx, true, false);
            tx.instant();
        }
        Phase::Up if d.cancelled() => report(s, d.data, tx, true, false),
        Phase::Up => report(s, at(d), tx, false, true),
    })
}

/// `v` with as many decimals as the step has (none for whole steps, two when continuous).
fn format_number(v: f32, step: f32) -> String {
    let places = if step <= 0.0 {
        2
    } else {
        let mut p = 0;
        while p < 4 && ((step * 10f32.powi(p)).round() - step * 10f32.powi(p)).abs() > 1e-3 {
            p += 1;
        }
        p as usize
    };
    format!("{v:.places$}")
}

#[cfg(test)]
#[path = "slider_tests.rs"]
mod tests;
