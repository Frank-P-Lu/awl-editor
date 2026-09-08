//! Finder "Open With" + cold-launch document handoff: macOS,
//! non-MAS desktop only.
//!
//! **THE GAP, verified rather than assumed.** `scripts/package-macos.sh`
//! wrote no `CFBundleDocumentTypes` (fixed alongside this file — see
//! `mac_open_documents_law`'s plist test), so Finder's Open With menu never
//! listed Awl at all. Separately, and independently of the plist: winit 0.30
//! OWNS the process's `NSApplicationDelegate` (a `declare_class!`d type named
//! `"WinitApplicationDelegate"`, in
//! `winit::platform_impl::macos::app_state`) and implements only
//! `applicationDidFinishLaunching:` / `applicationWillTerminate:` on it — read
//! straight out of the vendored crate source at
//! `~/.cargo/registry/src/*/winit-0.30.13/src/platform_impl/macos/app_state.rs`,
//! not inferred from its docs. It never implements `application:openURLs:`,
//! so even with the plist fixed, AppKit finds no handler for the open-
//! documents Apple Event and `open -a Awl note.md` launches the app with no
//! file. This file is the fix for that second half.
//!
//! **THE TECHNIQUE.** We can't edit winit's `declare_class!` block, so
//! [`install`] adds the missing selector to the ALREADY-REGISTERED class at
//! runtime via the raw Objective-C runtime function `class_addMethod` —
//! exactly the mechanism winit's own
//! `platform_impl::macos::app::override_send_event` uses to swizzle
//! `NSApplication::sendEvent:` (same file, same crate), except we are ADDING
//! a selector that does not exist yet rather than replacing one that does.
//! `objc2::runtime::ClassBuilder` cannot do this — it only builds a BRAND NEW
//! class pair (`objc_allocateClassPair`) and returns `None` for a name that
//! is already registered — so this drops to the raw `ffi::class_addMethod`
//! `ClassBuilder` itself calls internally.
//!
//! **TIMING, verified against winit 0.30.13's own source (not assumed):**
//! [`install`] runs from `crate::app::run`, right after `event_loop.create_proxy()`
//! and strictly BEFORE `event_loop.run_app(..)`. At that point the
//! `WinitApplicationDelegate` class already exists (`EventLoop::new` —
//! `platform_impl::macos::event_loop.rs` — constructs the delegate instance
//! and calls `app.setDelegate(..)` inside `EventLoopBuilder::build()`, which
//! has already returned), but the real `CFRunLoop` has not started pumping
//! yet — `-[NSApplication run]`, the only thing that can actually DELIVER a
//! queued Apple Event to a delegate method, is what `run_app` calls. So no
//! ordering of Apple's own "deliver the open-document event around launch"
//! behavior (documented informally, and not a hard OS contract this crate
//! controls) can ever race our `class_addMethod` call — it is unconditionally
//! done before the run loop that would need it starts running at all.
//!
//! **THE COLD-LAUNCH RACE THAT REMAINS, and why it's still handled.** Even
//! though `install` always wins the race above, a genuine cold launch (Finder
//! starts this PROCESS specifically to hand it a file) can still have AppKit
//! call `application:openURLs:` before winit's `resumed()` has built the
//! first window — `resumed()` itself runs synchronously inside
//! `applicationDidFinishLaunching:`, and Apple's own documented behavior for
//! this delegate method allows it to fire around/before that notification.
//! Posting straight through `EventLoopProxy::send_event` at that point would
//! still be *safe* (traced through `platform_impl::macos::app_state.rs`:
//! `Event::Resumed` is dispatched synchronously from
//! `dispatch_init_events()`, called directly from `did_finish_launching()`,
//! while a proxied `send_event` only wakes a `CFRunLoopSource` — draining it
//! into `App::user_event` waits for the run loop's later "before waiting"
//! observer (`cleared()`), which cannot fire until `did_finish_launching()`
//! has already returned) — but relying on that ordering being permanent
//! across future winit releases is exactly the kind of unstated-behavior bet
//! this crate's own conventions warn against. So instead: every URL is
//! queued (module [`queue`], a plain `Vec` behind a `Mutex`, unit-tested with
//! zero AppKit involved) until [`flush_after_first_frame`] — called once from
//! `App::on_gpu_ready`, this crate's own established "the first frame is up"
//! seam (see its doc: the live-probe's `probe_ready` signal fires from the
//! exact same spot) — drains it and posts every queued path through the
//! SAME door a live delivery uses. A drain with nothing queued (every launch
//! that isn't a Finder cold-open) is one lock and one empty-`Vec` check.
//!
//! **ONE PATH, NEVER A SECOND.** Every URL, warm or cold, is posted as
//! `AwlEvent::Daemon(DaemonEvent::OpenPath { waiter: None, .. })` — the exact
//! same event `crate::daemon`'s socket accept-loop posts for a fire-and-forget
//! CLI open, interpreted by the exact same `App::handle_daemon_event`
//! (`app/daemon.rs`) that already tolerates a not-yet-existent window (it
//! only focuses/raises the window when `frame.gpu()` is `Some`). Finder never
//! gets `--wait` semantics — there is no client blocked on a `done` reply —
//! so `waiter` is always `None`. [`mac_open_documents_law`] greps the whole
//! crate for the one place that directly constructs this shape, so a second
//! AppKit door built the same way anywhere else fails that law until it is
//! named there too.
#![cfg(all(target_os = "macos", not(feature = "mas"), not(target_arch = "wasm32")))]

