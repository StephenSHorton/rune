//! `--tree` reports the focused field's caret line, which `El::anchored_to_caret` overlays
//! hang from.
use super::*;
use crate::{Align, Side, TextEdit, TextStyle, col, text_input};

#[test]
fn the_tree_shows_the_focused_fields_caret_and_the_overlay_under_it() {
    fn view(s: &TextEdit, _: &mut Cx<TextEdit>) -> El {
        col().w_full().h_full()
            .child(text_input(s, TextStyle::new(14.0, 400.0)).key("input").w(200.0).on_edit(|s: &mut TextEdit| s))
            .child(crate::el().key("picker").size(80.0, 30.0).anchored_to_caret("input", Side::Bottom, Align::Start, 0.0))
    }
    let mut edit = TextEdit::new("abc");
    edit.focused = true;
    let mut rt = Runtime::new(edit, view, (300.0, 200.0), Env { scale: 1.0, system_dark: true });
    rt.frame(0.0);
    let tree = tree_json(&rt, 0.0);
    let els = tree["elements"].as_array().unwrap();
    let by = |key: &str| els.iter().find(|e| e["key"] == key).unwrap().clone();
    let caret: Vec<f64> = by("input")["caret"].as_array().expect("caret on the focused field").iter().map(|v| v.as_f64().unwrap()).collect();
    assert!(caret[0] > 8.0 && caret[3] > 10.0, "after \"abc\", a line tall: {caret:?}");
    let picker = by("picker")["rect"].as_array().unwrap().iter().map(|v| v.as_f64().unwrap()).collect::<Vec<_>>();
    assert!((picker[0] - caret[0]).abs() < 0.02 && (picker[1] - (caret[1] + caret[3])).abs() < 0.02, "{picker:?} under {caret:?}");
    assert!(els.iter().filter(|e| e["key"] != "input").all(|e| e.get("caret").is_none()), "only the focused field has one");
}
