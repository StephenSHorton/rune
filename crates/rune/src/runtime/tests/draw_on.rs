use super::*;
use crate::{Shape, canvas};

/// A 100-point line in 10 segments, drawn on when `.0` mounts it.
fn line_view(s: &(bool, Transition), _: &mut Cx<(bool, Transition)>) -> El {
    let line: Vec<[f32; 2]> = (0..=10).map(|i| [i as f32 * 10.0, 20.0]).collect();
    col().size(200.0, 100.0).children(s.0.then(|| canvas(Shape::new().stroke(line, 2.0, crate::Color::WHITE)).key("line").size(120.0, 40.0).transition(s.1)))
}

fn segments(rt: &Runtime<(bool, Transition)>) -> usize {
    rt.instances.iter().filter(|q| q.params[1] == KIND_SEGMENT).count()
}

#[test]
fn a_draw_transition_traces_a_shape_on_and_reduce_motion_draws_it_whole() {
    let mut rt = Runtime::new((false, Transition::draw()), line_view, (200.0, 100.0), Env::new(true, 1.0));
    rt.frame(0.0);
    rt.update(|s, _| s.0 = true);
    rt.frame(0.0);
    let mut seen = vec![segments(&rt)];
    for _ in 0..90 {
        rt.frame(1.0 / 120.0);
        seen.push(segments(&rt));
    }
    assert!(seen[0] < 3, "starts undrawn: {seen:?}");
    assert!(seen.windows(2).all(|w| w[0] <= w[1]), "draws on, never back: {seen:?}");
    assert!(seen.iter().any(|&n| n > 3 && n < 10), "passes through partly drawn: {seen:?}");
    assert_eq!(*seen.last().unwrap(), 10, "ends whole: {seen:?}");

    let mut rt = Runtime::new((false, Transition::draw()), line_view, (200.0, 100.0), Env::new(true, 1.0));
    rt.set_system_reduced_motion(true);
    rt.frame(0.0);
    rt.update(|s, _| s.0 = true);
    rt.frame(0.0);
    assert_eq!(segments(&rt), 10, "drawn whole the frame it mounts under Reduce Motion");
}

#[test]
fn a_live_trim_cuts_a_shape_every_frame_without_rebuilding() {
    type S = crate::Tween<[f32; 2]>;
    fn view(s: &S, _: &mut Cx<S>) -> El {
        let line: Vec<[f32; 2]> = (0..=10).map(|i| [i as f32 * 10.0, 20.0]).collect();
        col().size(200.0, 100.0).child(canvas(Shape::new().stroke(line, 2.0, crate::Color::WHITE)).key("line").size(120.0, 40.0).live_trim(s))
    }
    let count = |rt: &Runtime<S>| rt.instances.iter().filter(|q| q.params[1] == KIND_SEGMENT).count();
    let tween = crate::Tween::linear([0.0, 1.0], [1.0, 1.0], std::time::Duration::from_secs(1));
    let mut rt = Runtime::new(tween, view, (200.0, 100.0), Env::new(true, 1.0));
    rt.frame(0.0);
    assert_eq!(count(&rt), 10, "starts whole");
    let commits = rt.stats.commits;
    let mut seen = Vec::new();
    for _ in 0..10 {
        assert!(rt.frame(0.1 - 1e-4), "keeps drawing while it runs");
        seen.push(count(&rt));
    }
    assert!(seen.windows(2).all(|w| w[0] >= w[1]), "only shortens: {seen:?}");
    assert!(seen.contains(&5), "half gone at half time: {seen:?}");
    rt.frame(0.01);
    assert_eq!(count(&rt), 0, "gone at the end");
    assert!(!rt.frame(0.1), "settles");
    assert_eq!(rt.stats.commits, commits, "no view rebuild");

    rt.update(|s, _| *s = crate::Tween::linear([0.25, 0.5], [0.25, 0.5], std::time::Duration::ZERO));
    rt.frame(0.0);
    assert_eq!(count(&rt), 3, "a fixed span keeps its stretch, cut mid-segment at both ends");
}

/// The first dash's start along a dashed line, in device px.
fn first_dash(rt: &Runtime<()>) -> f32 {
    rt.instances.iter().filter(|q| q.params[1] == KIND_SEGMENT).map(|q| q.radii[0]).fold(f32::INFINITY, f32::min)
}

fn dashed_edge(_: &(), _: &mut Cx<()>) -> El {
    let edge = Shape::new().stroke(vec![[0.0, 20.0], [100.0, 20.0]], 2.0, crate::Color::WHITE).dashed(6.0, 4.0);
    col().size(200.0, 100.0).child(canvas(edge).key("edge").size(100.0, 40.0).dash_flow(20.0))
}

#[test]
fn a_dashed_stroke_draws_one_segment_per_dash() {
    fn view(_: &(), _: &mut Cx<()>) -> El {
        let edge = Shape::new().stroke(vec![[0.0, 20.0], [100.0, 20.0]], 2.0, crate::Color::WHITE).dashed(6.0, 4.0).dash_offset(2.0);
        col().size(200.0, 100.0).child(canvas(edge).size(100.0, 40.0))
    }
    let mut rt = Runtime::new((), view, (200.0, 100.0), Env::new(true, 1.0));
    rt.frame(0.0);
    let count = rt.instances.iter().filter(|q| q.params[1] == KIND_SEGMENT).count();
    // 100 points of 6 on and 4 off, slid 2 on: dashes start at 2, 12, ... 92.
    assert_eq!(count, 10);
    assert!(!rt.frame(1.0 / 60.0), "a still pattern asks for no frames");
}

#[test]
fn dash_flow_marches_the_dashes_without_rebuilding_and_stands_still_reduced() {
    let mut rt = Runtime::new((), dashed_edge, (200.0, 100.0), Env::new(true, 1.0));
    rt.frame(0.0);
    let commits = rt.stats.commits;
    let start = first_dash(&rt);
    assert!(rt.frame(0.1), "keeps drawing while it flows");
    // 20 points a second: 2 points on after 0.1 s, so a gap's sliver shows before the first dash.
    let moved = first_dash(&rt);
    assert!((moved - start - 2.0).abs() < 0.1, "{start} -> {moved}");
    for _ in 0..30 {
        assert!(rt.frame(1.0 / 60.0));
    }
    assert_eq!(rt.stats.commits, commits, "no view rebuild");

    let mut rt = Runtime::new((), dashed_edge, (200.0, 100.0), Env::new(true, 1.0));
    rt.set_system_reduced_motion(true);
    rt.frame(0.0);
    let still = first_dash(&rt);
    rt.frame(0.25);
    assert_eq!(first_dash(&rt), still, "Reduce Motion holds the dashes still");
}
