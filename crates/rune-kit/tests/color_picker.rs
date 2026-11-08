//! `ColorPicker`: swatches pick and fill the hex field, typing a hex reports it once it parses,
//! and the hue slider turns the colour.
use rune::{Color, Cx, El, Env, Key, KeyEvent, Mods, Runtime, col};
use rune_kit::{ColorPicker, ColorPickerState, ColorSwatch, Kit};

#[derive(Default)]
struct App {
    color: Option<Color>,
    picker: ColorPickerState,
    changes: usize,
}

fn view(s: &App, cx: &mut Cx<App>) -> El {
    let k = Kit::neutral(cx.system_dark());
    let picker = ColorPicker::new("tag", &s.picker, |s: &mut App| &mut s.picker)
        .value(s.color)
        .hue(true)
        .on_change(|s: &mut App, c, _| {
            s.color = Some(c);
            s.changes += 1;
        })
        .build(&k);
    col().w_full().h_full().padding(10.0).child(picker).child(ColorSwatch::new(Color::WHITE).build(&k).key("chip"))
}

fn app() -> Runtime<App> {
    let mut rt = Runtime::new(App::default(), view, (400.0, 400.0), Env::new(false, 1.0));
    rt.frame(0.0);
    rt
}

fn press(rt: &mut Runtime<App>, key: &str) {
    let (x, y) = rt.find(key).unwrap_or_else(|| panic!("no {key}"));
    rt.pointer_down(x, y);
    rt.frame(0.0);
    rt.pointer_up(x, y);
    rt.frame(0.0);
}

fn hex(c: Color) -> String {
    c.hex()[..7].to_owned()
}

#[test]
fn a_swatch_picks_its_colour_and_fills_the_field() {
    let mut rt = app();
    press(&mut rt, "swatch3");
    let c = rt.state.color.expect("picked");
    assert_eq!(rt.state.picker.hex.text, hex(c));
    let sw = rt.elements().into_iter().find(|e| e.key.as_deref() == Some("swatch3")).unwrap();
    assert_eq!(sw.role, "radio");
    assert_eq!(sw.fill, c);
    let tree = rt.accessibility_json().to_string();
    assert!(tree.contains("\"radio-group\""), "{tree}");
}

#[test]
fn typing_a_hex_reports_it_once_it_parses() {
    let mut rt = app();
    press(&mut rt, "tag-hex");
    for ch in ["#", "1", "2", "3"] {
        rt.key(KeyEvent { key: Key::Text(ch.into()), mods: Mods::default() });
        rt.frame(0.0);
    }
    // "#123" is a short hex: it parses.
    assert_eq!(rt.state.color.map(hex).as_deref(), Some("#112233"));
    let before = rt.state.changes;
    rt.key(KeyEvent { key: Key::Text("4".into()), mods: Mods::default() });
    rt.frame(0.0);
    assert_eq!(rt.state.changes, before, "#1234 doesn't parse: no report");
    for ch in ["5", "6"] {
        rt.key(KeyEvent { key: Key::Text(ch.into()), mods: Mods::default() });
        rt.frame(0.0);
    }
    assert_eq!(rt.state.color.map(hex).as_deref(), Some("#123456"));
}

#[test]
fn the_hue_slider_turns_the_colour() {
    let mut rt = app();
    press(&mut rt, "swatch0");
    let first = rt.state.color.unwrap();
    let (x, y) = rt.find("tag-hue").unwrap();
    rt.pointer_down(x + 40.0, y);
    rt.frame(0.0);
    rt.pointer_up(x + 40.0, y);
    rt.frame(0.0);
    let turned = rt.state.color.unwrap();
    assert_ne!(hex(turned), hex(first));
    assert_eq!(rt.state.picker.hex.text, hex(turned), "the field follows the hue");
    let (a, b) = (first.to_oklab(), turned.to_oklab());
    assert!((a[0] - b[0]).abs() < 0.03, "about the same lightness");
}

#[test]
fn a_swatch_without_a_handler_is_an_image() {
    let rt = app();
    let chip = rt.elements().into_iter().find(|e| e.key.as_deref() == Some("chip")).unwrap();
    assert_eq!((chip.role, chip.label.as_deref()), ("img", Some("#ffffff")));
}
