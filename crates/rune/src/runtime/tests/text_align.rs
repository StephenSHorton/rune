//! `El::align` aligns each line of text with line breaks, not only wrapped text: centred lines
//! share a centre and end-aligned lines an end.
use super::*;
use crate::Align;

fn view(align: &Align, _: &mut Cx<Align>) -> El {
    col().w_full()
        .child(crate::el::text("Hi\nA much longer line of text\nMiddle line").align(*align).w(300.0).key("lines"))
        .child(crate::el::text("One line").align(*align).w(300.0).key("one"))
}

/// Each line's (start, end) inside its layout, in points.
fn lines(rt: &Runtime<Align>, key: &str) -> Vec<(f32, f32)> {
    let layout = rt.nodes.iter().find(|n| n.key.as_deref() == Some(key)).and_then(|n| n.text_layout.as_ref()).unwrap();
    let s = rt.env.scale;
    layout.lines().map(|l| (l.metrics().offset / s, (l.metrics().offset + l.metrics().advance) / s)).collect()
}

#[test]
fn centred_and_end_aligned_lines_align_each_line() {
    for (align, name) in [(Align::Center, "centre"), (Align::End, "end")] {
        let mut rt = Runtime::new(align, view, (400.0, 200.0), Env::new(true, 2.0));
        rt.frame(0.0);
        let ls = lines(&rt, "lines");
        assert_eq!(ls.len(), 3);
        let at = |(a, b): (f32, f32)| if align == Align::Center { (a + b) / 2.0 } else { b };
        for l in &ls[1..] {
            assert!((at(*l) - at(ls[0])).abs() < 0.6, "{name}: {ls:?}");
        }
        assert!(ls[0].0 > 10.0, "{name}: the short line moved in: {ls:?}");
        assert_eq!(lines(&rt, "one")[0].0, 0.0, "{name}: one line sits at its layout's start");
    }
    let mut rt = Runtime::new(Align::Start, view, (400.0, 200.0), Env::new(true, 2.0));
    rt.frame(0.0);
    assert!(lines(&rt, "lines").iter().all(|l| l.0 == 0.0), "start-aligned lines start together");
}
