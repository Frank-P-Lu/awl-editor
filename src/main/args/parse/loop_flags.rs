//! The argument-token loop — read-only, delegates to `parse::Ctx`'s parent
//! for its own doc. Split out of `args/parse.rs` to stay under the file's
//! 500-line ceiling; `parse_flag_loop` is the whole content.

use super::*;

/// THE ONE ARGUMENT LOOP. `flags::lookup` is the only place a `--…` token
/// becomes a flag and `take_operands` the only place its operands leave the
/// stream, so the dispatch below reads DATA rather than argv: the roster
/// decides what exists, this match decides what it does. NO WILDCARD — a new
/// roster row fails to compile here until it is read.
///
/// A native-only body carries the `cfg` its own module already carries. The
/// ROW stays unconditional: `fn main` is the native entry (`wasm_start` never
/// calls this function), so a browser build has no command line to keep a
/// second, shorter roster for.
pub(super) fn parse_flag_loop(ctx: &mut Ctx) -> Result<()> {
    let mut args = std::env::args().skip(1).peekable();
    while let Some(arg) = args.next() {
        let Some(flag) = flags::lookup(arg.as_str()) else {
            if arg.starts_with("--") {
                bail!("unknown flag: {arg}");
            }
            ctx.file = Some(PathBuf::from(arg));
            continue;
        };
        let ops = flag.take_operands(&mut args)?;
        match flag.id {
            FlagId::BenchTyping => ctx.bench_typing = true,
            FlagId::BenchPerf => ctx.bench_perf = true,
            FlagId::BenchFrame => ctx.bench_frame = true,
            FlagId::BenchThemeBurst => ctx.bench_theme_burst = true,
            FlagId::BenchA11y => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    ctx.bench_a11y = true;
                }
            }
            FlagId::BenchZoomBurst => ctx.bench_zoom_burst = true,
            FlagId::BenchFrost => ctx.bench_frost = true,
            FlagId::BenchCaret => ctx.bench_caret = true,
            FlagId::BenchSuite => ctx.bench_suite = true,
            FlagId::SoakGpu => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    ctx.soak_gpu = true;
                }
            }
            FlagId::SoakGpuSeconds => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    ctx.soak_gpu_duration = parse_soak_seconds(ops.req(0))?;
                    ctx.soak_gpu_duration_seen = true;
                }
            }
            FlagId::BenchBaseline => ctx.bench_baseline = Some(PathBuf::from(ops.req(0))),
            FlagId::LiveScript => ctx.live_script = Some(ops.req(0).to_string()),
            FlagId::LiveShots => ctx.live_shots = Some(PathBuf::from(ops.req(0))),
            FlagId::Screenshot => {
                ctx.out = Some(PathBuf::from(ops.req(0)));
                ctx.capture_modes.push(flag.name());
            }
            FlagId::ScreenshotMotion => {
                ctx.out = Some(PathBuf::from(ops.req(0)));
                ctx.motion = true;
                ctx.capture_modes.push(flag.name());
            }
            FlagId::ScreenshotMotionVertical => {
                ctx.out = Some(PathBuf::from(ops.req(0)));
                ctx.motion_v = true;
                ctx.capture_modes.push(flag.name());
            }
            FlagId::ScreenshotMotionDiagonal => {
                ctx.out = Some(PathBuf::from(ops.req(0)));
                ctx.motion_d = true;
                ctx.capture_modes.push(flag.name());
            }
            FlagId::ScreenshotApp => {
                ctx.out = Some(PathBuf::from(ops.req(0)));
                #[cfg(not(target_arch = "wasm32"))]
                {
                    ctx.live_app = true;
                }
                ctx.capture_modes.push(flag.name());
            }
            FlagId::SemanticJson => ctx.capture_modes.push(flag.name()),
            FlagId::ScreenshotFrames => {
                // The frame COUNT then the output path (the count is the flag's
                // headline, mirroring the task's shape; OUT stays explicit like
                // every other screenshot flag).
                #[cfg(not(target_arch = "wasm32"))]
                {
                    ctx.frames = Some(ops.req(0).parse::<u32>().map_err(|e| {
                        anyhow::anyhow!("--screenshot-frames <N> must be an integer: {e}")
                    })?);
                }
                ctx.out = Some(PathBuf::from(ops.req(1)));
                ctx.capture_modes.push(flag.name());
            }
            FlagId::FrameStepMs => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    ctx.frame_step_ms = Some(ops.req(0).parse::<u64>().map_err(|e| {
                        anyhow::anyhow!("--frame-step-ms must be a positive integer: {e}")
                    })?);
                }
            }
            FlagId::CaptureTimeline => {
                // A cumulative-ms step sequence FOLLOWED by the output path.
                ctx.timeline_steps = Some(parse_steps(ops.req(0))?);
                ctx.out = Some(PathBuf::from(ops.req(1)));
                ctx.capture_modes.push(flag.name());
            }
            FlagId::CaptureHeld => {
                // A held arrow direction, a cumulative-ms step sequence, then
                // the output path.
                ctx.held = Some((parse_held_dir(ops.req(0))?, parse_steps(ops.req(1))?));
                ctx.out = Some(PathBuf::from(ops.req(2)));
                ctx.capture_modes.push(flag.name());
            }
            FlagId::Storyboard => {
                ctx.storyboard_arg = Some(PathBuf::from(ops.req(0)));
                ctx.capture_modes.push(flag.name());
            }
            FlagId::StoryboardOut => ctx.storyboard_out = Some(PathBuf::from(ops.req(0))),
            FlagId::Sel => ctx.opts.selection = Some(parse_sel(ops.req(0))?),
            FlagId::Zoom => ctx.opts.zoom = Some(parse_zoom(ops.req(0))?),
            FlagId::CaptureSize => ctx.capture_size = Some(parse_size(ops.req(0))?),
            FlagId::CaptureDpi => ctx.capture_dpi = Some(parse_dpi(ops.req(0))?),
            FlagId::Scroll => {
                // Keep the capture hook's row anchor and fixed-point remainder
                // together at the CLI boundary; normalization waits until shaping
                // has supplied real variable-row geometry.
                let parsed = crate::scroll_arg::parse(ops.req(0))?;
                let row = parsed.row;
                let px_q = parsed.px_q;
                ctx.opts.scroll = Some(crate::render::ScrollPos { row, px_q });
            }
            FlagId::Preedit => ctx.opts.preedit = Some(ops.req(0).to_string()),
            FlagId::Search => ctx.opts.search = Some(ops.req(0).to_string()),
            FlagId::SearchCase => ctx.opts.search_case_sensitive = true,
            FlagId::SearchReplace => {
                // Reveal the labeled REPLACE row + the key-hint line on the panel (the
                // fresh Cmd-R open state: find field focused, empty replacement). A
                // `--keys` replay can drive the panel further — typing the replacement,
                // replacing — through the shared search-key seam (`search::keys`).
                ctx.opts.search_replace_active = true;
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
                ctx.theme_flag = true;
            }
            FlagId::CaretMode => {
                // Pin the process-global caret mode so the headless render is
                // deterministic and verifiable. 'auto' clears any override and
                // falls back to the universal default (Block, on every world).
                let v = ops.req(0);
                match v.to_ascii_lowercase().as_str() {
                    "block" => caret::set_mode(caret::CaretMode::Block),
                    "morph" => caret::set_mode(caret::CaretMode::Morph),
                    "ibeam" => caret::set_mode(caret::CaretMode::Ibeam),
                    "auto" => {} // leave the universal Block default in effect
                    _ => bail!("unknown --caret-mode {v:?}; choose block, morph, ibeam, or auto"),
                }
                ctx.caret_flag = true;
            }
            FlagId::Measure => {
                let n = parse_measure(ops.req(0))?;
                // Setting a measure implies page mode ON (so the narrow column +
                // gradient margins are visible in the capture).
                page::set_measure(n);
                page::set_page_on(true);
                ctx.page_flag = true;
                ctx.measure_flag = true;
            }
            FlagId::Page => {
                let v = ops.req(0);
                match v.to_ascii_lowercase().as_str() {
                    "on" => page::set_page_on(true),
                    "off" => page::set_page_on(false),
                    _ => bail!("unknown --page {v:?}; choose on or off"),
                }
                ctx.page_flag = true;
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
            FlagId::Keys => ctx.keys_spec = Some(ops.req(0).to_string()),
            FlagId::StrictReplay => ctx.strict_replay = true,
            FlagId::Config => ctx.config_arg = Some(PathBuf::from(ops.req(0))),
            // THE DATA-ROOT SEED SLOT. Hermetic-scenario doors only — it gives
            // a `--screenshot-app` capture a starting store (an
            // unresolved-change record, a scratch stash, a session), which is
            // the one thing the sandbox has no other way to hold. See
            // `crate::scenario::data_root_seeds`.
            FlagId::SeedData => ctx.data_seed = Some(PathBuf::from(ops.req(0))),
            // THE PROJECT-TREE SEED SLOT. Hermetic-scenario doors only — it
            // gives a `--screenshot-app` capture a real multi-file root to open
            // files from, which `--root` alone cannot (that seeds a directory
            // marker, so the folder reads as empty). See
            // `crate::scenario::tree_seeds`.
            FlagId::SeedTree => ctx.tree_seed = Some(PathBuf::from(ops.req(0))),
            FlagId::Root => ctx.root = Some(PathBuf::from(ops.req(0))),
            FlagId::Workspace => ctx.workspace = Some(PathBuf::from(ops.req(0))),
            FlagId::DefaultFolder => ctx.default_folder = Some(PathBuf::from(ops.req(0))),
            FlagId::Wait => ctx.wait_flag = true,
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
    Ok(())
}
