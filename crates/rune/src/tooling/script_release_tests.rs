//! A script's release lands where the pointer is: a move during a tap's hold isn't undone by
//! its up, as it would be if the up went back to where the press started.
use std::cell::RefCell;
use std::time::Duration;

use super::*;

thread_local! {
    static HEARD: RefCell<Vec<bool>> = const { RefCell::new(Vec::new()) };
}

#[test]
fn a_tap_whose_pointer_leaves_during_the_hold_releases_outside() {
    let _turn = super::tests::HEADLESS
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    HEARD.with(|h| h.borrow_mut().clear());
    let parsed = parse_script(json!([{"t": 0.1, "type": "tap", "x": 8, "y": 8}]));
    assert!(
        matches!(
            parsed.last().map(|e| (&e.kind, &e.pos)),
            Some((Kind::Up, Pos::Pointer))
        ),
        "{parsed:?}"
    );
    let dir = std::env::temp_dir().join(format!("rune-release-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let script = dir.join("script.json");
    // Rest on the zone until its hover shows, tap it, and leave 50 ms into the 80 ms hold.
    std::fs::write(
        &script,
        r#"[{"t": 0.0, "type": "move", "x": 8, "y": 8},
        {"t": 0.2, "type": "tap", "x": 8, "y": 8},
        {"t": 0.25, "type": "move", "x": 60, "y": 60},
        {"t": 1.0, "type": "mark", "name": "end"}]"#,
    )
    .unwrap();
    let view = |_: &(), _: &mut crate::Cx<()>| {
        crate::col().w_full().h_full().child(
            crate::el()
                .key("zone")
                .w(16.0)
                .h(16.0)
                .on_press(|_: &mut ()| {})
                .on_hover_change(|_: &mut (), inside, _| {
                    HEARD.with(|h| h.borrow_mut().push(inside))
                })
                .hover_delays(Duration::from_millis(40), Duration::from_millis(100)),
        )
    };
    let spec = AppSpec::new("release", (80.0, 80.0), || (), view);
    let tree = dir.join("tree.json");
    let args = [
        "--script",
        script.to_str().unwrap(),
        "--scale",
        "1",
        "--fps",
        "60",
        "--tree",
        tree.to_str().unwrap(),
    ];
    headless(
        crate::App::new(spec),
        Args::parse_from(args.map(String::from).into()),
    );
    std::fs::remove_dir_all(&dir).unwrap();
    assert_eq!(
        HEARD.with(|h| h.borrow().clone()),
        [true, false],
        "the hover shows, then the release outside ends it"
    );
}
