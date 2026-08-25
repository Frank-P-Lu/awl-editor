//! `parse_args`'s own phases, pulled out of `args.rs` proper the same way
//! `flags.rs`/`modes.rs`/`parsers.rs` already are. Each of `parse_args`'s
//! comment-delimited stages (the argument-token loop, the hidden bench/soak
//! short-circuit, CLI validation, hermetic scenario filesystem setup, config
//! load, sticky-preference resolution, capture-opts assembly, final `Mode`
//! construction) is a named function, threading its state through [`Ctx`]
//! instead of ~20 separate locals. The phases split across three sibling
//! files (`loop_flags`/`validate`/`finish`) to stay under this file's own
//! 500-line ceiling; `Ctx` stays here, the one struct every phase shares.

use super::*;

#[path = "parse/finish.rs"]
mod finish;
#[path = "parse/loop_flags.rs"]
mod loop_flags;
#[path = "parse/validate.rs"]
mod validate;

/// Every value threaded between `parse_args`'s comment-delimited phases: the
/// argument-token loop's own outputs, plus each later phase's own derived
/// state, added to the same struct as it is produced. One phase-crossing
/// local per field — carrying them together (rather than as ~20 separate
/// function parameters) is what turns each commented phase below into a
/// named function instead of an inline block.
struct Ctx {
    // --- set by the argument-token loop (`loop_flags::parse_flag_loop`) ---
    out: Option<PathBuf>,
    motion: bool,
    motion_v: bool,
    motion_d: bool,
    // `--screenshot-frames N OUT.png`: the virtual-clock FRAME-LOOP capture — N
    // successive settled frames of the real App scheduling body stepped `--frame-step-ms`
    // per frame (None = not a frame-loop capture). Native-only (builds a hermetic App),
    // so the flag + its Mode do not exist on the CLI-less wasm target.
    #[cfg(not(target_arch = "wasm32"))]
    frames: Option<u32>,
    // `--screenshot-app OUT.png`: the LIVE-`App` capture — hermetic,
    // native-only, and the only door that photographs live-`App`-only state.
    #[cfg(not(target_arch = "wasm32"))]
    live_app: bool,
    #[cfg(not(target_arch = "wasm32"))]
    frame_step_ms: Option<u64>,
    // Every capture-mode flag seen, in order. More than one is a conflict (each
    // sets `out` + selects a Mode by precedence, so a second would silently win
    // or lose); checked after the loop via `ensure_single_capture_mode`.
    capture_modes: Vec<&'static str>,
    // `--capture-timeline "<ms,ms,...>"` cumulative step sequence (None = not a
    // timeline capture).
    timeline_steps: Option<Vec<u32>>,
    // `--capture-held DIR "<ms,ms,...>"` (None = not a held capture).
    held: Option<(capture::HeldDir, Vec<u32>)>,
    // `--capture-size WxH` PHYSICAL canvas dims (None = default 1200x800) and
    // `--capture-dpi N` renderer scale factor (None = 1.0). Both purely additive:
    // absent -> today's byte-identical capture. Threaded onto every capture mode.
    capture_size: Option<(u32, u32)>,
    capture_dpi: Option<f32>,
    file: Option<PathBuf>,
    opts: CaptureOpts,
    bench_typing: bool,
    bench_perf: bool,
    bench_frame: bool,
    bench_theme_burst: bool,
    #[cfg(not(target_arch = "wasm32"))]
    bench_a11y: bool,
    bench_zoom_burst: bool,
    bench_frost: bool,
    bench_caret: bool,
    bench_suite: bool,
    #[cfg(not(target_arch = "wasm32"))]
    soak_gpu: bool,
    #[cfg(not(target_arch = "wasm32"))]
    soak_gpu_duration: std::time::Duration,
    #[cfg(not(target_arch = "wasm32"))]
    soak_gpu_duration_seen: bool,
    // `--bench-baseline <path>`: only meaningful with `--bench-suite` (rejected
    // otherwise below, so it can never be silently dropped).
    bench_baseline: Option<PathBuf>,
    // `--keys` replay spec, kept RAW until after the arg loop so it parses THROUGH
    // the loaded config's keybinding overrides (the `--config` flag may appear after
    // `--keys` on the command line). Threaded into whichever screenshot Mode runs.
    keys_spec: Option<String>,
    root: Option<PathBuf>,
    workspace: Option<PathBuf>,
    default_folder: Option<PathBuf>,
    // `--config <path>` override for the config file location (also via `$AWL_CONFIG`),
    // so a test config can be pointed at headlessly.
    config_arg: Option<PathBuf>,
    // The `--seed-data` directory: awl's own data root, seeded into a hermetic
    // scenario sandbox. `None` on every ordinary run.
    data_seed: Option<PathBuf>,
    // The `--seed-tree` directory: a whole fixture PROJECT carried verbatim into
    // a hermetic scenario sandbox. `None` on every ordinary run.
    tree_seed: Option<PathBuf>,
    // Did the user pass an EXPLICIT sticky-pref flag? A flag always WINS over the
    // config's remembered value (flag > config > default), so the config is applied
    // only where its flag is absent. (Zoom rides `opts.zoom.is_some()` already.)
    theme_flag: bool,
    caret_flag: bool,
    page_flag: bool,
    measure_flag: bool,
    // `--wait` (single-instance daemon; `EDITOR=awl --wait` for git): only
    // meaningful for the windowed editor — see `crate::daemon`'s module doc.
    wait_flag: bool,
    // `--live-script "<steps>"` (+ optional `--live-shots DIR`): the LIVE PROBE
    // harness — windowed-editor-only, rejected alongside any capture mode below.
    live_script: Option<String>,
    live_shots: Option<PathBuf>,
    // `--strict-replay`: the strict replay gate on `--screenshot --keys` — see
    // `crate::replay`'s module doc. Parsed keys go through the STRICT door
    // (unbound chords error) and the replay aborts on Unsupported effects.
    strict_replay: bool,
    // `--storyboard <file.toml>` (+ optional `--storyboard-out <dir>`): the
    // scenario runner — always strict, always hermetic. Kept as the raw path
    // here; parsed in `validate::validate_cli` (its named file seeds the sandbox).
    storyboard_arg: Option<PathBuf>,
    storyboard_out: Option<PathBuf>,

