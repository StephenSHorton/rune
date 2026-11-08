//! `El::anchored_to_caret`: an overlay hangs from the focused field's caret line in the frame
//! the caret moves, flips when it doesn't fit, and falls back to the field's box without focus.
use rune::runtime::ElementInfo;
use rune::{Align, Cx, El, Env, Ime, Runtime, ScrollSpec, Side, TextEdit, TextStyle, col, el, text_input};

struct S {
    draft: TextEdit,
    /// Space above the field, to push it toward the bottom of the window.
    above: f32,
}

fn view(s: &S, _: &mut Cx<S>) -> El {
    let field = col().key("composer").w_full().padding(6.0)
        .child(text_input(&s.draft, TextStyle::new(14.0, 400.0)).key("input").w_full().on_edit(|s: &mut S| &mut s.draft));
    col().w_full().h_full()
        .child(col().key("scroll").w_full().h(240.0).scroll(ScrollSpec { stick_bottom: false })
            .child(col().w_full().child(el().h(s.above)).child(field).child(el().h(400.0))))
        // The picker is the view's last child: it never reads the caret itself.
        .child(el().key("picker").size(80.0, 30.0).anchored_to_caret("composer", Side::Bottom, Align::Start, 4.0))
}

fn find(rt: &Runtime<S>, key: &str) -> ElementInfo {
    rt.elements().into_iter().find(|e| e.key.as_deref() == Some(key)).unwrap_or_else(|| panic!("{key}"))
}

/// The picker's laid-out and drawn top-left, after a frame.
fn picker(rt: &mut Runtime<S>) -> ([f32; 2], [f32; 2]) {
    rt.frame(1.0 / 60.0);
    let p = find(rt, "picker");
    ([p.target[0], p.target[1]], [p.rect[0], p.rect[1]])
}

fn caret(rt: &Runtime<S>) -> [f32; 4] {
    find(rt, "input").caret.expect("the focused field's caret")
}

fn close(a: [f32; 2], b: [f32; 2]) -> bool {
    (a[0] - b[0]).abs() < 0.01 && (a[1] - b[1]).abs() < 0.01
}

#[test]
fn hangs_under_the_caret_through_typing_scrolling_and_composition() {
    let mut draft = TextEdit::new("hi");
    draft.focused = true;
    let mut rt = Runtime::new(S { draft, above: 40.0 }, view, (300.0, 300.0), Env::new(true, 2.0));
    rt.frame(0.0);
    let mut last = 0.0;
    for step in ["first frame", "typing", "scrolling", "composition"] {
        match step {
            "typing" => rt.ime(Ime::Commit(" there".into())),
            "scrolling" => rt.wheel(150.0, 60.0, -20.0),
            "composition" => rt.ime(Ime::Preedit("あい".into(), Some((3, 3)))),
            _ => {}
        }
        let (target, drawn) = picker(&mut rt);
        let c = caret(&rt);
        let under = [c[0], c[1] + c[3] + 4.0];
        assert!(close(target, under), "{step}: laid out under the caret: {target:?} vs {under:?}");
        assert!(close(drawn, under), "{step}: drawn under the caret: {drawn:?} vs {under:?}");
        if step == "typing" { assert!(c[0] > last + 20.0, "typing moves it right"); }
        last = c[0];
    }
}

#[test]
fn flips_above_the_caret_near_the_bottom_and_falls_back_without_focus() {
    let mut draft = TextEdit::new("hello");
    draft.focused = true;
    let mut rt = Runtime::new(S { draft, above: 205.0 }, view, (300.0, 260.0), Env::new(true, 2.0));
    rt.frame(0.0);
    let (target, _) = picker(&mut rt);
    let c = caret(&rt);
    assert!(close(target, [c[0], c[1] - 4.0 - 30.0]), "above the caret line: {target:?} vs caret {c:?}");

    rt.update(|s, _| s.draft.focused = false);
    for _ in 0..30 { rt.frame(1.0 / 60.0); }
    let (target, _) = picker(&mut rt);
    let field = find(&rt, "composer").target;
    // The field's box, not its caret line, is what it now flips above (and 8 points in).
    assert!(close(target, [field[0].max(8.0), field[1] - 4.0 - 30.0]), "beside the field once it has no caret: {target:?} vs {field:?}");
}
