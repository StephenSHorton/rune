//! `Slider`: drag, track press, keys, cancel, commit, disabled, and what it reports.
use rune::{Cx, El, Env, Key, KeyEvent, Mods, Runtime, col};
use rune_kit::{Kit, Slider};

#[derive(Default)]
struct App {
    v: f32,
    commits: Vec<f32>,
    disabled: bool,
}

fn view(s: &App, cx: &mut Cx<App>) -> El {
    let k = Kit::neutral(cx.system_dark());
    // 168 wide: the 16 point knob's centre travels 8..160, so x = 8 + 152 × value.
    let slider = Slider::new(s.v)
        .range(0.0, 1.0)
        .step(0.05)
        .width(168.0)
        .label("Volume")
        .disabled(s.disabled)
        .on_change(|s: &mut App, v, _| s.v = v)
        .on_commit(|s: &mut App, v, _| s.commits.push(v))
        .build(&k)
        .key("volume");
    col().w_full().h_full().pl(20.0).pt(20.0).child(slider)
}

fn app(v: f32) -> Runtime<App> {
    let mut rt = Runtime::new(App { v, ..App::default() }, view, (300.0, 100.0), Env::new(false, 1.0));
    rt.frame(0.0);
    rt
}

/// Window x for a value: the slider starts at 20.
fn x(v: f32) -> f32 {
    20.0 + 8.0 + 152.0 * v
}

fn key(rt: &mut Runtime<App>, k: Key) {
    rt.key(KeyEvent { key: k, mods: Mods::default() });
    rt.frame(0.0);
}

#[test]
fn dragging_the_knob_moves_the_value_and_commits_on_release() {
    let mut rt = app(0.5);
    rt.pointer_down(x(0.5) + 3.0, 30.0);
    rt.frame(0.0);
    assert_eq!(rt.state.v, 0.5, "grabbing the knob off-centre doesn't jump it");
    rt.pointer_move(x(0.75) + 3.0, 30.0);
    rt.frame(0.0);
    assert!((rt.state.v - 0.75).abs() < 1e-6, "{}", rt.state.v);
    rt.pointer_move(x(0.9) + 3.0, 30.0);
    rt.frame(0.0);
    rt.pointer_up(x(0.9) + 3.0, 30.0);
    rt.frame(0.0);
    assert!((rt.state.v - 0.9).abs() < 1e-6, "{}", rt.state.v);
    assert_eq!(rt.state.commits.len(), 1, "one commit per gesture");
    assert!((rt.state.commits[0] - 0.9).abs() < 1e-6);
}

#[test]
fn a_track_press_jumps_there_snapped_to_the_step() {
    let mut rt = app(0.0);
    rt.pointer_down(x(0.62), 30.0);
    rt.frame(0.0);
    assert!((rt.state.v - 0.6).abs() < 1e-6, "{}", rt.state.v);
    rt.pointer_up(x(0.62), 30.0);
    rt.frame(0.0);
    let (kx, _) = rt.find("volume/knob").unwrap();
    assert!((kx - x(0.6)).abs() < 0.5, "the knob's centre follows: {kx}");
}

#[test]
fn values_stay_in_the_range() {
    let mut rt = app(0.5);
    rt.pointer_down(x(0.5), 30.0);
    rt.frame(0.0);
    rt.pointer_move(400.0, 30.0);
    rt.frame(0.0);
    assert_eq!(rt.state.v, 1.0);
    rt.pointer_move(-50.0, 30.0);
    rt.frame(0.0);
    assert_eq!(rt.state.v, 0.0);
}

#[test]
fn keys_step_the_value() {
    let mut rt = app(0.5);
    key(&mut rt, Key::Tab);
    key(&mut rt, Key::Right);
    assert!((rt.state.v - 0.55).abs() < 1e-6, "{}", rt.state.v);
    key(&mut rt, Key::Left);
    key(&mut rt, Key::Left);
    assert!((rt.state.v - 0.45).abs() < 1e-6, "{}", rt.state.v);
    key(&mut rt, Key::End);
    assert_eq!(rt.state.v, 1.0);
    key(&mut rt, Key::Home);
    assert_eq!(rt.state.v, 0.0);
    assert_eq!(rt.state.commits.len(), 5, "each key step commits");
}

#[test]
fn it_reads_as_a_slider_with_its_value() {
    let rt = app(0.4);
    let e = rt.elements().into_iter().find(|e| e.key.as_deref() == Some("volume")).unwrap();
    assert_eq!(e.role, "slider");
    assert_eq!(e.label.as_deref(), Some("Volume"));
    assert!(e.focusable);
    let tree = rt.accessibility_json().to_string();
    assert!(tree.contains("40%"), "{tree}");
}

#[test]
fn a_disabled_slider_takes_no_input() {
    let mut rt = Runtime::new(App { v: 0.5, disabled: true, ..App::default() }, view, (300.0, 100.0), Env::new(false, 1.0));
    rt.frame(0.0);
    rt.pointer_down(x(0.9), 30.0);
    rt.frame(0.0);
    rt.pointer_up(x(0.9), 30.0);
    rt.frame(0.0);
    key(&mut rt, Key::Tab);
    key(&mut rt, Key::Right);
    assert_eq!(rt.state.v, 0.5);
    assert!(rt.state.commits.is_empty());
}

#[test]
fn a_cancelled_drag_puts_the_value_back() {
    let mut rt = app(0.5);
    rt.pointer_down(x(0.5), 30.0);
    rt.frame(0.0);
    rt.pointer_move(x(0.8), 30.0);
    rt.frame(0.0);
    assert!((rt.state.v - 0.8).abs() < 1e-6);
    key(&mut rt, Key::Escape);
    rt.pointer_up(x(0.8), 30.0);
    rt.frame(0.0);
    assert_eq!(rt.state.v, 0.5);
    assert!(rt.state.commits.is_empty(), "a cancel doesn't commit");
}