use std::path::PathBuf;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

use objc2::runtime::{AnyClass, Imp, Sel};
use objc2::{ffi, sel};
use objc2_app_kit::NSApplication;
use objc2_foundation::{MainThreadMarker, NSArray, NSObject, NSURL};

use crate::app::AwlEvent;
use crate::daemon::DaemonEvent;

/// The pure "queue until it's safe to post" half — no AppKit, no proxy,
/// unit-tested directly. See the module doc's "COLD-LAUNCH RACE" section for
/// why this exists at all.
mod queue {
    use std::path::PathBuf;
    use std::sync::Mutex;

    static PENDING: Mutex<Vec<PathBuf>> = Mutex::new(Vec::new());

    pub fn push(path: PathBuf) {
        PENDING.lock().unwrap_or_else(|e| e.into_inner()).push(path);
    }

    /// Empty the queue and return everything that was in it, IN ARRIVAL
    /// ORDER. `mem::take` (not a clone) is load-bearing: a second call
    /// returns nothing, which is the "drains exactly once" property a cold
    /// launch depends on — see the law test below.
    pub fn take_all() -> Vec<PathBuf> {
        std::mem::take(&mut *PENDING.lock().unwrap_or_else(|e| e.into_inner()))
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        /// THE COLD-LAUNCH QUEUE LAW. Mutation-proven: swap
        /// `mem::take` in `take_all` for a `.clone()` and the SECOND
        /// assertion below goes red — the same paths get redelivered, which
        /// on a real cold launch means the same Finder-opened file gets
        /// opened twice (harmless here, but a `--wait`-carrying waiter
        /// delivered twice would double-notify a client that only expects
        /// one `done`).
        #[test]
        fn drains_every_queued_path_in_arrival_order_exactly_once() {
            let _guard = crate::testlock::serial();
            // Defensive: nothing else in this binary touches this queue
            // outside this one test (the real AppKit callback never runs
            // under `cargo test`), but draining first keeps this test
            // self-contained regardless of run order.
            let _ = take_all();
            push(PathBuf::from("/tmp/a.md"));
            push(PathBuf::from("/tmp/b.md"));
            assert_eq!(
                take_all(),
                vec![PathBuf::from("/tmp/a.md"), PathBuf::from("/tmp/b.md")],
                "queued paths must drain once, in the order Finder handed them over"
            );
            assert_eq!(
                take_all(),
                Vec::<PathBuf>::new(),
                "a second drain must find nothing — the queue does not replay"
            );
        }
    }
}

/// Set once at startup by [`install`] — the SAME `EventLoopProxy` the daemon
/// accept thread was handed (`crate::app::run`'s one `event_loop.create_proxy()`
/// call), so a Finder-opened URL is posted through the identical door.
static PROXY: OnceLock<winit::event_loop::EventLoopProxy<AwlEvent>> = OnceLock::new();

/// Flips true the one time [`flush_after_first_frame`] runs. Before that,
/// every URL is queued rather than posted directly — see the module doc.
static READY: AtomicBool = AtomicBool::new(false);

