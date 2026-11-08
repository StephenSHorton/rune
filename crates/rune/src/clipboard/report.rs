//! `Tx::copy_checked`: whether a clipboard write landed, told to the handler that asked for it.
//!
//! The callback stays on the event thread, keyed by an id. The write carries only the id and a
//! channel back, so the clipboard worker (or the browser's promise) can answer from its thread.
use std::cell::Cell;
use std::rc::Rc;
use std::sync::mpsc;
use crate::{Tx, Waker};

/// What a checked copy hears: `Ok` once the clipboard holds it, else why it doesn't.
pub(crate) type CopyFn = Box<dyn FnOnce(crate::states::States<'_>, Result<(), String>, &mut Tx)>;

/// A `Tx::copy_checked` callback. `Tx` is `Clone`; clones share it and the runtime takes it once.
#[derive(Clone)]
pub(crate) struct CopyReport(Rc<Cell<Option<CopyFn>>>);
impl CopyReport {
    pub(crate) fn new(f: CopyFn) -> Self { Self(Rc::new(Cell::new(Some(f)))) }
    pub(crate) fn take(&self) -> Option<CopyFn> { self.0.take() }
}
impl std::fmt::Debug for CopyReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str("CopyReport") }
}

/// Outcomes on their way back to a runtime, by report id.
pub(crate) type Outcome = (u64, Result<(), String>);

/// A write's report id, and once the shell sends it, where its outcome goes.
#[derive(Clone)]
pub(crate) struct Report {
    pub(crate) id: u64,
    pub(crate) to: Option<(mpsc::Sender<Outcome>, Waker)>,
}
impl Report {
    pub(crate) fn send(&self, result: Result<(), String>) {
        if let Some((to, wake)) = &self.to && to.send((self.id, result)).is_ok() { wake(); }
    }
}

/// The channel a client's checked copies answer on.
pub(crate) struct Outcomes {
    pub(crate) sender: mpsc::Sender<Outcome>,
    pub(crate) receiver: mpsc::Receiver<Outcome>,
}
impl Default for Outcomes {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self { sender, receiver }
    }
}

impl Tx {
    /// [`Tx::copy`], then `f` hears whether the clipboard took it: `Ok(())` once written, or the
    /// platform's reason it wasn't (no clipboard, the browser refused, another app holds it),
    /// so the app can say "Couldn't copy". It runs as an update, after the write. A later copy
    /// in the same handler, or before the shell writes this one, replaces it and `f` never
    /// runs; so does a headless runtime with no clipboard.
    ///
    /// ```ignore
    /// tx.copy_checked(link, |s: &mut App, result, _| s.toast = Some(match result {
    ///     Ok(()) => "Copied".into(),
    ///     Err(_) => "Couldn't copy".into(),
    /// }));
    /// ```
    pub fn copy_checked<S: 'static>(&mut self, text: impl Into<String>, f: impl FnOnce(&mut S, Result<(), String>, &mut Tx) + 'static) {
        self.copy_rich_checked(crate::ClipboardContent::new(text), f);
    }

    /// [`Tx::copy_rich`] with [`Tx::copy_checked`]'s report.
    pub fn copy_rich_checked<S: 'static>(&mut self, content: crate::ClipboardContent, f: impl FnOnce(&mut S, Result<(), String>, &mut Tx) + 'static) {
        self.copy_rich(content);
        self.copy_report = Some(CopyReport::new(Box::new(move |s, result, tx| {
            if let Some(s) = s.cast::<S>() { f(s, result, tx); }
        })));
    }
}

/// A runtime's checked copies waiting on the clipboard, by report id.
#[derive(Default)]
pub(crate) struct CopyReports(Vec<(u64, CopyFn)>);
impl CopyReports {
    /// Gives `copy` the report id of `report`'s callback, if it has one, and drops the callback
    /// of the unsent write it replaces. Ids come from `next`, shared with clipboard reads.
    pub(crate) fn attach(&mut self, copy: &mut crate::ClipboardWrite, replaced: Option<&crate::ClipboardWrite>, report: Option<CopyReport>, next: &mut u64) {
        if let Some(old) = replaced.and_then(|c| c.report.as_ref()) { self.forget(old.id); }
        if let Some(f) = report.and_then(|r| r.take()) {
            *next += 1;
            self.0.push((*next, f));
            copy.report = Some(Report { id: *next, to: None });
        }
    }
    pub(crate) fn take(&mut self, id: u64) -> Option<CopyFn> {
        let i = self.0.iter().position(|(pending, _)| *pending == id)?;
        Some(self.0.remove(i).1)
    }
    /// Drops the callback of a write replaced before it was sent.
    pub(crate) fn forget(&mut self, id: u64) { self.0.retain(|(pending, _)| *pending != id); }
}
