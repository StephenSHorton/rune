//! Skeleton's pulse, the indeterminate Progress sweep, and Avatar's picture, initials and dot.
use rune::{Color, Cx, El, Env, Pixels, Runtime, col};
use rune_kit::{Avatar, AvatarShape, Kit, Progress, Skeleton};

struct Case {
    which: u32,
    photo: Pixels,
}

fn view(s: &Case, cx: &mut Cx<Case>) -> El {
    let k = Kit::neutral(cx.system_dark());
    let body = match s.which {
        0 => col().w(200.0).child(Skeleton::new().width(120.0).build(&k).key("line")),
        1 => col().w(200.0).child(Progress::indeterminate().height(4.0).color(Color::WHITE).build(&k).key("bar")),
        2 => col()
            .child(Avatar::new("Ada Lovelace").size(40.0).status(Color::WHITE).build(&k).key("ada"))
            .child(Avatar::new("Grace Hopper").image(&s.photo).shape(AvatarShape::Rounded).build(&k).key("grace")),
        _ => unreachable!(),
    };
    col().w_full().h_full().child(body)
}

fn run(which: u32, reduced: bool) -> Runtime<Case> {
    let mut rt = Runtime::new(Case { which, photo: Pixels::new() }, view, (300.0, 200.0), Env::new(false, 1.0));
    rt.set_system_reduced_motion(reduced);
    rt.frame(0.0);
    rt
}

/// The element keyed `key`, or for `a/b` the element keyed `b` under the one keyed `a`.
fn info(rt: &Runtime<Case>, key: &str) -> rune::ElementInfo {
    lookup(rt, key).unwrap_or_else(|| panic!("no {key}"))
}

fn lookup(rt: &Runtime<Case>, key: &str) -> Option<rune::ElementInfo> {
    let all = rt.elements();
    let found = match key.split_once('/') {
        None => all.iter().find(|e| e.key.as_deref() == Some(key)),
        Some((outer, inner)) => {
            let under = |mut i: Option<usize>| {
                while let Some(j) = i {
                    if all[j].key.as_deref() == Some(outer) {
                        return true;
                    }
                    i = all[j].parent;
                }
                false
            };
            all.iter().find(|e| e.key.as_deref() == Some(inner) && under(e.parent))
        }
    };
    found.cloned()
}

fn opacities(rt: &mut Runtime<Case>, key: &str) -> Vec<f32> {
    (0..20)
        .map(|_| {
            rt.frame(0.1);
            info(rt, key).opacity
        })
        .collect()
}

#[test]
fn a_skeleton_pulses() {
    let mut rt = run(0, false);
    let o = opacities(&mut rt, "line");
    let (lo, hi) = o.iter().fold((f32::MAX, f32::MIN), |(a, b), &x| (a.min(x), b.max(x)));
    assert!(hi > 0.95 && lo < 0.6, "pulses between full and half: {o:?}");
    assert_eq!(info(&rt, "line").rect[2], 120.0);
}

#[test]
fn a_skeleton_holds_still_and_opaque_under_reduced_motion() {
    let mut rt = run(0, true);
    let o = opacities(&mut rt, "line");
    assert!(o.iter().all(|&x| x > 0.99), "{o:?}");
}

#[test]
fn an_indeterminate_bar_sweeps() {
    let mut rt = run(1, false);
    assert_eq!(info(&rt, "bar/track").rect[3], 4.0);
    assert_eq!(info(&rt, "bar/fill").fill, Color::WHITE);
    let xs: Vec<f32> = (0..10)
        .map(|_| {
            rt.frame(0.1);
            info(&rt, "bar/fill").rect[0]
        })
        .collect();
    let moved = xs.windows(2).filter(|w| (w[0] - w[1]).abs() > 1.0).count();
    assert!(moved >= 5, "the segment moves: {xs:?}");
}

#[test]
fn an_avatar_shows_initials_its_dot_and_a_picture_over_them() {
    let mut rt = run(2, false);
    let ada = info(&rt, "ada");
    assert_eq!((ada.role, ada.label.as_deref()), ("img", Some("Ada Lovelace")));
    assert_eq!(ada.rect[2], 40.0);
    let initials = rt.elements().into_iter().find(|e| e.text.as_deref() == Some("AL"));
    assert!(initials.is_some(), "initials without a picture");
    assert_eq!(info(&rt, "ada/status").fill, Color::WHITE);
    assert!(lookup(&rt, "grace/image").is_some(), "the picture layers over the initials");
    assert!(lookup(&rt, "grace/status").is_none());
    let face = info(&rt, "grace/face");
    assert!(face.radius < 16.0, "rounded, not round: {}", face.radius);
    rt.frame(0.0);
}
