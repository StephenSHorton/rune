//! `PopupList::at_caret` and `Popover::at_caret` hang from the focused field's caret, and
//! `PopupList::header` puts a heading above the rows that is never an option.
use rune::runtime::ElementInfo;
use rune::{Cx, El, Env, Ime, Runtime, TextEdit, col, el, text};
use rune_kit::{Kit, Placement, PopupItem, PopupList, Popover, TextInput, root};

struct State {
    draft: TextEdit,
    items: usize,
    popover: bool,
}

fn view(s: &State, _: &mut Cx<State>) -> El {
    let k = Kit::neutral(true);
    let mut layer = Vec::new();
    let field = TextInput::new(&s.draft).on_edit(|s: &mut State| &mut s.draft).build(&k).key("composer").w(260.0);
    if s.popover {
        Popover::new("tip").open(true).at_caret("composer").placement(Placement::BottomStart)
            .on_close(|_: &mut State| {}).child(text("hint")).build(&k, &mut layer);
    } else {
        PopupList::<State>::new("mentions", "composer")
            .at_caret("composer")
            .placement(Placement::BottomStart)
            .header("At most 10 mentions")
            .items((0..s.items).map(|i| PopupItem::new(format!("p{i}"), format!("Person {i}"))))
            .active(Some(0))
            .build(&k, &mut layer);
    }
    root(col().w_full().h_full().padding(40.0).child(el().h(20.0)).child(field), layer)
}

fn find(rt: &Runtime<State>, key: &str) -> Option<ElementInfo> {
    rt.elements().into_iter().find(|e| e.key.as_deref() == Some(key))
}

fn caret(rt: &Runtime<State>) -> [f32; 4] {
    rt.elements().into_iter().find_map(|e| e.caret).expect("the focused field's caret")
}

fn settle(rt: &mut Runtime<State>) {
    for _ in 0..40 { rt.frame(1.0 / 60.0); }
}

fn runtime(popover: bool, items: usize) -> Runtime<State> {
    let mut draft = TextEdit::new("Hello @an");
    draft.focused = true;
    let mut rt = Runtime::new(State { draft, items, popover }, view, (600.0, 400.0), Env::new(true, 2.0));
    settle(&mut rt);
    rt
}

#[test]
fn the_popup_list_hangs_under_the_caret_and_follows_typing() {
    let mut rt = runtime(false, 3);
    let field = find(&rt, "composer").unwrap().rect;
    for step in ["start", "typed"] {
        if step == "typed" {
            rt.ime(Ime::Commit("na and more".into()));
            settle(&mut rt);
        }
        let c = caret(&rt);
        let list = find(&rt, "mentions-content").unwrap().target;
        assert!((list[0] - c[0]).abs() < 0.01, "{step}: starts at the caret's x, not the field's: {list:?} {c:?} {field:?}");
        assert!((list[1] - (c[1] + c[3] + 4.0)).abs() < 0.01, "{step}: under the caret line: {list:?} {c:?}");
        assert!(c[0] > field[0] + 30.0, "{step}: the caret is well inside the field");
    }
}

#[test]
fn the_header_sits_above_the_rows_and_is_not_an_option() {
    let rt = runtime(false, 3);
    let header = find(&rt, "header").expect("the header row");
    let first = find(&rt, "mentions-option-p0").unwrap();
    assert!(header.rect[1] + header.rect[3] <= first.rect[1] + 0.01, "above the first row");
    let all = rt.elements();
    let heading = all.iter().find(|e| e.text.as_deref() == Some("At most 10 mentions")).expect("its text");
    assert_eq!(heading.role, "text");
    let options: Vec<_> = all.iter().filter(|e| e.role == "option").collect();
    assert_eq!(options.len(), 3, "the header is not counted among the options");

    let empty = runtime(false, 0);
    assert!(find(&empty, "header").is_some(), "it stays above the empty message");
}

#[test]
fn a_popover_opens_at_the_caret() {
    let rt = runtime(true, 0);
    let c = caret(&rt);
    let content = find(&rt, "tip-content").unwrap().target;
    assert!((content[0] - c[0]).abs() < 0.01 && (content[1] - (c[1] + c[3] + 4.0)).abs() < 0.01, "{content:?} under {c:?}");
}
