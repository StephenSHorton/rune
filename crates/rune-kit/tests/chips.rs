//! Badge chips (pill, height, padding, border, truncation, plain text), IconButton glyph sizes
//! and ghost role colours, and an opener ColorSwatch.
use rune::{Color, Cx, El, Env, Runtime, col, row};
use rune_kit::*;

struct Case(u32);

const LONG: &str = "A rather long chip label that cannot fit";

fn view(s: &Case, cx: &mut Cx<Case>) -> El {
    let k = Kit::neutral(cx.system_dark());
    // Each sits in a row so a column doesn't stretch it to its width.
    let body = match s.0 {
        0 => col()
            .child(row().child(Badge::new("Done").build(&k).key("default")))
            .child(
                row().child(
                    Badge::new("Owner")
                        .variant(BadgeVariant::Success)
                        .pill()
                        .height(20.0)
                        .padding(8.0)
                        .border_width(1.0)
                        .plain()
                        .build(&k)
                        .key("chip"),
                ),
            )
            .child(row().child(Badge::new(LONG).max_width(80.0).build(&k).key("long"))),
        1 => col()
            .child(row().child(IconButton::new(k.glyph.close).label("Close").build(&k).key("regular")))
            .child(row().child(IconButton::new(k.glyph.close).label("Mute").glyph_size(14.0).build(&k).key("small")))
            .child(
                row().child(
                    IconButton::new(k.glyph.close)
                        .label("Hang up")
                        .variant(IconButtonVariant::GhostDanger)
                        .build(&k)
                        .key("hang"),
                ),
            )
            .child(
                row().child(
                    IconButton::new(k.glyph.check)
                        .label("Answer")
                        .variant(IconButtonVariant::GhostSuccess)
                        .build(&k)
                        .key("answer"),
                ),
            )
            .child(
                row().child(
                    ColorSwatch::new(Color::WHITE).opener(true).on_press(|_: &mut Case| {}).build(&k).key("opener"),
                ),
            )
            .child(row().child(ColorSwatch::new(Color::WHITE).on_press(|_: &mut Case| {}).build(&k).key("radio"))),
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

#[test]
fn an_icon_buttons_glyph_takes_its_own_size() {
    let rt = run(1);
    assert_eq!(child(&rt, "regular").rect[2], 16.0);
    assert_eq!(child(&rt, "small").rect[2], 14.0);
    assert_eq!(info(&rt, "small").rect[2], 24.0, "the button keeps its size");
}

#[test]
fn ghost_role_buttons_ink_their_icon_and_rest_clear() {
    let k = Kit::neutral(false);
    let rt = run(1);
    for (key, ink) in [("hang", k.c.danger.fg), ("answer", k.c.success.fg)] {
        assert_eq!(info(&rt, key).fill, Color::TRANSPARENT, "{key} has no fill at rest");
        assert_eq!(child(&rt, key).text_color, Some(ink), "{key} ink");
    }
}

#[test]
fn an_opener_swatch_is_a_button_not_a_radio() {
    let rt = run(1);
    let opener = info(&rt, "opener");
    assert_eq!((opener.role, opener.expanded), ("button", Some(true)));
    assert_eq!(info(&rt, "radio").role, "radio");
    let tree = rt.accessibility_json().to_string();
    let at = tree.find("\"opener\"").expect("the opener in the tree");
    let node = &tree[tree[..at].rfind('{').unwrap()..];
    let node = &node[..node.find('}').unwrap()];
    assert!(!node.contains("checked"), "no checked state: {node}");
}
