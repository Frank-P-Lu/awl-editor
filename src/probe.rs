//! `--live-script` drives real-window input and compositor screenshots. It covers
//! cache, redraw, and presentation faults unavailable to offscreen capture.
//!
//! ## Grammar (deliberately dumb)
//!
//! `--live-script "<step>; <step>; ..."` — semicolon-separated steps:
//!   - `keys <chordspec>` — space-separated chords fed through the REAL keymap
//!     path exactly as keystrokes would (same dispatch tail as
//!     `WindowEvent::KeyboardInput`; see `App::dispatch_pressed_key`). Chords
//!     within one `keys` step are posted back-to-back (a burst); use `sleep`
//!     between steps to dwell.
//!   - `sleep <ms>` — the driver thread pauses; the app runs its normal live
//!     loop (debounces fire, frames present) for that long.
//!   - `move <x> <y>` — move the pointer to PHYSICAL (x, y) through the real
//!     `on_cursor_moved`; while a picker is open this HOVER-previews the row
//!     under the cursor (a hover SWEEP is many `move`s with `sleep`s between —
//!     the dense `CursorMoved` stream no keyboard burst reproduces).
//!   - `wheel <n>` — mouse wheel by n notches (wheel-up positive) through the
//!     real `on_mouse_wheel`; an open picker advances + previews, coordinate-free.
//!   - `shot <name>` — screenshot the real window into `<shots-dir>/<name>.png`
//!     (`--live-shots DIR`, default the system temp dir). Every shot prints one
//!     `LIVE-PROBE shot …` stdout line, tailed with this window's own
//!     `surface=WxH dpi=S` — the canvas the script renders its reference on.
//!   - `quit` — clean exit through the same `Action::Quit` a Cmd-Q takes.
//!     Appended automatically if the script doesn't end with one, so a probe
//!     run always terminates.
//!
//! ## Capture gate + isolation
//!
//! Native-live-only, exactly like the daemon: the flag exists only on
//! `Mode::Windowed`, the driver spawns only inside `crate::app::run`, and no
//! headless `--screenshot`/`--keys` path can ever reach it. The wrapping
//! script (`scripts/live-probe.sh`) points `HOME`/`XDG_CONFIG_HOME`/
//! `XDG_DATA_HOME` at a temp dir so a probe run can never touch the user's
//! real config/session/history — and `app::run` additionally skips the
//! single-instance daemon entirely when a live script is armed, so a probe can
//! never hand its file off to (or hijack the socket of) the user's real
//! running instance, even when launched without the wrapper.

// The TYPES + parser below are portable (so `Mode::Windowed` can carry the
// field on every target); the DRIVER thread and the capture backend — the
// parts that touch an OS — are native-only (`spawn_driver`) and macOS-only
// (`cgshot`). The wasm build parses no CLI, so `LiveScript` is never
// constructed there — the same "field exists, value never does" shape as
// `wait`.

use std::path::PathBuf;

use anyhow::{Result, bail};

/// One parsed `--live-script` step. See the module doc for the grammar.
#[derive(Debug, Clone, PartialEq)]
pub enum Step {
    /// Feed these chords through the real keymap dispatch, back-to-back.
    Keys(Vec<crate::keyspec::Chord>),
    /// Driver-side pause (ms) while the app's live loop runs normally.
    Sleep(u64),
    /// Screenshot the real window to `<shots-dir>/<name>.png`.
    Shot(String),
    /// Move the pointer to PHYSICAL (x, y) — the real `on_cursor_moved` seam, so
    /// an open picker HOVER-previews the row under the cursor exactly like a live
    /// mouse move (`overlay_hover` → `retint_theme_preview`). A hover SWEEP is
    /// many `move` steps with small `sleep`s between (the dense `CursorMoved`
    /// stream a real sweep produces, which no keyboard burst reproduces).
    MouseMove(f64, f64),
    /// Mouse WHEEL by N notches (sign = direction, wheel-up positive) — the real
    /// `on_mouse_wheel` seam; an open picker advances its selection + previews
    /// (`overlay_wheel` → `retint_theme_preview`), coordinate-free.
    Wheel(f32),
    /// Print the accumulated THEME-PICKER MOVEMENT-LATENCY distribution
    /// (event → first presented frame) as one `LIVE-PROBE latency …` stdout line,
    /// the live companion to the offscreen `--bench-theme-burst` — this one times
    /// the REAL end-to-end path (dispatch, relayout, encode/submit, and the actual
    /// compositor present), not the pipeline's reshape cost alone. A no-op-safe
    /// report when no movement has been sampled yet.
    Latency,
    /// Clean exit via `Action::Quit`.
    Quit,
}

