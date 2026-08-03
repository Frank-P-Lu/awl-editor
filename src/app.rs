use crate::clock::Instant;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

#[cfg(test)]
mod crossing;
#[cfg(test)]
mod present_txn;

#[cfg(not(target_arch = "wasm32"))]
use arboard::Clipboard;
#[cfg(target_arch = "wasm32")]
use web_clipboard::Clipboard;

#[cfg(target_arch = "wasm32")]
mod web_clipboard {
    //! Best-effort async browser clipboard adapter. Copy never blocks; paste
    //! remains internal because browser reads require unreliable user activation.
    pub struct Clipboard;

    impl Clipboard {
        pub fn new() -> Result<Self, &'static str> {
            Ok(Self)
        }

        pub fn set_text(&mut self, text: String) -> Result<(), &'static str> {
            let Some(window) = web_sys::window() else {
                return Err("no window (headless/detached wasm context)");
            };
            let clipboard = window.navigator().clipboard();
            let promise = clipboard.write_text(&text);
            wasm_bindgen_futures::spawn_local(async move {
                let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
            });
            Ok(())
        }

        pub fn get_text(&mut self) -> Result<String, &'static str> {
            Err("clipboard read unavailable on web (see WEB.md)")
        }
    }
}

const AUTOSAVE_DEBOUNCE: Duration = Duration::from_millis(400);

/// Idle delay before atomic document autosave; blur, switch, and quit flush.
const AUTOSAVE_IDLE: Duration = Duration::from_secs(1);

const TOAST_LIFETIME: Duration = Duration::from_millis(2500);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum NoticeKind {
    Toast,
    #[default]
    Sticky,
}

const ZOOM_PERSIST_DEBOUNCE: Duration = Duration::from_millis(500);

/// The LEADING-EDGE-PLUS-TRAILING-COALESCE window shared by both halves of
/// `theme_font_debounce::theme_font_reshape_decision` (that module's own doc
/// has the full mechanism story) — the isolated-vs-burst cooldown AND the
/// trailing coalesce duration are the SAME window. Shared by every input
/// modality that funnels through `App::retint_theme_preview` (keyboard nav,
/// pointer hover, wheel). `AWL_THEME_FONT_DEBOUNCE_MS` is the A/B override;
/// see docs/fonts.md for the live before/after numbers on both axes.
const THEME_FONT_DEBOUNCE_DEFAULT_MS: u64 = 100;

// A reversion to 0 fails the BUILD (item 202); `theme_debounce_item202.rs`
// covers the non-const half — the real, env-overridable predicate also
// doesn't fire at elapsed 0.
const _: () = assert!(
    THEME_FONT_DEBOUNCE_DEFAULT_MS > 0,
    "item 202: must be nonzero"
);

fn parse_theme_font_debounce_ms(raw: Option<&str>) -> u64 {
    raw.filter(|s| !s.is_empty())
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(THEME_FONT_DEBOUNCE_DEFAULT_MS)
}

fn theme_font_debounce() -> Duration {
    static ONCE: std::sync::OnceLock<u64> = std::sync::OnceLock::new();
    let ms = *ONCE.get_or_init(|| {
        parse_theme_font_debounce_ms(std::env::var("AWL_THEME_FONT_DEBOUNCE_MS").ok().as_deref())
    });
    Duration::from_millis(ms)
}

#[cfg(test)]
#[test]
fn theme_font_debounce_ms_env_parse() {
    let default = THEME_FONT_DEBOUNCE_DEFAULT_MS;
    assert_eq!(parse_theme_font_debounce_ms(None), default);
    assert_eq!(parse_theme_font_debounce_ms(Some("")), default);
    // An explicit "0" is still honored as a real override (the A/B escape
    // hatch item 202's own measurement rode) — only an ABSENT/empty/garbage
    // value falls back to the default, never a deliberate zero.
    assert_eq!(parse_theme_font_debounce_ms(Some("0")), 0);
    assert_eq!(parse_theme_font_debounce_ms(Some("150")), 150);
    assert_eq!(parse_theme_font_debounce_ms(Some("garbage")), default);
}

/// AMBIENT LAVA TICK period — the lava-lamp ground's slow drift cadence
/// (`crate::lava::LAVA_TICK_MS`). A single `WaitUntil` this far out in
/// `about_to_wait` advances the phase + requests one redraw + re-arms, so a lava
/// world costs ~10 sparse frames/sec (NEVER the caret spring's hot per-frame
/// loop), and a static world costs zero (the tick never arms). See
/// `App::tick_lava`.
const LAVA_TICK: Duration = Duration::from_millis(crate::lava::LAVA_TICK_MS);

/// Quiet period after the last LIVE-RESIZE `Resized` tick before the macOS
/// Core-Animation-transaction present sync (`Gpu::set_presents_with_
/// transaction`) is flipped back OFF (debounce; macOS-only — see
/// `resize_settle_at`'s doc for the full mechanism). A fast drag re-stamps the
/// deadline on every tick (`App::arm_live_resize_sync`), so this only fires
/// once the drag genuinely stops. TASTE TUNABLE (independent of the theme-font
/// debounce below — the two diverged at item 202, which lowered the theme
/// debounce off this same 150ms without this one needing to follow): short
/// enough that the transaction-sync cost (Apple's own documented throughput
/// trade-off for `presentsWithTransaction`) is paid only while actually
/// dragging, long enough that a brief pause mid-drag doesn't flap it on/off.
const RESIZE_SYNC_SETTLE: Duration = Duration::from_millis(150);

/// Quiet period after the last `Moved` tick before the MOVE stream is considered
/// settled: the lamp resumes, ONE settle redraw fires, and presents go back to
/// async (`App::finish_move_settle`). DELIBERATELY LONGER than
/// `RESIZE_SYNC_SETTLE`: a resize stream's ticks stop exactly when the drag
/// stops, but a MOVE stream's quiet gaps include mid-drag stationary HOLDS with
/// the title bar still grabbed — at the old 150ms, a hesitation un-paused the
/// lamp mid-grab, and the resumed ambient presents raced the window-server's
/// move transaction the instant the drag continued (the "flash while moving is
/// kinda back" report, 2026-07-15 — the same compositor-race class the
/// resize-stretch fix closed). One second outlasts an ordinary hesitation; the
/// lamp drifts so slowly (~67 s loop) that the longer hold is imperceptible.
/// TASTE TUNABLE — flagged for live review.
const MOVE_SETTLE: Duration = Duration::from_millis(1000);

/// Quiet period a THEME-PREVIEW lava-boundary crossing keeps the present-
/// transaction sync armed (debounce; macOS-only effect). When a preview step
/// swaps a ticking lava world for a static non-lava one (or back), the ~10 fps
/// ambient present cadence starts/stops underfoot; arming the transaction sync
/// makes the crossing frame (and one settle follow-up) JOIN the compositor's
/// transaction instead of racing it, so the swapchain can't strand a stale
/// drawable — the "writing surface vanishes" report. Each further crossing
/// re-stamps the deadline (`retint_theme_preview`), so a rapid arrow burst
/// through the boundary keeps it armed and settles once you rest — the same
/// single-`WaitUntil` shape as `RESIZE_SYNC_SETTLE` / the theme-font debounce.
/// Sized like `RESIZE_SYNC_SETTLE`: long enough to bracket the crossing frame +
/// its follow-up, short enough that the sync cost is paid only around a crossing.
const CROSSING_SYNC_SETTLE: Duration = Duration::from_millis(150);

use glyphon::Cache;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, Ime, Modifiers, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, ModifiersState};
// Exposes `KeyEvent::key_without_modifiers()` — the logical key BEFORE OS modifier
// composition. Used to undo macOS Option dead-key composition (Option-f -> 'ƒ') for
// Meta chords without breaking Option-accent text input. The trait lives on the
// DESKTOP backends (macOS / Windows / X11 / Wayland); the web backend has no such
// composition layer, so on wasm `key_without_modifiers` falls back to the plain
// logical key (see the cfg-split helper near the bottom of this file).
#[cfg(not(target_arch = "wasm32"))]
use winit::platform::modifier_supplement::KeyEventExtModifierSupplement;
use winit::window::Window;

use crate::actions;
use crate::buffer::Buffer;
use crate::config::Config;
use crate::keymap::Action;
use crate::render::{self, TextPipeline, ViewState};

