//! The capture/launch MODE a parsed command line selects. One enum, lifted out
//! of the parser so growing the roster does not grow `args.rs`.

use std::path::PathBuf;

use crate::capture::{self, CaptureOpts};
use crate::config::Config;
use crate::keymap::KeymapState;
use crate::keyspec;

// A command mode is parsed once and immediately consumed, so boxing would
// add indirection without benefit.
#[allow(clippy::large_enum_variant)]
pub(crate) enum Mode {
    Windowed {
        file: Option<PathBuf>,
        /// The armed LIVE PROBE (`--live-script` [+ `--live-shots`]) — scripted
        /// chords + compositor-side window shots against the real windowed app.
        /// `None` on every normal launch; never constructible from a headless
        /// capture mode (the flag combination is rejected below). See
        /// `crate::probe`'s module doc for the harness contract.
        live: Option<crate::probe::LiveScript>,
        /// The ACTIVE project root (`--root`). When absent it defaults to the
        /// launch file's parent (or cwd) in `app::run`.
        root: Option<PathBuf>,
        /// The RAW `--workspace` flag (None = unset). Folded with the config inside
        /// `App::new` so a later live config reload can re-apply precedence.
        workspace: Option<PathBuf>,
        /// The RAW `--default-folder` flag (None = unset). Folded with the config
        /// (flag > config > `~/notes`) inside `App::new`; kept raw so reload keeps
        /// flag wins. The FIRST-RUN fallback only — never the active
        /// folder once running.
        default_folder: Option<PathBuf>,
        /// The loaded persistent config (keybinding overrides + folder defaults +
        /// the Settings-open path). Empty/all-None when no config file exists.
        config: Config,
        /// The raw `--wait` flag (single-instance daemon; `EDITOR=awl --wait` for
        /// git). Native-only meaning — see `crate::daemon`'s module doc for the
        /// documented scope of what it does and doesn't block on.
        wait: bool,
    },
    /// Deterministic one-frame capture with the caret AT REST (the resting amber
    /// rounded square on the glyph), plus optional zoom / scroll / selection
    /// verification overrides. `keys` is an optional `--keys` replay applied to
    /// the buffer BEFORE the capture, so the PNG + sidecar reflect post-replay
    /// state (cursor / selection / search).
    Screenshot {
        out: PathBuf,
        file: Option<PathBuf>,
        opts: CaptureOpts,
        keys: Vec<keyspec::Chord>,
        /// The keymap the replay loop resolves `keys` through, chord by chord
        /// (config `[keys]` rebinds + the `linux_keep_emacs` door applied) —
        /// resolution happens INSIDE the replay so the search guard can
        /// intercept a chord before the keymap ever sees it.
        km: KeymapState,
        /// The active project root for the capture (`--root`); scopes the go-to
        /// overlay and populates the sidecar `project` block.
        root: Option<PathBuf>,
        /// Optional workspace parent (`--workspace`): its child dirs are the
        /// switch-project candidates a replayed `C-x p` lists (with git markers).
        workspace: Option<PathBuf>,
        /// The EFFECTIVE default folder (`--default-folder`), surfaced ONLY in the
        /// sidecar `project.default_folder` field. It does not scope the `C-x m`
        /// move-dest picker, which walks the ACTIVE root like Browse.
        default_folder: PathBuf,
        /// The loaded persistent config: supplies the `[keys]` overrides reflected in
        /// the palette's effective bindings, and the Settings-open target.
        config: Config,
        /// STRICT REPLAY (`--strict-replay`, opt-in): abort on any unbound
        /// chord (checked at replay time by `keyspec::ChordResolver`, AFTER
        /// the search guard has had its chance to consume the chord), any
        /// Unsupported effect, or a missing layout oracle — naming the exact
        /// offender — instead of the legacy permissive warn-and-continue. The
        /// scenario-runner default the later harness phases plumb through;
        /// see `crate::replay`'s module doc. Also HERMETIC: by the time this
        /// Mode exists the process fs has been swapped to the seeded sandbox
        /// (`crate::scenario::install_hermetic_fs`, called before the config
        /// loaded), so the whole run never touches the user's real files.
        strict: bool,
    },
    /// Deterministic one-frame capture of a caret MID-GLIDE (dropped to the
    /// baseline and stretched into a trailing underline streak), so the temporal
    /// effect is inspectable from a still.
    ScreenshotMotion {
        out: PathBuf,
        file: Option<PathBuf>,
        keys: Vec<keyspec::Chord>,
        km: KeymapState,
    },
    /// Like [`Mode::ScreenshotMotion`] but a VERTICAL glide: the caret slid to a
    /// thin bar on the cell's left edge, trailing up the lines it passed.
    ScreenshotMotionVertical {
        out: PathBuf,
        file: Option<PathBuf>,
        keys: Vec<keyspec::Chord>,
        km: KeymapState,
    },
    /// Like [`Mode::ScreenshotMotion`] but a DIAGONAL glide (different row AND
    /// column): the trail is a true slanted tracer from source to target.
    ScreenshotMotionDiagonal {
        out: PathBuf,
        file: Option<PathBuf>,
        keys: Vec<keyspec::Chord>,
        km: KeymapState,
    },
    /// The virtual-clock FRAME-LOOP capture (`--screenshot-frames N OUT.png`): N
    /// successive SETTLED frames driven by a REAL `App`'s `about_to_wait_impl`
    /// scheduling body stepped `step_ms` per frame under a `VirtualClock`, so a
    /// LIVE-ONLY cross-frame behaviour (the which-key debounce summoning EXACTLY at
    /// its 500 ms pause deadline) is inspectable — the class the single settled
    /// `--screenshot` frame is blind to. `file` is the stationary document backdrop.
    /// Native-only (builds a hermetic App via `InMemoryFs`). See `capture::frames`.
    #[cfg(not(target_arch = "wasm32"))]
    ScreenshotFrames {
        out: PathBuf,
        file: Option<PathBuf>,
        frames: u32,
        step_ms: u64,
    },
    /// THE LIVE-`App` CAPTURE (`--screenshot-app OUT.png [file]`): the
    /// only capture door that can photograph a live-`App`-only transition. Hermetic
    /// and native-only; contract in `main/run/live_app.rs`.
    #[cfg(not(target_arch = "wasm32"))]
    ScreenshotApp { out: PathBuf, spec: LiveAppSpec },
    /// Print the exact renderer-independent semantic snapshot produced by a
    /// real headless `App`. Native accessibility is intentionally separate
    /// from the browser DOM contract.
    #[cfg(not(target_arch = "wasm32"))]
    SemanticJson(LiveAppSpec),
    /// DETERMINISTIC TIMELINE capture: after the `--keys` replay sets up a
    /// NAVIGATION caret move (a glide, not an edit-snap), advance a VIRTUAL clock
    /// by the given cumulative-ms `steps` with an INJECTED dt, writing a frame
    /// (`OUT.t<ms>.png` + `.json`) after each step so an animation's TRAJECTORY is
    /// inspectable. `keys` is split: all-but-last set up the origin, the LAST chord
    /// is the navigation move that glides.
    CaptureTimeline {
        out: PathBuf,
        file: Option<PathBuf>,
        keys: Vec<keyspec::Chord>,
        km: KeymapState,
        /// Cumulative ms since the move started; the dt for step i is `t[i]-t[i-1]`.
        steps: Vec<u32>,
        root: Option<PathBuf>,
        /// `--capture-size` physical canvas dims (None = default 1200x800).
        canvas: Option<(u32, u32)>,
        /// `--capture-dpi` renderer scale factor (None = 1.0).
        dpi: Option<f32>,
    },
    /// DETERMINISTIC HELD-MOTION capture: reproduce a HELD arrow (OS auto-repeat)
    /// by re-targeting the caret one char/line in `dir` at EACH virtual-clock step
    /// with `held=true`, advancing the spring by the injected dt, and writing a
    /// frame (`OUT.t<ms>.png` + `.json`) per step. The `--keys` replay sets the
    /// ORIGIN the held burst starts from; the per-step sidecar records the drawn
    /// trail (length/endpoints/holding) so the held streak is machine-verifiable.
    CaptureHeld {
        out: PathBuf,
        file: Option<PathBuf>,
        keys: Vec<keyspec::Chord>,
        km: KeymapState,
        dir: capture::HeldDir,
        /// Cumulative ms; the dt for step i is `t[i]-t[i-1]`. One held re-target is
        /// applied per entry.
        steps: Vec<u32>,
        root: Option<PathBuf>,
        /// `--capture-size` physical canvas dims (None = default 1200x800).
        canvas: Option<(u32, u32)>,
        /// `--capture-dpi` renderer scale factor (None = 1.0).
        dpi: Option<f32>,
    },
    /// STORYBOARD run (`--storyboard <file.toml>`): a checked-in scenario file
    /// drives one HERMETIC, STRICT replay session end-to-end (see
    /// `crate::storyboard` + `crate::story`), emitting per-step PNG+sidecar
    /// artifacts, deterministic film frames on the virtual clock, a byte-stable
    /// `trace.json`, and (via a local ffmpeg, when present) a WebM/MP4 film.
    /// Like `--strict-replay`, by the time this Mode exists the process fs has
    /// been swapped to the seeded sandbox (`crate::scenario`).
    Storyboard {
        board: crate::storyboard::Storyboard,
        /// The board's document resolved against the storyboard file's own
        /// directory (`None` = scratch); already seeded into the sandbox.
        file: Option<PathBuf>,
        /// Where the run's artifacts land (`--storyboard-out`, defaulting to
        /// `<storyboard-stem>.run/` beside the storyboard file — gitignored).
        out_dir: PathBuf,
        root: Option<PathBuf>,
        workspace: Option<PathBuf>,
        default_folder: PathBuf,
        config: Config,
        km: KeymapState,
    },
    /// Hidden performance harness: time the per-keystroke update path (append a
    /// char -> reshape) on documents of 100/1000/5000 lines, BEFORE (whole-buffer
    /// reshape) vs AFTER (incremental), and print the numbers. Opens no window.
    BenchTyping,
    /// Hidden performance harness: time the FIVE traced hot paths (motion oracle,
    /// ornament marks, rule conceal, theme reshape) over the long fixtures under
    /// `benches/fixtures`, printing median ns per call. Opens no window.
    BenchPerf,
    /// Hidden performance harness: per-stage FRAME profile of the exact live
    /// redraw sequence (each `prepare` sub-call, render encode, submit+poll,
    /// atlas trim) over the real repo docs (CAPTURE.md / CLAUDE.md) with their
    /// real spell-squiggle load, at the live-report 2910x1720 @2x canvas,
    /// printing a stage | median ms | % table. Opens no window.
    BenchFrame,
    /// Hidden performance harness: the THEME-BURST profile — N successive
    /// font-changing theme switches (the faceted picker's live preview) over
    /// CLAUDE.md with its real spell load at the live-report 5120x2756 @2x
    /// zoom-1.1 canvas, timing `sync_theme` (the reshape) AND the first frame
    /// after each switch (the new face's atlas rasterization), two laps
    /// (cold/warm) to expose atlas retention. Opens no window.
    BenchThemeBurst,
    /// Hidden performance harness: the ACCESSIBILITY PROJECTION WITNESS — one
    /// keystroke mid-document at 100 / 1 000 / 10 000 / 50 000 lines, timed
    /// through the retired monolithic path (whole snapshot + whole
    /// `TreeUpdate` on every redraw) and through the retained incremental
    /// projection, printing the counts — runs re-read, document bytes read,
    /// nodes published — beside the milliseconds. Opens no window. Native
    /// only: the projection it measures is, because AccessKit has no canvas
    /// adapter and the browser build carries no accessibility tree at all.
    #[cfg(not(target_arch = "wasm32"))]
    BenchA11y,
    /// Hidden performance harness: replay a rapid adjacent-level zoom burst at
    /// the reported 3538x2610 @2x / 60% posture, comparing the old eager
    /// per-input reflow with latest-wins present-boundary coalescing. Opens no
    /// window.
    BenchZoomBurst,
    /// Hidden performance harness: the FROST steady-frame profiler — the
    /// organic glyph-seeded frost field's real workload over a heading-rich
    /// page-mode lava fixture with a populated outline + gutter, for both lava
    /// worlds, witnessing a nonzero seed field + zero-rebuild steady frames + one
    /// rebuild after a zoom / margin-text change. Opens no window.
    BenchFrost,
    /// Hidden performance harness: the CARET LOOKUP WITNESS — places the
    /// caret at the document top/middle/tail on a long fixture and records, per
    /// position, the prefix runs a whole-doc walk would touch (grows), the
    /// target-line-local glyph count the fixed lookup visits (nonzero, constant), and
    /// the median old-walk vs new-lookup cost — proving the caret glyph lookup cost is
    /// independent of document position. Opens no window.
    BenchCaret,
    /// Hidden performance harness: the UNIFIED BENCH SUITE — deterministic
    /// corpus tiers x interaction scenarios, every cell witnessed, printed as
    /// a table and written to `bench.json` beside the invocation. `baseline`
    /// (`--bench-baseline <path>`) additionally diffs the run against a
    /// checked-in machine-keyed baseline, exiting nonzero on a >20% cell
    /// regression (the `scripts/bench.sh` merge-day gate). Opens no window.
    BenchSuite { baseline: Option<PathBuf> },
    /// Hidden bounded REAL native-window/surface robustness probe. The live App
    /// runs isolated from daemon/session/history/user config state.
    #[cfg(not(target_arch = "wasm32"))]
    SoakGpu(crate::soak_gpu::SoakConfig),
}

/// What a live-`App` door needs to stand one up: the document, the chords to
/// replay into it, where it is rooted, and its config. Both live-`App` modes
/// carry exactly this, so they carry one type rather than two copies of five
/// fields.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) struct LiveAppSpec {
    pub file: Option<PathBuf>,
    pub keys: Vec<keyspec::Chord>,
    pub root: Option<PathBuf>,
    pub workspace: Option<PathBuf>,
    pub config: Config,
}
