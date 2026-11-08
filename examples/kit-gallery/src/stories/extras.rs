//! Rune's own components with no Storybook story: Slider, Avatar, Skeleton, ColorSwatch, the
//! state badges and tinted buttons.

use rune::{Color, El, col, row};
use rune_kit::*;
use rune_theme_opencode::icons as ic;

use crate::G;

/// `rune-kit--extras`.
pub fn kit_extras(s: &G, k: &Kit, _layer: &mut Vec<El>) -> El {
    let volume: f32 = s.pick.get("volume").and_then(|v| v.parse().ok()).unwrap_or(0.4);
    let swatch: Option<usize> = s.pick.get("swatch").and_then(|v| v.parse().ok());
    let c = &k.c;
    let slider = row()
        .gap(16.0)
        .center_y()
        .child(
            Slider::new(volume)
                .step(0.05)
                .label("Volume")
                .on_change(|s: &mut G, v, _| {
                    s.pick.insert("volume".into(), v.to_string());
                })
                .build(k),
        )
        .child(Slider::new(0.7).label("Disabled").disabled(true).build(k));
    let avatars = row()
        .gap(12.0)
        .center_y()
        .child(Avatar::new("Ada Lovelace").size(40.0).status(c.success.icon).build(k))
        .child(Avatar::new("Grace Hopper").status(c.warning.icon).build(k))
        .child(Avatar::new("Alan Turing").shape(AvatarShape::Rounded).build(k))
        .child(Avatar::new("Edsger Dijkstra").size(20.0).build(k));
    let skeleton = row()
        .gap(12.0)
        .w(320.0)
        .child(Skeleton::circle(40.0).build(k))
        .child(
            col()
                .gap(8.0)
                .grow(1.0)
                .child(Skeleton::new().width(140.0).label("Loading profile").build(k))
                .child(Skeleton::new().build(k)),
        );
    let badges = row()
        .gap(8.0)
        .child(Badge::new("Done").variant(BadgeVariant::Success).build(k))
        .child(Badge::new("Stale").variant(BadgeVariant::Warning).build(k))
        .child(Badge::new("Failed").variant(BadgeVariant::Danger).build(k))
        .child(Badge::new("New").variant(BadgeVariant::Info).build(k));
    let buttons = row()
        .gap(8.0)
        .center_y()
        .child(Button::new("Join").tint(c.success.icon, Color::WHITE).build(k))
        .child(IconButton::new(&ic::CLOSE).label("Leave").tint(c.danger.icon, Color::WHITE).build(k))
        .child(IconButton::new(&ic::CLOSE).label("Delete").variant(IconButtonVariant::Danger).build(k))
        .child(IconButton::new(&ic::CHECK).label("Accept").variant(IconButtonVariant::Success).build(k));
    let mut swatches = row().gap(6.0).role("radiogroup").label("Colour");
    for (i, hue) in [25.0, 85.0, 145.0, 205.0, 265.0, 325.0].into_iter().enumerate() {
        swatches = swatches.child(
            ColorSwatch::new(rune::color::oklch(0.66, 0.16, hue))
                .round()
                .selected(swatch == Some(i))
                .on_press(move |s: &mut G| {
                    s.pick.insert("swatch".into(), i.to_string());
                })
                .build(k),
        );
    }
    let progress = col().w(320.0).gap(12.0).child(Progress::indeterminate().height(4.0).build(k)).child(
        Progress::new(Some(0.6)).height(4.0).color(c.success.icon).track(c.layer_03).build(k),
    );
    col().padding(24.0).gap(20.0).children([slider, avatars, skeleton, badges, buttons, swatches, progress])
}