const MULTICLICK_MS: u64 = 400;
const INITIAL_ZOOM: f32 = 0.8;
const WHEEL_LINES_PER_NOTCH: f32 = 3.0;
const WHEEL_PIXELS_PER_LINE: f32 = 16.0;
/// Physical-px SLOP a text-selection drag must travel past the press position
/// before it arms (extends the selection) — the phantom-selection-click fix. Below
/// this, a `CursorMoved` while `dragging` is pointer jitter (or a WYSIWYG reveal
/// reflow under a stationary pointer) and must not move the cursor away from the
/// press's own hit-test result. Matches the multi-click "same spot" tolerance
/// (`bump_click_count`'s own `4.0`) — both answer "did the pointer really move",
/// just for two different gestures. See `PointerInput::exceeds_drag_slop`.
///
/// `pub(crate)` so `overlay::nav::HOVER_MOVE_SLOP_PX` (item 106) can read this
/// SAME constant rather than declaring its own copy of the number — the two
/// gates answer the identical "did the pointer really move, or did content
/// relocate under a stationary one" question for two different gestures (a
/// text-selection drag arming vs. a picker's hover re-selecting), and awl does
/// not grow two independently-tuned pointer-jitter constants that could drift
/// apart under a future retune of either.
pub(crate) const DRAG_ARM_SLOP_PX: f32 = 4.0;

#[derive(Clone, Copy)]
enum CaretImpact {
    Type,
    Delete,
    Gulp,
    Land,
    Copy,
}

#[derive(Default)]
struct ZoomReflow {
    pending: bool,
}

impl ZoomReflow {
    fn queue(&mut self) {
        self.pending = true;
    }

    fn take(&mut self) -> bool {
        std::mem::take(&mut self.pending)
    }

    fn clear(&mut self) {
        self.pending = false;
    }
}

