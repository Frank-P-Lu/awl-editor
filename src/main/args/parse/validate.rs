//! The hidden bench/soak short-circuit and CLI-validation phases of
//! `parse::parse_args` — split out of `args/parse.rs` to stay under the
//! file's 500-line ceiling.

use super::*;

/// [`validate_and_parse_storyboard`]'s return: the parsed board (alongside
/// its own path) and its resolved document.
type ParsedStoryboard = (
    Option<(crate::storyboard::Storyboard, PathBuf)>,
    Option<PathBuf>,
);

/// The hidden benchmark / soak-GPU short-circuit modes: each opens no window
/// and no capture, so a match is resolved (or a conflicting-flag combination
/// refused) before any of the CLI-validation phase below ever runs. `Some`
/// short-circuits `parse_args` with that `Mode`; `None` means keep going.
pub(super) fn resolve_early_mode(ctx: &Ctx) -> Result<Option<Mode>> {
    #[cfg(not(target_arch = "wasm32"))]
    if ctx.soak_gpu {
        if ctx.file.is_some()
            || ctx.out.is_some()
            || ctx.keys_spec.is_some()
            || ctx.wait_flag
            || ctx.live_script.is_some()
            || ctx.config_arg.is_some()
            || ctx.root.is_some()
            || ctx.workspace.is_some()
            || ctx.default_folder.is_some()
        {
            bail!(
                "--soak-gpu is isolated; file/capture/input/config/folder arguments do not apply"
            );
        }
        return Ok(Some(Mode::SoakGpu(crate::soak_gpu::SoakConfig {
            duration: ctx.soak_gpu_duration,
        })));
    }
    #[cfg(not(target_arch = "wasm32"))]
    if ctx.soak_gpu_duration_seen {
        bail!("--soak-gpu-seconds requires --soak-gpu");
    }
    if ctx.bench_suite {
        return Ok(Some(Mode::BenchSuite {
            baseline: ctx.bench_baseline.clone(),
        }));
    }
    if ctx.bench_baseline.is_some() {
        bail!("--bench-baseline requires --bench-suite");
    }
    if ctx.bench_typing {
        return Ok(Some(Mode::BenchTyping));
    }
    if ctx.bench_perf {
        return Ok(Some(Mode::BenchPerf));
    }
    if ctx.bench_frame {
        return Ok(Some(Mode::BenchFrame));
    }
    if ctx.bench_theme_burst {
        return Ok(Some(Mode::BenchThemeBurst));
    }
    #[cfg(not(target_arch = "wasm32"))]
    if ctx.bench_a11y {
        return Ok(Some(Mode::BenchA11y));
    }
    if ctx.bench_zoom_burst {
        return Ok(Some(Mode::BenchZoomBurst));
    }
    if ctx.bench_frost {
        return Ok(Some(Mode::BenchFrost));
    }
    if ctx.bench_caret {
        return Ok(Some(Mode::BenchCaret));
    }
    Ok(None)
}

