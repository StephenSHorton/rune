//! A headless run's clipboard: a checked copy answers, so an app can show its "Copied" state
//! in a `--snap` or `--script` run, and the run's clipboard holds what was copied.
use std::cell::RefCell;

use super::*;

thread_local! {
    static HEARD: RefCell<Vec<Result<(), String>>> = const { RefCell::new(Vec::new()) };
}

fn copy_run(fail: Option<&'static str>) -> (Vec<Result<(), String>>, Option<String>) {
    HEARD.with(|h| h.borrow_mut().clear());
    let dir = std::env::temp_dir().join(format!("rune-headless-copy-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let script = dir.join("script.json");
    std::fs::write(&script, r#"[{"t": 0.1, "type": "tap", "key": "copy"}]"#).unwrap();
    let view =
        |_: &(), _: &mut crate::Cx<()>| {
            crate::col().w_full().h_full().child(
                crate::el()
                    .key("copy")
                    .w(20.0)
                    .h(20.0)
                    .on_press_tx(|_: &mut (), tx| {
                        tx.copy_checked("link", |_: &mut (), result, _| {
                            HEARD.with(|h| h.borrow_mut().push(result))
                        });
                    }),
            )
        };
    // `init` runs once the run is published, so it can set the clipboard up first.
    let init: fn() = if fail.is_some() {
        || {
            headless_run()
                .unwrap()
                .clipboard()
                .fail_writes(Some("no clipboard here"))
        }
    } else {
        || {}
    };
    let spec = AppSpec::new("copy", (40.0, 40.0), init, view);
    let tree = dir.join("tree.json");
    let args = [
        "--script",
        script.to_str().unwrap(),
        "--scale",
        "1",
        "--fps",
        "30",
        "--tree",
        tree.to_str().unwrap(),
    ];
    headless(
        crate::App::new(spec),
        Args::parse_from(args.map(String::from).into()),
    );
    std::fs::remove_dir_all(&dir).unwrap();
    let copied = headless_run()
        .unwrap()
        .clipboard()
        .text()
        .map(|t| t.as_str().to_owned());
    (HEARD.with(|h| h.borrow().clone()), copied)
}

#[test]
fn a_checked_copy_in_a_headless_run_hears_its_outcome() {
    let _turn = super::tests::HEADLESS
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    assert_eq!(
        copy_run(None),
        (vec![Ok(())], Some("link".to_owned())),
        "written to the run's clipboard"
    );
    assert_eq!(
        copy_run(Some("no clipboard here")),
        (vec![Err("no clipboard here".to_owned())], None),
        "a failing clipboard says why"
    );
}
