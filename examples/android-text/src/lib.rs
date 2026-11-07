//! A Rune text field that takes Android's on-screen keyboard: composing, autocorrect, swipe.
//! The activity is `dev.rune.RuneActivity` (`crates/rune/android`), whose view holds the
//! keyboard's connection and passes its edits to Rune (`rune::android`'s `ime`).
//! `scripts/android-apk.sh android-text` builds the APK with Gradle; `cargo run -p android-text`
//! opens the same view on the desktop.

use rune::tok::{self, type_};
use rune::{AppSpec, Cx, El, TextEdit, col, el, text, text_input};

#[derive(Default)]
pub struct State {
    note: TextEdit,
    sent: Vec<String>,
}

fn view(s: &State, _: &mut Cx<State>) -> El {
    let t = tok::theme(true);
    // In the top third: NativeActivity doesn't resize for the keyboard, which covers the rest.
    let field = text_input(&s.note, rune::TextStyle::new(17.0, 400.0)).key("note").single_line()
        .placeholder("Type here", t.text_secondary)
        .w_full().px(14.0).py(12.0).radius(10.0).fill(t.control_off)
        .on_edit(|s: &mut State| &mut s.note)
        .on_submit(|s: &mut State, _| {
            let line = std::mem::take(&mut s.note).text.clone();
            if !line.trim().is_empty() { s.sent.insert(0, line); }
        });
    let echo = text(format!("{} characters", s.note.text.chars().count())).font(type_::LABEL).color(t.text_secondary);
    let mut sent = col().gap(6.0);
    for line in s.sent.iter().take(6) {
        sent = sent.child(text(line.as_str()).color(t.text));
    }
    col().w_full().h_full().px(20.0).py(48.0).gap(12.0).fill(t.bg)
        .child(text("Rune on Android").font(type_::TITLE).color(t.text))
        .child(field)
        .child(echo)
        .child(el().h(8.0))
        .child(sent)
}

pub fn spec() -> AppSpec<State> {
    AppSpec {
        scenes: &[("typed", |s| s.note = TextEdit::new("Hello from Gboard"))],
        ..AppSpec::new("Android text", (360.0, 640.0), State::default, view)
    }
}

/// RuneActivity's entry point, as NativeActivity's (`android-activity`).
#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: rune::android::AndroidApp) {
    rune::android::init(app);
    rune::run(spec());
}