/// The whole armed probe: parsed steps + where shots land. The type is PORTABLE
/// (so `Mode::Windowed` can carry the `Option<LiveScript>` field on every target,
/// the "field exists, value never does" shape shared with `wait`); the fields are
/// only READ by the native driver (`spawn_driver`), so on wasm — where no
/// `LiveScript` is ever constructed — they are legitimately dead.
#[cfg_attr(target_arch = "wasm32", allow(dead_code))]
#[derive(Debug, Clone)]
pub struct LiveScript {
    pub steps: Vec<Step>,
    pub shots_dir: PathBuf,
}

/// What the driver thread posts into the winit loop (via `EventLoopProxy`,
/// the daemon's own precedent — never cross-thread `App` access). `Sleep`
/// never crosses the channel: the driver sleeps on its own thread. Native-only:
/// the driver + the winit-side handler are both native, so the wasm build (which
/// never arms a probe) never names this type.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, Clone)]
pub enum ProbeEvent {
    /// One chord: dispatched through the same tail a real key press takes.
    Chord(crate::keyspec::Chord),
    /// Screenshot the real window to this exact path (main-thread capture).
    Shot(PathBuf),
    /// Move the pointer to PHYSICAL (x, y) through the real `on_cursor_moved`.
    MouseMove(f64, f64),
    /// Mouse wheel by N notches through the real `on_mouse_wheel`.
    Wheel(f32),
    /// Print the movement-latency distribution (main-thread, so it can
    /// read + format the samples without crossing a lock into the driver thread).
    Latency,
    /// Clean exit through `Action::Quit`.
    Quit,
}

/// THE PROBE-MODE PROCESS GLOBAL: `true` iff this launch armed `--live-script`.
/// Set ONCE in `crate::app::run` before any GPU exists; read by `Gpu::new`
/// (adds `COPY_SRC` to the surface usage) and `Gpu::redraw` (mirrors every
/// PRESENTED frame into the probe's shot texture). `false` on every other
/// launch, keeping the production surface config byte-identical. Mirrors the
/// `debug::debug_on` process-global precedent.
#[cfg(not(target_arch = "wasm32"))]
static LIVE_ACTIVE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[cfg(not(target_arch = "wasm32"))]
pub fn set_live_active() {
    LIVE_ACTIVE.store(true, std::sync::atomic::Ordering::Relaxed);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn live_active() -> bool {
    LIVE_ACTIVE.load(std::sync::atomic::Ordering::Relaxed)
}

/// THE FLIGHT RECORDER (native live-App only) — the user's black box for the
/// live "the page vanishes while previewing a theme" report. That bug is a
/// present/compositor race: awl renders the correct frame but the macOS
/// window-server shows a stale/blank drawable, so a readback of OUR OWN frame
/// (the probe mirror) would look fine — the diagnostic signal is the PRESENT
/// PATH itself (was the frame presented or skipped? was the transaction bracket
/// armed? did a redraw get scheduled?), not a pixel of our own render. The
/// vanish also will not reproduce under the automated probe (its non-key window
/// is unfocused, so the ambient tick pauses and present races differ), while the
/// user reproduces it constantly on their real focused window — so the honest
/// tool is to hand them the recorder and read the trace of the next repro.
///
/// Armed by `AWL_FLIGHT_RECORDER=<path>` at launch (`init_flight`, called once
/// from `crate::app::run`, the ONE native live door — a headless capture never
/// reaches it, mirroring the daemon/probe capture gate). When armed, every
/// diagnostic `trace` line ALSO appends to that file (flushed per line, so a
/// crash/force-quit keeps the black box), and the `recording`-gated trace points
/// across the app fire in the NORMAL live session, not just under the probe.
/// Absent env = a total no-op, production byte-identical.
#[cfg(not(target_arch = "wasm32"))]
static FLIGHT_ACTIVE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[cfg(not(target_arch = "wasm32"))]
static FLIGHT_SINK: std::sync::Mutex<Option<std::io::BufWriter<std::fs::File>>> =
    std::sync::Mutex::new(None);

/// Process-start monotonic anchor: each flight line is stamped `+<ms>` since this,
/// so present gaps (the vanish signature) read directly off the log while the
/// header carries the wall-clock start for correlating with the user's "it
/// vanished at HH:MM".
#[cfg(not(target_arch = "wasm32"))]
static FLIGHT_START: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();

/// Arm the flight recorder from `AWL_FLIGHT_RECORDER` if the user set it. Opens the
/// file in APPEND mode (a session adds to the black box, never truncates a prior
/// repro). A missing/empty var or an open failure leaves the recorder OFF — never
/// blocks launch. Idempotent-safe (re-arming just replaces the sink).
#[cfg(not(target_arch = "wasm32"))]
pub fn init_flight() {
    let Some(path) = std::env::var_os("AWL_FLIGHT_RECORDER") else {
        return;
    };
    if path.is_empty() {
        return;
    }
    arm_flight(std::path::Path::new(&path));
}

/// The env-free arming core (so a test can drive it without mutating `std::env` —
/// the `set_var`/`var` data-race hazard). Opens the black box in APPEND mode and
/// writes a header line stamping the build + wall-clock start.
#[cfg(not(target_arch = "wasm32"))]
fn arm_flight(path: &std::path::Path) {
    match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        Ok(f) => {
            let _ = FLIGHT_START.set(std::time::Instant::now());
            if let Ok(mut sink) = FLIGHT_SINK.lock() {
                *sink = Some(std::io::BufWriter::new(f));
            }
            FLIGHT_ACTIVE.store(true, std::sync::atomic::Ordering::Relaxed);
            let wall = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);
            trace(format_args!(
                "=== flight-recorder armed (awl {}, pid {}, unix {wall}) ===",
                env!("CARGO_PKG_VERSION"),
                std::process::id(),
            ));
        }
        Err(e) => eprintln!("awl: AWL_FLIGHT_RECORDER open failed ({e}); flight recorder off"),
    }
}

