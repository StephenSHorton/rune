//! `HoverCard::hover`: the kit opens the card once the pointer rests on the trigger, keeps it
//! open while the pointer crosses into the card, and closes it once the pointer has left both.
use std::time::Duration;

use rune::{Cx, El, Env, Runtime, col, el, text};
use rune_kit::{HoverCard, HoverCardState, Kit, root};

#[derive(Default)]
struct App {
    card: HoverCardState,
}

fn view(s: &App, cx: &mut Cx<App>) -> El {
    let k = Kit::neutral(cx.system_dark());
    let mut layer = Vec::new();
    let card = HoverCard::new("who")
        .trigger(el().key("name").size(80.0, 20.0))
        .hover(&s.card, |s: &mut App| &mut s.card)
        .delays(Duration::from_millis(100), Duration::from_millis(80))
        .child(text("Profile").key("profile"))
        .build(&k, &mut layer);
    let body = col().w_full().h_full().pl(20.0).pt(20.0).child(el().w(80.0).child(card));
    root(body, layer)
}

fn app() -> Runtime<App> {
    let mut rt = Runtime::new(App::default(), view, (400.0, 400.0), Env::new(false, 1.0));
    rt.frame(0.0);
    rt.pointer_move(300.0, 300.0);
    rt.frame(0.0);
    rt
}

fn shown(rt: &Runtime<App>) -> bool {
    rt.elements().iter().any(|e| e.key.as_deref() == Some("who-card") && !e.exiting)
}

fn rest_on_trigger(rt: &mut Runtime<App>) {
    rt.pointer_move(40.0, 30.0);
    rt.frame(0.12);
    rt.frame(0.0);
    rt.settle_all();
    rt.frame(0.0);
}

#[test]
fn it_opens_after_the_pointer_rests_on_the_trigger() {
    let mut rt = app();
    rt.pointer_move(40.0, 30.0);
    rt.frame(0.05);
    assert!(!shown(&rt), "not before the open delay");
    rt.frame(0.06);
    rt.frame(0.0);
    assert!(rt.state.card.is_open());
    assert!(shown(&rt));
}

#[test]
fn a_pass_shorter_than_the_delay_opens_nothing() {
    let mut rt = app();
    rt.pointer_move(40.0, 30.0);
    rt.frame(0.05);
    rt.pointer_move(300.0, 300.0);
    rt.frame(0.3);
    assert!(!rt.state.card.is_open());
}

#[test]
fn it_stays_open_crossing_into_the_card_and_closes_after_leaving_both() {
    let mut rt = app();
    rest_on_trigger(&mut rt);
    let card = rt.elements().into_iter().find(|e| e.key.as_deref() == Some("who-card")).unwrap();
    let [x, y, w, h] = card.rect;
    // Across the gap and into the card, within the close delay.
    rt.pointer_move(x + w / 2.0, y + h / 2.0);
    rt.frame(0.02);
    rt.frame(0.2);
    rt.frame(0.0);
    assert!(rt.state.card.is_open(), "the card holds it open");
    assert!(shown(&rt));
    rt.pointer_move(390.0, 390.0);
    rt.frame(0.05);
    assert!(rt.state.card.is_open(), "within the close delay");
    rt.frame(0.05);
    rt.frame(0.0);
    assert!(!rt.state.card.is_open(), "closed after leaving both");
}

#[test]
fn leaving_the_trigger_alone_closes_it() {
    let mut rt = app();
    rest_on_trigger(&mut rt);
    rt.pointer_move(390.0, 20.0);
    rt.frame(0.1);
    rt.frame(0.0);
    assert!(!rt.state.card.is_open());
}

#[test]
fn close_shuts_it_at_once() {
    let mut s = HoverCardState::default();
    assert!(!s.is_open());
    s.close();
    assert_eq!(s, HoverCardState::default());
}
