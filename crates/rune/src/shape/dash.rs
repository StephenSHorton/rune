//! Dashed strokes: a pattern of dash and gap lengths, and an offset that slides it along the
//! stroke (`Shape::dashed`, `Shape::dash_offset`, `El::dash_flow`). The pattern is cut on the
//! CPU at paint time into one segment per dash piece, so a dashed stroke draws in the same pass
//! as a solid one and an offset that moves every frame rebuilds nothing.

/// A stroke's dash pattern, in points along the stroke.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Dash {
    /// The length of each dash. Zero with round caps draws dots.
    pub(crate) on: f32,
    /// The gap after each dash.
    pub(crate) off: f32,
    /// How far the pattern has slid forward along the stroke, from its start toward its end.
    pub(crate) offset: f32,
}

impl Dash {
    /// A pattern, or `None` when it would draw nothing different from a solid line (no gap)
    /// or isn't a length.
    pub(crate) fn new(on: f32, off: f32) -> Option<Dash> {
        (on.is_finite() && off.is_finite() && on >= 0.0 && off > 0.0).then_some(Dash { on, off, offset: 0.0 })
    }

    /// One dash and one gap.
    pub(crate) fn period(&self) -> f32 {
        self.on + self.off
    }
}

/// The most pieces one dashed stroke draws; a finer pattern draws solid instead of flooding
/// the frame with instances.
const MAX_PIECES: usize = 20_000;

/// Calls `piece(a, b)` for every dash of the polyline through `point(0..n)` (device pixels;
/// `closed` joins the last point back to the first), with the pattern scaled by `k` (device
/// pixels per point) and slid on by `extra` points beyond its own offset. A dash that turns a
/// corner is one piece per straight run. Returns false, drawing nothing, when the pattern is too
/// fine to cut, so the caller draws the stroke solid. It allocates nothing.
pub(crate) fn pieces(n: usize, point: impl Fn(usize) -> [f32; 2], closed: bool, dash: Dash, k: f32, extra: f32, mut piece: impl FnMut([f32; 2], [f32; 2])) -> bool {
    let (on, period) = (dash.on * k, dash.period() * k);
    let count = if closed { n } else { n.saturating_sub(1) };
    let length: f32 = (0..count).map(|i| dist(point(i), point((i + 1) % n))).sum();
    if !(period > 0.25) || length / period > MAX_PIECES as f32 {
        return false;
    }
    // Where in the pattern the stroke starts: sliding the pattern forward by `offset` puts the
    // start that far back in it.
    let mut at = (-(dash.offset + extra) * k).rem_euclid(period);
    if !at.is_finite() {
        at = 0.0;
    }
    // Each step below either reaches the end of a segment or lands `at` exactly on the next
    // pattern boundary (the end of the dash, or the end of the period, which wraps to 0). So the
    // pattern state always moves on, even where `t + remaining` rounds back to `t` in f32, and
    // two boundaries in a row move `t` by about a period, which is far above an ulp of any `t`
    // the piece cap allows (`length <= MAX_PIECES * period`). `steps` is a backstop only.
    let snap = period * 1e-5;
    let mut steps = 4 * (MAX_PIECES + count) + 16;
    for i in 0..count {
        let (a, b) = (point(i), point((i + 1) % n));
        let len = dist(a, b);
        if !(len > 0.0) {
            continue;
        }
        let along = |t: f32| {
            let u = t / len;
            [a[0] + (b[0] - a[0]) * u, a[1] + (b[1] - a[1]) * u]
        };
        let mut t = 0.0;
        while t < len {
            steps = match steps.checked_sub(1) {
                Some(s) => s,
                None => return true,
            };
            let drawing = at < on;
            // A zero-length dash is a dot where the pattern starts over.
            if !drawing && on == 0.0 && at == 0.0 {
                piece(along(t), along(t));
            }
            let boundary = if drawing { on } else { period };
            let want = t + (boundary - at);
            let end = want.min(len);
            // `end` can round back to `t` when the dash has less than half an ulp of `t` left;
            // that sliver draws nothing.
            if drawing && end > t {
                piece(along(t), along(end));
            }
            if want < len {
                at = boundary;
            } else {
                at = (at + (len - t)).min(boundary);
                if boundary - at <= snap {
                    at = boundary;
                }
            }
            if at >= period {
                at = 0.0;
            }
            t = end;
        }
    }
    true
}

fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
    ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt()
}

#[cfg(test)]
#[path = "dash_tests.rs"]
mod tests;