/// Inject `application:openURLs:` onto winit's `NSApplicationDelegate` class
/// and remember `proxy` for the callback to post through. Call once, from
/// `crate::app::run`, after `event_loop.create_proxy()` and before
/// `event_loop.run_app(..)` — see the module doc's TIMING section for why
/// that window is always safe. A no-op off the main thread (should never
/// happen — this only ever runs from the process's own startup sequence).
pub fn install(proxy: winit::event_loop::EventLoopProxy<AwlEvent>) {
    let Some(_mtm) = MainThreadMarker::new() else {
        return;
    };
    let _ = PROXY.set(proxy);
    let Some(class) = AnyClass::get(c"WinitApplicationDelegate") else {
        // winit renamed or restructured its delegate class — nothing to hang
        // the selector on. Loud in a debug build (this is a load-bearing
        // assumption about a pinned dependency version), silent in release
        // rather than crashing an otherwise-working editor over Open With.
        debug_assert!(
            false,
            "winit's ApplicationDelegate class (\"WinitApplicationDelegate\") \
             must already be registered by EventLoop::build() — Open With will \
             not work this launch"
        );
        return;
    };
    // SAFETY: adding a selector that does not exist on this class yet (a
    // fresh `class_addMethod`, not a swizzle — nobody has ever sent
    // `application:openURLs:` to this delegate, so there is no prior
    // implementation to race or clobber), with a type encoding matching the
    // real signature (`void application:(NSApplication *)app
    // openURLs:(NSArray<NSURL *> *)urls` → "v@:@@", the same simplified,
    // offset-free encoding `objc2::runtime::ClassBuilder` itself writes for
    // every method it adds), called before the run loop has started pumping
    // (see the module doc's TIMING section) so nothing can be mid-dispatch
    // through this selector when it is added.
    unsafe {
        let imp: Imp = std::mem::transmute::<OpenUrlsImp, Imp>(application_open_urls);
        let added = ffi::class_addMethod(
            (class as *const AnyClass).cast_mut(),
            sel!(application:openURLs:),
            imp,
            c"v@:@@".as_ptr(),
        );
        debug_assert!(
            added.as_bool(),
            "application:openURLs: must not already exist on WinitApplicationDelegate"
        );
    }
}

/// Drain [`queue`] and mark the door open for direct delivery from here on.
/// Call exactly once, from `App::on_gpu_ready` — see the module doc's
/// "COLD-LAUNCH RACE" section. Idempotent: `on_gpu_ready` also runs after a
/// GPU-fault rebuild, and a second (or hundredth) call here just finds an
/// empty queue.
pub fn flush_after_first_frame() {
    READY.store(true, Ordering::Release);
    for path in queue::take_all() {
        post(path);
    }
}

fn post(path: PathBuf) {
    if let Some(proxy) = PROXY.get() {
        let _ = proxy.send_event(AwlEvent::Daemon(DaemonEvent::OpenPath {
            path,
            waiter: None,
        }));
    }
}

/// Route one Finder-handed path: straight through if the first frame has
/// already happened, queued otherwise. See the module doc.
fn handle_open(path: PathBuf) {
    if READY.load(Ordering::Acquire) {
        post(path);
    } else {
        queue::push(path);
    }
}

/// The exact `application:openURLs:` signature (`self`, `_cmd`, the
/// `NSApplication`, the `NSArray<NSURL>`), matching what
/// `objc2_app_kit::NSApplication`'s generated `NSApplicationDelegate` trait
/// declares for `application_openURLs` — used only for its type, never
/// implemented as that trait (we cannot; winit owns the `impl` block for
/// this class), which is why this goes through raw `class_addMethod` instead.
type OpenUrlsImp = extern "C" fn(&NSObject, Sel, &NSApplication, &NSArray<NSURL>);

/// The injected method body. Runs on the main thread (AppKit delegate
/// callbacks always do) whenever Finder hands this process one or more
/// documents to open — a fresh cold launch via double-click/Open With, a
/// drag onto the Dock tile, or `open -a Awl file.md` while Awl is already
/// running. `urls` can carry more than one entry (multi-select Open With);
/// each becomes its own posted/queued path, exactly as if the user had run
/// the CLI once per file against the daemon.
extern "C" fn application_open_urls(
    _this: &NSObject,
    _cmd: Sel,
    _app: &NSApplication,
    urls: &NSArray<NSURL>,
) {
    for url in urls.iter() {
        let Some(path) = url.path() else { continue };
        handle_open(PathBuf::from(path.to_string()));
    }
}
