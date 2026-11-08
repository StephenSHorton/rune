use super::*;

#[test]
fn hex_parses_short_and_long_with_or_without_the_hash() {
    assert_eq!(parse_hex("#ff0000").map(hex6).as_deref(), Some("#ff0000"));
    assert_eq!(parse_hex(" 0f0 ").map(hex6).as_deref(), Some("#00ff00"));
    assert_eq!(parse_hex("#12345"), None);
    assert_eq!(parse_hex("#ggg"), None);
    assert_eq!(parse_hex("#ff000080"), None, "alpha isn't the field's");
}

#[test]
fn hue_turns_at_the_same_lightness_and_chroma() {
    // Muted enough that every hue at its lightness and chroma is inside sRGB.
    let red = rune::color::oklch(0.6, 0.08, 30.0);
    let h = hue_of(red).unwrap();
    let turned = with_hue(red, (h + 120.0) % 360.0);
    let (a, b) = (red.to_oklab(), turned.to_oklab());
    assert!((a[0] - b[0]).abs() < 1e-3, "lightness kept");
    assert!((a[1].hypot(a[2]) - b[1].hypot(b[2])).abs() < 1e-3, "chroma kept");
    assert!((hue_of(turned).unwrap() - (h + 120.0) % 360.0).abs() < 0.5);
}

#[test]
fn a_grey_has_no_hue_and_takes_colour_when_turned() {
    let grey = parse_hex("#808080").unwrap();
    assert_eq!(hue_of(grey), None);
    assert!(hue_of(with_hue(grey, 200.0)).is_some());
}

#[test]
fn twelve_default_presets_one_per_hue_step() {
    let p = default_presets();
    assert_eq!(p.len(), 12);
    let mut hexes: Vec<String> = p.iter().map(|c| hex6(*c)).collect();
    hexes.dedup();
    assert_eq!(hexes.len(), 12);
}