    // --- set by later phases ---
    /// Whether `--semantic-json` was the chosen capture-mode flag.
    semantic_json: bool,
    /// The armed LIVE PROBE, parsed from `live_script`/`live_shots` in
    /// `validate::validate_cli`.
    live: Option<crate::probe::LiveScript>,
    /// The parsed `--storyboard` file, alongside its own path (for a
    /// default `--storyboard-out`). Parsed in `validate::validate_cli` so its
    /// document can seed the hermetic sandbox in `validate::install_hermetic_fs`.
    storyboard: Option<(crate::storyboard::Storyboard, PathBuf)>,
    /// The board's document, resolved against the storyboard file's own
    /// directory (so a checked-in `scenarios/demo.toml` names its fixture as
    /// `demo.md`).
    storyboard_file: Option<PathBuf>,
    /// The STRUCTURAL parse of `keys_spec` — chords stay unresolved (see
    /// `finish::resolve_sticky_prefs`'s doc).
    keys: Vec<keyspec::Chord>,
}

impl Ctx {
    fn new() -> Self {
        Self {
            out: None,
            motion: false,
            motion_v: false,
            motion_d: false,
            #[cfg(not(target_arch = "wasm32"))]
            frames: None,
            #[cfg(not(target_arch = "wasm32"))]
            live_app: false,
            #[cfg(not(target_arch = "wasm32"))]
            frame_step_ms: None,
            capture_modes: Vec::new(),
            timeline_steps: None,
            held: None,
            capture_size: None,
            capture_dpi: None,
            file: None,
            opts: CaptureOpts::default(),
            bench_typing: false,
            bench_perf: false,
            bench_frame: false,
            bench_theme_burst: false,
            #[cfg(not(target_arch = "wasm32"))]
            bench_a11y: false,
            bench_zoom_burst: false,
            bench_frost: false,
            bench_caret: false,
            bench_suite: false,
            #[cfg(not(target_arch = "wasm32"))]
            soak_gpu: false,
            #[cfg(not(target_arch = "wasm32"))]
            soak_gpu_duration: crate::soak_gpu::DEFAULT_DURATION,
            #[cfg(not(target_arch = "wasm32"))]
            soak_gpu_duration_seen: false,
            bench_baseline: None,
            keys_spec: None,
            root: None,
            workspace: None,
            default_folder: None,
            config_arg: None,
            data_seed: None,
            tree_seed: None,
            theme_flag: false,
            caret_flag: false,
            page_flag: false,
            measure_flag: false,
            wait_flag: false,
            live_script: None,
            live_shots: None,
            strict_replay: false,
            storyboard_arg: None,
            storyboard_out: None,
            semantic_json: false,
            live: None,
            storyboard: None,
            storyboard_file: None,
            keys: Vec::new(),
        }
    }
}

/// Parse the process's own `std::env::args()` into a launch [`Mode`] — the
/// native entry's one door (`fn main` calls this, then hands the result to
/// `run::run`). Each phase below is one comment-delimited stage of the old
/// monolith: the token loop, the hidden bench/soak short-circuit, CLI
/// validation, hermetic scenario filesystem setup, config load,
/// sticky-preference resolution, capture-opts assembly, final `Mode`
/// construction.
pub(crate) fn parse_args() -> Result<Mode> {
    let mut ctx = Ctx::new();
    loop_flags::parse_flag_loop(&mut ctx)?;
    if let Some(mode) = validate::resolve_early_mode(&ctx)? {
        return Ok(mode);
    }
    validate::validate_cli(&mut ctx)?;
    validate::install_hermetic_fs(&ctx)?;
    // Load the persistent CONFIG (flag/$AWL_CONFIG/XDG path — resolved inside
    // the hermetic sandbox for a strict run, where an un-seeded path degrades
    // to pure defaults). Absent file = all defaults, so this is purely
    // additive.
    let config = Config::load(config::config_path(ctx.config_arg.clone()));
    finish::resolve_sticky_prefs(&mut ctx, &config)?;
    let km = finish::build_keymap(&config);
    let (default_folder_resolved, workspace_folded) = finish::fold_launch_precedence(&ctx, &config);
    finish::assemble_capture_opts(&mut ctx, &config);
    finish::build_mode(ctx, config, km, default_folder_resolved, workspace_folded)
}