/// A pending ZOOM ANCHOR: the document char + the screen y that char should hold, so
/// the next `sync_view` (which reshapes to the just-changed zoom) keeps that point
/// fixed on screen instead of anchoring at the viewport top. Captured at the OLD zoom
/// BEFORE the deferred reshape (both zoom paths arm it — the wheel with the POINTER's
/// char + y, the keyboard with the CARET's, or the viewport-centre char when the caret
/// is off-screen), consumed once by `sync_view` via [`TextPipeline::zoom_anchor_scroll`]
/// (the one owner of the anchored-scroll math). Live-only: the headless capture never
/// builds an `App`, so its single-frame scroll stays cursor-follow (unchanged).
#[derive(Clone, Copy, Debug)]
struct ZoomAnchor {
    line: usize,
    col: usize,
    screen_y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GpuLifecycle {
    AwaitingWindow,
    Active { oom_skips: u8 },
    Suspended,
    Rebuilding,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GpuFaultAction {
    RetryOneFrame,
    Rebuild,
    NoticeOnly,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GpuSkipAction {
    WaitForWake,
    RetryAfter(Duration),
    RetryWithNoticeAfter(Duration, &'static str),
    HoldWithNotice(&'static str),
}
const GPU_SURFACE_RETRY: Duration = Duration::from_millis(16);
fn gpu_fault_action(lifecycle: GpuLifecycle, kind: gpu::GpuFaultKind) -> GpuFaultAction {
    match kind {
        gpu::GpuFaultKind::OutOfMemory
            if matches!(lifecycle, GpuLifecycle::Active { oom_skips: 0 }) =>
        {
            GpuFaultAction::RetryOneFrame
        }
        gpu::GpuFaultKind::Validation => GpuFaultAction::NoticeOnly,
        _ => GpuFaultAction::Rebuild,
    }
}
fn gpu_skip_action(skip: gpu::GpuFrameSkip, timeout_streak: u8) -> GpuSkipAction {
    match skip {
        gpu::GpuFrameSkip::Occluded => GpuSkipAction::WaitForWake,
        gpu::GpuFrameSkip::Timeout => {
            GpuSkipAction::RetryAfter(Duration::from_millis(16_u64 << timeout_streak.min(5)))
        }
        gpu::GpuFrameSkip::SurfaceReconfigured => GpuSkipAction::RetryAfter(GPU_SURFACE_RETRY),
        gpu::GpuFrameSkip::SurfaceRecreated => {
            GpuSkipAction::RetryWithNoticeAfter(GPU_SURFACE_RETRY, "graphics surface recovered")
        }
        gpu::GpuFrameSkip::PrepareFailed => {
            GpuSkipAction::HoldWithNotice("graphics skipped one frame — editing is safe")
        }
    }
}
/// THE FOLLOW-UP-FRAME DECISION, and why it takes TWO animation terms.
///
/// `stepped` is what [`crate::render::TextPipeline::advance`] reported — read
/// BEFORE `Gpu::redraw`, because the spring must advance by this frame's `dt`
/// before the frame is drawn. `band_ease_started` is what `prepare` — which runs
/// INSIDE that same `Gpu::redraw` call — did afterwards: the selection band is
/// the one animator whose target is set at draw time rather than at the apply
/// seam, so the frame that starts its ease is invisible to `stepped` by
/// construction (see
/// [`crate::render::TextPipeline::take_band_ease_started`] for the full
/// mechanism and the user-visible symptom). Dropping the second term parks the
/// loop on the very frame an ease began, which is the every-other-input defect.
///
/// `frame_presented` still gates both: a failed acquire must never drive the
/// Poll loop, or an occluded window prepares thousands of unseen frames.
fn keep_gpu_loop_hot(stepped: bool, band_ease_started: bool, frame_presented: bool) -> bool {
    (stepped || band_ease_started) && frame_presented
}
/// Map a live GPU skip cause onto the soak probe's [`crate::soak_gpu::SkipKind`]
/// so each cause is counted SEPARATELY (the collapse into one `skipped` total is
/// what hid the zero-drawable occlusion investigation).
#[cfg(not(target_arch = "wasm32"))]
fn soak_skip_kind(skip: gpu::GpuFrameSkip) -> crate::soak_gpu::SkipKind {
    match skip {
        gpu::GpuFrameSkip::Timeout => crate::soak_gpu::SkipKind::Timeout,
        gpu::GpuFrameSkip::Occluded => crate::soak_gpu::SkipKind::Occluded,
        gpu::GpuFrameSkip::SurfaceReconfigured => crate::soak_gpu::SkipKind::SurfaceReconfigured,
        gpu::GpuFrameSkip::SurfaceRecreated => crate::soak_gpu::SkipKind::SurfaceRecreated,
        gpu::GpuFrameSkip::PrepareFailed => crate::soak_gpu::SkipKind::PrepareFailed,
    }
}
/// `WindowEvent::Occluded`: whether an occlusion CHANGE should schedule a
/// repaint. The GPU skip path parks `Occluded → WaitForWake` with no retry
/// timer, so an un-occlusion (`false`) is the wake that must request a redraw;
/// becoming occluded (`true`) needs nothing — the next acquire returns
/// `Occluded` and re-parks the loop. Pure so it is unit-testable off-window.
fn occluded_change_wants_redraw(occluded: bool) -> bool {
    !occluded
}

struct Gpu {
    instance: wgpu::Instance,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    /// The format the frame is RENDERED through — always the sRGB variant of the
    /// surface's config format. On native it EQUALS `config.format` (the platform
    /// already offers an `*-Srgb` surface format). On the web the canvas only
    /// permits a NON-srgb config format (`bgra8unorm`/`rgba8unorm`; the WebGPU
    /// spec forbids an `*-srgb` primary canvas format), so we configure the base
    /// format, list its srgb variant in `config.view_formats`, and render through
    /// an srgb VIEW — otherwise the shader-linearised grounds/selection/caret get
    /// written WITHOUT the sRGB encode and the whole scene reads too dark (the
    /// margins collapse to near-black). See `Gpu::new`.
    view_format: wgpu::TextureFormat,
    pipeline: TextPipeline,
    window: Arc<Window>,
    #[cfg(not(target_arch = "wasm32"))]
    backend_name: String,
    faults: gpu::GpuFaultInbox,
    #[cfg(not(target_arch = "wasm32"))]
    inject_surface_loss: bool,
    /// LIVE PROBE frame mirror (`--live-script` only, else forever `None`): a
    /// persistent texture every PRESENTED frame is blitted into just before
    /// `present()`, so a probe `shot` can read back what the compositor was
    /// LAST HANDED — without forcing a redraw that would repaint over exactly
    /// the stale-frame / missed-redraw bug classes the probe hunts. See
    /// `Gpu::mirror_presented_frame` + `crate::probe`'s module doc.
    #[cfg(not(target_arch = "wasm32"))]
    probe_mirror: Option<wgpu::Texture>,
    /// LIVE-ONLY (DEBUG): the last presented frame's cost SPLIT — `(prepare_ms,
    /// present_ms)` — recorded by `Gpu::redraw` from the perf stamps it already reads
    /// (no new clock read; `None` when the debug panel is off, or on a skipped frame).
    /// The frame loop reads it right after a settled theme-switch present to attribute
    /// the atlas (prepare) + first-present phases of the settle readout
    /// (`crate::themeswitch`). Off the headless path — `redraw`'s stamps are gated on
    /// `debug_on()`, so a capture never sets it.
    debug_frame_split: Option<(f32, f32)>,
}

struct ThemeSettleInFlight {
    input_at: Instant,
    phases: crate::themeswitch::SwitchPhases,
}

/// The winit USER EVENT type, and the conversions the event loop needs to
/// post one — lifted out so the enum grows without the root file growing.
mod awl_event;
/// GPU surface + frame loop (device/queue/surface, prepare/render).
mod files;
/// Re-exported crate-wide so the render fixtures and the docs-vocabulary law
/// read the SAME constant the running app displays, never a retyped copy.
#[cfg(test)]
pub(crate) use files::CHANGED_ELSEWHERE_NOTICE;
/// The one live owner of frame timing, presentation bookkeeping, render state,
/// surface lifecycle, and notices.
mod frame;
mod gpu;
mod gpu_recovery;
mod input;
mod lifecycle;
mod location;
/// The `about_to_wait` scheduling body: every debounce / settle deadline, the
/// ambient (lava/stars) tick, event-toast expiry, GPU acquire retries + soak
/// drive — one `WaitUntil` each, lifted out of the trait method (a thin
/// delegate now) so the file's #1 collision seam has its own home.
mod schedule;
pub(crate) use awl_event::AwlEvent;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod semantic;
mod startup;
/// ITEM 202's leading-edge-plus-trailing-coalesce rule for the theme-picker
/// preview's deferred font reshape — a pure scheduling decision, extracted
/// out of this file because it needs neither `App` nor a GPU.
mod theme_font_debounce;
mod viewstate;
mod window;
/// The SUMMONED-UI LAYER owner (item 172): the overlay/search/popover
/// precedence ladder, behind one type with private fields.
mod workspace;
#[cfg(any(test, not(target_arch = "wasm32")))]
pub(crate) use schedule::RecordingScheduler;
pub(crate) use theme_font_debounce::{ThemeFontReshapeDecision, theme_font_reshape_decision};
mod apply;
mod apply_context;
/// ITEM 188 — the live `App`'s own SIDECAR FOLD + its capture constructor.
/// Native-only, like the `--screenshot-app` mode that is its only consumer.
#[cfg(not(target_arch = "wasm32"))]
mod capture_state;
mod daemon;
/// The active whole-slot, background registry, previous target, checker, and
/// every buffer-scoped App cache.
mod document;
mod menu;
/// The APP-GLOBAL SAVE LEDGER (item 172): the fresh-document autosave
/// debounce+version pair, the save-feedback clocks, the title dirty cache.
mod persistence;
/// ITEM 183's HEADLESS PRESS DOOR — real chords into the live `App`, off-window.
#[cfg(any(test, not(target_arch = "wasm32")))]
mod press;
mod probe;
/// The one redraw-request door. GPU ownership stays on `App`; callers ask for
/// a frame without reaching through that owner to winit's window verb.
mod redraw;
mod session;
mod stats;
mod streaks;

pub struct App {
    /// One owner for the active whole slot, background registry, previous-file
    /// target, spell checker, and every buffer-scoped cache.
    document: document::DocumentSession,
    /// THE SUMMONED-UI LAYER (item 172's first owner — `app/workspace.rs`):
    /// the modal picker, the find/replace panel, and the format popover's
    /// summon bit, with their PRECEDENCE LADDER. The three former `App`
    /// fields (`overlay`/`search`/`popover_open`) are private to that module
    /// now, so the ladder cannot be re-derived by a consumer — it used to be
    /// the same conjunction hand-written across five files. Item 173 grows the
    /// typed summoned-workspace lifecycle inside this type.
    workspace_state: workspace::WorkspaceState,
    /// THE APP-GLOBAL SAVE LEDGER (item 172's second owner —
    /// `app/persistence.rs`): the fresh-document autosave debounce + the
    /// version it last wrote (one ledger, not two fields), the two
    /// save-feedback clocks, and the window title's dirty cache. The
    /// per-buffer half of saving stays in `files::BufferExtra`, travelling
    /// with the active slot.
    persistence: persistence::PersistenceRuntime,
    /// All live keyboard, IME, pointer, and press→drag→release state. The
    /// typed owner keeps its independent substates from leaking back onto
    /// root `App`; see `app/input/mod.rs`.
    input: input::InputRuntime,
    /// One frame lifecycle. Render-affecting state and the deadlines which
    /// settle it cannot drift as separate root field bags.
    frame: frame::FrameRuntime,
    #[cfg(not(target_arch = "wasm32"))]
    soak: Option<crate::soak_gpu::Controller>,
    #[cfg(not(target_arch = "wasm32"))]
    soak_recovery_pending: Option<crate::soak_gpu::FaultKind>,
    #[cfg(not(target_arch = "wasm32"))]
    soak_passed: Option<bool>,
    /// LIVE PROBE (`--live-script`): the one-shot "first frame is up" signal
    /// the driver thread blocks on before feeding scripted input — sent (and
    /// the sender dropped) from `on_gpu_ready`, alongside a `focus_window()`
    /// so the probe window is frontmost/unoccluded (the wgpu macOS occlusion
    /// tripwire: a non-visible window presents nothing). `None` outside a
    /// probe run — zero cost on a normal launch.
    #[cfg(not(target_arch = "wasm32"))]
    probe_ready: Option<std::sync::mpsc::Sender<()>>,
    clipboard: Option<Clipboard>,
    clipboard_last_written: Option<String>,
    /// The live root plus its derived project state and persisted MRUs.
    project_location: location::ProjectLocation,
    /// Newest unacknowledged crash-log filename. It never occupies the center
    /// notice: About and Settings surface it passively until Report a Problem
    /// acknowledges it.
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    pending_crash: Option<String>,
    /// Persisted configuration plus CLI and first-run folder policy.
    config: location::ConfigurationRuntime,
    /// SESSION RESTORE (native only): the window FRAME a previous session left
    /// (already clamped to whatever screens were connected at THAT save time —
    /// re-clamped again in `resumed()` against the CURRENT screens), applied
    /// once when the window is first created. `None` when there was nothing to
    /// restore (no session file, the kill-switch is off, or this platform never
    /// captures one) — `resumed()` then falls back to the fixed 1200x800
    /// default, unchanged from before this round.
    #[cfg(not(target_arch = "wasm32"))]
    restored_window: Option<crate::session::WindowFrame>,
    /// LIFETIME STATS odometer (native only, `stats` config-gated): the persisted
    /// running counters, loaded once at launch and flushed on the autosave
    /// triggers (idle/blur/switch/quit). Lives ONLY on the live `App` — the
    /// headless capture never constructs one, so `stats.toml` is untouchable
    /// there (tripwire: `headless_replay_never_touches_the_stats_file`).
    #[cfg(not(target_arch = "wasm32"))]
    stats: crate::stats::Stats,
    #[cfg(not(target_arch = "wasm32"))]
    stats_origin: Instant,
    #[cfg(not(target_arch = "wasm32"))]
    stats_last_input_ms: Option<u64>,
    /// The caret's last-sampled DOCUMENT-space position (scroll-independent), for
    /// the caret-travel accumulator — diffed against the current position each
    /// `sync_view`, but only ADDED when the logical cursor actually moved (so a
    /// scroll or a reshape never fakes distance). `None` until the first sample.
    #[cfg(not(target_arch = "wasm32"))]
    stats_last_caret_xy: Option<(f32, f32)>,
    #[cfg(not(target_arch = "wasm32"))]
    stats_last_cursor: Option<(usize, usize)>,
    /// Whether the odometer has unsaved increments since the last flush, so a
    /// flush with nothing new skips the atomic write.
    #[cfg(not(target_arch = "wasm32"))]
    stats_dirty: bool,
    #[cfg(not(target_arch = "wasm32"))]
    streaks: crate::streaks::Streaks,
    /// The active buffer's word count at the last streaks sample — the `last` side
    /// of the per-flush word DELTA. `None` until the first sample of a buffer (a
    /// fresh launch or right after a buffer swap), so opening a file's existing
    /// words is ANCHORED (never counted as "written"); the next flush records the
    /// delta from there. Reset to `None` on every buffer swap (`streaks_reset_baseline`).
    #[cfg(not(target_arch = "wasm32"))]
    streaks_baseline: Option<usize>,
    /// Whether the streaks record has unpersisted changes since the last flush, so
    /// a flush that recorded nothing (an anchor, or a no-net-change idle) skips the
    /// atomic write.
    #[cfg(not(target_arch = "wasm32"))]
    streaks_dirty: bool,
    /// SINGLE-INSTANCE DAEMON (native only, and compiled out under `mas` — see
    /// `crate::daemon`'s module doc): the socket special file's path, so
    /// `daemon::daemon_shutdown` can unlink it on a clean quit — `None` when this
    /// launch never became the instance (a socket error degraded to a normal,
    /// non-singleton launch; see `crate::app::run`).
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "mas")))]
    daemon_socket_path: Option<PathBuf>,
    /// SINGLE-INSTANCE DAEMON (native only, and compiled out under `mas`):
    /// every daemon `--wait` client's still-
    /// open connection, keyed by the [`crate::buffers::BufferKey`] of the buffer it
    /// is waiting on. `Action::FinishBuffer` (Cmd-W) notifies + drains the entry for
    /// the buffer being finished; `daemon::daemon_shutdown` drains everything on
    /// quit (a dropped `Waiter` closes its socket, which the client treats as done
    /// too — see `crate::daemon`'s module doc).
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "mas")))]
    wait_conns: std::collections::HashMap<crate::buffers::BufferKey, Vec<crate::daemon::Waiter>>,
    /// NATIVE MACOS MENU BAR: the event-loop proxy stashed at construction so
    /// `resumed()` can install the menu bar (and register muda's event
    /// handler) once NSApp/the window exists — menu install needs to happen
    /// AFTER window creation, but the proxy is only obtainable in
    /// `crate::app::run`, before control ever reaches `resumed()`. Taken
    /// (`Option::take`) the one time it is used, so a second `resumed()` call
    /// (there isn't one today, but the existing `gpu.is_some()` guard already
    /// covers that) can never double-install. `None` after install, or in any
    /// test build that never goes through `crate::app::run`.
    #[cfg(target_os = "macos")]
    menu_proxy: Option<winit::event_loop::EventLoopProxy<AwlEvent>>,
    /// The installed menu bar's Rust-side handle, kept alive for the app's
    /// whole lifetime. **This field's only job is to never be dropped before
    /// `App` itself is.** `crate::menu::install`'s doc explains why: every
    /// native `NSMenuItem` stashes a raw (non-retaining) pointer back into
    /// this value's owned `Rc<RefCell<MenuChild>>` chain, so letting it drop
    /// (the v1 bug — the return value used to be an unstored local) leaves
    /// every menu item pointing at freed memory, and clicking ANY of them —
    /// About, Quit, a routed item — is a use-after-free. Never read after
    /// `resumed()` stores it; `Option` only so the field can start `None`
    /// before the window/NSApp exist.
    #[cfg(target_os = "macos")]
    _menu_bar: Option<crate::menu::InstalledMenu>,
}

