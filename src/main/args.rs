//! CLI argument parsing and capture-mode selection.

use std::path::PathBuf;

use anyhow::{Result, bail};

use crate::capture::{self, CaptureOpts};
use crate::config::{self, Config};
use crate::keymap::KeymapState;
use crate::{caret, debug, hud, keyspec, lifetime, page, theme, whichkey};

// THE FLAG ROSTER — the one owner of "what flags exist". `parse_args` below
// compares no argument against a literal: every `--…` token resolves through
// `flags::lookup`, every operand comes off the stream through
// `Flag::take_operands`, and the dispatch is a no-wildcard match on `FlagId`, so
// a roster row with no arm fails to compile. `--help` and the reference's
// command-line section are both generated from the same table — which is why the
// module is `pub(crate)`: `crate::reference::rows::cli` reads the roster, and
// reading it is the whole point. Nothing outside PARSES with it; `lookup` and
// `take_operands` have exactly one caller, the loop below.
#[path = "args/flags.rs"]
pub(crate) mod flags;
#[path = "args/modes.rs"]
mod modes;
#[path = "args/parsers.rs"]
mod parsers;
use flags::FlagId;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use modes::LiveAppSpec;
pub(crate) use modes::Mode;
use parsers::*;

/// Parse a `--sel L0:C0-L1:C1` argument into ordered line/col endpoints.
pub(crate) fn parse_args() -> Result<Mode> {
    let mut args = std::env::args().skip(1).peekable();
    let mut out: Option<PathBuf> = None;
    let mut motion = false;
    let mut motion_v = false;
    let mut motion_d = false;
    // `--screenshot-frames N OUT.png`: the virtual-clock FRAME-LOOP capture — N
    // successive settled frames of the real App scheduling body stepped `--frame-step-ms`
    // per frame (None = not a frame-loop capture). Native-only (builds a hermetic App),
    // so the flag + its Mode do not exist on the CLI-less wasm target.
    #[cfg(not(target_arch = "wasm32"))]
    let mut frames: Option<u32> = None;
    // `--screenshot-app OUT.png`: the LIVE-`App` capture — hermetic,
    // native-only, and the only door that photographs live-`App`-only state.
    #[cfg(not(target_arch = "wasm32"))]
    let mut live_app = false;
    #[cfg(not(target_arch = "wasm32"))]
    let mut frame_step_ms: Option<u64> = None;
    // Every capture-mode flag seen, in order. More than one is a conflict (each
    // sets `out` + selects a Mode by precedence, so a second would silently win
    // or lose); checked after the loop via `ensure_single_capture_mode`.
    let mut capture_modes: Vec<&str> = Vec::new();
    // `--capture-timeline "<ms,ms,...>"` cumulative step sequence (None = not a
    // timeline capture).
    let mut timeline_steps: Option<Vec<u32>> = None;
    // `--capture-held DIR "<ms,ms,...>"` (None = not a held capture).
    let mut held: Option<(capture::HeldDir, Vec<u32>)> = None;
    // `--capture-size WxH` PHYSICAL canvas dims (None = default 1200x800) and
    // `--capture-dpi N` renderer scale factor (None = 1.0). Both purely additive:
    // absent -> today's byte-identical capture. Threaded onto every capture mode.
    let mut capture_size: Option<(u32, u32)> = None;
    let mut capture_dpi: Option<f32> = None;
    let mut file: Option<PathBuf> = None;
    let mut opts = CaptureOpts::default();
    let mut bench_typing = false;
    let mut bench_perf = false;
    let mut bench_frame = false;
    let mut bench_theme_burst = false;
    #[cfg(not(target_arch = "wasm32"))]
    let mut bench_a11y = false;
    let mut bench_zoom_burst = false;
    let mut bench_frost = false;
    let mut bench_caret = false;
    let mut bench_suite = false;
    #[cfg(not(target_arch = "wasm32"))]
    let mut soak_gpu = false;
    #[cfg(not(target_arch = "wasm32"))]
    let mut soak_gpu_duration = crate::soak_gpu::DEFAULT_DURATION;
    #[cfg(not(target_arch = "wasm32"))]
    let mut soak_gpu_duration_seen = false;
    // `--bench-baseline <path>`: only meaningful with `--bench-suite` (rejected
    // otherwise below, so it can never be silently dropped).
    let mut bench_baseline: Option<PathBuf> = None;
    // `--keys` replay spec, kept RAW until after the arg loop so it parses THROUGH
    // the loaded config's keybinding overrides (the `--config` flag may appear after
    // `--keys` on the command line). Threaded into whichever screenshot Mode runs.
    let mut keys_spec: Option<String> = None;
    let mut root: Option<PathBuf> = None;
    let mut workspace: Option<PathBuf> = None;
    let mut default_folder: Option<PathBuf> = None;
    // `--config <path>` override for the config file location (also via `$AWL_CONFIG`),
    // so a test config can be pointed at headlessly.
    let mut config_arg: Option<PathBuf> = None;
    // The `--seed-data` directory: awl's own data root, seeded into a hermetic
    // scenario sandbox. `None` on every ordinary run.
    let mut data_seed: Option<PathBuf> = None;
    // The `--seed-tree` directory: a whole fixture PROJECT carried verbatim into
    // a hermetic scenario sandbox. `None` on every ordinary run.
    let mut tree_seed: Option<PathBuf> = None;
    // Did the user pass an EXPLICIT sticky-pref flag? A flag always WINS over the
    // config's remembered value (flag > config > default), so the config is applied
    // only where its flag is absent. (Zoom rides `opts.zoom.is_some()` already.)
    let mut theme_flag = false;
    let mut caret_flag = false;
    let mut page_flag = false;
    let mut measure_flag = false;
    // `--wait` (single-instance daemon; `EDITOR=awl --wait` for git): only
    // meaningful for the windowed editor — see `crate::daemon`'s module doc.
    let mut wait_flag = false;
    // `--live-script "<steps>"` (+ optional `--live-shots DIR`): the LIVE PROBE
    // harness — windowed-editor-only, rejected alongside any capture mode below.
    let mut live_script: Option<String> = None;
    let mut live_shots: Option<PathBuf> = None;
    // `--strict-replay`: the strict replay gate on `--screenshot --keys` — see
    // `crate::replay`'s module doc. Parsed keys go through the STRICT door
    // (unbound chords error) and the replay aborts on Unsupported effects.
    let mut strict_replay = false;
    // `--storyboard <file.toml>` (+ optional `--storyboard-out <dir>`): the
    // scenario runner — always strict, always hermetic. Kept as the raw path
    // here; parsed after the loop (its named file seeds the sandbox).
    let mut storyboard_arg: Option<PathBuf> = None;
    let mut storyboard_out: Option<PathBuf> = None;

    // THE ONE ARGUMENT LOOP. `flags::lookup` is the only place a `--…` token
    // becomes a flag and `take_operands` the only place its operands leave the
    // stream, so the dispatch below reads DATA rather than argv: the roster
    // decides what exists, this match decides what it does. NO WILDCARD — a new
    // roster row fails to compile here until it is read.
    //
    // A native-only body carries the `cfg` its own module already carries. The
    // ROW stays unconditional: `fn main` is the native entry (`wasm_start` never
    // calls this function), so a browser build has no command line to keep a
    // second, shorter roster for.
    while let Some(arg) = args.next() {
        let Some(flag) = flags::lookup(arg.as_str()) else {
            if arg.starts_with("--") {
                bail!("unknown flag: {arg}");
            }
            file = Some(PathBuf::from(arg));
            continue;
        };
        let ops = flag.take_operands(&mut args)?;
        match flag.id {
            FlagId::BenchTyping => bench_typing = true,
            FlagId::BenchPerf => bench_perf = true,
            FlagId::BenchFrame => bench_frame = true,
            FlagId::BenchThemeBurst => bench_theme_burst = true,
            FlagId::BenchA11y => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    bench_a11y = true;
                }
            }
            FlagId::BenchZoomBurst => bench_zoom_burst = true,
            FlagId::BenchFrost => bench_frost = true,
            FlagId::BenchCaret => bench_caret = true,
            FlagId::BenchSuite => bench_suite = true,
            FlagId::SoakGpu => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    soak_gpu = true;
                }
            }
            FlagId::SoakGpuSeconds => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    soak_gpu_duration = parse_soak_seconds(ops.req(0))?;
                    soak_gpu_duration_seen = true;
                }
            }
            FlagId::BenchBaseline => bench_baseline = Some(PathBuf::from(ops.req(0))),
            FlagId::LiveScript => live_script = Some(ops.req(0).to_string()),
            FlagId::LiveShots => live_shots = Some(PathBuf::from(ops.req(0))),
            FlagId::Screenshot => {
                out = Some(PathBuf::from(ops.req(0)));
                capture_modes.push(flag.name());
            }
            FlagId::ScreenshotMotion => {
                out = Some(PathBuf::from(ops.req(0)));
                motion = true;
                capture_modes.push(flag.name());
            }
            FlagId::ScreenshotMotionVertical => {
                out = Some(PathBuf::from(ops.req(0)));
                motion_v = true;
                capture_modes.push(flag.name());
            }
            FlagId::ScreenshotMotionDiagonal => {
                out = Some(PathBuf::from(ops.req(0)));
                motion_d = true;
                capture_modes.push(flag.name());
            }
            FlagId::ScreenshotApp => {
                out = Some(PathBuf::from(ops.req(0)));
                #[cfg(not(target_arch = "wasm32"))]
                {
                    live_app = true;
                }
                capture_modes.push(flag.name());
            }
            FlagId::SemanticJson => capture_modes.push(flag.name()),
            FlagId::ScreenshotFrames => {
                // The frame COUNT then the output path (the count is the flag's
                // headline, mirroring the task's shape; OUT stays explicit like
                // every other screenshot flag).
                #[cfg(not(target_arch = "wasm32"))]
                {
                    frames = Some(ops.req(0).parse::<u32>().map_err(|e| {
                        anyhow::anyhow!("--screenshot-frames <N> must be an integer: {e}")
                    })?);
                }
                out = Some(PathBuf::from(ops.req(1)));
                capture_modes.push(flag.name());
            }
            FlagId::FrameStepMs => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    frame_step_ms = Some(ops.req(0).parse::<u64>().map_err(|e| {
                        anyhow::anyhow!("--frame-step-ms must be a positive integer: {e}")
                    })?);
                }
            }
            FlagId::CaptureTimeline => {
                // A cumulative-ms step sequence FOLLOWED by the output path.
                timeline_steps = Some(parse_steps(ops.req(0))?);
                out = Some(PathBuf::from(ops.req(1)));
                capture_modes.push(flag.name());
            }
            FlagId::CaptureHeld => {
                // A held arrow direction, a cumulative-ms step sequence, then
                // the output path.
                held = Some((parse_held_dir(ops.req(0))?, parse_steps(ops.req(1))?));
                out = Some(PathBuf::from(ops.req(2)));
                capture_modes.push(flag.name());
            }
            FlagId::Storyboard => {
                storyboard_arg = Some(PathBuf::from(ops.req(0)));
                capture_modes.push(flag.name());
            }
            FlagId::StoryboardOut => storyboard_out = Some(PathBuf::from(ops.req(0))),
            FlagId::Sel => opts.selection = Some(parse_sel(ops.req(0))?),
            FlagId::Zoom => opts.zoom = Some(parse_zoom(ops.req(0))?),
            FlagId::CaptureSize => capture_size = Some(parse_size(ops.req(0))?),
            FlagId::CaptureDpi => capture_dpi = Some(parse_dpi(ops.req(0))?),
            FlagId::Scroll => {
                // Keep the capture hook's row anchor and fixed-point remainder
                // together at the CLI boundary; normalization waits until shaping
                // has supplied real variable-row geometry.
                let parsed = super::scroll_arg::parse(ops.req(0))?;
                let row = parsed.row;
                let px_q = parsed.px_q;
                opts.scroll = Some(crate::render::ScrollPos { row, px_q });
            }
            FlagId::Preedit => opts.preedit = Some(ops.req(0).to_string()),
            FlagId::Search => opts.search = Some(ops.req(0).to_string()),
            FlagId::SearchCase => opts.search_case_sensitive = true,
            FlagId::SearchReplace => {
                // Reveal the labeled REPLACE row + the key-hint line on the panel (the
                // fresh Cmd-R open state: find field focused, empty replacement). A
                // `--keys` replay can drive the panel further — typing the replacement,
                // replacing — through the shared search-key seam (`search::keys`).
                opts.search_replace_active = true;
            }
            FlagId::Theme => {
                // Set the process-global active theme NOW so it composes with any
                // capture mode (the headless render reads the active theme). Order
                // among flags is irrelevant since the active theme is global.
                let v = ops.req(0);
                theme::set_active_by_name(v).ok_or_else(|| {
                    // The roster comes from the one code-owned source
                    // (`theme::world_names`) — never a hand-copied list that
                    // can drift from the real `theme::THEMES`.
                    anyhow::anyhow!(
                        "unknown --theme {v:?}; choose one of {}",
                        theme::world_names().join(", ")
                    )
                })?;
                theme_flag = true;
            }
            FlagId::CaretMode => {
                // Pin the process-global caret mode so the headless render is
                // deterministic and verifiable. 'auto' clears any override and
                // falls back to the font-derived default (Block on mono).
                let v = ops.req(0);
                match v.to_ascii_lowercase().as_str() {
                    "block" => caret::set_mode(caret::CaretMode::Block),
                    "morph" => caret::set_mode(caret::CaretMode::Morph),
                    "ibeam" => caret::set_mode(caret::CaretMode::Ibeam),
                    "auto" => {} // leave the font-derived default in effect
                    _ => bail!("unknown --caret-mode {v:?}; choose block, morph, ibeam, or auto"),
                }
                caret_flag = true;
            }
            FlagId::Measure => {
                let n = parse_measure(ops.req(0))?;
                // Setting a measure implies page mode ON (so the narrow column +
                // gradient margins are visible in the capture).
                page::set_measure(n);
                page::set_page_on(true);
                page_flag = true;
                measure_flag = true;
            }
            FlagId::Page => {
                let v = ops.req(0);
                match v.to_ascii_lowercase().as_str() {
                    "on" => page::set_page_on(true),
                    "off" => page::set_page_on(false),
                    _ => bail!("unknown --page {v:?}; choose on or off"),
                }
                page_flag = true;
            }
            FlagId::Debug => {
                // Opt-in DEBUG panel. Sets the process-global so it composes with any
                // capture mode; the frametime line shows a FIXED placeholder with no
                // live clock (deterministic), while the rest of the panel is a pure
                // function of the view state — so an explicit `--debug` capture stays
                // stable and a plain capture (panel OFF) is byte-identical.
                debug::set_debug_on(true);
            }
            FlagId::Hud => {
                // Summon the HELD STATS HUD for the capture. Sets the process-global
                // so it composes with any capture mode; the clock / file-date fields
                // render FIXED placeholders (no live clock), so an explicit `--hud`
                // capture is deterministic while a plain capture (HUD released) is
                // byte-identical. The live window summons it by HOLDING the binding
                // (Option-Cmd-I) instead.
                hud::set_held(true);
            }
            FlagId::MenuBar => {
                // Show the WEB/LINUX MENU BAR for the capture (mirrors `--hud`). Sets
                // the process-global so it composes with any capture mode; the bar is
                // pure geometry + theme (no clock), so an explicit `--menu-bar` capture
                // is deterministic while a plain capture (default OFF on macOS) is
                // byte-identical. On web/Linux the live app shows it by default.
                crate::menubar::set_menu_bar_on(true);
            }
            FlagId::MenuOpen => {
                // Show the menu bar AND drop the dropdown for menu index N (0 = the App
                // menu), so a capture can exercise the open-dropdown render + sidecar
                // `menubar.open_menu` deterministically. N is a numeric operand, so a
                // file argument (never a plain integer) is left on the stream rather
                // than eaten; a bad/out-of-range index still just shows the closed bar.
                crate::menubar::set_menu_bar_on(true);
                if let Some(n) = ops.opt(0).and_then(|s| s.parse::<usize>().ok()) {
                    crate::menubar::set_open(Some(n));
                }
            }
            FlagId::Lifetime => {
                // Summon the LIFETIME STATS card for the capture (mirrors `--hud`).
                // Sets the process-global so it composes with any capture mode; the
                // odometer figures render FIXED "—" placeholders (no live persisted
                // store), so an explicit `--lifetime` capture is deterministic while a
                // plain capture (card closed) is byte-identical. The live app summons
                // it via the palette "Lifetime stats" command instead.
                lifetime::set_open(true);
            }
            FlagId::Streaks => {
                // Summon the WRITING STREAKS card for the capture (mirrors `--lifetime`).
                // Sets the process-global so it composes with any capture mode; the card
                // renders the FIXED synthetic `streaks::placeholder` year + streak numbers
                // (no live persisted store), so an explicit `--streaks` capture is
                // deterministic + byte-stable while a plain capture (card closed) is
                // byte-identical. The live app summons it via the palette "Writing
                // streaks" command instead.
                crate::streaks::set_open(true);
            }
            FlagId::Peek => {
                // Summon the HOLD-⌘ SHORTCUT PEEK for the capture (mirrors `--hud` /
                // `--lifetime`). Sets the process-global so it composes with any capture
                // mode; the card renders the curated STARTER SIX (no live ledger to
                // personalize from), so an explicit `--peek` capture is deterministic and
                // byte-stable while a plain capture (not summoned) is byte-identical. The
                // live app summons it by HOLDING the active convention's bare arming
                // modifier (⌘ on Mac, Ctrl on Linux — `peek::arming_modifier`) for ~600ms
                // instead.
                crate::peek::set_open(true);
            }
            FlagId::WhichKey => {
                // Summon the WHICH-KEY continuation panel for the capture. Sets the
                // process-global so it composes with any capture mode; `run.rs` then
                // derives the `C-x` rows from the catalog + config and renders the
                // SETTLED summoned panel (the live 500ms pause is windowed). A plain
                // capture (unset) draws no panel and stays byte-identical. The live
                // window summons it by pressing `C-x` and PAUSING instead.
                whichkey::set_force_shown(true);
            }
            // Kept RAW until after the loop so it parses THROUGH the loaded
            // config's keybinding overrides (`--config` may appear after `--keys`).
            FlagId::Keys => keys_spec = Some(ops.req(0).to_string()),
            FlagId::StrictReplay => strict_replay = true,
            FlagId::Config => config_arg = Some(PathBuf::from(ops.req(0))),
            // THE DATA-ROOT SEED SLOT. Hermetic-scenario doors only — it gives
            // a `--screenshot-app` capture a starting store (an
            // unresolved-change record, a scratch stash, a session), which is
            // the one thing the sandbox has no other way to hold. See
            // `crate::scenario::data_root_seeds`.
            FlagId::SeedData => data_seed = Some(PathBuf::from(ops.req(0))),
            // THE PROJECT-TREE SEED SLOT. Hermetic-scenario doors only — it
            // gives a `--screenshot-app` capture a real multi-file root to open
            // files from, which `--root` alone cannot (that seeds a directory
            // marker, so the folder reads as empty). See
            // `crate::scenario::tree_seeds`.
            FlagId::SeedTree => tree_seed = Some(PathBuf::from(ops.req(0))),
            FlagId::Root => root = Some(PathBuf::from(ops.req(0))),
            FlagId::Workspace => workspace = Some(PathBuf::from(ops.req(0))),
            FlagId::DefaultFolder => default_folder = Some(PathBuf::from(ops.req(0))),
            FlagId::Wait => wait_flag = true,
            FlagId::ListWorlds => {
                // Machine-readable roster dump — one world name per line, in
                // `theme::THEMES` cycle order — read straight off the ONE
                // code-owned source (`--help` once drifted to only ten of the
                // twenty shipped worlds; a script that shells out to THIS flag
                // can never drift the same way, since it never keeps its own
                // copy of the list). See `scripts/capture-worlds.sh`.
                for name in theme::world_names() {
                    println!("{name}");
                }
                std::process::exit(0);
            }
            // THE ICON EXPORT MANIFEST — the same one-owner move as
            // `--list-worlds`, one step richer: the offline icon compositor in
            // `scripts/icons/` needs each world's four icon palette tokens and
            // its display face, so it SHELLS OUT for them rather than keeping a
            // second copy of the palette that could drift from `worlds.rs`.
            // Reads the bundled faces from `assets/fonts` relative to the
            // CURRENT DIRECTORY (run it from the repo root) — never a path
            // baked in at build time, which would end up personal-machine
            // specific in a public repo.
            FlagId::IconManifest => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let dir = PathBuf::from(crate::icon_manifest::DEFAULT_FONTS_DIR);
                    print!("{}", crate::icon_manifest::manifest_json(&dir)?);
                    std::process::exit(0);
                }
            }
            // The A/B/C ground audition: see `icon_manifest::ground_audition_json`.
            FlagId::GroundAudition => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let dir = PathBuf::from(crate::icon_manifest::DEFAULT_FONTS_DIR);
                    print!(
                        "{}",
                        crate::icon_manifest::ground_audition_json(ops.req(0), &dir)?
                    );
                    std::process::exit(0);
                }
            }
            // THE PACK STEP — cut every shipped world's rendered tiles into a
            // real `.icns`, write the canonical bundle icon, and regenerate the
            // embedded table. Run from the repo root, AFTER the web compositor
            // has rendered the tiles; `scripts/export-icons.sh` does both in
            // order. Packing lives in the binary rather than in a shell script
            // so the container format has one owner and the determinism law can
            // re-pack a committed asset inside `cargo test`.
            FlagId::PackIcns => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let tiles = ops
                        .opt(0)
                        .map(PathBuf::from)
                        .unwrap_or_else(|| PathBuf::from("assets/macos/candidates/tiles"));
                    let root = PathBuf::from(".");
                    let written = crate::app_icon::pack_all(&tiles, &root)?;
                    let total: usize = written.iter().map(|(_, n)| n).sum();
                    for (world, bytes) in &written {
                        println!("{world:<12} {bytes:>8} bytes");
                    }
                    println!(
                        "packed {} worlds ({total} bytes) -> {}/  +  {} (canonical: {})",
                        written.len(),
                        crate::app_icon::WORLD_ICON_DIR,
                        crate::app_icon::CANONICAL_ICNS,
                        crate::app_icon::canonical_world().name
                    );
                    std::process::exit(0);
                }
            }
            // THE LINUX DESKTOP ICON — cut the 256px PNG straight out of the
            // committed canonical `.icns` via `app_icon::icns::unpack`, the
            // same parser the app-icon law tests use as their oracle. Reuses
            // the existing pipeline's OUTPUT rather than adding a second
            // rendered source: this flag reads no font, no theme, no browser —
            // only bytes `scripts/export-icons.sh` already produced and
            // committed. `scripts/package-appimage.sh` is the one caller, run
            // from the repo root (relative to CWD, same convention as
            // `--pack-icns`).
            FlagId::ExportLinuxIcon => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let out = PathBuf::from(ops.req(0));
                    let bytes = crate::app_icon::export_linux_icon(&out)?;
                    println!("wrote {} ({bytes} bytes)", out.display());
                    std::process::exit(0);
                }
            }
            // Generated from the roster above, so a flag cannot be added
            // without `--help` learning about it (or the reference's
            // command-line section saying so, for a flag kept unlisted).
            FlagId::Help => {
                println!("{}", flags::help_text());
                std::process::exit(0);
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    if soak_gpu {
        if file.is_some()
            || out.is_some()
            || keys_spec.is_some()
            || wait_flag
            || live_script.is_some()
            || config_arg.is_some()
            || root.is_some()
            || workspace.is_some()
            || default_folder.is_some()
        {
            bail!(
                "--soak-gpu is isolated; file/capture/input/config/folder arguments do not apply"
            );
        }
        return Ok(Mode::SoakGpu(crate::soak_gpu::SoakConfig {
            duration: soak_gpu_duration,
        }));
    }
    #[cfg(not(target_arch = "wasm32"))]
    if soak_gpu_duration_seen {
        bail!("--soak-gpu-seconds requires --soak-gpu");
    }
    if bench_suite {
        return Ok(Mode::BenchSuite {
            baseline: bench_baseline,
        });
    }
    if bench_baseline.is_some() {
        bail!("--bench-baseline requires --bench-suite");
    }
    if bench_typing {
        return Ok(Mode::BenchTyping);
    }
    if bench_perf {
        return Ok(Mode::BenchPerf);
    }
    if bench_frame {
        return Ok(Mode::BenchFrame);
    }
    if bench_theme_burst {
        return Ok(Mode::BenchThemeBurst);
    }
    #[cfg(not(target_arch = "wasm32"))]
    if bench_a11y {
        return Ok(Mode::BenchA11y);
    }
    if bench_zoom_burst {
        return Ok(Mode::BenchZoomBurst);
    }
    if bench_frost {
        return Ok(Mode::BenchFrost);
    }
    if bench_caret {
        return Ok(Mode::BenchCaret);
    }
    // CLI VALIDATION (error paths only — valid runs are unaffected).
    // 1) At most ONE capture-mode flag. With more than one, the Mode chosen below
    //    would silently follow a precedence and drop the rest; refuse instead.
    ensure_single_capture_mode(&capture_modes)?;
    // 1b) The LIVE PROBE is the windowed editor only — it composes with the
    //     normal launch flags (file/--theme/--config/--root/…) and with nothing
    //     headless (the whole point is the real window; a capture mode would
    //     silently swallow the script).
    // Derived, not tracked: `capture_modes` already records every capture flag
    // in order, and on wasm the arm that pushes this one does not exist.
    let semantic_json = capture_modes.contains(&flags::name_of(FlagId::SemanticJson));
    if live_script.is_some() && (out.is_some() || storyboard_arg.is_some() || semantic_json) {
        bail!("--live-script drives the real windowed app; it does not compose with capture modes");
    }
    if live_shots.is_some() && live_script.is_none() {
        bail!("--live-shots requires --live-script");
    }
    let live = match live_script {
        Some(spec) => Some(crate::probe::LiveScript {
            steps: crate::probe::parse_script(&spec)?,
            shots_dir: live_shots.unwrap_or_else(std::env::temp_dir),
        }),
        None => None,
    };
    // 2) Reject verification hooks the chosen mode would silently ignore. After the
    //    single-mode check above at most one mode category is active, so
    //    `resolve_capture_kind` need only classify which one.
    // `live_app` is native-only; a wasm build has no `--screenshot-app` door, so
    // its `kind` can only ever fall through to `Screenshot` here.
    #[cfg(not(target_arch = "wasm32"))]
    let is_screenshot_app = live_app;
    #[cfg(target_arch = "wasm32")]
    let is_screenshot_app = false;
    // `frames` is native-only for the identical reason (`--screenshot-frames`
    // builds a hermetic `App`; the flag + its `Mode` do not exist on wasm).
    #[cfg(not(target_arch = "wasm32"))]
    let is_screenshot_frames = frames.is_some();
    #[cfg(target_arch = "wasm32")]
    let is_screenshot_frames = false;
    let kind = resolve_capture_kind(
        out.is_some(),
        held.is_some(),
        timeline_steps.is_some(),
        motion || motion_v || motion_d,
        is_screenshot_app,
        is_screenshot_frames,
    );
    let supplied = SuppliedHooks {
        sel: opts.selection.is_some(),
        zoom: opts.zoom.is_some(),
        scroll: opts.scroll.is_some(),
        preedit: opts.preedit.is_some(),
        search: opts.search.is_some(),
        search_case: opts.search_case_sensitive,
        search_replace: opts.search_replace_active,
        capture_size: capture_size.is_some(),
        capture_dpi: capture_dpi.is_some(),
        root: root.is_some(),
        workspace: workspace.is_some(),
        default_folder: default_folder.is_some(),
    };
    let unused = unused_hooks(kind, &supplied);
    if !unused.is_empty() {
        bail!(
            "{} not honored by the chosen capture mode",
            unused.join(", ")
        );
    }
    // `--screenshot-frames` drives the App's SCHEDULING loop under a virtual
    // clock, not a `--keys` replay — the document is a stationary backdrop
    // (see `capture::frames`'s module doc) — so `Mode::ScreenshotFrames`
    // carries no `keys` field to land a replay in. Refuse the combination
    // here (the `unused_hooks` table above only covers `SuppliedHooks`'
    // fields, which `--keys` is not one of) rather than let it silently parse
    // and do nothing, the same shape as `--storyboard`'s own `--keys` refusal
    // below.
    #[cfg(not(target_arch = "wasm32"))]
    if frames.is_some() && keys_spec.is_some() {
        bail!("--screenshot-frames drives its own scheduling loop; --keys does not apply");
    }
    // `--strict-replay` gates a `--keys` replay, and only the plain
    // `--screenshot` mode threads the strict engine (the motion/timeline/held
    // variants stay permissive one-offs); refuse the combinations that would
    // silently ignore it. Validated BEFORE the hermetic install below so a
    // refused flag combination never swaps the process filesystem first.
    if strict_replay {
        if keys_spec.is_none() {
            bail!("--strict-replay requires --keys (there is no replay to be strict about)");
        }
        if kind != CaptureKind::Screenshot {
            bail!(
                "--strict-replay only applies to --screenshot (not motion/timeline/held captures)"
            );
        }
    }
    // `--storyboard` drives its own input/document; refuse the flags it would
    // silently ignore, then parse the scenario file NOW (std::fs — the one
    // boundary crossing before the sandbox exists) so its named document can
    // seed the hermetic sandbox below, exactly like `--strict-replay`'s file.
    if storyboard_arg.is_some() {
        if keys_spec.is_some() {
            bail!("--storyboard drives its own steps; --keys does not apply");
        }
        if file.is_some() {
            bail!(
                "--storyboard takes its document from the storyboard file; drop the file argument"
            );
        }
        if wait_flag {
            bail!("--wait only applies to the windowed editor (no capture mode)");
        }
        if strict_replay {
            bail!("--storyboard is always strict; --strict-replay does not apply");
        }
    } else if storyboard_out.is_some() {
        bail!("--storyboard-out requires --storyboard");
    }
    let storyboard: Option<(crate::storyboard::Storyboard, PathBuf)> = match &storyboard_arg {
        Some(p) => {
            let src = std::fs::read_to_string(p)
                .map_err(|e| anyhow::anyhow!("reading storyboard {}: {e}", p.display()))?;
            let stem = p
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "storyboard".to_string());
            let board = crate::storyboard::parse(&src, &stem)
                .map_err(|e| e.context(format!("parsing storyboard {}", p.display())))?;
            Some((board, p.clone()))
        }
        None => None,
    };
    // The board's document, resolved against the storyboard file's own directory
    // (so a checked-in `scenarios/demo.toml` names its fixture as `demo.md`).
    let storyboard_file: Option<PathBuf> = storyboard.as_ref().and_then(|(b, p)| {
        b.file.as_ref().map(|f| match p.parent() {
            Some(dir) if !dir.as_os_str().is_empty() => dir.join(f),
            _ => PathBuf::from(f),
        })
    });
    // HERMETIC SCENARIO FILESYSTEM — the ONE production call (`crate::scenario`'s
    // module doc is the contract): a scenario run swaps the process fs to an
    // in-memory sandbox seeded from exactly the CLI-named inputs BEFORE the
    // config loads, so the load below — and every fs consumer after it — reads
    // the sandbox, never the user's real files. The legacy permissive paths
    // never install it (real-fs behavior kept byte-for-byte). Three doors select
    // it: `--strict-replay`, `--storyboard` (which seeds the BOARD's document,
    // resolved above, plus its parent-directory marker), and
    // `--screenshot-app`, whose claim is the strongest — it drives a real `App`,
    // which PERFORMS the writes a replay only records.
    //
    // `--seed-data` only means anything HERE, so it is refused everywhere else
    // rather than silently ignored: a run that named a store and then did not
    // get one would photograph the wrong starting state and look like a product
    // bug. The slot exists because a `--screenshot-app` capture of an
    // external-change conflict has to START conflicted: nothing can raise one
    // mid-run, since the change must come from outside awl.
    // `live_app` is a native-only binding (the wasm build compiles no live-`App`
    // capture door), so the hermetic predicate is spelled per target. The wasm
    // arm installs no sandbox at all, which is why its answer is unconditional.
    #[cfg(not(target_arch = "wasm32"))]
    let hermetic = strict_replay || storyboard.is_some() || live_app || semantic_json;
    #[cfg(target_arch = "wasm32")]
    let hermetic = false;
    if data_seed.is_some() && !hermetic {
        bail!(
            "--seed-data only applies to a hermetic scenario run (--screenshot-app, \
             --semantic-json, --storyboard, or --screenshot --keys --strict-replay)"
        );
    }
    // Same refusal, same reason: a run that named a fixture project and then did
    // not get one would photograph an empty folder and read as a product bug.
    if tree_seed.is_some() && !hermetic {
        bail!(
            "--seed-tree only applies to a hermetic scenario run (--screenshot-app, \
             --semantic-json, --storyboard, or --screenshot --keys --strict-replay)"
        );
    }
    #[cfg(not(target_arch = "wasm32"))]
    if hermetic {
        crate::scenario::install_hermetic_fs(
            crate::scenario::seed_document(
                storyboard.is_some(),
                storyboard_file.as_deref(),
                file.as_deref(),
            ),
            config_arg.as_deref(),
            root.as_deref(),
            data_seed.as_deref(),
            tree_seed.as_deref(),
        )?;
    }
    // Load the persistent CONFIG (flag/$AWL_CONFIG/XDG path — resolved inside
    // the hermetic sandbox for a strict run, where an un-seeded path degrades
    // to pure defaults). Absent file = all defaults, so this is purely
    // additive. Parse `--keys` THROUGH the config's keybinding overrides so a
    // replay exercises rebound chords.
    let config = Config::load(config::config_path(config_arg));
    // STICKY PREFERENCES: restore the remembered THEME / PAGE / CARET onto the
    // process-globals (the same globals the flags set), honouring flag > config —
    // a config value is applied only where its flag was ABSENT, so an explicit flag
    // still wins. These globals serve BOTH the windowed editor and the headless
    // capture, so a `--config` with theme/page/caret set produces a capture reflecting
    // them. ZOOM is per-instance (not a global): the capture folds it into `opts.zoom`
    // below and the windowed `App::new` reads `config.zoom`.
    //
    // The page-width MEASURE is now a per-KIND sticky pref (`page_width_prose` /
    // `page_width_code`) — resolve the STARTING buffer's class from the launch
    // `file` argument (no `Buffer` exists yet here) so the very first frame reads
    // the right one; a later buffer switch re-resolves against whichever kind is
    // then active (`App::sync_page_measure` / the headless `--keys` Goto switch).
    let initial_page_class =
        page::PageClass::of_path(storyboard_file.as_deref().or(file.as_deref()));
    config.apply_sticky_globals(
        theme_flag,
        page_flag,
        caret_flag,
        measure_flag,
        initial_page_class,
    );
    // `--keys` only makes sense with a capture mode (it mutates the buffer for a
    // one-frame capture); refuse it for the windowed editor where live typing is
    // the input path.
    if keys_spec.is_some() && out.is_none() && !semantic_json {
        bail!("--keys requires a capture mode (e.g. --screenshot OUT.png)");
    }
    // `--wait` is a windowed-editor-only concern (the single-instance daemon's
    // handoff); a capture mode has no daemon to wait on (see `crate::daemon`'s
    // CAPTURE GATE).
    if wait_flag && (out.is_some() || semantic_json) {
        bail!("--wait only applies to the windowed editor (no capture mode)");
    }
    // STRUCTURAL parse only — a garbled token still errors right here. The
    // chords stay UNRESOLVED: the replay loop resolves them one press at a time
    // through `km` below (`keyspec::ChordResolver`), interleaved with the
    // search guard, so a chord an open search panel consumes never reaches the
    // keymap — and the STRICT unbound/dangling-prefix refusals fire there,
    // where "was this chord for the keymap at all" is actually decidable.
    let keys: Vec<keyspec::Chord> = match &keys_spec {
        Some(spec) => keyspec::parse_chords(spec)?,
        None => Vec::new(),
    };
    // The keymap every capture replay resolves through: config `[keys]`
    // rebinds + the `linux_keep_emacs` door, exactly what live `App::new` builds.
    let km = KeymapState::with_overrides_and_keep(&config.keys, &config.effective_linux_keep());
    // PRECEDENCE: explicit flag > config > built-in default. Fold the config value in
    // BEHIND the flag (the flag wins via `.or`) before the existing resolvers add the
    // built-in default. The Windowed path keeps the RAW flag + config so a live reload
    // can re-fold; capture modes fold here (one-shot, no reload).
    let default_folder_resolved = resolve_default_folder(
        &default_folder
            .clone()
            .or_else(|| config.default_folder.clone()),
    );
    let workspace_folded = workspace.clone().or_else(|| config.workspace.clone());
    // Thread the capture canvas size + dpi onto the screenshot opts (timeline/held
    // carry them on their Mode variants). Absent flags -> None -> byte-stable default.
    opts.canvas = capture_size;
    opts.dpi = capture_dpi;
    // STICKY ZOOM (capture): fold the remembered zoom in BEHIND `--zoom` (the flag
    // wins). The windowed editor applies `config.zoom` in `App::new` instead.
    if opts.zoom.is_none() {
        opts.zoom = config.zoom;
    }
    // STORYBOARD mode: everything below the sandbox install composes normally
    // (config from the sandbox, km with its rebinds); the run outputs land in
    // `--storyboard-out`, defaulting to `<storyboard>.run/` beside the board.
    if let Some((board, board_path)) = storyboard {
        let out_dir = storyboard_out.unwrap_or_else(|| board_path.with_extension("run"));
        return Ok(Mode::Storyboard {
            board,
            file: storyboard_file,
            out_dir,
            root,
            workspace: workspace_folded,
            default_folder: default_folder_resolved,
            config,
            km,
        });
    }
    #[cfg(not(target_arch = "wasm32"))]
    if semantic_json {
        return Ok(Mode::SemanticJson(LiveAppSpec {
            file,
            keys,
            root,
            workspace: workspace_folded,
            config,
            // Always `None`: this mode renders no PNG and its `CaptureKind::Windowed`
            // classification already refuses the flags above rather than discarding them.
            canvas: capture_size,
            dpi: capture_dpi,
        }));
    }
    Ok(match out {
        Some(out) if held.is_some() => {
            let (dir, steps) = held.unwrap();
            Mode::CaptureHeld {
                out,
                file,
                keys,
                km,
                dir,
                steps,
                root,
                canvas: capture_size,
                dpi: capture_dpi,
            }
        }
        Some(out) if timeline_steps.is_some() => Mode::CaptureTimeline {
            out,
            file,
            keys,
            km,
            steps: timeline_steps.unwrap(),
            root,
            canvas: capture_size,
            dpi: capture_dpi,
        },
        #[cfg(not(target_arch = "wasm32"))]
        Some(out) if live_app => Mode::ScreenshotApp {
            out,
            spec: LiveAppSpec {
                file,
                keys,
                root,
                workspace: workspace_folded,
                config,
                // Honored for real — `capture_live_app` applies these before rendering.
                canvas: capture_size,
                dpi: capture_dpi,
            },
        },
        #[cfg(not(target_arch = "wasm32"))]
        Some(out) if frames.is_some() => Mode::ScreenshotFrames {
            out,
            file,
            frames: frames.unwrap(),
            step_ms: frame_step_ms.unwrap_or(capture::DEFAULT_FRAME_STEP_MS),
            // Honored for real — `capture::capture_frames` reads them off the
            // `CaptureOpts` `run.rs`'s handler builds.
            canvas: capture_size,
            dpi: capture_dpi,
        },
        Some(out) if motion_d => Mode::ScreenshotMotionDiagonal {
            out,
            file,
            keys,
            km,
        },
        Some(out) if motion_v => Mode::ScreenshotMotionVertical {
            out,
            file,
            keys,
            km,
        },
        Some(out) if motion => Mode::ScreenshotMotion {
            out,
            file,
            keys,
            km,
        },
        Some(out) => Mode::Screenshot {
            out,
            file,
            opts,
            keys,
            km,
            root,
            workspace: workspace_folded,
            default_folder: default_folder_resolved,
            config,
            strict: strict_replay,
        },
        None => Mode::Windowed {
            file,
            live,
            root,
            workspace,
            default_folder,
            config,
            wait: wait_flag,
        },
    })
}

/// Resolve the DEFAULT-FOLDER CANDIDATE: explicit `--default-folder`, else
/// `~/notes` (`$HOME/notes`), else `./notes` if HOME is unset.
///
/// The candidate is a first-run launch fallback only when the CLI or config
/// explicitly supplied the setting. An unconfigured launch uses awl's data
/// root; `run::location::resolve_launch_context` owns and tests that gate.
pub(crate) fn resolve_default_folder(default_folder: &Option<PathBuf>) -> PathBuf {
    if let Some(n) = default_folder {
        return n.clone();
    }
    match crate::fs::home_dir() {
        Some(home) => home.join("notes"),
        None => PathBuf::from("notes"),
    }
}

#[cfg(test)]
#[path = "args/tests.rs"]
mod tests;