/// TEST-ONLY: arm the flight recorder at `path` and, [`disarm_flight_for_test`],
/// put it away again. The two doors exist so a law can READ the trace this file
/// writes instead of re-describing it — the event→present chain is
/// asserted over the recorder's own lines, which is the only oracle that cannot
/// be satisfied by a parallel reimplementation of the chain. Both statics are
/// process-global, so every caller holds `crate::testlock::serial()`.
#[cfg(all(test, not(target_arch = "wasm32")))]
pub(crate) fn arm_flight_for_test(path: &std::path::Path) {
    arm_flight(path);
}

#[cfg(all(test, not(target_arch = "wasm32")))]
pub(crate) fn disarm_flight_for_test() {
    FLIGHT_ACTIVE.store(false, std::sync::atomic::Ordering::Relaxed);
    if let Ok(mut sink) = FLIGHT_SINK.lock() {
        *sink = None;
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn flight_active() -> bool {
    FLIGHT_ACTIVE.load(std::sync::atomic::Ordering::Relaxed)
}

/// Should the `recording`-gated diagnostic trace points fire? True under EITHER
/// the automated live PROBE (`--live-script`) OR the user's FLIGHT RECORDER
/// (`AWL_FLIGHT_RECORDER`). The vanish-hunt trace points guard on THIS (not the
/// narrower `live_active`) so the same well-placed seams serve both readers —
/// one set of trace points, two consumers, never a parallel copy.
#[cfg(not(target_arch = "wasm32"))]
pub fn recording() -> bool {
    live_active() || flight_active()
}

/// The LIVE PROBE window's fixed LOGICAL size (px): small + corner-anchored (see
/// the window-attrs branch in `App::resumed`), so a probe window never sits
/// center-stage stealing the eye — the companion to the Prohibited activation
/// policy (`crate::app::run`) that keeps it from stealing keyboard FOCUS. The
/// wrapping script (`scripts/live-probe.sh`) sizes the probe WINDOW from this and
/// KEEPS IT IN LOCKSTEP as its own `PROBE_CANVAS` (law-tested in `tests`). It is
/// NOT what that script renders its REFERENCES at — see its `ref_for`: layout is
/// not dpi-invariant, and a replay reference never gets the LAUNCH ZOOM.
#[cfg(not(target_arch = "wasm32"))]
pub const PROBE_LOGICAL_W: f64 = 900.0;
#[cfg(not(target_arch = "wasm32"))]
pub const PROBE_LOGICAL_H: f64 = 600.0;

/// Prototype-only window-size override for live affordance galleries whose
/// subject lives in the page margin and is intentionally hidden at the normal
/// compact probe width. The default remains the fixed 900×600 contract above;
/// ordinary launches and every offscreen capture never call this function
/// because `App::resumed` reads it only inside the `live_active()` branch.
#[cfg(not(target_arch = "wasm32"))]
pub fn probe_logical_size() -> (f64, f64) {
    std::env::var("AWL_PROBE_WINDOW_SIZE")
        .ok()
        .and_then(|raw| {
            let (w, h) = raw.split_once('x')?;
            let w = w.parse::<f64>().ok()?;
            let h = h.parse::<f64>().ok()?;
            (w >= 640.0 && h >= 400.0).then_some((w, h))
        })
        .unwrap_or((PROBE_LOGICAL_W, PROBE_LOGICAL_H))
}

/// ONE owner of the `PROBE-TRACE …` diagnostic line — the present/crossing/move
/// trace the vanish hunt reads (stamped with a wall-clock `Instant` so the
/// ordering of retint → present-txn → present → settle is legible in the log).
/// Call sites guard on [`live_active`] BEFORE building the `format_args!` (so a
/// normal launch pays nothing), then route the actual print through here — which
/// keeps every trace print in THIS file, so the println-audit (`println_audit`)
/// has exactly one site to account for instead of a scatter across the app
/// modules. stderr, so it never mixes with the `LIVE-PROBE …` stdout protocol
/// the wrapping script asserts on.
#[cfg(not(target_arch = "wasm32"))]
pub fn trace(args: std::fmt::Arguments) {
    // The PROBE reads its ordering off stderr (`PROBE-TRACE …`); only the live
    // probe prints there, so a flight-recorder-only session stays silent on the
    // terminal (the user's normal editor must not spew).
    if live_active() {
        eprintln!("PROBE-TRACE {args} t={:?}", std::time::Instant::now());
    }
    // The FLIGHT RECORDER appends the same line to the user's file, stamped `+<ms>`
    // since arm so present gaps read directly. Flushed per line so a force-quit
    // mid-repro keeps the tail. A poisoned lock or write error is swallowed —
    // diagnostics must never crash the editor.
    if flight_active()
        && let Ok(mut guard) = FLIGHT_SINK.lock()
        && let Some(w) = guard.as_mut()
    {
        use std::io::Write;
        let ms = FLIGHT_START
            .get()
            .map(|s| s.elapsed().as_millis())
            .unwrap_or(0);
        let _ = writeln!(w, "+{ms}ms {args}");
        let _ = w.flush();
    }
}

/// MOVEMENT-LATENCY SAMPLES (native live-only): a FIFO of pending marks, one
/// `Instant` pushed per THEME-PICKER movement step that begins its real
/// relayout work ([`mark_movement_input`], called from
/// `App::retint_theme_preview` — the ONE owner every input kind, keyboard nav /
/// mouse hover / wheel, funnels a world-changing preview through) and popped in
/// arrival order as each is closed out against its own FIRST frame actually
/// presented afterward ([`note_presented_frame`], called from `Gpu::redraw` at
/// the exact point the existing `"present"` trace already fires). Every preview
/// step unconditionally reshapes and presents on its own turn (no deferred or
/// coalesced settle), so every mark has exactly one present to pair with, in
/// the same order they armed.
#[cfg(not(target_arch = "wasm32"))]
static LATENCY_PENDING: std::sync::Mutex<std::collections::VecDeque<std::time::Instant>> =
    std::sync::Mutex::new(std::collections::VecDeque::new());

/// The accumulated samples, in whole nanoseconds (event → first presented frame),
/// one per theme-picker movement step that actually got a frame out the door.
#[cfg(not(target_arch = "wasm32"))]
static LATENCY_SAMPLES: std::sync::Mutex<Vec<u128>> = std::sync::Mutex::new(Vec::new());

/// Arm the latency clock for ONE theme-picker movement step. A cheap no-op unless
/// [`recording`] — the same gate every other diagnostic trace point uses, so a
/// plain launch never even reads the clock. Pushed onto the pending queue rather
/// than overwriting a single slot: an overwriting single slot collapses an
/// N-step burst to a single sample (`n=1`) regardless of N, which is not a
/// harmless simplification — a burst's true count is exactly the thing this
/// distribution exists to report. There is no reason for one mark to evict
/// another: every step gets its own present (see [`LATENCY_PENDING`]'s doc), so
/// every step earns its own sample.
#[cfg(not(target_arch = "wasm32"))]
pub fn mark_movement_input() {
    if !recording() {
        return;
    }
    if let Ok(mut pending) = LATENCY_PENDING.lock() {
        pending.push_back(std::time::Instant::now());
    }
}

/// Close out the OLDEST pending movement mark against a frame that was just
/// PRESENTED (called unconditionally from `Gpu::redraw`'s present point —
/// cheap, and a no-op both when nothing is pending — an ordinary frame
/// unrelated to any picker movement — and when recording is off). FIFO pop, so
/// marks and presents pair in arrival order even under a fast burst. Pushes the
/// elapsed duration into [`LATENCY_SAMPLES`] and traces it via the shared
/// [`trace`] door.
#[cfg(not(target_arch = "wasm32"))]
pub fn note_presented_frame() {
    if !recording() {
        return;
    }
    let armed = LATENCY_PENDING.lock().ok().and_then(|mut g| g.pop_front());
    if let Some(t0) = armed {
        let ns = t0.elapsed().as_nanos();
        if let Ok(mut samples) = LATENCY_SAMPLES.lock() {
            samples.push(ns);
        }
        trace(format_args!("movement-latency {:.3}ms", ns as f64 / 1.0e6));
    }
}

/// Format the collected movement-latency distribution — count + min/p50/p95/max
/// in milliseconds — or `None` when nothing has been recorded yet. Sorts a COPY
/// (never drains `LATENCY_SAMPLES`), so a script that reports mid-run keeps
/// accumulating and a later report reflects the running total.
#[cfg(not(target_arch = "wasm32"))]
pub fn latency_distribution() -> Option<String> {
    let mut ns: Vec<u128> = LATENCY_SAMPLES.lock().ok()?.clone();
    if ns.is_empty() {
        return None;
    }
    ns.sort_unstable();
    let n = ns.len();
    let ms = |v: u128| v as f64 / 1.0e6;
    let pctl = |p: f64| ms(ns[(((n - 1) as f64 * p).round() as usize).min(n - 1)]);
    Some(format!(
        "n={n} min={:.3}ms p50={:.3}ms p95={:.3}ms max={:.3}ms",
        ms(ns[0]),
        pctl(0.5),
        pctl(0.95),
        ms(ns[n - 1]),
    ))
}

/// Parse the `--live-script` grammar. A malformed step names itself in the
/// error (this is our own harness input — fail fast, the lenient-user-config
/// posture does not apply). Appends a trailing [`Step::Quit`] when absent so a
/// probe run always terminates.
pub fn parse_script(spec: &str) -> Result<Vec<Step>> {
    let mut steps = Vec::new();
    for raw in spec.split(';') {
        let s = raw.trim();
        if s.is_empty() {
            continue;
        }
        let (verb, rest) = match s.split_once(char::is_whitespace) {
            Some((v, r)) => (v, r.trim()),
            None => (s, ""),
        };
        match verb {
            "keys" => {
                if rest.is_empty() {
                    bail!("--live-script: `keys` needs a chord spec (e.g. \"keys Cmd-T Down\")");
                }
                steps.push(Step::Keys(crate::keyspec::parse_chords(rest)?));
            }
            "sleep" => {
                let ms: u64 = rest.parse().map_err(|_| {
                    anyhow::anyhow!("--live-script: `sleep` needs ms, got {rest:?}")
                })?;
                steps.push(Step::Sleep(ms));
            }
            "shot" => {
                if rest.is_empty()
                    || !rest
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
                {
                    bail!("--live-script: `shot` needs a [A-Za-z0-9._-] name, got {rest:?}");
                }
                steps.push(Step::Shot(rest.to_string()));
            }
            "move" => {
                let mut it = rest.split_whitespace();
                let (x, y) = (it.next(), it.next());
                match (
                    x.and_then(|s| s.parse::<f64>().ok()),
                    y.and_then(|s| s.parse::<f64>().ok()),
                ) {
                    (Some(x), Some(y)) if it.next().is_none() => steps.push(Step::MouseMove(x, y)),
                    _ => bail!(
                        "--live-script: `move` needs PHYSICAL x y (e.g. \"move 900 640\"), got {rest:?}"
                    ),
                }
            }
            "wheel" => {
                let n: f32 = rest
                    .parse()
                    .map_err(|_| anyhow::anyhow!("--live-script: `wheel` needs a notch count (e.g. \"wheel -2\"), got {rest:?}"))?;
                steps.push(Step::Wheel(n));
            }
            "latency" => steps.push(Step::Latency),
            "quit" => steps.push(Step::Quit),
            other => bail!(
                "--live-script: unknown step {other:?} (keys|sleep|shot|move|wheel|latency|quit)"
            ),
        }
    }
    if steps.is_empty() {
        bail!("--live-script: empty script");
    }
    if steps.last() != Some(&Step::Quit) {
        steps.push(Step::Quit);
    }
    Ok(steps)
}

/// Spawn the driver thread: wait for the app's ready signal (the first
/// GPU-ready, sent by `App::on_gpu_ready`), then walk the steps — sleeping
/// locally, posting everything else into the winit loop through `post` (a
/// `EventLoopProxy::send_event` wrapper; returns `false` once the loop is
/// gone, which ends the walk). The extra settle sleep after the ready signal
/// gives the very first frame time to present before any scripted input.
#[cfg(not(target_arch = "wasm32"))]
pub fn spawn_driver(
    script: LiveScript,
    ready: std::sync::mpsc::Receiver<()>,
    post: impl Fn(ProbeEvent) -> bool + Send + 'static,
) {
    std::thread::Builder::new()
        .name("awl-live-probe".into())
        .spawn(move || {
            if ready
                .recv_timeout(std::time::Duration::from_secs(15))
                .is_err()
            {
                eprintln!("LIVE-PROBE error: app never signalled ready; quitting");
                let _ = post(ProbeEvent::Quit);
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(300));
            for step in script.steps {
                let ok = match step {
                    Step::Sleep(ms) => {
                        std::thread::sleep(std::time::Duration::from_millis(ms));
                        true
                    }
                    Step::Keys(chords) => chords.into_iter().all(|c| post(ProbeEvent::Chord(c))),
                    Step::MouseMove(x, y) => post(ProbeEvent::MouseMove(x, y)),
                    Step::Wheel(n) => post(ProbeEvent::Wheel(n)),
                    Step::Shot(name) => post(ProbeEvent::Shot(
                        script.shots_dir.join(format!("{name}.png")),
                    )),
                    Step::Latency => post(ProbeEvent::Latency),
                    Step::Quit => {
                        let _ = post(ProbeEvent::Quit);
                        return;
                    }
                };
                if !ok {
                    return; // event loop closed underneath us
                }
            }
        })
        .expect("spawn live-probe driver thread");
}

// --- The compositor-side window capture (macOS) -------------------------------
//
// `CGWindowListCreateImage` asks the WINDOW SERVER for its current composited
// image of ONE window — our own. Capturing your own process's windows is
// exempt from the Screen Recording TCC permission (the restriction guards
// OTHER apps' content), so this needs no grant, no prompt, and it reads the
// pixels the compositor is actually holding — which is exactly where the
// "page vanishes" class of bug lives. Deprecated API (macOS 14+ points at
// ScreenCaptureKit), but SCK requires the TCC grant even for self-capture;
// this stays the right tool for a self-inspecting harness. A plain C API, so
// declared here directly rather than growing `mac_chrome.rs`'s objc2 surface
// (only the NSWindow number lookup lives there).

#[cfg(target_os = "macos")]
mod cgshot {
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct CGPoint {
        x: f64,
        y: f64,
    }
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct CGSize {
        w: f64,
        h: f64,
    }
    #[repr(C)]
    #[derive(Clone, Copy)]
    struct CGRect {
        origin: CGPoint,
        size: CGSize,
    }

    // kCGWindowListOptionIncludingWindow = 1 << 3 (capture exactly this window).
    const INCLUDING_WINDOW: u32 = 1 << 3;
    // kCGWindowImageBoundsIgnoreFraming (1<<0): no shadow/framing effects;
    // kCGWindowImageBestResolution (1<<3): native (retina) resolution.
    const IMAGE_OPTS: u32 = (1 << 0) | (1 << 3);

    #[link(name = "CoreGraphics", kind = "framework")]
    unsafe extern "C" {
        static CGRectNull: CGRect;
        fn CGWindowListCreateImage(
            bounds: CGRect,
            list_option: u32,
            window_id: u32,
            image_option: u32,
        ) -> *mut core::ffi::c_void; // CGImageRef
        fn CGImageGetWidth(image: *mut core::ffi::c_void) -> usize;
        fn CGImageGetHeight(image: *mut core::ffi::c_void) -> usize;
        fn CGImageGetBytesPerRow(image: *mut core::ffi::c_void) -> usize;
        fn CGImageGetBitsPerPixel(image: *mut core::ffi::c_void) -> usize;
        fn CGImageGetBitmapInfo(image: *mut core::ffi::c_void) -> u32;
        fn CGImageGetDataProvider(image: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
        fn CGDataProviderCopyData(provider: *mut core::ffi::c_void) -> *mut core::ffi::c_void; // CFDataRef
    }
    #[link(name = "CoreFoundation", kind = "framework")]
    unsafe extern "C" {
        fn CFDataGetBytePtr(data: *mut core::ffi::c_void) -> *const u8;
        fn CFDataGetLength(data: *mut core::ffi::c_void) -> isize;
        fn CFRelease(cf: *mut core::ffi::c_void);
    }

    /// Ask the window server for its composited image of `window_id` as RGBA.
    /// Returns a short human error on any failure. NOTE: on a machine without
    /// the Screen Recording TCC grant, macOS quietly hands back a tiny generic
    /// PLACEHOLDER thumbnail instead of the window's pixels (observed
    /// empirically: ~194x192 white card for a 2400x1664 window) — the CALLER
    /// must validate the returned dimensions against the real surface size and
    /// fall back to the frame mirror on a mismatch (`App::probe_shot`).
    pub fn capture_window_image(window_id: u32) -> Result<image::RgbaImage, String> {
        // SAFETY: plain C calls; every CF object created here is released on
        // every path before return, and the byte slice is copied out before
        // its owning CFData is released.
        unsafe {
            let image =
                CGWindowListCreateImage(CGRectNull, INCLUDING_WINDOW, window_id, IMAGE_OPTS);
            if image.is_null() {
                return Err("CGWindowListCreateImage returned null (window gone?)".into());
            }
            let (w, h) = (CGImageGetWidth(image), CGImageGetHeight(image));
            let stride = CGImageGetBytesPerRow(image);
            let bpp = CGImageGetBitsPerPixel(image);
            let info = CGImageGetBitmapInfo(image);
            let provider = CGImageGetDataProvider(image);
            if provider.is_null() || w == 0 || h == 0 || bpp != 32 {
                CFRelease(image);
                return Err(format!("unusable window image ({w}x{h}, {bpp}bpp)"));
            }
            let data = CGDataProviderCopyData(provider);
            if data.is_null() {
                CFRelease(image);
                return Err("CGDataProviderCopyData returned null".into());
            }
            let len = CFDataGetLength(data) as usize;
            let bytes = std::slice::from_raw_parts(CFDataGetBytePtr(data), len);
            // Window-server images are 32bpp; byte order little (kCGBitmapByteOrder32Little,
            // 2 << 12) means BGRA in memory, otherwise ARGB (alpha-first big-endian).
            let little = (info & (3 << 12)) == (2 << 12);
            let mut rgba = vec![0u8; w * h * 4];
            for y in 0..h {
                let row = &bytes[y * stride..y * stride + w * 4];
                for x in 0..w {
                    let px = &row[x * 4..x * 4 + 4];
                    let (r, g, b, a) = if little {
                        (px[2], px[1], px[0], px[3])
                    } else {
                        (px[1], px[2], px[3], px[0])
                    };
                    let o = (y * w + x) * 4;
                    rgba[o..o + 4].copy_from_slice(&[r, g, b, a]);
                }
            }
            CFRelease(data);
            CFRelease(image);
            image::RgbaImage::from_raw(w as u32, h as u32, rgba)
                .ok_or_else(|| "rgba buffer size mismatch".to_string())
        }
    }
}

#[cfg(target_os = "macos")]
pub use cgshot::capture_window_image;

#[cfg(test)]
mod tests;