impl App {
    fn new(
        file: Option<PathBuf>,
        root: PathBuf,
        cli_workspace: Option<PathBuf>,
        cli_default_folder: Option<PathBuf>,
        config: Config,
    ) -> Self {
        // ACCESSIBILITY TIER 1 — REDUCE MOTION: resolve the config->OS ladder
        // ONCE, here, at live startup (native + wasm both construct `App`
        // through this one seam). See `motion.rs`'s module doc for the full
        // resolution ladder + the determinism guarantee this call site is the
        // ONLY place in the whole codebase that may consult OS/browser motion
        // detection — never a headless capture path.
        crate::motion::apply_at_startup(&config);
        // ITEM 77 — THE ONE CAPABILITY OWNER: an explicit CLI/OS-open LAUNCH
        // argument that isn't openable text is refused HERE, before it can
        // ever reach `Buffer::from_file` — the SAME door `App::load_path`
        // guards (see `crate::openable`'s module doc). A refusal falls
        // through to the ordinary no-argument path below (scratch/stash
        // restore) exactly as if no file had been named at all; the refusal
        // message surfaces as a sticky notice once `app` exists, below.
        let file_refusal = file
            .as_ref()
            .and_then(|p| crate::openable::classify(p).refusal_message());
        let file = if file_refusal.is_some() { None } else { file };
        // SESSION RESTORE (native only) reads this BEFORE `file` moves into the
        // struct literal below — see `Self::apply_session_restore`'s doc for why
        // a launch WITH a file argument still restores the rest of the session
        // (just never lets it override the active buffer).
        #[cfg(not(target_arch = "wasm32"))]
        let file_arg_given = file.is_some();
        // SCRATCH RESTORE: a no-argument launch resumes the persistent scratch
        // buffer from its stash (written by the autosave engine on idle/blur/
        // quit). Path stays None — still a true scratch, still markdown-first.
        // ONLY the live App restores; the headless `load_buffer` never reads the
        // stash, so a default no-file capture stays byte-identical.
        let stash = crate::fs::scratch_stash_path();
        let buffer = match &file {
            Some(p) => Buffer::from_file(p),
            None => match crate::fs::active().read_to_string(&stash) {
                Ok(s) if !s.is_empty() => Buffer::from_str(&s),
                Ok(_) => Buffer::scratch(), // present but empty: nothing to preserve
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Buffer::scratch(),
                Err(_) => {
                    // PRESERVE-ON-CORRUPT (the scratch stash IS a manuscript):
                    // the file exists but failed to decode as UTF-8 text — a
                    // real corruption signal, not a fresh install. Back up
                    // the raw bytes to a `.corrupt-*` sibling before falling
                    // back to a blank scratch buffer, so those bytes are
                    // never silently discarded (and never overwritten away
                    // by the very next scratch-stash flush).
                    if let Ok(raw) = crate::fs::active().read(&stash) {
                        crate::durable::preserve_corrupt(&stash, &raw);
                    }
                    Buffer::scratch()
                }
            },
        };
        let disk_baseline = file
            .as_deref()
            .map(crate::external::Seen::at)
            .unwrap_or_default();
        let scratch_baseline = if file.is_none() {
            crate::external::Seen::at(&stash)
        } else {
            crate::external::Seen::Absent
        };
        let config = location::ConfigurationRuntime::new(config, cli_workspace, cli_default_folder);
        let project_location = location::ProjectLocation::new(root, &config.location_policy());
        let mut keys_with_web_alt = config.keys.clone();
        keys_with_web_alt.extend(crate::commands::web_alternate_keys(
            &config.keys,
            crate::convention::Convention::current(),
            crate::commands::Platform::current(),
        ));
        let keymap = startup::keymap(&keys_with_web_alt, &config.effective_linux_keep());
        let zoom = render::clamp_zoom(config.zoom.unwrap_or(INITIAL_ZOOM));
        let scroll_sensitivity = input::initial_scroll_sensitivity(config.scroll_sensitivity);
        crate::settings::set_scroll_sensitivity(scroll_sensitivity);
        // THE ONE TIME OWNER: the shipped `RealClock` (a pure `Instant::now()`
        // pass-through). Built before the literal so the session-timer origin
        // reads it (a `clock.now()` BORROW), then the box is moved into the
        // `clock` field. A deterministic clock would swap only this one line.
        let clock: Box<dyn crate::clock::Clock> = Box::new(crate::clock::RealClock);
        #[cfg(not(target_arch = "wasm32"))]
        let stats_origin = clock.now();
        let document = document::DocumentSession::new(buffer, disk_baseline, scratch_baseline);
        let mut app = Self {
            document,
            workspace_state: workspace::WorkspaceState::default(),
            persistence: persistence::PersistenceRuntime::default(),
            input: input::InputRuntime::new(keymap, scroll_sensitivity),
            frame: frame::FrameRuntime::new(zoom, clock),
            #[cfg(not(target_arch = "wasm32"))]
            soak: None,
            #[cfg(not(target_arch = "wasm32"))]
            soak_recovery_pending: None,
            #[cfg(not(target_arch = "wasm32"))]
            soak_passed: None,
            #[cfg(not(target_arch = "wasm32"))]
            probe_ready: None,
            clipboard: match Clipboard::new() {
                Ok(c) => Some(c),
                Err(e) => {
                    eprintln!("system clipboard disabled: {e}");
                    None
                }
            },
            clipboard_last_written: None,
            project_location,
            pending_crash: None,
            config,
            #[cfg(not(target_arch = "wasm32"))]
            restored_window: None,
            // LIFETIME STATS: load the persisted odometer through the same
            // `FileSystem` seam the recent-* MRUs use (degrades to an empty
            // `Stats` on a fresh install), and start the active-writing clock.
            // Only ever reached on the live `App` — never the headless capture.
            #[cfg(not(target_arch = "wasm32"))]
            stats: crate::stats::load(&crate::stats::stats_path()),
            #[cfg(not(target_arch = "wasm32"))]
            stats_origin,
            #[cfg(not(target_arch = "wasm32"))]
            stats_last_input_ms: None,
            #[cfg(not(target_arch = "wasm32"))]
            stats_last_caret_xy: None,
            #[cfg(not(target_arch = "wasm32"))]
            stats_last_cursor: None,
            #[cfg(not(target_arch = "wasm32"))]
            stats_dirty: false,
            #[cfg(not(target_arch = "wasm32"))]
            streaks: crate::streaks::load(&crate::streaks::streaks_path()),
            #[cfg(not(target_arch = "wasm32"))]
            streaks_baseline: None,
            #[cfg(not(target_arch = "wasm32"))]
            streaks_dirty: false,
            #[cfg(all(not(target_arch = "wasm32"), not(feature = "mas")))]
            daemon_socket_path: None,
            #[cfg(all(not(target_arch = "wasm32"), not(feature = "mas")))]
            wait_conns: std::collections::HashMap::new(),
            #[cfg(target_os = "macos")]
            menu_proxy: None,
            #[cfg(target_os = "macos")]
            _menu_bar: None,
        };
        // i18n WRITE-BACK-ONCE (see `files::write_back_lang_tag_once`'s doc):
        // covers the `awl somefile.md` LAUNCH-ARGUMENT open, mirroring the
        // C-x f / C-x b / goto path's own call in `App::load_path` — a real
        // FILE only (never the no-argument scratch/stash-restore buffer,
        // which isn't "opening a document").
        if app.document.buffer().path().is_some() {
            app.write_back_lang_tag_once();
        }
        if let Some(msg) = file_refusal {
            app.set_sticky_notice(msg);
        }
        // SESSION RESTORE (native only, kill-switch gated): the OTHER open
        // files (parked into the buffer registry) and, on a bare launch, the
        // ACTIVE file + its cursor/scroll. Composes with — never replaces —
        // whatever the scratch-stash restore above already picked.
        #[cfg(not(target_arch = "wasm32"))]
        app.apply_session_restore(file_arg_given);
        // RELAUNCH RECOVERY, once every startup buffer decision has settled.
        app.adopt_unresolved_after_startup();
        // WRITING STREAKS: set the INITIAL word-delta anchor now that every startup
        // buffer decision (scratch-stash restore + session restore, which can swap
        // the active buffer) has settled. An awl-CREATED scratch (no path — fresh
        // empty OR resumed stash) anchors EAGERLY at its birth word count, so words
        // typed before the first idle flush are recorded rather than swallowed by a
        // lazy first-flush anchor (the anchor-swallow bug); a resumed stash's own
        // words are anchored, never miscounted as today's writing. An opened FILE
        // (CLI arg or session-restored active) keeps the LAZY anchor — its
        // pre-existing words are not "writing" — so `streaks_baseline` stays `None`.
        #[cfg(not(target_arch = "wasm32"))]
        if app.document.buffer().path().is_none() {
            app.streaks_anchor_now();
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let dir = crate::crashlog::crashes_dir();
            app.pending_crash = crate::crashlog::pending_notice(&dir);
        }
        // USER (personal) DICTIONARY: fold the on-disk word list ("Add to
        // dictionary" words) into the checker so an added word stays un-squiggled
        // ACROSS RESTARTS. Absent file = empty (no error). ZERO-NETWORK — a file,
        // never a fetch. Runs AFTER the spell field is built (the struct literal
        // above) so the session checker exists when the words load.
        app.load_user_dictionary();
        app
    }

