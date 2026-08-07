//! THE ROSTER ITSELF — every command-line flag awl has, once.
//!
//! Read [`super`]'s module doc first: this file is data, and the mechanism that
//! makes it load-bearing lives there. Two rules govern an edit here:
//!
//! ORDER IS `--help`'s ORDER within each block, so a row added in the middle
//! moves a line a user reads. Append unless the new flag genuinely belongs
//! beside a neighbour.
//!
//! A `need` string is the TAIL of the refusal a missing operand earns, printed
//! as `<flag> <need>` — so it starts with its own verb (`requires a path`,
//! `needs a WORLD name`) and the two verbs the roster uses are the two the
//! parser has always used.

use super::{Flag, HelpBlock, Listing, Operand};

flag_roster! {
    // ---- CAPTURE MODES: `awl <flag> …`, listed at the top of `--help`. -------
    Screenshot: &["--screenshot"], Modes, Shown,
        &[Operand::req("OUT.png", "requires an output path")],
        "caret at rest (rounded square)";
    ScreenshotMotion: &["--screenshot-motion"], Modes, Shown,
        &[Operand::req("OUT.png", "requires an output path")],
        "caret mid-glide (centred trailing streak)";
    ScreenshotMotionVertical: &["--screenshot-motion-v"], Modes, Shown,
        &[Operand::req("OUT.png", "requires an output path")],
        "caret mid-glide vertical (left-edge bar)";
    ScreenshotMotionDiagonal: &["--screenshot-motion-d"], Modes, Shown,
        &[Operand::req("OUT.png", "requires an output path")],
        "caret mid-glide diagonal (slanted tracer)";
    ScreenshotApp: &["--screenshot-app"], Modes, Shown,
        &[Operand::req("OUT.png", "requires an output path")],
        concat!(
            "drive --keys into a REAL headless App (hermetic) and capture ITS state — the ",
            "only door that sees a live-App-only transition; sidecar carries driver: \"live-app\"",
        );
    CaptureTimeline: &["--capture-timeline"], Modes, Shown,
        &[
            Operand::req("\"0,16,50,150\"", "requires a \"<ms,ms,...>\" step sequence"),
            Operand::req("OUT.png", "requires an output path after the steps"),
        ],
        concat!(
            "deterministic timeline: step the caret glide by injected ms, frame per step ",
            "(OUT.t<ms>.png)",
        );
    CaptureHeld: &["--capture-held"], Modes, Shown,
        &[
            Operand::req("DIR", "requires a direction (left|right|up|down)"),
            Operand::req("\"0,30,60,90\"", "requires a \"<ms,ms,...>\" step sequence"),
            Operand::req("OUT.png", "requires an output path after the steps"),
        ],
        concat!(
            "deterministic HELD arrow (DIR=left|right|up|down): re-target one char/line per ",
            "step (held=true), frame per step with trail geometry",
        );

    // ---- CAPTURE MODES `--help` does not list. ------------------------------
    ScreenshotFrames: &["--screenshot-frames"], Modes, Hidden,
        &[
            Operand::req("N", "requires <N> <out.png>"),
            Operand::req("OUT.png", "requires an output path after <N>"),
        ],
        concat!(
            "capture N successive settled frames of the real App scheduling body, stepped ",
            "--frame-step-ms per frame",
        );
    SemanticJson: &["--semantic-json"], Modes, Hidden, &[],
        "print the headless App's accessibility tree as JSON instead of a PNG";

    // ---- OPTIONS: the `verification hooks` block, in its printed order. -----
    Sel: &["--sel"], Options, Shown,
        &[Operand::req("L0:C0-L1:C1", "requires L0:C0-L1:C1")],
        "selection highlight from (l0,c0)..(l1,c1)";
    Zoom: &["--zoom"], Options, Shown,
        &[Operand::req("F", "requires a factor (e.g. 1.6)")],
        "zoom factor (0.5..3.0)";
    Scroll: &["--scroll"], Options, Shown,
        &[Operand::req("N[:Q]", "requires ROW[:SUBPX]")],
        "scroll to row N plus Q fixed 1/64px units";
    Preedit: &["--preedit"], Options, Shown,
        &[Operand::req("STR", "requires a string")],
        "render STR as an IME preedit at the caret";
    Search: &["--search"], Options, Shown,
        &[Operand::req("STR", "requires a query")],
        "open isearch panel for STR + highlight hits";
    SearchCase: &["--search-case"], Options, Shown, &[],
        "make --search case-sensitive";
    Theme: &["--theme"], Options, Shown,
        &[Operand::req("NAME", "requires a world name")],
        "set the active color theme ({worlds})";
    ListWorlds: &["--list-worlds"], Options, Shown, &[],
        concat!(
            "print every theme name, one per line, then exit (the roster `--theme` accepts; ",
            "see scripts/capture-worlds.sh)",
        );
    IconManifest: &["--icon-manifest"], Options, Shown, &[],
        concat!(
            "print the app-icon export manifest as JSON (per world: icon palette tokens + ",
            "display face + its logo-cursor; per face: the bundled font files), then exit — ",
            "run from the repo root; see scripts/icons/",
        );
    GroundAudition: &["--ground-audition"], Options, Shown,
        &[Operand::req("W", "needs a WORLD name")],
        "item 121's A/B/C ground-audition manifest, exit";
    PackIcns: &["--pack-icns"], Options, Shown, &[Operand::opt("DIR")],
        concat!(
            "cut every world's rendered tiles (default assets/macos/candidates/tiles) into ",
            "assets/macos/world/<World>.icns + the canonical assets/macos/Awl.icns, and ",
            "regenerate src/app_icon/embedded.rs, then exit — run from the repo root AFTER ",
            "scripts/export-icons.sh",
        );
    ExportLinuxIcon: &["--export-linux-icon"], Options, Shown,
        &[Operand::req("OUT.png", "needs an output path")],
        concat!(
            "cut the 256px PNG out of the committed canonical assets/macos/Awl.icns, then ",
            "exit — run from the repo root; see scripts/package-appimage.sh",
        );
    CaretMode: &["--caret-mode"], Options, Shown,
        &[Operand::req("MODE", "requires 'block' or 'morph'")],
        "caret look: block, morph, ibeam, or auto (default: mono->block, proportional->morph)";
    CaptureSize: &["--capture-size"], Options, Shown,
        &[Operand::req("WxH", "requires WxH (e.g. 2400x1600)")],
        "physical canvas size for the capture (default 1200x800)";
    CaptureDpi: &["--capture-dpi"], Options, Shown,
        &[Operand::req("N", "requires a factor (e.g. 2.0)")],
        "renderer scale factor (default 1.0); WxH at dpi N == (W/N)x(H/N) logical retina window";
    Measure: &["--measure"], Options, Shown,
        &[Operand::req("N", "requires a char count")],
        "page-mode column width in chars (default 70 for prose, 100 for code; implies --page on)";
    Page: &["--page"], Options, Shown,
        &[Operand::req("on|off", "requires 'on' or 'off'")],
        "page mode: centered column (on, default) vs edge-to-edge (off)";
    Debug: &["--debug"], Options, Shown, &[],
        concat!(
            "DEBUG: draw the dim top-left dev panel — frametime/zoom/viewport/cursor/theme/",
            "md+syn (OFF by default; frametime is a fixed placeholder in a headless capture)",
        );
    Hud: &["--hud"], Options, Shown, &[],
        concat!(
            "summon the HELD stats HUD (live: hold Option-Cmd-I; clock/file-date fields are ",
            "fixed placeholders in a capture)",
        );
    MenuBar: &["--menu-bar"], Options, Shown, &[],
        concat!(
            "show the web/Linux MENU BAR (default on web/Linux, off on macOS which has the ",
            "native bar); --menu-open N drops menu N's dropdown",
        );
    Peek: &["--peek"], Options, Shown, &[],
        concat!(
            "summon the HOLD-⌘ shortcut peek (live: hold the convention's bare arming ",
            "modifier — ⌘ on Mac, Ctrl on Linux — ~600ms; a capture shows the curated ",
            "starter six)",
        );
    Streaks: &["--streaks"], Options, Shown, &[],
        concat!(
            "summon the WRITING STREAKS card (live: palette \"Writing streaks\"; a capture ",
            "shows a fixed synthetic year + streak numbers)",
        );
    WhichKey: &["--whichkey"], Options, Shown, &[],
        concat!(
            "summon the WHICH-KEY panel: the C-x prefix's follow-up keys (live: press C-x ",
            "and pause ~500ms)",
        );
    DefaultFolder: &["--default-folder"], Options, Shown,
        &[Operand::req("DIR", "requires a directory")],
        "fallback active folder for a first launch with nothing remembered (default ~/notes)";
    Config: &["--config"], Options, Shown,
        &[Operand::req("PATH", "requires a path")],
        "load settings from PATH (default ~/.config/awl/config.toml)";
    Wait: &["--wait"], Options, Shown, &[],
        concat!(
            "windowed editor only: single-instance daemon — hand `file` to an ",
            "already-running awl and block until C-x # finishes it (EDITOR=awl --wait for git)",
        );
    Keys: &["--keys"], Options, Shown,
        &[Operand::req("\"SPEC\"", "requires a key-spec string")],
        "replay emacs chords (e.g. \"C-n C-n M->\") then capture";
    SeedData: &["--seed-data"], Options, Shown,
        &[Operand::req("DIR", "requires a directory")],
        concat!(
            "seed awl's own DATA ROOT (the unresolved-change record, the scratch stash, ",
            "session.toml, history) into a hermetic scenario sandbox from DIR's files — the ",
            "only way a --screenshot-app run can START from state awl already had; refused ",
            "outside a hermetic door",
        );
    StrictReplay: &["--strict-replay"], Options, Shown, &[],
        concat!(
            "with --screenshot --keys: abort (naming the offender) on an unbound chord, a ",
            "live-only effect the replay can't perform, or a missing layout oracle; runs ",
            "HERMETIC (an in-memory fs seeded from the named file + --config — a replayed ",
            "save never touches the real file, the user's own config/notes/history are never ",
            "read or written)",
        );
    Storyboard: &["--storyboard"], Options, Shown,
        &[Operand::req("TOML", "requires a storyboard .toml path")],
        concat!(
            "run a scenario storyboard (press/type/pause/run_for/expect steps — see ",
            "scenarios/): strict + hermetic, emitting per-step PNG+JSON, deterministic film ",
            "frames, a byte-stable trace.json, and (with ffmpeg on PATH) film.webm/film.mp4",
        );
    StoryboardOut: &["--storyboard-out"], Options, Shown,
        &[Operand::req("DIR", "requires an output directory")],
        "where the storyboard run's artifacts land (default: <storyboard>.run/ beside the .toml)";

    // ---- OPTIONS `--help` does not list. ------------------------------------
    Help: &["--help", "-h"], Options, Hidden, &[],
        "print the usage summary above and exit";
    FrameStepMs: &["--frame-step-ms"], Options, Hidden,
        &[Operand::req("MS", "requires a millisecond step")],
        "milliseconds of virtual clock between --screenshot-frames frames";
    SearchReplace: &["--search-replace"], Options, Hidden, &[],
        "open the search panel's labelled replace row — the fresh Cmd-R state";
    MenuOpen: &["--menu-open"], Options, Hidden, &[Operand::opt("N")],
        "show the menu bar and drop menu N's dropdown (0 = the App menu)";
    Lifetime: &["--lifetime"], Options, Hidden, &[],
        concat!(
            "summon the LIFETIME STATS card (live: the palette's \"Lifetime stats\"; a ",
            "capture renders fixed placeholders rather than a live store)",
        );
    Root: &["--root"], Options, Hidden,
        &[Operand::req("DIR", "requires a directory")],
        concat!(
            "the active project root: scopes the go-to overlay and fills the sidecar's ",
            "project block (default: the launch file's parent, else the working directory)",
        );
    Workspace: &["--workspace"], Options, Hidden,
        &[Operand::req("DIR", "requires a directory")],
        "workspace parent whose child directories are the switch-project candidates";
    LiveScript: &["--live-script"], Options, Hidden,
        &[Operand::req(
            "\"STEPS\"",
            "requires a step string (e.g. \"keys Cmd-T; sleep 300; shot open\")",
        )],
        concat!(
            "drive the REAL windowed app through a live-probe step script; refused ",
            "alongside any capture mode",
        );
    LiveShots: &["--live-shots"], Options, Hidden,
        &[Operand::req("DIR", "requires a directory")],
        "where --live-script writes its shots (default: the system temp directory)";

    // ---- BENCHMARKS AND SOAKS. Every one opens no window. -------------------
    BenchTyping: &["--bench-typing"], Options, Hidden, &[],
        concat!(
            "time the per-keystroke update path on 100/1000/5000-line documents, ",
            "whole-buffer reshape against incremental",
        );
    BenchPerf: &["--bench-perf"], Options, Hidden, &[],
        "time the traced hot paths over the long fixtures under benches/fixtures";
    BenchFrame: &["--bench-frame"], Options, Hidden, &[],
        "per-stage frame profile of the live redraw sequence over the real repo docs";
    BenchThemeBurst: &["--bench-theme-burst"], Options, Hidden, &[],
        concat!(
            "profile successive font-changing theme switches — the reshape and the first ",
            "frame after each — cold and warm",
        );
    BenchA11y: &["--bench-a11y"], Options, Hidden, &[],
        concat!(
            "time one keystroke's accessibility projection at 100 to 50 000 lines, the ",
            "whole-snapshot path against the incremental one",
        );
    BenchZoomBurst: &["--bench-zoom-burst"], Options, Hidden, &[],
        "replay a rapid adjacent-level zoom burst: eager reflow against latest-wins coalescing";
    BenchFrost: &["--bench-frost"], Options, Hidden, &[],
        "profile the frost field's steady frames and its rebuilds, for both lava worlds";
    BenchCaret: &["--bench-caret"], Options, Hidden, &[],
        "record the caret glyph lookup's cost at the document top, middle and tail";
    BenchSuite: &["--bench-suite"], Options, Hidden, &[],
        concat!(
            "run the unified bench suite — corpus tiers by interaction scenarios — printing ",
            "a table and writing bench.json beside the invocation",
        );
    BenchBaseline: &["--bench-baseline"], Options, Hidden,
        &[Operand::req("PATH", "requires a path (e.g. benches/baseline.json)")],
        "diff --bench-suite against a machine-keyed baseline, exiting nonzero on a >20% cell";
    SoakGpu: &["--soak-gpu"], Options, Hidden, &[],
        concat!(
            "run the bounded native window/surface robustness probe, isolated from the ",
            "daemon, session, history and user config",
        );
    SoakGpuSeconds: &["--soak-gpu-seconds"], Options, Hidden,
        &[Operand::req("SECONDS", "requires a positive number")],
        "how long --soak-gpu runs, in seconds (default 900)";
}
