//! State badges, coloured buttons, and the widths of a select's listbox and a popover.
use rune::{Color, Cx, El, Env, Runtime, col, el, text};
use rune_kit::*;

struct Case(u32);

fn view(s: &Case, cx: &mut Cx<Case>) -> El {
    let k = Kit::neutral(cx.system_dark());
    let mut layer = Vec::new();
    let body = match s.0 {
        0 => col()
            .child(Badge::new("Done").variant(BadgeVariant::Success).build(&k).key("success"))
            .child(Badge::new("Stale").variant(BadgeVariant::Warning).build(&k).key("warning"))
            .child(Badge::new("Failed").variant(BadgeVariant::Danger).build(&k).key("danger"))
            .child(Badge::new("New").variant(BadgeVariant::Info).build(&k).key("info")),
        1 => col()
            .child(Button::new("Join").tint(GREEN, Color::WHITE).build(&k).key("join"))
            .child(IconButton::new(k.glyph.close).label("Leave").tint(RED, Color::WHITE).build(&k).key("leave"))
            .child(
                IconButton::new(k.glyph.close)
                    .label("Delete")
                    .variant(IconButtonVariant::Danger)
                    .build(&k)
                    .key("delete"),
            )
            .child(
                IconButton::new(k.glyph.check)
                    .label("Accept")
                    .variant(IconButtonVariant::Success)
                    .build(&k)
                    .key("accept"),
            ),
        2 => Select::new("wide", &["A", "B"])
            .appearance(SelectAppearance::Field)
            .width(420.0)
            .open(true)
            .build(&k, &mut layer),
        3 => Select::new("set", &["A", "B"]).menu_width(240.0).open(true).build(&k, &mut layer),
        4 => Popover::new("pop")
            .trigger(el().size(20.0, 20.0))
            .max_width(480.0)
            .open(true)
            .child(el().w(440.0).h(10.0).key("wide-body"))
            .build(&k, &mut layer),
        5 => Popover::new("pop")
            .trigger(el().size(20.0, 20.0))
            .open(true)
            .child(text("x").key("narrow-body"))
            .build(&k, &mut layer),
        _ => unreachable!(),
    };
    root(col().w_full().h_full().child(body), layer)
}

const GREEN: Color = Color { r: 0.1, g: 0.6, b: 0.3, a: 1.0 };
const RED: Color = Color { r: 0.8, g: 0.15, b: 0.15, a: 1.0 };

fn run(case: u32) -> Runtime<Case> {
    let mut rt = Runtime::new(Case(case), view, (800.0, 600.0), Env::new(false, 1.0));
    rt.frame(0.0);
    rt.settle_all();
    rt.frame(0.0);
    rt
}

fn info(rt: &Runtime<Case>, key: &str) -> rune::ElementInfo {
    rt.elements().into_iter().find(|e| e.key.as_deref() == Some(key)).unwrap_or_else(|| panic!("no {key}"))
}

#[test]
fn state_badges_take_their_roles_colours() {
    let k = Kit::neutral(false);
    let rt = run(0);
    for (key, role) in [("success", k.c.success), ("warning", k.c.warning), ("danger", k.c.danger), ("info", k.c.info)]
    {
        let b = info(&rt, key);
        assert_eq!(b.fill, role.bg, "{key} fill");
        assert_eq!(b.role, "status");
    }
}

#[test]
fn tinted_and_state_buttons_paint_their_colours() {
    let k = Kit::neutral(false);
    let rt = run(1);
    assert_eq!(info(&rt, "join").fill, GREEN);
    assert_eq!(info(&rt, "leave").fill, RED);
    let ink = |key: &str| {
        let i = rt.elements().into_iter().position(|e| e.key.as_deref() == Some(key)).unwrap();
        rt.elements().into_iter().find(|e| e.parent == Some(i) && e.key.as_deref() == Some("icon")).unwrap().text_color
    };
    assert_eq!(ink("delete"), Some(k.c.danger.fg));
    assert_eq!(ink("accept"), Some(k.c.success.fg));
    assert_eq!(ink("leave"), Some(Color::WHITE));
}

#[test]
fn a_field_selects_listbox_is_as_wide_as_its_trigger() {
    let rt = run(2);
    let trigger = info(&rt, "wide-trigger").rect[2];
    let menu = info(&rt, "wide-content").rect[2];
    assert_eq!(trigger, 420.0);
    assert!(menu >= trigger, "menu {menu} under a {trigger} trigger");
}

#[test]
fn a_selects_listbox_takes_a_width() {
    let rt = run(3);
    assert_eq!(info(&rt, "set-content").rect[2], 240.0);
}

#[test]
fn a_popover_grows_to_its_max_width() {
    let wide = run(4);
    assert!(info(&wide, "pop-content").rect[2] > 320.0, "past the old cap");
    let narrow = run(5);
    assert_eq!(info(&narrow, "pop-content").rect[2], 200.0, "the default minimum");
}
