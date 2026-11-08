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
