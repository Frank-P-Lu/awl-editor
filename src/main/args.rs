//! CLI argument parsing and capture-mode selection.

use std::path::PathBuf;

use anyhow::{Result, bail};

use crate::capture::{self, CaptureOpts};
use crate::config::{self, Config};
use crate::keymap::KeymapState;
use crate::{caret, debug, hud, keyspec, lifetime, page, theme, whichkey};

#[path = "args/modes.rs"]
mod modes;
#[path = "args/parsers.rs"]
mod parsers;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use modes::LiveAppSpec;
pub(crate) use modes::Mode;
use parsers::*;

/// Parse a `--sel L0:C0-L1:C1` argument into ordered line/col endpoints.
pub(crate) fn parse_args() -> Result<Mode> {
    let mut args = std::env::args().skip(1);
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
    // `--screenshot-app OUT.png`: the LIVE-`App` capture (item 188) — hermetic,
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

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--bench-typing" => {
                bench_typing = true;
            }
            "--bench-perf" => {
                bench_perf = true;
            }
            "--bench-frame" => {
                bench_frame = true;
            }
            "--bench-theme-burst" => {
                bench_theme_burst = true;
            }
            #[cfg(not(target_arch = "wasm32"))]
            "--bench-a11y" => {
                bench_a11y = true;
            }
            "--bench-zoom-burst" => {
                bench_zoom_burst = true;
            }
            "--bench-frost" => {
                bench_frost = true;
            }
            "--bench-caret" => {
                bench_caret = true;
            }
            "--bench-suite" => {
                bench_suite = true;
            }
            #[cfg(not(target_arch = "wasm32"))]
            "--soak-gpu" => {
                soak_gpu = true;
            }
            #[cfg(not(target_arch = "wasm32"))]
            "--soak-gpu-seconds" => {
                let v = args.next().ok_or_else(|| {
                    anyhow::anyhow!("--soak-gpu-seconds requires a positive number")
                })?;
                soak_gpu_duration = parse_soak_seconds(&v)?;
                soak_gpu_duration_seen = true;
            }
            "--bench-baseline" => {
                let v = args.next().ok_or_else(|| {
                    anyhow::anyhow!("--bench-baseline requires a path (e.g. benches/baseline.json)")
                })?;
                bench_baseline = Some(PathBuf::from(v));
            }
            "--live-script" => {
                let v = args.next().ok_or_else(|| {
                    anyhow::anyhow!(
                        "--live-script requires a step string (e.g. \"keys Cmd-T; sleep 300; shot open\")"
                    )
                })?;
                live_script = Some(v);
            }
            "--live-shots" => {
                let v = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--live-shots requires a directory"))?;
                live_shots = Some(PathBuf::from(v));
            }
            "--screenshot" => {
                let p = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--screenshot requires an output path"))?;
                out = Some(PathBuf::from(p));
                capture_modes.push("--screenshot");
            }
            "--screenshot-motion" => {
                let p = args.next().ok_or_else(|| {
                    anyhow::anyhow!("--screenshot-motion requires an output path")
                })?;
                out = Some(PathBuf::from(p));
                motion = true;
                capture_modes.push("--screenshot-motion");
            }
            "--screenshot-motion-v" => {
                let p = args.next().ok_or_else(|| {
                    anyhow::anyhow!("--screenshot-motion-v requires an output path")
                })?;
                out = Some(PathBuf::from(p));
                motion_v = true;
                capture_modes.push("--screenshot-motion-v");
            }
            "--screenshot-motion-d" => {
                let p = args.next().ok_or_else(|| {
                    anyhow::anyhow!("--screenshot-motion-d requires an output path")
                })?;
                out = Some(PathBuf::from(p));
                motion_d = true;
                capture_modes.push("--screenshot-motion-d");
            }
            #[cfg(not(target_arch = "wasm32"))]
            "--screenshot-app" => {
                let p = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--screenshot-app requires an output path"))?;
                out = Some(PathBuf::from(p));
                live_app = true;
                capture_modes.push("--screenshot-app");
            }
            #[cfg(not(target_arch = "wasm32"))]
            "--semantic-json" => capture_modes.push("--semantic-json"),
            #[cfg(not(target_arch = "wasm32"))]
            "--screenshot-frames" => {
                // `--screenshot-frames N OUT.png`: the frame COUNT then the output path
                // (the count is the flag's headline, mirroring the task's shape; OUT
                // stays explicit like every other screenshot flag).
                let n = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--screenshot-frames requires <N> <out.png>"))?;
                let n: u32 = n.parse().map_err(|e| {
                    anyhow::anyhow!("--screenshot-frames <N> must be an integer: {e}")
                })?;
                let p = args.next().ok_or_else(|| {
                    anyhow::anyhow!("--screenshot-frames requires an output path after <N>")
                })?;
                out = Some(PathBuf::from(p));
                frames = Some(n);
                capture_modes.push("--screenshot-frames");
            }
            #[cfg(not(target_arch = "wasm32"))]
            "--frame-step-ms" => {
                let v = args.next().ok_or_else(|| {
                    anyhow::anyhow!("--frame-step-ms requires a millisecond step")
                })?;
                frame_step_ms = Some(v.parse().map_err(|e| {
                    anyhow::anyhow!("--frame-step-ms must be a positive integer: {e}")
                })?);
            }
            "--capture-timeline" => {
                // `--capture-timeline "<ms,ms,...>" OUT.png`: a cumulative-ms step
                // sequence FOLLOWED by the output path.
                let spec = args.next().ok_or_else(|| {
                    anyhow::anyhow!("--capture-timeline requires a \"<ms,ms,...>\" step sequence")
                })?;
                let p = args.next().ok_or_else(|| {
                    anyhow::anyhow!("--capture-timeline requires an output path after the steps")
                })?;
                timeline_steps = Some(parse_steps(&spec)?);
                out = Some(PathBuf::from(p));
                capture_modes.push("--capture-timeline");
            }
            "--capture-held" => {
                // `--capture-held DIR "<ms,ms,...>" OUT.png`: a held arrow
                // direction, a cumulative-ms step sequence, then the output path.
                let d = args.next().ok_or_else(|| {
                    anyhow::anyhow!("--capture-held requires a direction (left|right|up|down)")
                })?;
                let spec = args.next().ok_or_else(|| {
                    anyhow::anyhow!("--capture-held requires a \"<ms,ms,...>\" step sequence")
                })?;
                let p = args.next().ok_or_else(|| {
                    anyhow::anyhow!("--capture-held requires an output path after the steps")
                })?;
                held = Some((parse_held_dir(&d)?, parse_steps(&spec)?));
                out = Some(PathBuf::from(p));
                capture_modes.push("--capture-held");
            }
            "--storyboard" => {
                let p = args.next().ok_or_else(|| {
                    anyhow::anyhow!("--storyboard requires a storyboard .toml path")
                })?;
                storyboard_arg = Some(PathBuf::from(p));
                capture_modes.push("--storyboard");
            }
            "--storyboard-out" => {
                let p = args.next().ok_or_else(|| {
                    anyhow::anyhow!("--storyboard-out requires an output directory")
                })?;
                storyboard_out = Some(PathBuf::from(p));
            }
            "--sel" => {
                let v = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--sel requires L0:C0-L1:C1"))?;
                opts.selection = Some(parse_sel(&v)?);
            }
            "--zoom" => {
                let v = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--zoom requires a factor (e.g. 1.6)"))?;
                opts.zoom = Some(parse_zoom(&v)?);
            }
            "--capture-size" => {
                let v = args.next().ok_or_else(|| {
                    anyhow::anyhow!("--capture-size requires WxH (e.g. 2400x1600)")
                })?;
                capture_size = Some(parse_size(&v)?);
            }
            "--capture-dpi" => {
                let v = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--capture-dpi requires a factor (e.g. 2.0)"))?;
                capture_dpi = Some(parse_dpi(&v)?);
            }
            "--scroll" => {
                // Keep the capture hook's row anchor and fixed-point remainder
                // together at the CLI boundary; normalization waits until shaping
                // has supplied real variable-row geometry.
                let v = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--scroll requires ROW[:SUBPX]"))?;
                let parsed = super::scroll_arg::parse(&v)?;
                let row = parsed.row;
                let px_q = parsed.px_q;
                opts.scroll = Some(crate::render::ScrollPos { row, px_q });
            }
            "--preedit" => {
                let v = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--preedit requires a string"))?;
                opts.preedit = Some(v);
            }
            "--search" => {
                let v = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--search requires a query"))?;
                opts.search = Some(v);
            }
            "--search-case" => {
                opts.search_case_sensitive = true;
            }
            "--search-replace" => {
                // Reveal the labeled REPLACE row + the key-hint line on the panel (the
                // fresh Cmd-R open state: find field focused, empty replacement). A
                // `--keys` replay can drive the panel further — typing the replacement,
                // replacing — through the shared search-key seam (`search::keys`).
                opts.search_replace_active = true;
            }
            "--theme" => {
                let v = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--theme requires a world name"))?;
                // Set the process-global active theme NOW so it composes with any
                // capture mode (the headless render reads the active theme). Order
                // among flags is irrelevant since the active theme is global.
                theme::set_active_by_name(&v).ok_or_else(|| {
                    // The roster comes from the one code-owned source
                    // (`theme::world_names`, item 68) — never a hand-copied
                    // list that can drift from the real `theme::THEMES`.
                    anyhow::anyhow!(
                        "unknown --theme {v:?}; choose one of {}",
                        theme::world_names().join(", ")
                    )
                })?;
                theme_flag = true;
            }
            "--caret-mode" => {
                let v = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--caret-mode requires 'block' or 'morph'"))?;
                // Pin the process-global caret mode so the headless render is
                // deterministic and verifiable. 'auto' clears any override and
                // falls back to the font-derived default (Block on mono).
                match v.to_ascii_lowercase().as_str() {
                    "block" => caret::set_mode(caret::CaretMode::Block),
                    "morph" => caret::set_mode(caret::CaretMode::Morph),
                    "ibeam" => caret::set_mode(caret::CaretMode::Ibeam),
                    "auto" => {} // leave the font-derived default in effect
                    _ => bail!("unknown --caret-mode {v:?}; choose block, morph, ibeam, or auto"),
                }
                caret_flag = true;
            }
            "--measure" => {
                let v = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--measure requires a char count"))?;
                let n = parse_measure(&v)?;
                // Setting a measure implies page mode ON (so the narrow column +
                // gradient margins are visible in the capture).
                page::set_measure(n);
                page::set_page_on(true);
                page_flag = true;
                measure_flag = true;
            }
            "--page" => {
                let v = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--page requires 'on' or 'off'"))?;
                match v.to_ascii_lowercase().as_str() {
                    "on" => page::set_page_on(true),
                    "off" => page::set_page_on(false),
                    _ => bail!("unknown --page {v:?}; choose on or off"),
                }
                page_flag = true;
            }
            "--debug" => {
                // Opt-in DEBUG panel. Sets the process-global so it composes with any
                // capture mode; the frametime line shows a FIXED placeholder with no
                // live clock (deterministic), while the rest of the panel is a pure
                // function of the view state — so an explicit `--debug` capture stays
                // stable and a plain capture (panel OFF) is byte-identical.
                debug::set_debug_on(true);
            }
            "--hud" => {
                // Summon the HELD STATS HUD for the capture. Sets the process-global
                // so it composes with any capture mode; the clock / file-date fields
                // render FIXED placeholders (no live clock), so an explicit `--hud`
                // capture is deterministic while a plain capture (HUD released) is
                // byte-identical. The live window summons it by HOLDING the binding
                // (Option-Cmd-I) instead.
                hud::set_held(true);
            }
            "--menu-bar" => {
                // Show the WEB/LINUX MENU BAR for the capture (mirrors `--hud`). Sets
                // the process-global so it composes with any capture mode; the bar is
                // pure geometry + theme (no clock), so an explicit `--menu-bar` capture
                // is deterministic while a plain capture (default OFF on macOS) is
                // byte-identical. On web/Linux the live app shows it by default.
                crate::menubar::set_menu_bar_on(true);
            }
            "--menu-open" => {
                // Show the menu bar AND drop the dropdown for menu index N (0 = the App
                // menu), so a capture can exercise the open-dropdown render + sidecar
                // `menubar.open_menu` deterministically. A bad/absent index just shows
                // the closed bar.
                crate::menubar::set_menu_bar_on(true);
                if let Some(n) = args.next().and_then(|s| s.parse::<usize>().ok()) {
                    crate::menubar::set_open(Some(n));
                }
            }
            "--lifetime" => {
                // Summon the LIFETIME STATS card for the capture (mirrors `--hud`).
                // Sets the process-global so it composes with any capture mode; the
                // odometer figures render FIXED "—" placeholders (no live persisted
                // store), so an explicit `--lifetime` capture is deterministic while a
                // plain capture (card closed) is byte-identical. The live app summons
                // it via the palette "Lifetime stats" command instead.
                lifetime::set_open(true);
            }
            "--streaks" => {
                // Summon the WRITING STREAKS card for the capture (mirrors `--lifetime`).
                // Sets the process-global so it composes with any capture mode; the card
                // renders the FIXED synthetic `streaks::placeholder` year + streak numbers
                // (no live persisted store), so an explicit `--streaks` capture is
                // deterministic + byte-stable while a plain capture (card closed) is
                // byte-identical. The live app summons it via the palette "Writing
                // streaks" command instead.
                crate::streaks::set_open(true);
            }
            "--peek" => {
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
            "--whichkey" => {
                // Summon the WHICH-KEY continuation panel for the capture. Sets the
                // process-global so it composes with any capture mode; `run.rs` then
                // derives the `C-x` rows from the catalog + config and renders the
                // SETTLED summoned panel (the live 500ms pause is windowed). A plain
                // capture (unset) draws no panel and stays byte-identical. The live
                // window summons it by pressing `C-x` and PAUSING instead.
                whichkey::set_force_shown(true);
            }
            "--keys" => {
                let v = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--keys requires a key-spec string"))?;
                keys_spec = Some(v);
            }
            "--strict-replay" => {
                strict_replay = true;
            }
            "--config" => {
                let v = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--config requires a path"))?;
                config_arg = Some(PathBuf::from(v));
            }
            // THE DATA-ROOT SEED SLOT. Hermetic-scenario doors only — it gives
            // a `--screenshot-app` capture a starting store (an
            // unresolved-change record, a scratch stash, a session), which is
            // the one thing the sandbox has no other way to hold. See
            // `crate::scenario::data_root_seeds`.
            "--seed-data" => {
                let v = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--seed-data requires a directory"))?;
                data_seed = Some(PathBuf::from(v));
            }
            "--root" => {
                let v = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--root requires a directory"))?;
                root = Some(PathBuf::from(v));
            }
            "--workspace" => {
                let v = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--workspace requires a directory"))?;
                workspace = Some(PathBuf::from(v));
            }
            "--default-folder" => {
                let v = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--default-folder requires a directory"))?;
                default_folder = Some(PathBuf::from(v));
            }
            "--wait" => {
                wait_flag = true;
            }
            "--list-worlds" => {
                // Machine-readable roster dump — one world name per line, in
                // `theme::THEMES` cycle order — read straight off the ONE
                // code-owned source (item 68: `--help` once drifted to only
                // ten of the twenty shipped worlds; a script that shells
                // out to THIS flag can never drift the same way, since it
                // never keeps its own copy of the list). See
                // `scripts/capture-worlds.sh`.
                for name in theme::world_names() {
                    println!("{name}");
                }
                std::process::exit(0);
            }
            // THE ICON EXPORT MANIFEST (item 92) — the same one-owner move as
            // `--list-worlds`, one step richer: the offline icon compositor in
            // `scripts/icons/` needs each world's four icon palette tokens and
            // its display face, so it SHELLS OUT for them rather than keeping a
            // second copy of the palette that could drift from `worlds.rs`.
            // Reads the bundled faces from `assets/fonts` relative to the
            // CURRENT DIRECTORY (run it from the repo root) — never a path
            // baked in at build time, which would end up personal-machine
            // specific in a public repo.
            #[cfg(not(target_arch = "wasm32"))]
            "--icon-manifest" => {
                let dir = PathBuf::from(crate::icon_manifest::DEFAULT_FONTS_DIR);
                print!("{}", crate::icon_manifest::manifest_json(&dir)?);
                std::process::exit(0);
            }
            // ITEM 121's ground audition: see `icon_manifest::ground_audition_json`.
            #[cfg(not(target_arch = "wasm32"))]
            "--ground-audition" => {
                let world = args
                    .next()
                    .ok_or_else(|| anyhow::anyhow!("--ground-audition needs a WORLD name"))?;
                let dir = PathBuf::from(crate::icon_manifest::DEFAULT_FONTS_DIR);
                print!(
                    "{}",
                    crate::icon_manifest::ground_audition_json(&world, &dir)?
                );
                std::process::exit(0);
            }
            // THE PACK STEP (item 92) — cut every shipped world's rendered
            // tiles into a real `.icns`, write the canonical bundle icon, and
            // regenerate the embedded table. Run from the repo root, AFTER the
            // web compositor has rendered the tiles; `scripts/export-icons.sh`
            // does both in order. Packing lives in the binary rather than in a
            // shell script so the container format has one owner and the
            // determinism law can re-pack a committed asset inside `cargo test`.
            #[cfg(not(target_arch = "wasm32"))]
            "--pack-icns" => {
                let tiles = args
                    .next()
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
            "-h" | "--help" => {
                // Built from the same one-owner roster as `--theme`'s error
                // and `--list-worlds` (item 68) — never a hand-copied list.
                let world_names_csv = theme::world_names().join(", ");
                println!(
                    "awl [file]\n\
                     awl --screenshot OUT.png [file]         caret at rest (rounded square)\n\
                     awl --screenshot-motion OUT.png [file]  caret mid-glide (centred trailing streak)\n\
                     awl --screenshot-motion-v OUT.png [file] caret mid-glide vertical (left-edge bar)\n\
                     awl --screenshot-motion-d OUT.png [file] caret mid-glide diagonal (slanted tracer)\n\
                     awl --screenshot-app OUT.png [file]     drive --keys into a REAL \
                     headless App (hermetic) and capture ITS state — the only door that sees a \
                     live-App-only transition; sidecar carries driver: \"live-app\"\n\
                     awl --capture-timeline \"0,16,50,150\" OUT.png [file]  deterministic timeline: step the caret glide by injected ms, frame per step (OUT.t<ms>.png)\n\
                     awl --capture-held DIR \"0,30,60,90\" OUT.png [file]  deterministic HELD arrow (DIR=left|right|up|down): re-target one char/line per step (held=true), frame per step with trail geometry\n\
                     \n\
                     verification hooks (compose with --screenshot):\n\
                     \x20 --sel L0:C0-L1:C1   selection highlight from (l0,c0)..(l1,c1)\n\
                     \x20 --zoom F            zoom factor (0.5..3.0)\n\
                     \x20 --scroll N[:Q]     scroll to row N plus Q fixed 1/64px units\n\
                     \x20 --preedit STR       render STR as an IME preedit at the caret\n\
                     \x20 --search STR        open isearch panel for STR + highlight hits\n\
                     \x20 --search-case       make --search case-sensitive\n\
                     \x20 --theme NAME        set the active color theme ({world_names_csv})\n\
                     \x20 --list-worlds       print every theme name, one per line, then exit (the roster `--theme` accepts; see scripts/capture-worlds.sh)\n\
                     \x20 --icon-manifest     print the app-icon export manifest as JSON (per world: icon palette tokens + display face + its logo-cursor; per face: the bundled font files), then exit — run from the repo root; see scripts/icons/\n\
                     \x20 --ground-audition W item 121's A/B/C ground-audition manifest, exit\n\
                     \x20 --pack-icns [DIR]   cut every world's rendered tiles (default assets/macos/candidates/tiles) into assets/macos/world/<World>.icns + the canonical assets/macos/Awl.icns, and regenerate src/app_icon/embedded.rs, then exit — run from the repo root AFTER scripts/export-icons.sh\n\
                     \x20 --caret-mode MODE   caret look: block, morph, ibeam, or auto (default: mono->block, proportional->morph)\n\
                     \x20 --capture-size WxH  physical canvas size for the capture (default 1200x800)\n\
                     \x20 --capture-dpi N      renderer scale factor (default 1.0); WxH at dpi N == (W/N)x(H/N) logical retina window\n\
                     \x20 --measure N         page-mode column width in chars (default 80; implies --page on)\n\
                     \x20 --page on|off       page mode: centered column (on, default) vs edge-to-edge (off)\n\
                     \x20 --debug             DEBUG: draw the dim top-left dev panel — frametime/zoom/viewport/cursor/theme/md+syn (OFF by default; frametime is a fixed placeholder in a headless capture)\n\
                     \x20 --hud               summon the HELD stats HUD (live: hold Option-Cmd-I; clock/file-date fields are fixed placeholders in a capture)\n\
                     \x20 --menu-bar          show the web/Linux MENU BAR (default on web/Linux, off on macOS which has the native bar); --menu-open N drops menu N's dropdown\n\
                     \x20 --peek              summon the HOLD-⌘ shortcut peek (live: hold the convention's bare arming modifier — ⌘ on Mac, Ctrl on Linux — ~600ms; a capture shows the curated starter six)\n\
                     \x20 --streaks           summon the WRITING STREAKS card (live: palette \"Writing streaks\"; a capture shows a fixed synthetic year + streak numbers)\n\
                     \x20 --whichkey          summon the WHICH-KEY panel: the C-x prefix's follow-up keys (live: press C-x and pause ~500ms)\n\
                     \x20 --default-folder DIR    fallback active folder for a first launch with nothing remembered (default ~/notes)\n\
                     \x20 --config PATH       load settings from PATH (default ~/.config/awl/config.toml)\n\
                     \x20 --wait              windowed editor only: single-instance daemon — hand `file` to an already-running awl and block until C-x # finishes it (EDITOR=awl --wait for git)\n\
                     \x20 --keys \"SPEC\"        replay emacs chords (e.g. \"C-n C-n M->\") then capture\n\
                     \x20 --seed-data DIR     seed awl's own DATA ROOT (the \
                     unresolved-change record, the scratch stash, session.toml, history) into a \
                     hermetic scenario sandbox from DIR's files — the only way a --screenshot-app \
                     run can START from state awl already had; refused outside a hermetic door\n\
                     \x20 --strict-replay     with --screenshot --keys: abort (naming the offender) on an unbound chord, a live-only effect the replay can't perform, or a missing layout oracle; runs HERMETIC (an in-memory fs seeded from the named file + --config — a replayed save never touches the real file, the user's own config/notes/history are never read or written)\n\
                     \x20 --storyboard TOML   run a scenario storyboard (press/type/pause/run_for/expect steps — see scenarios/): strict + hermetic, emitting per-step PNG+JSON, deterministic film frames, a byte-stable trace.json, and (with ffmpeg on PATH) film.webm/film.mp4\n\
                     \x20 --storyboard-out DIR where the storyboard run's artifacts land (default: <storyboard>.run/ beside the .toml)"
                );
                std::process::exit(0);
            }
            s if s.starts_with("--") => bail!("unknown flag: {s}"),
            s => file = Some(PathBuf::from(s)),
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
    let semantic_json = capture_modes.contains(&"--semantic-json");
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
    //    single-mode check above at most one mode category is active, so this
    //    mirrors the Mode construction's precedence (held > timeline > motion >
    //    screenshot; no output = windowed).
    let kind = if out.is_none() {
        CaptureKind::Windowed
    } else if held.is_some() {
        CaptureKind::Held
    } else if timeline_steps.is_some() {
        CaptureKind::Timeline
    } else if motion || motion_v || motion_d {
        CaptureKind::Motion
    } else {
        CaptureKind::Screenshot
    };
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
    // resolved above, plus its parent-directory marker), and item 188's
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
        );
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
            },
        },
        #[cfg(not(target_arch = "wasm32"))]
        Some(out) if frames.is_some() => Mode::ScreenshotFrames {
            out,
            file,
            frames: frames.unwrap(),
            step_ms: frame_step_ms.unwrap_or(capture::DEFAULT_FRAME_STEP_MS),
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

/// Resolve the DEFAULT FOLDER (item 76 — the first-run launch fallback ONLY,
/// never the active folder once running): explicit `--default-folder`, else
/// `~/notes` (`$HOME/notes`), else `./notes` if HOME is unset. The directory is
/// created lazily on first use.
pub(crate) fn resolve_default_folder(default_folder: &Option<PathBuf>) -> PathBuf {
    if let Some(n) = default_folder {
        return n.clone();
    }
    match std::env::var_os("HOME") {
        Some(home) => PathBuf::from(home).join("notes"),
        None => PathBuf::from("notes"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sel_orders_endpoints_and_rejects_malformed() {
        assert_eq!(parse_sel("0:0-2:3").unwrap(), ((0, 0), (2, 3)));
        assert_eq!(parse_sel("2:3-0:0").unwrap(), ((0, 0), (2, 3)));
        assert_eq!(parse_sel(" 1:2 - 1:5 ").unwrap(), ((1, 2), (1, 5)));
        parse_sel("0:0").unwrap_err();
        parse_sel("00-23").unwrap_err();
        parse_sel("a:b-c:d").unwrap_err();
    }

    #[test]
    fn parse_steps_reads_ms_and_rejects_junk() {
        assert_eq!(parse_steps("0,16,50,150").unwrap(), vec![0, 16, 50, 150]);
        // Whitespace + trailing/empty entries are tolerated.
        assert_eq!(parse_steps(" 0 , 30 ,").unwrap(), vec![0, 30]);
        // Empty / all-blank / non-numeric are errors.
        parse_steps("").unwrap_err();
        parse_steps("  ,  ").unwrap_err();
        parse_steps("0,x,2").unwrap_err();
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn parse_soak_duration_accepts_short_runs_and_rejects_non_positive() {
        assert_eq!(
            parse_soak_seconds("0.25").unwrap(),
            std::time::Duration::from_millis(250)
        );
        assert_eq!(
            parse_soak_seconds("900").unwrap(),
            crate::soak_gpu::DEFAULT_DURATION
        );
        for bad in ["0", "-1", "NaN", "inf", "nope"] {
            assert!(parse_soak_seconds(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn parse_size_accepts_both_separators_and_rejects_zero() {
        assert_eq!(parse_size("2400x1600").unwrap(), (2400, 1600));
        assert_eq!(parse_size("800X600").unwrap(), (800, 600));
        // Missing separator, zero dimension, non-numeric are errors.
        parse_size("1200").unwrap_err();
        parse_size("0x600").unwrap_err();
        parse_size("800x0").unwrap_err();
        parse_size("axb").unwrap_err();
    }

    #[test]
    fn parse_held_dir_accepts_aliases_and_rejects_bad() {
        assert!(parse_held_dir("left").unwrap() == capture::HeldDir::Left);
        assert!(parse_held_dir("L").unwrap() == capture::HeldDir::Left);
        assert!(parse_held_dir("RIGHT").unwrap() == capture::HeldDir::Right);
        assert!(parse_held_dir("u").unwrap() == capture::HeldDir::Up);
        assert!(parse_held_dir("Down").unwrap() == capture::HeldDir::Down);
        assert!(parse_held_dir("sideways").is_err());
        assert!(parse_held_dir("").is_err());
    }

    #[test]
    fn parse_dpi_requires_finite_positive() {
        assert_eq!(parse_dpi("2.0").unwrap(), 2.0);
        assert_eq!(parse_dpi(" 1 ").unwrap(), 1.0);
        // Zero, negative, non-finite, and non-numeric are all errors (mirrors
        // parse_size's non-zero guard).
        parse_dpi("0").unwrap_err();
        parse_dpi("-1.5").unwrap_err();
        parse_dpi("inf").unwrap_err();
        parse_dpi("nan").unwrap_err();
        parse_dpi("x").unwrap_err();
    }

    #[test]
    fn parse_zoom_requires_finite_positive() {
        assert_eq!(parse_zoom("1.6").unwrap(), 1.6);
        assert_eq!(parse_zoom(" 0.5 ").unwrap(), 0.5);
        // Zero, negative, non-finite, and non-numeric are all errors (mirrors
        // parse_dpi's guard) — a NaN factor would otherwise poison every
        // zoom-derived metric downstream.
        parse_zoom("0").unwrap_err();
        parse_zoom("-1").unwrap_err();
        parse_zoom("inf").unwrap_err();
        parse_zoom("nan").unwrap_err();
        parse_zoom("x").unwrap_err();
    }

    #[test]
    fn clamp_zoom_never_returns_non_finite() {
        // The LAST line of defence behind the --zoom / config seams above:
        // `render::clamp_zoom` must yield a finite in-range factor for ANY input.
        // (Tested here beside the zoom-flag seam; render/tests/geometry.rs owns
        // the geometry suite.) NaN — the propagating poison — falls back to the 1.0
        // default; ±inf saturates through the ordinary clamp.
        use crate::range::ZOOM;
        use crate::render::clamp_zoom;
        let (zmin, zmax) = (ZOOM.min, ZOOM.max);
        for z in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 0.0, -7.0, 1e30] {
            let c = clamp_zoom(z);
            assert!(
                c.is_finite() && (zmin..=zmax).contains(&c),
                "clamp_zoom({z}) -> {c} must be finite in [{zmin}, {zmax}]"
            );
        }
        assert_eq!(clamp_zoom(f32::NAN), 1.0, "NaN falls back to the default");
        assert_eq!(clamp_zoom(f32::INFINITY), zmax, "+inf saturates high");
        assert_eq!(clamp_zoom(f32::NEG_INFINITY), zmin, "-inf saturates low");
        // A normal factor still step-rounds + clamps exactly as before.
        assert!(
            (clamp_zoom(1.234) - 1.2).abs() < 1e-5,
            "step rounding unchanged"
        );
        assert_eq!(clamp_zoom(9.0), zmax);
        assert_eq!(clamp_zoom(0.0), zmin);
    }

    #[test]
    fn parse_measure_requires_positive() {
        assert_eq!(parse_measure("80").unwrap(), 80);
        assert_eq!(parse_measure(" 40 ").unwrap(), 40);
        // Zero and non-numeric are errors (mirrors parse_size's non-zero guard).
        parse_measure("0").unwrap_err();
        parse_measure("-1").unwrap_err();
        parse_measure("x").unwrap_err();
    }

    #[test]
    fn single_capture_mode_rejects_conflicts() {
        // Zero or one capture-mode flag is fine.
        ensure_single_capture_mode(&[]).unwrap();
        ensure_single_capture_mode(&["--screenshot"]).unwrap();
        // Two distinct modes — or the same flag twice — is a conflict.
        assert!(ensure_single_capture_mode(&["--screenshot", "--capture-held"]).is_err());
        assert!(ensure_single_capture_mode(&["--screenshot", "--screenshot"]).is_err());
        // The error names every conflicting flag.
        let msg = ensure_single_capture_mode(&["--screenshot", "--screenshot-motion"])
            .unwrap_err()
            .to_string();
        assert!(msg.contains("--screenshot") && msg.contains("--screenshot-motion"));
    }

    #[test]
    fn unused_hooks_flags_only_what_a_mode_drops() {
        // A plain screenshot honors every hook → nothing unused.
        let all = SuppliedHooks {
            sel: true,
            zoom: true,
            scroll: true,
            preedit: true,
            search: true,
            search_case: true,
            search_replace: true,
            capture_size: true,
            capture_dpi: true,
            root: true,
            workspace: true,
            default_folder: true,
        };
        assert!(unused_hooks(CaptureKind::Screenshot, &all).is_empty());

        // Motion threads only keys/file: every other hook is dropped.
        let motion = unused_hooks(CaptureKind::Motion, &all);
        for f in [
            "--sel",
            "--zoom",
            "--scroll",
            "--preedit",
            "--search",
            "--search-case",
            "--search-replace",
            "--capture-size",
            "--capture-dpi",
            "--root",
            "--workspace",
            "--default-folder",
        ] {
            assert!(motion.contains(&f), "motion should drop {f}");
        }

        // Timeline / held carry root + canvas/dpi but still drop the per-frame
        // render hooks and workspace/default-folder.
        for kind in [CaptureKind::Timeline, CaptureKind::Held] {
            let u = unused_hooks(kind, &all);
            assert!(u.contains(&"--sel") && u.contains(&"--search-case"));
            assert!(u.contains(&"--workspace") && u.contains(&"--default-folder"));
            assert!(!u.contains(&"--root"));
            assert!(!u.contains(&"--capture-size") && !u.contains(&"--capture-dpi"));
        }

        // The windowed editor honors project context but not capture hooks.
        let win = unused_hooks(CaptureKind::Windowed, &all);
        assert!(win.contains(&"--sel") && win.contains(&"--capture-size"));
        assert!(!win.contains(&"--root"));
        assert!(!win.contains(&"--workspace") && !win.contains(&"--default-folder"));

        // Nothing supplied → nothing unused, for every mode.
        let none = SuppliedHooks::default();
        for kind in [
            CaptureKind::Windowed,
            CaptureKind::Screenshot,
            CaptureKind::Motion,
            CaptureKind::Timeline,
            CaptureKind::Held,
        ] {
            assert!(unused_hooks(kind, &none).is_empty());
        }
    }
}
