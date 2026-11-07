//! The on-screen keyboard on Android, through a Kotlin host view (`dev.rune.RuneActivity` and
//! `dev.rune.RuneInputView`, in `crates/rune/android`). NativeActivity alone can't take
//! composing, autocorrect or swipe typing: those need a `View` that answers
//! `onCreateInputConnection`. The host keeps an `Editable` the keyboard reads and edits, and it
//! and Rune swap the same messages the web shell's hidden textarea does
//! (`runtime/web/runtime.rs`):
//!
//! - Kotlin to Rust, `RuneActivity.runeInput(json)`, on the UI thread: `{seq, kind, ...}` where
//!   `kind` is `edit` or `preedit` (`key`, `base`, `expected`, `start`, `end`, `text`,
//!   `anchor`, `focus`: UTF-16 offsets in `expected`, the window of text last mirrored, as
//!   `Runtime::web_edit` takes them), `selection`, or `key` (`key: "Enter"`). The call only
//!   queues it and wakes the event loop; the loop applies it (`receive`).
//! - Rust to Kotlin, `RuneActivity.runeSync(json)`, after a frame: `{key, text, base, anchor,
//!   focus, show, reshow, applied}`. `key` is the focused field's path ("" when none), `text` at
//!   most 512 UTF-16 units each side of the selection, `show` whether the keyboard should be up,
//!   `reshow` that a finger went down since the last push (Back can hide the keyboard while the
//!   field keeps focus; a tap brings it back), `applied` the last `seq` Rune took. It goes when
//!   any of them changed, and after every batch of edits even when nothing did, so the host
//!   learns an edit was dropped. The host ignores a sync older than its last edit, and otherwise
//!   takes it as the truth.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

use jni::objects::{JObject, JString};
use jni::refs::Global;
use jni::{EnvUnowned, JValue, jni_sig, jni_str};

use crate::runtime::web_runtime::Mirror;
use crate::runtime::{Runtime, Waker};

static QUEUE: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());
static WAKER: OnceLock<Waker> = OnceLock::new();
static HOSTED: AtomicBool = AtomicBool::new(true);
static TOUCHED: AtomicBool = AtomicBool::new(false);

/// A finger went down: the next sync asks the host to bring the keyboard back if a field still
/// has focus, since Back can dismiss it without Rune hearing.
pub(crate) fn touched() {
    TOUCHED.store(true, Ordering::Relaxed);
}

/// Lets the host's native calls wake the event loop. The process has one host.
pub(crate) fn connect(wake: Waker) {
    let _ = WAKER.set(wake);
}

/// `RuneActivity.runeInput(String)`: queue an edit from the keyboard, then wake the loop. It
/// never waits on the loop, which may be busy drawing.
#[jni::jni_mangle("dev.rune.RuneActivity")]
pub fn rune_input<'local>(mut env: EnvUnowned<'local>, _this: JObject<'local>, json: JString<'local>) {
    env.with_env(|env| -> jni::errors::Result<()> {
        let json = json.try_to_string(env)?;
        QUEUE.lock().unwrap_or_else(|e| e.into_inner()).push_back(json);
        if let Some(wake) = WAKER.get() { wake(); }
        Ok(())
    }).resolve::<jni::errors::LogErrorAndDefault>()
}

thread_local! {
    /// android_main's: only the event loop's thread reads or writes it.
    static BRIDGE: RefCell<Bridge> = RefCell::default();
}

/// Applies the edits the host queued. True when one may have changed the view.
pub(crate) fn receive<S: 'static>(rt: &mut Runtime<S>) -> bool {
    BRIDGE.with_borrow_mut(|b| b.receive(rt))
}

/// Tells the host about the focused field after a frame. `focused`: the window has focus.
pub(crate) fn sync<S: 'static>(rt: &Runtime<S>, focused: bool) {
    BRIDGE.with_borrow_mut(|b| b.sync(rt, focused))
}

/// What Rune last told the host, and the edits it has taken since.
#[derive(Default)]
struct Bridge {
    mirror: Mirror,
    show: bool,
    applied: u64,
    told_applied: u64,
}

impl Bridge {
    fn receive<S: 'static>(&mut self, rt: &mut Runtime<S>) -> bool {
        let events: Vec<String> = QUEUE.lock().unwrap_or_else(|e| e.into_inner()).drain(..).collect();
        for json in &events {
            let Ok(event) = serde_json::from_str::<serde_json::Value>(json) else { continue };
            if let Some(seq) = event["seq"].as_u64() { self.applied = self.applied.max(seq); }
            match event["kind"].as_str() {
                Some("edit" | "preedit" | "selection") => rt.web_edit(&event),
                Some("key") => { rt.web_key(&event); }
                _ => {}
            }
        }
        !events.is_empty()
    }

    /// Pushes when anything the host knows changed, or edits were taken since the last push.
    fn sync<S: 'static>(&mut self, rt: &Runtime<S>, focused: bool) {
        let mirrored = rt.web_context(&mut self.mirror);
        let show = focused && rt.ime_area.is_some() && !self.mirror.key.is_empty() && !self.mirror.secure;
        let reshow = TOUCHED.swap(false, Ordering::Relaxed) && show;
        if !mirrored && !reshow && show == self.show && self.applied == self.told_applied { return; }
        self.show = show;
        self.told_applied = self.applied;
        let m = &self.mirror;
        let json = serde_json::json!({
            "key": m.key, "text": m.text, "base": m.base, "anchor": m.anchor, "focus": m.focus,
            "show": show, "reshow": reshow, "applied": self.applied,
        }).to_string();
        if HOSTED.load(Ordering::Relaxed) && send(&json).is_err() {
            // A plain NativeActivity (no Kotlin host) has no `runeSync`: stop asking.
            HOSTED.store(false, Ordering::Relaxed);
        }
    }
}

/// Calls `RuneActivity.runeSync` on this thread (android_main's, which android-activity
/// attached); the host moves it to the UI thread.
fn send(json: &str) -> Result<(), String> {
    let app = super::app()?;
    let vm = unsafe { jni::JavaVM::from_raw(app.vm_as_ptr().cast()) };
    vm.attach_current_thread(|env| -> jni::errors::Result<()> {
        let raw: jni::sys::jobject = app.activity_as_ptr().cast();
        // Borrowed: android-activity owns this global reference.
        let activity = unsafe { env.as_cast_raw::<Global<JObject>>(&raw)? };
        let text = env.new_string(json)?;
        let called = env.call_method(activity.as_ref(), jni_str!("runeSync"), jni_sig!("(Ljava/lang/String;)V"),
            &[JValue::Object(&text)]);
        if called.is_err() { env.exception_clear(); }
        called.map(|_| ())
    }).map_err(|e| e.to_string())
}