    /// Hold an action-required notice until its owner explicitly clears it.
    /// Toast notices use the separate timed helper below.
    /// This is deliberately separate from transient editor feedback.
    ///
    fn set_sticky_notice(&mut self, text: impl Into<String>) {
        self.frame.set_sticky_notice(text.into());
    }

    fn set_toast_notice(&mut self, text: impl Into<String>) {
        // A real window is the live/capture boundary: unit tests and headless
        // replay keep the text deterministic but never arm a wall-clock expiry.
        let expires_at = self.frame.gpu().map(|_| self.frame.now() + TOAST_LIFETIME);
        self.frame.set_toast_notice(text.into(), expires_at);
    }

    fn clear_notice(&mut self) {
        self.frame.clear_notice();
    }
}

/// TEST HERMETICITY: the ONE door every test that needs a real `App` should
/// build it through, instead of calling `App::new` directly. `App::new` reads
/// two pieces of ambient state a plain test never intends to touch:
///
///  - **Session restore** (`apply_session_restore`, native-only): unless the
///    passed `Config` disables it, this reads `~/.local/share/awl/session.toml`
///    (or wherever `$XDG_DATA_HOME` points) through the REAL `FileSystem`
///    backend and PARKS every surviving buffer it names into the registry —
///    regardless of whether `file` is `Some` or `None`. On the developer's own
///    machine this is his ACTUAL live session: whatever files happen to be
///    open in a real `awl` right now leak into the test's `buffer_registry`,
///    and `open_buffer_count()`/similar assertions silently start tracking his
///    editing session instead of the test's fixture (`d93109e` fixed one
///    instance of exactly this leak — this closes the door everywhere else it
///    was still open).
///  - **Scratch stash**: a `file: None` launch reads the scratch buffer's
///    stash (`~/.local/share/awl/scratch.md`) through the SAME real backend —
///    UNCONDITIONALLY; unlike session restore there is no config gate for it
///    at all. A test with no fake FS installed gets the developer's real
///    scratch content loaded as the initial buffer.
///
/// This constructor closes both doors by (a) forcing `session_restore:
/// Some(false)` into the passed `Config` and (b) installing a throwaway,
/// empty `InMemoryFs` for the SCOPE of construction only (via `fs::with_fs`,
/// which restores whatever backend was active before, on return) — so both
/// reads land on a fake with nothing in it, and any directory scan the
/// constructor does along the way (`crate::index::build_index`,
/// `crate::project::Project::resolve`'s `.git` probe) also finds nothing
/// rather than walking a real directory.
///
/// **Explicitly NOT closed, and why that's fine:**
///  - The **daemon socket**: `App::new` itself never binds one — only
///    `crate::app::run` does (see `crate::daemon`'s module doc) — there is
///    nothing here to guard.
///  - The **config file path**: `Config` is a plain value passed in by the
///    caller; `App::new` never re-reads `config.toml` off disk itself (only
///    `main::run`'s startup path does that, before `App::new` is ever built).
///  - **Sticky prefs** (theme / caret mode / the zoom PERSISTED default /
///    etc.): these are process-globals restored by `main::run` before
///    `App::new` runs; `App::new` only reads the *passed* `Config`'s `zoom`
///    field for the per-instance zoom, never re-derives a sticky preference
///    from the environment on its own.
///
/// **When NOT to use this:** a test that genuinely needs the App to see REAL
/// file content (verifying an actual save landed on disk, or that a second
/// real file's bytes reach the view after `load_path`) cannot use this — the
/// injected `InMemoryFs` would make `Buffer::from_file` find nothing. Call
/// `App::new` directly instead, but still merge `Config{session_restore:
/// Some(false), ..}` into the passed config yourself, and hold
/// `fs::TEST_LOCK` for the test's life — see
/// `open_serves_the_new_files_text_despite_equal_buffer_versions` below, or
/// `app/daemon.rs`'s `finish_buffer_saves_notifies_the_waiter_and_switches_
/// to_the_previous_buffer`, for the pattern. `real_fs_app_new_calls_are_
/// all_accounted_for` (in this file's test module) is the structural guard
/// making sure a raw `App::new` call never gets added silently without one
/// of these two treatments.
#[cfg(test)]
impl App {
    pub(crate) fn new_hermetic(file: Option<PathBuf>, root: PathBuf, config: Config) -> Self {
        // Pin reduce-motion OFF for hermetic App-level tests too (mirrors the
        // `session_restore` override above): `App::new` calls
        // `motion::apply_at_startup`, and a test runner whose machine actually
        // has the OS "Reduce Motion" preference on must not silently flip every
        // spring/flinch test's animation behavior to instant-settle.
        let config = Config {
            session_restore: Some(false),
            reduce_motion: Some(false),
            ..config
        };
        let fake: Arc<dyn crate::fs::FileSystem> = Arc::new(crate::fs::InMemoryFs::new());
        crate::fs::with_fs(fake, || Self::new(file, root, None, None, config))
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl App {
    /// Build a HERMETIC, `gpu`-less App for the FRAME-LOOP capture
    /// (`--screenshot-frames`) — a native, non-test sibling of the `#[cfg(test)]`
    /// `new_hermetic`, deterministic by construction: an `InMemoryFs` (empty — no
    /// config/session/stash to read, no user disk touched, the same door
    /// `crate::scenario` uses for a strict replay) and reduce-motion/session-restore
    /// pinned off, exactly like `new_hermetic`. The capture harness swaps a
    /// [`crate::clock::VirtualClock`] in ([`set_clock`](Self::set_clock)) and steps
    /// the real scheduling body; this App renders nothing itself (`gpu: None`), so
    /// its buffer is just the scheduling driver — the harness draws the document +
    /// the panel its state reports through its OWN offscreen pipeline. Constructs via
    /// `Self::new` (not the raw open-paren needle the accounting guard scans for), so
    /// that guard is unaffected.
    ///
    /// Installs the `InMemoryFs` via the production `fs::set_active` (the SAME door
    /// `crate::scenario::install_hermetic_fs` uses for a strict replay), restoring the
    /// prior backend when construction returns — no test-only `with_fs`/serial lock
    /// (this is a single-threaded one-shot CLI, never a concurrent test). Routes
    /// through `Self::new`, not the raw constructor's open-paren needle, so the
    /// real-FS-constructor accounting guard is unaffected.
    pub(crate) fn new_headless_scheduler(root: PathBuf, config: Config) -> Self {
        let config = Config {
            session_restore: Some(false),
            reduce_motion: Some(false),
            ..config
        };
        let prev = crate::fs::active();
        crate::fs::set_active(Arc::new(crate::fs::InMemoryFs::new()));
        let app = Self::new(None, root, None, None, config);
        crate::fs::set_active(prev);
        app
    }
}

impl App {
    /// Shared post-GPU-init: fold the monitor's DPI scale into the metrics BEFORE
    /// the first sync (so the opening frame is proportioned like the capture on a
    /// HiDPI screen), push the initial view, and request the opening frame. Called
    /// inline after the NATIVE blocking init, and from `window_event` once the WASM
    /// async init deposits its GPU.
    fn on_gpu_ready(&mut self) {
        if self.frame.gpu().is_none() {
            return;
        }
        // `Gpu::new` owns a fresh surface/CAMetalLayer. The value shadowed for
        // the previous layer cannot suppress application to this one.
        self.sync_present_txn();
        let Some(gpu) = self.frame.gpu() else { return };
        let sf = gpu.window.scale_factor() as f32;
        self.frame.set_dpi(sf);
        if let Some(gpu) = self.frame.gpu_mut() {
            gpu.pipeline.set_dpi(sf);
        }
        #[cfg(not(target_arch = "wasm32"))]
        if let (Some(soak), Some(gpu)) = (self.soak.as_mut(), self.frame.gpu()) {
            soak.observe_backend(gpu.backend_name().to_string());
        }
        // WASM: the surface was configured inside the async `Gpu::new` against the
        // canvas's size AT CREATION — which is 1x1 before the browser lays the page
        // out, and the `Resized` events carrying the real canvas size fired WHILE the
        // GPU future was still pending (so they were dropped by the `gpu.is_none()`
        // guard). winit still tracked the latest observed size, so re-read it now and
        // resize the surface to the true canvas size, else the first frame draws into
        // a 1x1 surface (a blank page). Native's size is already correct here, so the
        // fix is web-only and leaves the native path untouched.
        #[cfg(target_arch = "wasm32")]
        {
            let Some(gpu) = self.frame.gpu() else { return };
            let size = gpu.window.inner_size();
            if let Some(gpu) = self.frame.gpu_mut() {
                gpu.resize(size.width.max(1), size.height.max(1));
            }
        }
        self.sync_view(true);
        self.request_frame();
        // LIVE PROBE ready signal: the window + GPU exist, so the driver thread
        // may start feeding scripted input. FIRST make the window unoccludable:
        // the wgpu macOS occlusion gate returns `SurfaceError::Occluded` before
        // `nextDrawable()` for a window without `NSWindowOcclusionStateVisible`
        // — and a probe launched from a (fullscreen) terminal opens BEHIND it,
        // so without this the run presents ZERO frames (observed: every shot
        // "no frame has presented yet"). AlwaysOnTop guarantees visibility for
        // the run's few seconds regardless of the launching terminal;
        // `focus_window` additionally asks for key status so the run matches
        // the reported live conditions (focused editing).
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(tx) = self.probe_ready.take() {
            if let Some(gpu) = self.frame.gpu() {
                gpu.window
                    .set_window_level(winit::window::WindowLevel::AlwaysOnTop);
                gpu.window.focus_window();
            }
            let _ = tx.send(());
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn drive_gpu_soak(&mut self, event_loop: &ActiveEventLoop) {
        if self.soak.is_none() {
            return;
        }
        if self.soak_passed.is_some() {
            return;
        }
        let now = self.frame.now();
        let metal = self.frame.gpu().and_then(Gpu::current_gpu_bytes);
        let (finished, stimuli) = {
            let Some(soak) = self.soak.as_mut() else {
                return;
            };
            soak.sample_if_due(now, metal);
            let finished = soak.finished(now);
            let mut stimuli = Vec::new();
            if !finished {
                for _ in 0..32 {
                    let Some(stimulus) = soak.next_stimulus(now) else {
                        break;
                    };
                    let stop = matches!(
                        stimulus,
                        crate::soak_gpu::Stimulus::Inject(_)
                            | crate::soak_gpu::Stimulus::Resize { .. }
                    );
                    stimuli.push(stimulus);
                    if stop {
                        break;
                    }
                }
            }
            (finished, stimuli)
        };
        if finished {
            let Some(soak) = self.soak.as_ref() else {
                return;
            };
            let report = soak.report(now);
            self.soak_passed = Some(report.passed());
            report.print();
            event_loop.exit();
            return;
        }
        for stimulus in stimuli.iter().copied() {
            match stimulus {
                crate::soak_gpu::Stimulus::SetLavaTheme => {
                    let _ = crate::theme::set_active_by_name("Mangrove");
                    self.retint_theme_now();
                    self.sync_view(true);
                }
                crate::soak_gpu::Stimulus::Resize { width, height } => {
                    if let Some(w) = self.frame.recovery_window().as_ref() {
                        let _ = w.request_inner_size(winit::dpi::PhysicalSize::new(width, height));
                    }
                }
                crate::soak_gpu::Stimulus::ThemeNext => {
                    crate::theme::cycle(1);
                    self.retint_theme_now();
                    self.sync_view(true);
                    if let Some(s) = self.soak.as_mut() {
                        s.observe_theme_switch();
                    }
                }
                crate::soak_gpu::Stimulus::Overlay { open } => {
                    let action = if open {
                        Action::OpenCommandPalette
                    } else {
                        Action::Cancel
                    };
                    let _ = self.apply(action, false, event_loop, crate::stats::Door::Chord);
                    if !open
                        && !self.workspace_state.overlay_open()
                        && let Some(s) = self.soak.as_mut()
                    {
                        s.observe_overlay_cycle();
                    }
                }
                crate::soak_gpu::Stimulus::Inject(kind) => {
                    self.soak_recovery_pending = Some(kind);
                    if let Some(gpu) = self.frame.gpu_mut() {
                        match kind {
                            crate::soak_gpu::FaultKind::OutOfMemory => {
                                gpu.inject_fault(gpu::GpuFaultInjection::OutOfMemory)
                            }
                            crate::soak_gpu::FaultKind::SurfaceLost => gpu.inject_surface_loss(),
                            crate::soak_gpu::FaultKind::DeviceLost => {
                                gpu.inject_fault(gpu::GpuFaultInjection::DeviceLost)
                            }
                        }
                    }
                }
            }
        }
        // Keep the surface presenting every tick while the soak runs and is not
        // yet finished — not only when this tick emitted stimuli. The tail of a
        // slow run (item 53) emits NO new stimuli while the App is still
        // confirming the last resize/recovery through its ordinary frames; the
        // loop must keep waking so those `observe_*` calls land and
        // `soak.finished` can flip on schedule completion. `finished` is false
        // here (the finished branch returned above).
        self.request_frame();
        if self.frame.last_frame().is_none() {
            event_loop.set_control_flow(control_flow_with_deadline(
                event_loop.control_flow(),
                now + if stimuli.len() == 32 {
                    Duration::from_millis(1)
                } else {
                    Duration::from_millis(100)
                },
            ));
        }
    }

    fn stamp_input(&mut self) {
        if crate::debug::debug_on() {
            let now = self.frame.now();
            self.frame.stamp_input_if_absent(now);
        }
    }
}

/// Has a DEBOUNCE window elapsed? `dirty` is when the action was last seen, `window`
/// the quiet period to wait, `now` the current instant: true once `now` has reached
/// `dirty + window` (fire the deferred write), false while still inside the window
/// (keep waiting — a fresh action re-stamps `dirty`, sliding the deadline). Pure, so
/// the debounce decision is unit-testable without an event loop.
#[cfg(test)]
fn debounce_due(dirty: Instant, window: Duration, now: Instant) -> bool {
    now.saturating_duration_since(dirty) >= window
}

/// Should the CAMetalLayer's `presentsWithTransaction` be armed? ONE owner of
/// the composition: armed while ANY source needs it — a RESIZE drag, a MOVE
/// drag, or a THEME-PREVIEW lava-boundary crossing — disarmed only once ALL have
/// settled (a corner drag streams both resize+move; a crossing can overlap a
/// drag; the settle of one source must never strip another's protection). Pure,
/// so the composition is unit-testable without a window; `App::sync_present_txn`
/// is the sole applier.
fn present_sync_armed(resize_active: bool, move_active: bool, crossing_active: bool) -> bool {
    resize_active || move_active || crossing_active
}

/// Compose one idle deadline with the event loop's current intent. A hot `Poll`
/// always wins; an unscheduled `Wait` accepts the proposal; and two deadlines
/// resolve to the earlier one so a slow ambient concern cannot delay a faster
/// sibling timer. Pure, keeping the shared lava/toast scheduling law testable
/// without a window or event loop.
fn control_flow_with_deadline(current: ControlFlow, proposed: Instant) -> ControlFlow {
    match current {
        ControlFlow::Poll => ControlFlow::Poll,
        ControlFlow::Wait => ControlFlow::WaitUntil(proposed),
        ControlFlow::WaitUntil(current) => ControlFlow::WaitUntil(current.min(proposed)),
    }
}

/// Pure notice lifetime law: only a Toast carrying a reached live deadline may
/// disappear. Sticky state and clockless/headless toasts never expire.
#[cfg(test)]
fn notice_expired(kind: NoticeKind, deadline: Option<Instant>, now: Instant) -> bool {
    kind == NoticeKind::Toast && deadline.is_some_and(|d| now >= d)
}

fn scroll_zoom_intent(mods: ModifiersState) -> bool {
    mods.contains(ModifiersState::SUPER)
}

#[cfg(test)]
#[test]
fn zoom_reflow_gate_collapses_a_burst_to_one_present_opportunity() {
    let mut gate = ZoomReflow::default();
    for _ in 0..12 {
        gate.queue();
    }
    assert!(gate.take(), "a queued burst owes exactly one reflow");
    assert!(
        !gate.take(),
        "the same present opportunity cannot reflow twice"
    );
    gate.queue();
    gate.clear();
    assert!(
        !gate.take(),
        "an intervening ordinary sync consumes the debt"
    );
}

/// Has the held stats HUD's summon chord been BROKEN by a modifier release? The HUD is a
/// momentary hold: `summon` is the modifier set held when it was summoned, `now` is the
/// current set. Any summoning modifier dropping (so `now` no longer CONTAINS all of
/// `summon`) breaks the hold and must dismiss the HUD — this is the macOS path where the
/// trigger letter's key-UP is never delivered while Cmd is down. Pressing EXTRA modifiers
/// (a superset) does not break it. Pure, so it's unit-testable without a window.
fn hud_mods_broken(summon: ModifiersState, now: ModifiersState) -> bool {
    !now.contains(summon)
}

/// Does a held Shift on this CHORD signal SELECT-INTENT (Shift+motion extends
/// the selection, GUI style)? The rule keys on the pressed CHORD, not just the
/// `Action`, because BufferStart/BufferEnd are reached two very different ways:
///   * `M-<` / `M->` (emacs) need Shift just to TYPE the `<` / `>` glyph — a
///     `Key::Character` — so that Shift is INCIDENTAL (Emacs treats them as pure
///     motion; you select via the mark, `C-Space`) and must NOT extend.
///   * Shift+Cmd-Up/Down (macOS) and Shift+Ctrl-Home/End (Linux) reach the SAME
///     actions through a `Key::Named` navigation key — a genuine GUI
///     select-intent Shift the platform text fields all honor — and MUST extend.
///     So the ONE discriminator is the key's shape: a named navigation key extends,
///     a printable glyph whose Shift is needed just to type it does not. Every OTHER
///     action keeps Shift's normal select-extend meaning regardless of key. Pure, so
///     it's unit-testable without a window/event loop. THE ONE OWNER of the rule:
///     both the live key dispatch (`app/input/keys.rs`, passing the resolved logical
///     key) and the headless `--keys` replay
///     (`main/run.rs::ReplaySession::apply_chord`, passing the chord's key) derive
///     their `apply_transition` shift flag through this fn, so an `S-` chord in a spec
///     signals select-intent exactly as a live held Shift does — never a parallel
///     copy of the rule.
pub(crate) fn motion_honors_shift_select(action: &Action, key: &Key) -> bool {
    match action {
        Action::BufferStart | Action::BufferEnd => matches!(key, Key::Named(_)),
        _ => true,
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn key_without_modifiers(event: &winit::event::KeyEvent) -> Key {
    event.key_without_modifiers()
}
#[cfg(target_arch = "wasm32")]
fn key_without_modifiers(event: &winit::event::KeyEvent) -> Key {
    event.logical_key.clone()
}

/// Run the windowed editor for an optional file with an active project `root`
/// (and optional `workspace` parent for switch-project). `wait` is the raw
/// `--wait` flag (native-only meaning — see `crate::daemon`'s module doc for
/// the documented scope of what it does and doesn't block on); ignored on wasm.
// CLI parsing passes these independently so native-only state stays cfg-gated at the boundary.
#[allow(clippy::too_many_arguments)]
pub fn run(
    file: Option<PathBuf>,
    root: PathBuf,
    cli_workspace: Option<PathBuf>,
    cli_default_folder: Option<PathBuf>,
    config: Config,
    wait: bool,
    #[cfg(not(target_arch = "wasm32"))] soak: Option<crate::soak_gpu::SoakConfig>,
    #[cfg(not(target_arch = "wasm32"))] live: Option<crate::probe::LiveScript>,
) -> anyhow::Result<()> {
    // CRASH VISIBILITY (native only — mirrors the daemon's own CAPTURE GATE
    // exactly): install the panic hook FIRST, before any window/GPU/daemon
    // work, so a panic anywhere downstream — including the daemon dance right
    // below — still gets a local crash log. `crate::crashlog::install_hook` is
    // called only here; headless capture modes never reach `crate::app::run`, so never
    // installs it (tripwire: `main::run::tests::
    // headless_screenshot_never_installs_the_crash_hook`).
    #[cfg(not(target_arch = "wasm32"))]
    if soak.is_none() {
        crate::crashlog::install_hook();
    }

    // FLIGHT RECORDER (native live-App only, capture-gated exactly like the crash
    // hook / daemon above — headless `--screenshot`/`--keys`/`--bench-*` never
    // reach `run`): if `AWL_FLIGHT_RECORDER=<path>` is set, arm the append-only
    // present/bracket/redraw trace so the user's next live theme-preview "page
    // vanishes" repro leaves a black box. A no-op when the env is absent.
    #[cfg(not(target_arch = "wasm32"))]
    crate::probe::init_flight();

    // SINGLE-INSTANCE DAEMON (native only, and compiled out entirely under
    // `mas` — see `crate::daemon`'s module doc for the full CAPTURE GATE
    // argument: this whole block lives ONLY on this live-App startup path,
    // never on any headless `--screenshot`/`--bench-*` mode). Runs the
    // bind-or-handoff dance BEFORE any window/GPU work, so handing off to an
    // already-running instance exits in milliseconds with no window ever
    // created. Under `mas`, Launch Services already refuses a second launch
    // and there is no CLI to hand a path off with, so this is simply absent.
    // The LIVE PROBE additionally skips the daemon outright (defense in depth
    // beyond the wrapper script's env isolation): a probe launch must never
    // hand its file off to — or bind over the socket of — the user's real
    // running instance. See `crate::probe`'s module doc.
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "mas")))]
    let instance_listener = if soak.is_some() || live.is_some() {
        None
    } else {
        match crate::daemon::startup(file.as_deref(), wait) {
            Ok(crate::daemon::StartupOutcome::HandedOff) => return Ok(()),
            Ok(crate::daemon::StartupOutcome::Instance(l)) => Some(l),
            Err(e) => {
                // Never let a socket hiccup (permissions, a full /tmp, a bad XDG
                // path, …) block opening the editor — degrade to a normal,
                // non-singleton launch.
                eprintln!("awl: single-instance socket unavailable ({e}); continuing without it");
                None
            }
        }
    };

    // Mark this LIVE session's start, so the History picker's Session lens has a
    // floor to bucket versions against. Live-launch-only (never the headless capture,
    // which never reaches `run`), so a capture's Session lens stays inert.
    crate::history::mark_session_start();

    // MAS SECURITY-SCOPED BOOKMARKS: resolve + start accessing every folder
    // grant persisted from an earlier launch, so this launch's FIRST touch of
    // a previously-granted root needs no fresh powerbox panel. Native macOS
    // `mas` builds only — see `src/mas.rs`'s module doc. Lives on this exact
    // live-App startup path (never `--screenshot`/`--keys`), matching every
    // other native-only startup door's capture gate above.
    #[cfg(all(feature = "mas", target_os = "macos"))]
    if soak.is_none() {
        crate::mas::restore_all_grants();
    }

    // LIVE PROBE (`--live-script`): launch WITHOUT STEALING FOCUS. Three winit
    // defaults each steal it and must all be turned off (the Accessory policy
    // alone does NOT — it only governs Dock/cmd-tab presence; the app still
    // activates and auto-keys its window, verified the hard way):
    //   1. ACTIVATION POLICY → Prohibited: the app can never be ACTIVATED, so
    //      `activateIgnoringOtherApps` is a no-op and no window of ours can become
    //      key (Accessory was insufficient — a `Focused(true)` still fired). No
    //      Dock icon, no cmd-tab entry, no menu-bar takeover either.
    //   2. `activate_ignoring_other_apps` defaults to TRUE → winit calls
    //      `NSApp.activateIgnoringOtherApps(true)` at launch, yanking the whole
    //      app (and the user's keyboard) to the foreground. Forced OFF here.
    //   3. (paired with the window's `with_active(false)` below → `orderFront`
    //      instead of `makeKeyAndOrderFront`, so the window shows but never
    //      becomes KEY.)
    // Net: the probe window appears on screen (visible + unoccluded — the wgpu
    // occlusion gate is about display VISIBILITY, not key status, so presents
    // still fire) while the user keeps typing into whatever they were using. The
    // driver injects chords straight into the event loop, never OS key focus, so
    // nothing the probe needs is lost. A normal launch stays Regular + active —
    // byte-identical activation to before.
    #[cfg(not(target_arch = "wasm32"))]
    let event_loop = {
        #[allow(unused_mut)]
        let mut builder = EventLoop::<AwlEvent>::with_user_event();
        #[cfg(target_os = "macos")]
        if live.is_some() {
            use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};
            // PROHIBITED (not Accessory): Accessory still lets the app ACTIVATE on
            // launch, and an active app auto-makes its front window key — which
            // stole the user's keyboard (observed: a `Focused(true)` still fired).
            // A Prohibited app can never be activated, so `activateIgnoringOtherApps`
            // is a no-op and no window of ours can become key. The window is still
            // shown (`orderFront`) and composited, so presents/occlusion are
            // unaffected (verified nonzero in the smoke run).
            builder.with_activation_policy(ActivationPolicy::Prohibited);
            builder.with_activate_ignoring_other_apps(false);
        }
        builder.build()?
    };
    #[cfg(target_arch = "wasm32")]
    let event_loop = EventLoop::<AwlEvent>::with_user_event().build()?;
    #[cfg(not(target_arch = "wasm32"))]
    let proxy = event_loop.create_proxy();
    // `mut` is only needed on native (the macOS-menu-proxy stash + the
    // `run_app(&mut app)` call below); on wasm `app` is moved straight into
    // `spawn_app` without ever being mutated. Kept as ONE call site (never
    // duplicated across a `#[cfg]` split) — a law test below counts every
    // raw constructor call in this file.
    #[cfg(not(target_arch = "wasm32"))]
    let config = if soak.is_some() {
        crate::fs::set_active(Arc::new(crate::fs::InMemoryFs::new()));
        Config {
            session_restore: Some(false),
            autosave: Some(false),
            stats: Some(false),
            reduce_motion: Some(false),
            ..config
        }
    } else {
        config
    };
    #[allow(unused_mut)]
    let mut app = App::new(file, root, cli_workspace, cli_default_folder, config);
    #[cfg(not(target_arch = "wasm32"))]
    {
        app.soak = soak.map(crate::soak_gpu::Controller::new);
        app.frame.set_accessibility_proxy(proxy.clone());
    }
    #[cfg(target_os = "macos")]
    {
        if app.soak.is_none() {
            app.menu_proxy = Some(proxy.clone());
        }
    }
    // LIVE PROBE (`--live-script`): arm the ready signal and spawn the driver
    // thread (the daemon's own EventLoopProxy precedent — scripted steps are
    // posted into the winit loop, never cross-thread `App` access). Shots land
    // in the script's shots dir, created here so the very first `shot` step
    // can't fail on a missing directory.
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(script) = live {
        // Arm the process-global FIRST — `Gpu::new` (called from `resumed`,
        // strictly later) reads it to add COPY_SRC + the frame mirror.
        crate::probe::set_live_active();
        if let Err(e) = std::fs::create_dir_all(&script.shots_dir) {
            eprintln!(
                "LIVE-PROBE error: cannot create shots dir {}: {e}",
                script.shots_dir.display()
            );
        }
        let (tx, rx) = std::sync::mpsc::channel();
        app.probe_ready = Some(tx);
        let probe_proxy = proxy.clone();
        crate::probe::spawn_driver(script, rx, move |e| {
            probe_proxy.send_event(AwlEvent::Probe(e)).is_ok()
        });
    }
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "mas")))]
    if let Some(listener) = instance_listener {
        app.daemon_socket_path = Some(crate::daemon::socket_path());
        crate::daemon::spawn_accept_thread(listener, proxy, AwlEvent::Daemon);
    }
    // MAS: no daemon exists to hand `--wait` off to (see the module doc) —
    // the flag is simply inert on this flavor, mirroring the wasm no-op below.
    #[cfg(all(not(target_arch = "wasm32"), feature = "mas"))]
    let _ = wait;

    // NATIVE: `run_app` blocks this thread driving the OS event loop to exit.
    #[cfg(not(target_arch = "wasm32"))]
    {
        event_loop.run_app(&mut app)?;
        if app.soak.is_some() && app.soak_passed != Some(true) {
            anyhow::bail!("native GPU soak did not meet its verification contract");
        }
    }

    #[cfg(target_arch = "wasm32")]
    {
        use winit::platform::web::EventLoopExtWebSys;
        let _ = wait; // no daemon on wasm; the flag is a native-only concern
        event_loop.spawn_app(app);
    }

    Ok(())
}

#[cfg(test)]
mod clock_law;
#[cfg(test)]
mod tests;
