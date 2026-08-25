//! The sticky-preference, keymap, and final `Mode`-construction phases of
//! `parse::parse_args` — split out of `args/parse.rs` to stay under the
//! file's 500-line ceiling.

use super::*;

/// STICKY PREFERENCES: restore the remembered THEME / PAGE / CARET onto the
/// process-globals (the same globals the flags set), honouring flag > config —
/// a config value is applied only where its flag was ABSENT, so an explicit flag
/// still wins. These globals serve BOTH the windowed editor and the headless
/// capture, so a `--config` with theme/page/caret set produces a capture reflecting
/// them. ZOOM is per-instance (not a global): the capture folds it into `opts.zoom`
/// in `assemble_capture_opts`; the windowed `App::new` reads `config.zoom`.
///
/// The page-width MEASURE is now a per-KIND sticky pref (`page_width_prose` /
/// `page_width_code`) — resolve the STARTING buffer's class from the launch
/// `file` argument (no `Buffer` exists yet here) so the very first frame reads
/// the right one; a later buffer switch re-resolves against whichever kind is
/// then active (`App::sync_page_measure` / the headless `--keys` Goto switch).
///
/// Also does the STRUCTURAL parse of `keys_spec` — a garbled token still
/// errors right here. The chords stay UNRESOLVED: the replay loop resolves
/// them one press at a time through `km` (built right after, in
/// `build_keymap`), interleaved with the search guard, so a chord an open
/// search panel consumes never reaches the keymap — and the STRICT
/// unbound/dangling-prefix refusals fire there, where "was this chord for
/// the keymap at all" is actually decidable.
pub(super) fn resolve_sticky_prefs(ctx: &mut Ctx, config: &Config) -> Result<()> {
    let initial_page_class =
        page::PageClass::of_path(ctx.storyboard_file.as_deref().or(ctx.file.as_deref()));
    config.apply_sticky_globals(
        ctx.theme_flag,
        ctx.page_flag,
        ctx.caret_flag,
        ctx.measure_flag,
        initial_page_class,
    );
    // `--keys` only makes sense with a capture mode (it mutates the buffer for a
    // one-frame capture); refuse it for the windowed editor where live typing is
    // the input path.
    if ctx.keys_spec.is_some() && ctx.out.is_none() && !ctx.semantic_json {
        bail!("--keys requires a capture mode (e.g. --screenshot OUT.png)");
    }
    // `--wait` is a windowed-editor-only concern (the single-instance daemon's
    // handoff); a capture mode has no daemon to wait on (see `crate::daemon`'s
    // CAPTURE GATE).
    if ctx.wait_flag && (ctx.out.is_some() || ctx.semantic_json) {
        bail!("--wait only applies to the windowed editor (no capture mode)");
    }
    ctx.keys = match &ctx.keys_spec {
        Some(spec) => keyspec::parse_chords(spec)?,
        None => Vec::new(),
    };
    Ok(())
}

/// The keymap every capture replay resolves through: config `[keys]`
/// rebinds + the `linux_keep_emacs` door, exactly what live `App::new` builds.
pub(super) fn build_keymap(config: &Config) -> KeymapState {
    KeymapState::with_overrides_and_keep(
        &config.keys,
        &config.effective_linux_keep(),
        config.keymap_flavor() == crate::keymap::KeymapFlavor::Emacs,
    )
}

/// PRECEDENCE: explicit flag > config > built-in default. Fold the config value in
/// BEHIND the flag (the flag wins via `.or`) before the existing resolvers add the
/// built-in default. The Windowed path keeps the RAW flag + config so a live reload
/// can re-fold; capture modes fold here (one-shot, no reload).
pub(super) fn fold_launch_precedence(ctx: &Ctx, config: &Config) -> (PathBuf, Option<PathBuf>) {
    let default_folder_resolved = resolve_default_folder(
        &ctx.default_folder
            .clone()
            .or_else(|| config.default_folder.clone()),
    );
    let workspace_folded = ctx.workspace.clone().or_else(|| config.workspace.clone());
    (default_folder_resolved, workspace_folded)
}

/// Thread the capture canvas size + dpi onto the screenshot opts (timeline/held
/// carry them on their Mode variants). Absent flags -> None -> byte-stable default.
/// STICKY ZOOM (capture): fold the remembered zoom in BEHIND `--zoom` (the flag
/// wins). The windowed editor applies `config.zoom` in `App::new` instead.
pub(super) fn assemble_capture_opts(ctx: &mut Ctx, config: &Config) {
    ctx.opts.canvas = ctx.capture_size;
    ctx.opts.dpi = ctx.capture_dpi;
    if ctx.opts.zoom.is_none() {
        ctx.opts.zoom = config.zoom;
    }
}

/// FINAL MODE CONSTRUCTION. STORYBOARD mode: everything above (sandbox
/// install, config, km) composes normally; the run outputs land in
/// `--storyboard-out`, defaulting to `<storyboard>.run/` beside the board.
/// Otherwise the plain capture-mode precedence — held > timeline >
/// screenshot-app > screenshot-frames > motion(diagonal/vertical/plain) >
/// screenshot > windowed — mirrors `resolve_capture_kind`'s own.
pub(super) fn build_mode(
    ctx: Ctx,
    config: Config,
    km: KeymapState,
    default_folder_resolved: PathBuf,
    workspace_folded: Option<PathBuf>,
) -> Result<Mode> {
    let Ctx {
        out,
        motion,
        motion_v,
        motion_d,
        #[cfg(not(target_arch = "wasm32"))]
        frames,
        #[cfg(not(target_arch = "wasm32"))]
        live_app,
        #[cfg(not(target_arch = "wasm32"))]
        frame_step_ms,
        timeline_steps,
        held,
        capture_size,
        capture_dpi,
        file,
        opts,
        root,
        workspace,
        default_folder,
        wait_flag,
        strict_replay,
        storyboard_out,
        semantic_json,
        live,
        storyboard,
        storyboard_file,
        keys,
        ..
    } = ctx;

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
