use rune::{Cx, El, Env, Runtime, row};
use rune_kit::{Dialog, Kit, Placement, Select, SelectAppearance, root};

struct State { dialog: bool, select: bool, value: String }
fn view(s: &State, _: &mut Cx<State>) -> El {
    let k = Kit::neutral(true);
    let mut layer = Vec::new();
    // Build the portal before its modal, as nested component builders naturally do.
    let select = Select::new("choice", &["First", "Second"]).current(Some(&s.value))
        .appearance(SelectAppearance::Field).width(100.0).placement(Placement::RightStart)
        .open(s.select).on_toggle(|s: &mut State| s.select = !s.select)
        .on_close(|s: &mut State| s.select = false)
        .on_choose(|s: &mut State, v, _| { s.value = v.into(); s.select = false; })
        .build(&k, &mut layer);
    Dialog::new("dialog", "Choose").open(s.dialog).on_close(|s: &mut State| s.dialog = false)
        .child(row().w_full().justify_end().child(select)).build(&k, &mut layer);
    root(rune::el().w_full().h_full(), layer)
}

#[test]
fn select_portal_inside_dialog_accepts_an_option_outside_the_panel() {
    let mut rt = Runtime::new(State { dialog: true, select: false, value: "First".into() }, view,
        (1400.0, 800.0), Env::new(true, 1.0));
    rt.frame(0.0);
    let (x, y) = rt.find("choice-trigger").unwrap();
    rt.pointer_down(x, y); rt.pointer_up(x, y); rt.frame(0.0);
    assert!(rt.state.select && rt.state.dialog);
    for _ in 0..60 { rt.frame(1.0 / 60.0); }
    let elements = rt.elements();
    let panel = elements.iter().find(|e| e.key.as_deref() == Some("dialog-dialog")).unwrap();
    let option = elements.iter().find(|e| e.key.as_deref() == Some("option1")).unwrap();
    assert!(!option.inert);
    let (x, y) = rt.find("option1").unwrap();
    assert!(x < panel.rect[0] || x > panel.rect[0] + panel.rect[2]
        || y < panel.rect[1] || y > panel.rect[1] + panel.rect[3],
        "option ({x}, {y}) must be outside panel {:?}", panel.rect);
    rt.pointer_down(x, y); rt.pointer_up(x, y); rt.frame(0.0);
    assert_eq!(rt.state.value, "Second");
    assert!(rt.state.dialog && !rt.state.select);
}

#[test]
fn a_fit_dialog_is_as_tall_as_its_content() {
    fn view(fit: &bool, _: &mut Cx<bool>) -> El {
        let k = Kit::neutral(true);
        let mut layer = Vec::new();
        let buttons = row().key("buttons").w_full().padding(16.0).justify_end().child(rune::el().size(80.0, 28.0));
        let d = Dialog::new("delete", "Delete session").description("Delete session \"x\"?").width(400.0)
            .open(true).on_close(|_: &mut bool| {}).child(buttons);
        if *fit { d.fit() } else { d }.build(&k, &mut layer);
        root(rune::el().w_full().h_full(), layer)
    }
    let rect = |fit: bool, key: &str| {
        let mut rt = Runtime::new(fit, view, (1000.0, 800.0), Env::new(true, 1.0));
        for _ in 0..60 { rt.frame(1.0 / 60.0); }
        rt.elements().into_iter().find(|e| e.key.as_deref() == Some(key)).unwrap().rect
    };
    assert_eq!(rect(false, "delete-dialog")[3], 368.0, "the standard height");
    let (panel, buttons) = (rect(true, "delete-dialog"), rect(true, "buttons"));
    // 16 + title 20 + 4 + description 20 + 16, then the 60 point button row.
    assert_eq!(panel[3], 76.0 + 60.0);
    assert_eq!((panel[0], panel[1]), ((1000.0 - 400.0) / 2.0, (800.0 - panel[3]) / 2.0), "centred in the layer");
    assert_eq!(buttons[1] + buttons[3], panel[1] + panel[3], "the buttons sit at the bottom edge");
}

fn panel_view(case: &u8, _: &mut Cx<u8>) -> El {
    let k = Kit::neutral(true);
    let mut layer = Vec::new();
    let title = "Allow this agent to keep editing files in the workspace while you are away from the keyboard";
    let description = "It can create, change and delete files under the project folder. You can take this back at any time from the agent's menu.";
    let rows = (0..if *case == 2 { 40 } else { 1 }).map(|i| rune::el().key(format!("row{i}")).w_full().h(30.0));
    let d = Dialog::new("ask", title).description(description).width(360.0).open(true).on_close(|_: &mut u8| {})
        .child(rune::col().w_full().children(rows));
    if *case == 0 { d } else { d.fit() }.build(&k, &mut layer);
    root(rune::el().w_full().h_full(), layer)
}

fn rects(case: u8, window: (f32, f32)) -> impl Fn(&str) -> [f32; 4] {
    let mut rt = Runtime::new(case, panel_view, window, Env::new(true, 1.0));
    for _ in 0..60 { rt.frame(1.0 / 60.0); }
    let els = rt.elements();
    move |key: &str| els.iter().find(|e| e.key.as_deref() == Some(key)).unwrap_or_else(|| panic!("{key}")).rect
}

fn inside(inner: [f32; 4], outer: [f32; 4]) -> bool {
    inner[0] >= outer[0] - 0.01 && inner[1] >= outer[1] - 0.01
        && inner[0] + inner[2] <= outer[0] + outer[2] + 0.01 && inner[1] + inner[3] <= outer[1] + outer[3] + 0.01
}

#[test]
fn a_long_title_and_description_wrap_inside_the_panel_beside_the_close_button() {
    for case in [0, 1] {
        let rect = rects(case, (800.0, 600.0));
        let (panel, title, close) = (rect("ask-dialog"), rect("title"), rect("close"));
        assert!(inside(title, panel) && inside(rect("description"), panel), "{case}: text inside {panel:?}: {title:?}");
        assert!(inside(close, panel), "{case}: the close button inside the panel: {close:?} in {panel:?}");
        assert!(title[0] + title[2] <= close[0], "{case}: the title stops before the close button");
        assert!(close[0] + close[2] > panel[0] + panel[2] - 20.0 && close[1] < panel[1] + 20.0, "{case}: in the top-right corner: {close:?}");
        assert!(title[3] > 60.0, "{case}: the title and description take several lines: {title:?}");
    }
}

#[test]
fn a_fit_dialog_taller_than_the_window_scrolls_its_body() {
    let rect = rects(2, (800.0, 500.0));
    let (panel, header, body) = (rect("ask-dialog"), rect("header"), rect("body"));
    assert!(panel[1] >= 16.0 - 0.01 && panel[1] + panel[3] <= 500.0 - 16.0 + 0.01, "inside the window's margins: {panel:?}");
    assert!(body[1] >= header[1] + header[3] - 0.01 && body[1] + body[3] <= panel[1] + panel[3] + 0.01, "under the header: {body:?}");
    assert!(rect("row39")[1] > body[1] + body[3], "the last rows wait below, to scroll to");
}
