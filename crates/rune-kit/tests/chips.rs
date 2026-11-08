//! Badge chips (pill, height, padding, border, truncation, plain text), IconButton glyph sizes
//! and ghost role colours, and an opener ColorSwatch.
use rune::{Cx, El, Env, Runtime, col, row};
use rune_kit::*;

struct Case(u32);

const LONG: &str = "A rather long chip label that cannot fit";

fn view(s: &Case, cx: &mut Cx<Case>) -> El {
    let k = Kit::neutral(cx.system_dark());
    // Each sits in a row so a column doesn't stretch it to its width.
    let body = match s.0 {
        0 => col()
            .child(row().child(Badge::new("Done").build(&k).key("default")))
            .child(row().child(
                Badge::new("Owner")
                    .variant(BadgeVariant::Success)
                    .pill()
                    .height(20.0)
                    .padding(8.0)
                    .border_width(1.0)
                    .plain()
                    .build(&k)
                    .key("chip"),
            ))
            .child(row().child(Badge::new(LONG).max_width(80.0).build(&k).key("long"))),
        _ => unreachable!(),
    };
    col().w_full().h_full().child(body)
}

fn run(case: u32) -> Runtime<Case> {
    let mut rt = Runtime::new(Case(case), view, (600.0, 400.0), Env::new(false, 1.0));
    rt.frame(0.0);
    rt.settle_all();
    rt.frame(0.0);
    rt
}

fn info(rt: &Runtime<Case>, key: &str) -> rune::ElementInfo {
    rt.elements().into_iter().find(|e| e.key.as_deref() == Some(key)).unwrap_or_else(|| panic!("no {key}"))
}

/// The first child of the element keyed `key`.
fn child(rt: &Runtime<Case>, key: &str) -> rune::ElementInfo {
    let all = rt.elements();
    let i = all.iter().position(|e| e.key.as_deref() == Some(key)).unwrap_or_else(|| panic!("no {key}"));
    all.into_iter().find(|e| e.parent == Some(i)).unwrap_or_else(|| panic!("{key} has no child"))
}

#[test]
fn a_default_badge_is_a_small_status() {
    let rt = run(0);
    let b = info(&rt, "default");
    assert_eq!((b.role, b.label.as_deref()), ("status", Some("Done")));
    assert_eq!(b.rect[3], 16.0);
    assert_eq!(b.radius, 2.0);
}

#[test]
fn a_chip_is_a_plain_pill_of_its_own_height() {
    let k = Kit::neutral(false);
    let rt = run(0);
    let chip = info(&rt, "chip");
    assert_eq!(chip.rect[3], 20.0);
    assert_eq!(chip.radius, 10.0, "fully rounded");
    assert_ne!(chip.role, "status");
    assert_eq!(chip.label, None, "the text speaks for itself");
    assert_eq!(chip.fill, k.c.success.bg);
    let text = child(&rt, "chip");
    assert_eq!(text.text.as_deref(), Some("Owner"));
    assert_eq!(text.rect[0] - chip.rect[0], 8.0, "its own padding");
}

#[test]
fn a_long_badge_is_cut_short_with_its_text_as_a_tooltip() {
    let rt = run(0);
    let long = info(&rt, "long");
    assert!(long.rect[2] <= 80.0, "held to its max width: {}", long.rect[2]);
    assert_eq!(long.label.as_deref(), Some(LONG), "assistive technology still reads all of it");
    assert_eq!(child(&rt, "long").help.as_deref(), Some(LONG));
}
