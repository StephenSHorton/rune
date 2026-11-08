//! `PointerKind`: a press, drag or pointer move says whether a mouse or a finger drove it, so an
//! app can open on a tap what a mouse opens on hover.
use rune::{Cx, El, Env, PointerKind, Runtime, TouchPhase, col, el};

#[derive(Default)]
struct Heard { presses: Vec<PointerKind>, moves: Vec<PointerKind>, drags: Vec<PointerKind> }

fn view(_: &Heard, _: &mut Cx<Heard>) -> El {
    col().w_full().h_full()
        .child(el().key("button").w(40.0).h(40.0)
            .on_press_tx(|s: &mut Heard, tx| s.presses.push(tx.pointer_kind()))
            .on_pointer_move_local(|s: &mut Heard, e, _| s.moves.push(e.kind)))
        .child(el().key("knob").w(40.0).h(40.0)
            .on_drag(|s: &mut Heard, d, tx| { assert_eq!(tx.pointer_kind(), d.pointer_kind()); s.drags.push(d.pointer_kind()) }))
}

fn app() -> Runtime<Heard> {
    let mut rt = Runtime::new(Heard::default(), view, (100.0, 100.0), Env::new(true, 1.0));
    rt.frame(0.0);
    rt
}

#[test]
fn a_click_is_a_mouse_and_a_tap_is_a_touch() {
    let mut rt = app();
    rt.pointer_move(20.0, 20.0);
    rt.pointer_down(20.0, 20.0);
    rt.pointer_up(20.0, 20.0);
    rt.frame(0.0);
    rt.touch(1, TouchPhase::Started, 21.0, 21.0, 1.0);
    rt.touch(1, TouchPhase::Ended, 21.0, 21.0, 1.05);
    rt.frame(0.0);
    // A mouse click after the tap is a mouse again.
    rt.pointer_move(20.0, 20.0);
    rt.pointer_down(20.0, 20.0);
    rt.pointer_up(20.0, 20.0);
    rt.frame(0.0);
    assert_eq!(rt.state.presses, [PointerKind::Mouse, PointerKind::Touch, PointerKind::Mouse]);
    assert_eq!(rt.state.moves.first(), Some(&PointerKind::Mouse));
    assert!(rt.state.moves.contains(&PointerKind::Touch), "the finger landing moves the pointer: {:?}", rt.state.moves);
}

#[test]
fn a_finger_drag_is_a_touch_on_every_phase() {
    let mut rt = app();
    rt.touch(7, TouchPhase::Started, 20.0, 60.0, 1.0);
    rt.touch(7, TouchPhase::Moved, 34.0, 60.0, 1.02);
    rt.touch(7, TouchPhase::Ended, 34.0, 60.0, 1.04);
    rt.frame(0.0);
    assert!(rt.state.drags.len() >= 2, "{:?}", rt.state.drags);
    assert!(rt.state.drags.iter().all(|k| *k == PointerKind::Touch), "{:?}", rt.state.drags);
    rt.pointer_move(20.0, 60.0);
    rt.pointer_down(20.0, 60.0);
    rt.pointer_up(30.0, 60.0);
    rt.frame(0.0);
    assert_eq!(rt.state.drags.last(), Some(&PointerKind::Mouse));
}
