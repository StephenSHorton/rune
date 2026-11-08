use super::*;

fn cut(points: &[[f32; 2]], closed: bool, dash: Dash, k: f32, extra: f32) -> Vec<([f32; 2], [f32; 2])> {
    let mut out = Vec::new();
    assert!(pieces(points.len(), |i| points[i], closed, dash, k, extra, |a, b| out.push((a, b))));
    out
}

fn close(a: [f32; 2], b: [f32; 2]) -> bool {
    (a[0] - b[0]).abs() < 1e-3 && (a[1] - b[1]).abs() < 1e-3
}

#[test]
fn a_line_cuts_into_dashes_and_gaps_from_its_start() {
    let d = Dash::new(4.0, 2.0).unwrap();
    let got = cut(&[[0.0, 0.0], [20.0, 0.0]], false, d, 1.0, 0.0);
    let want = [([0.0, 0.0], [4.0, 0.0]), ([6.0, 0.0], [10.0, 0.0]), ([12.0, 0.0], [16.0, 0.0]), ([18.0, 0.0], [20.0, 0.0])];
    assert_eq!(got.len(), want.len(), "{got:?}");
    assert!(got.iter().zip(&want).all(|(g, w)| close(g.0, w.0) && close(g.1, w.1)), "{got:?}");
}

#[test]
fn an_offset_slides_the_pattern_forward_and_wraps_after_a_period() {
    let mut d = Dash::new(4.0, 2.0).unwrap();
    d.offset = 1.0;
    let got = cut(&[[0.0, 0.0], [12.0, 0.0]], false, d, 1.0, 0.0);
    // The first dash has moved 1 point on: it starts at 1, and 1 point of the previous one's
    // gap shows before it.
    assert!(close(got[0].0, [1.0, 0.0]) && close(got[0].1, [5.0, 0.0]), "{got:?}");
    // A whole period on is the same pattern; `extra` adds to the offset.
    d.offset = 7.0;
    assert_eq!(cut(&[[0.0, 0.0], [12.0, 0.0]], false, d, 1.0, -6.0), got);
    // Backwards: the start sits 1 point into the first dash.
    d.offset = -1.0;
    let back = cut(&[[0.0, 0.0], [12.0, 0.0]], false, d, 1.0, 0.0);
    assert!(close(back[0].0, [0.0, 0.0]) && close(back[0].1, [3.0, 0.0]), "{back:?}");
}

#[test]
fn a_dash_turns_corners_and_a_closed_stroke_runs_round_to_its_start() {
    let d = Dash::new(6.0, 4.0).unwrap();
    let got = cut(&[[0.0, 0.0], [4.0, 0.0], [4.0, 10.0]], false, d, 1.0, 0.0);
    // 4 points along the first leg and 2 down the second make the first dash.
    assert!(close(got[0].1, [4.0, 0.0]) && close(got[1].0, [4.0, 0.0]) && close(got[1].1, [4.0, 2.0]), "{got:?}");
    let square = [[0.0, 0.0], [10.0, 0.0], [10.0, 10.0], [0.0, 10.0]];
    let ring = cut(&square, true, d, 1.0, 0.0);
    let drawn: f32 = ring.iter().map(|(a, b)| dist(*a, *b)).sum();
    assert!((drawn - 24.0).abs() < 1e-3, "four dashes of 6 in 40 points: {drawn}");
}

#[test]
fn lengths_scale_with_the_device_scale() {
    let d = Dash::new(4.0, 2.0).unwrap();
    let got = cut(&[[0.0, 0.0], [24.0, 0.0]], false, d, 2.0, 0.0);
    assert_eq!(got.len(), 2);
    assert!(close(got[0].1, [8.0, 0.0]) && close(got[1].0, [12.0, 0.0]), "{got:?}");
}

#[test]
fn zero_length_dashes_are_dots_and_degenerate_patterns_are_refused() {
    let d = Dash::new(0.0, 5.0).unwrap();
    let dots = cut(&[[0.0, 0.0], [12.0, 0.0]], false, d, 1.0, 0.0);
    assert_eq!(dots.len(), 3, "{dots:?}");
    assert!(dots.iter().all(|(a, b)| a == b));
    assert!(Dash::new(4.0, 0.0).is_none(), "no gap is a solid line");
    assert!(Dash::new(-1.0, 2.0).is_none());
    assert!(Dash::new(f32::NAN, 2.0).is_none());
    let fine = Dash::new(0.01, 0.01).unwrap();
    assert!(!pieces(2, |i| [i as f32 * 100.0, 0.0], false, fine, 1.0, 0.0, |_, _| {}), "too fine to cut: drawn solid");
    let huge = Dash::new(0.5, 0.5).unwrap();
    assert!(!pieces(2, |i| [i as f32 * 1.0e6, 0.0], false, huge, 1.0, 0.0, |_, _| {}), "too many pieces: drawn solid");
}