/// CLI VALIDATION (error paths only — valid runs are unaffected): reject
/// every flag combination the chosen capture mode would silently drop or
/// ignore, then resolve the LIVE PROBE + storyboard's own document (its
/// parse is here, not in a later phase, because `install_hermetic_fs` needs
/// `storyboard_file` to seed the sandbox).
pub(super) fn validate_cli(ctx: &mut Ctx) -> Result<()> {
    // 1) At most ONE capture-mode flag. With more than one, the Mode chosen below
    //    would silently follow a precedence and drop the rest; refuse instead.
    ensure_single_capture_mode(&ctx.capture_modes)?;
    // 1b) The LIVE PROBE is the windowed editor only — it composes with the
    //     normal launch flags (file/--theme/--config/--root/…) and with nothing
    //     headless (the whole point is the real window; a capture mode would
    //     silently swallow the script).
    // Derived, not tracked: `capture_modes` already records every capture flag
    // in order, and on wasm the arm that pushes this one does not exist.
    let semantic_json = ctx
        .capture_modes
        .contains(&flags::name_of(FlagId::SemanticJson));
    if ctx.live_script.is_some()
        && (ctx.out.is_some() || ctx.storyboard_arg.is_some() || semantic_json)
    {
        bail!("--live-script drives the real windowed app; it does not compose with capture modes");
    }
    if ctx.live_shots.is_some() && ctx.live_script.is_none() {
        bail!("--live-shots requires --live-script");
    }
    let live = match &ctx.live_script {
        Some(spec) => Some(crate::probe::LiveScript {
            steps: crate::probe::parse_script(spec)?,
            shots_dir: ctx.live_shots.clone().unwrap_or_else(std::env::temp_dir),
        }),
        None => None,
    };
    // 2) Reject verification hooks the chosen mode would silently ignore. After the
    //    single-mode check above at most one mode category is active, so
    //    `resolve_capture_kind` need only classify which one.
    // `live_app` is native-only; a wasm build has no `--screenshot-app` door, so
    // its `kind` can only ever fall through to `Screenshot` here.
    #[cfg(not(target_arch = "wasm32"))]
    let is_screenshot_app = ctx.live_app;
    #[cfg(target_arch = "wasm32")]
    let is_screenshot_app = false;
    // `frames` is native-only for the identical reason (`--screenshot-frames`
    // builds a hermetic `App`; the flag + its `Mode` do not exist on wasm).
    #[cfg(not(target_arch = "wasm32"))]
    let is_screenshot_frames = ctx.frames.is_some();
    #[cfg(target_arch = "wasm32")]
    let is_screenshot_frames = false;
    let kind = resolve_capture_kind(
        ctx.out.is_some(),
        ctx.held.is_some(),
        ctx.timeline_steps.is_some(),
        ctx.motion || ctx.motion_v || ctx.motion_d,
        is_screenshot_app,
        is_screenshot_frames,
    );
    let supplied = SuppliedHooks {
        sel: ctx.opts.selection.is_some(),
        zoom: ctx.opts.zoom.is_some(),
        scroll: ctx.opts.scroll.is_some(),
        preedit: ctx.opts.preedit.is_some(),
        search: ctx.opts.search.is_some(),
        search_case: ctx.opts.search_case_sensitive,
        search_replace: ctx.opts.search_replace_active,
        capture_size: ctx.capture_size.is_some(),
        capture_dpi: ctx.capture_dpi.is_some(),
        root: ctx.root.is_some(),
        workspace: ctx.workspace.is_some(),
        default_folder: ctx.default_folder.is_some(),
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
    if ctx.frames.is_some() && ctx.keys_spec.is_some() {
        bail!("--screenshot-frames drives its own scheduling loop; --keys does not apply");
    }
    // `--strict-replay` gates a `--keys` replay, and only the plain
    // `--screenshot` mode threads the strict engine (the motion/timeline/held
    // variants stay permissive one-offs); refuse the combinations that would
    // silently ignore it. Validated BEFORE the hermetic install below so a
    // refused flag combination never swaps the process filesystem first.
    if ctx.strict_replay {
        if ctx.keys_spec.is_none() {
            bail!("--strict-replay requires --keys (there is no replay to be strict about)");
        }
        if kind != CaptureKind::Screenshot {
            bail!(
                "--strict-replay only applies to --screenshot (not motion/timeline/held captures)"
            );
        }
    }
    let (storyboard, storyboard_file) = validate_and_parse_storyboard(ctx)?;

    ctx.semantic_json = semantic_json;
    ctx.live = live;
    ctx.storyboard = storyboard;
    ctx.storyboard_file = storyboard_file;
    Ok(())
}

/// `--storyboard` drives its own input/document; refuse the flags it would
/// silently ignore, then parse the scenario file NOW (std::fs — the one
/// boundary crossing before the sandbox exists) so its named document can
/// seed the hermetic sandbox in `install_hermetic_fs`, exactly like
/// `--strict-replay`'s file. Returns the parsed board (alongside its own
/// path, for a default `--storyboard-out`) and its document, resolved
/// against the storyboard file's own directory (so a checked-in
/// `scenarios/demo.toml` names its fixture as `demo.md`).
fn validate_and_parse_storyboard(ctx: &Ctx) -> Result<ParsedStoryboard> {
    if ctx.storyboard_arg.is_some() {
        if ctx.keys_spec.is_some() {
            bail!("--storyboard drives its own steps; --keys does not apply");
        }
        if ctx.file.is_some() {
            bail!(
                "--storyboard takes its document from the storyboard file; drop the file argument"
            );
        }
        if ctx.wait_flag {
            bail!("--wait only applies to the windowed editor (no capture mode)");
        }
        if ctx.strict_replay {
            bail!("--storyboard is always strict; --strict-replay does not apply");
        }
    } else if ctx.storyboard_out.is_some() {
        bail!("--storyboard-out requires --storyboard");
    }
    let storyboard: Option<(crate::storyboard::Storyboard, PathBuf)> = match &ctx.storyboard_arg {
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
    let storyboard_file: Option<PathBuf> = storyboard.as_ref().and_then(|(b, p)| {
        b.file.as_ref().map(|f| match p.parent() {
            Some(dir) if !dir.as_os_str().is_empty() => dir.join(f),
            _ => PathBuf::from(f),
        })
    });
    Ok((storyboard, storyboard_file))
}

/// HERMETIC SCENARIO FILESYSTEM — the ONE production call (`crate::scenario`'s
/// module doc is the contract): a scenario run swaps the process fs to an
/// in-memory sandbox seeded from exactly the CLI-named inputs BEFORE the
/// config loads, so the load after this — and every fs consumer after it —
/// reads the sandbox, never the user's real files. The legacy permissive
/// paths never install it (real-fs behavior kept byte-for-byte). Three doors
/// select it: `--strict-replay`, `--storyboard` (which seeds the BOARD's
/// document, resolved in `validate_cli`, plus its parent-directory marker),
/// and `--screenshot-app`, whose claim is the strongest — it drives a real
/// `App`, which PERFORMS the writes a replay only records.
///
/// `--seed-data` only means anything HERE, so it is refused everywhere else
/// rather than silently ignored: a run that named a store and then did not
/// get one would photograph the wrong starting state and look like a product
/// bug. The slot exists because a `--screenshot-app` capture of an
/// external-change conflict has to START conflicted: nothing can raise one
/// mid-run, since the change must come from outside awl.
/// `live_app` is a native-only binding (the wasm build compiles no live-`App`
/// capture door), so the hermetic predicate is spelled per target. The wasm
/// arm installs no sandbox at all, which is why its answer is unconditional.
pub(super) fn install_hermetic_fs(ctx: &Ctx) -> Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    let hermetic =
        ctx.strict_replay || ctx.storyboard.is_some() || ctx.live_app || ctx.semantic_json;
    #[cfg(target_arch = "wasm32")]
    let hermetic = false;
    if ctx.data_seed.is_some() && !hermetic {
        bail!(
            "--seed-data only applies to a hermetic scenario run (--screenshot-app, \
             --semantic-json, --storyboard, or --screenshot --keys --strict-replay)"
        );
    }
    // Same refusal, same reason: a run that named a fixture project and then did
    // not get one would photograph an empty folder and read as a product bug.
    if ctx.tree_seed.is_some() && !hermetic {
        bail!(
            "--seed-tree only applies to a hermetic scenario run (--screenshot-app, \
             --semantic-json, --storyboard, or --screenshot --keys --strict-replay)"
        );
    }
    #[cfg(not(target_arch = "wasm32"))]
    if hermetic {
        crate::scenario::install_hermetic_fs(
            crate::scenario::seed_document(
                ctx.storyboard.is_some(),
                ctx.storyboard_file.as_deref(),
                ctx.file.as_deref(),
            ),
            ctx.config_arg.as_deref(),
            ctx.root.as_deref(),
            ctx.data_seed.as_deref(),
            ctx.tree_seed.as_deref(),
        )?;
    }
    Ok(())
}