/// A small seeded generator, so the fuzz cases are the same on every run.
struct Lcg(u64);

impl Lcg {
    fn unit(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.0 >> 40) as f32 / (1u64 << 24) as f32
    }

    fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.unit()
    }
}

/// Cuts the polyline and checks the dashes: each fits in one dash, none is a zero-length sliver
/// (unless the pattern is dots), and together they cover what the pattern says, worked out in
/// f64 from where the pattern starts.
fn check_cover(points: &[[f32; 2]], closed: bool, dash: Dash, k: f32, extra: f32, label: &str) {
    let mut got = Vec::new();
    assert!(pieces(points.len(), |i| points[i], closed, dash, k, extra, |a, b| got.push((a, b))), "{label}: refused");
    let (on, period) = ((dash.on * k) as f64, (dash.period() * k) as f64);
    let count = if closed { points.len() } else { points.len() - 1 };
    let length: f64 = (0..count).map(|i| dist(points[i], points[(i + 1) % points.len()]) as f64).sum();
    let start = (-(dash.offset + extra) * k).rem_euclid(dash.period() * k) as f64;
    let covered = |x: f64| (x / period).floor() * on + (x.rem_euclid(period)).min(on);
    let want = covered(start + length) - covered(start);
    let drawn: f64 = got.iter().map(|(a, b)| dist(*a, *b) as f64).sum();
    // f32 positions along a segment are good to about an ulp of its length per cut.
    let tol = 0.05 + got.len() as f64 * length.max(1.0) * 4.0e-7 + length * 1.0e-4;
    assert!((drawn - want).abs() <= tol, "{label}: drew {drawn}, pattern covers {want} (tol {tol}, {} pieces)", got.len());
    for (a, b) in &got {
        let l = dist(*a, *b) as f64;
        assert!(l <= on * (1.0 + 1e-4) + 1e-2, "{label}: a piece of {l} is longer than a dash of {on}");
        assert!(dash.on == 0.0 || l > 0.0, "{label}: a zero-length piece");
    }
}

#[test]
fn random_fractional_polylines_always_finish_and_cover_the_pattern() {
    let mut rng = Lcg(0x5eed_da54);
    let patterns: [(f32, f32); 8] = [(5.0, 5.0), (4.0, 4.0), (6.0, 4.0), (3.0, 2.0), (0.0, 4.0), (1.0, 7.5), (8.0, 3.0), (2.5, 2.5)];
    for case in 0..4000 {
        let n = 2 + (rng.unit() * 30.0) as usize;
        let mut p = [rng.range(0.0, 400.0), rng.range(0.0, 400.0)];
        let mut points = vec![p];
        for _ in 1..n {
            // Mostly short steps, like a curve flattened into a polyline, with some long runs.
            let reach = if rng.unit() < 0.8 { 8.0 } else { 160.0 };
            p = [p[0] + rng.range(-reach, reach), p[1] + rng.range(-reach, reach)];
            points.push(p);
        }
        let (on, off) = if rng.unit() < 0.5 { patterns[case % patterns.len()] } else { (rng.range(0.0, 12.0), rng.range(0.3, 12.0)) };
        let mut dash = Dash::new(on, off).unwrap();
        dash.offset = rng.range(-50.0, 50.0);
        let k = [1.0, 2.0, 1.5, 3.0, 1.25][case % 5];
        let extra = rng.range(-30.0, 30.0);
        check_cover(&points, case % 7 == 0, dash, k, extra, &format!("case {case} ({on}/{off} k{k})"));
    }
}

#[test]
fn long_lines_finish_far_from_their_start() {
    let mut rng = Lcg(77);
    for case in 0..60 {
        // One straight run of up to 1e5 px, then a few fractional legs out at that distance,
        // where an f32 ulp is about 0.008 px.
        let far = rng.range(1.0e4, 1.0e5);
        let mut points = vec![[rng.range(0.0, 1.0), rng.range(0.0, 1.0)], [far, rng.range(0.0, 3.0)]];
        for _ in 0..6 {
            let last = *points.last().unwrap();
            points.push([last[0] + rng.range(-7.0, 7.0), last[1] + rng.range(-7.0, 7.0)]);
        }
        let (on, off) = [(5.0, 5.0), (6.0, 4.0), (4.0, 4.0), (0.0, 6.0)][case % 4];
        let mut dash = Dash::new(on, off).unwrap();
        dash.offset = rng.range(0.0, 10.0);
        check_cover(&points, false, dash, 1.0, rng.range(0.0, 3.0), &format!("long {case} ({far} px, {on}/{off})"));
    }
}
