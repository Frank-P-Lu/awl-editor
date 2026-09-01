//! Single-frame capture entry points and shared snapshot helpers.

use anyhow::{Context, Result};
use glyphon::Cache;
use std::path::Path;

use crate::buffer::Buffer;
use crate::overlay::OverlayKind;
use crate::render::{self, TextPipeline, ViewState};

use super::gpu::{headless_device, offscreen_target, read_frame};
use super::opts::{CaptureOpts, ProjectInfo};
use super::sidecar::write_sidecar;
use super::{CANVAS_HEIGHT, CANVAS_WIDTH, FORMAT};

/// Build a capture [`ViewState`] on the canonical [`ViewState::base`] with the
/// project-derived fields (`gutter_name`, `gutter_project`, `doc_dir`,
/// `is_markdown`, `syn_lang`, `eol`) filled in — every search / overlay field
/// inherits `base()`'s inert default, so a NEW ViewState field is defaulted once
/// in `base()` and this path inherits it automatically. The timeline / held paths
/// use this verbatim (overriding only `held`); the single-frame path overrides the
/// search / overlay / selection fields it actually drives.
pub(super) fn base_viewstate(
    buffer: &Buffer,
    project: &Option<ProjectInfo>,
    cursor: (usize, usize),
    zoom: f32,
    misspelled: Vec<crate::spell::Misspelling>,
    held: bool,
) -> ViewState {
    ViewState {
        text: buffer.text(),
        cursor_line: cursor.0,
        cursor_col: cursor.1,
        // Carry the buffer's caret wrap affinity into the capture so a `--keys`
        // replay of C-e / End / Cmd-Right at a shared soft-wrap boundary renders the
        // caret on the SAME visual row the live app would (Upstream → upper row).
        caret_affinity: buffer.affinity(),
        zoom,
        misspelled,
        held,
        // PAGE-MODE GUTTER: the buffer display name over the project name (empty when
        // there is no project), filled here so the gutter is verifiable from a capture.
        gutter_name: buffer.display_name(),
        gutter_project: project.as_ref().map(|p| p.name.clone()).unwrap_or_default(),
        is_markdown: buffer.is_markdown(),
        // INLINE IMAGES: a relative image path resolves against the captured
        // document's own directory (its buffer path's parent), so a `samples/foo.md`
        // referencing `foo.png` beside it renders in a headless capture.
        doc_dir: buffer
            .path()
            .and_then(|p| p.parent())
            .map(|d| d.to_path_buf()),
        syn_lang: buffer.syntax_lang(),
        // LINE ENDINGS: the buffer's real on-disk ending — a pure buffer fact, so a
        // CRLF fixture reports "CRLF" and an LF fixture "LF" in the sidecar's hud.eol.
        eol: buffer.eol(),
        // Every remaining field is the inert default (`ViewState::base()`): the
        // search / overlay / selection fields the single-frame path overrides itself,
        // and the caret-preview / overlay_spell / overlay_window_rows the still-open
        // overlay fills in later.
        ..ViewState::base()
    }
}

/// How the caret is posed for a headless capture. Both modes are fully
/// deterministic (no clock): the same input yields a byte-identical PNG.
#[derive(Clone, Copy, PartialEq)]
enum CaretMode {
    /// Caret settled exactly on target: the resting amber rounded square on the
    /// glyph.
    Rest,
    /// Caret part-way through a synthetic horizontal glide: a trailing amber
    /// underline streak dropped to the baseline.
    Motion,
    /// Caret part-way through a synthetic VERTICAL glide: a thin amber bar slid to
    /// the cell's left edge, trailing up the lines it passed.
    MotionVertical,
    /// Caret part-way through a synthetic DIAGONAL glide (different row AND column):
    /// a true slanted amber tracer from source to target.
    MotionDiagonal,
}

/// Render the loaded `buffer` to an offscreen 1200x800 texture and write
/// `<out>.png` and the sidecar `<out>.json`. Opens NO window. The caret is drawn
/// AT REST (the resting amber rounded square on the glyph) at the buffer's current
/// cursor position, so the capture is byte-deterministic. Deterministic for a
/// fixed set of options.
pub fn capture_with(out_png: &Path, buffer: &Buffer, opts: &CaptureOpts) -> Result<()> {
    pollster::block_on(capture_async(out_png, buffer, CaretMode::Rest, opts))
}

/// Like [`capture`], but renders ONE frame of a caret MID-GLIDE — a synthetic,
/// deterministic still showing the caret dropped to the baseline and stretched
/// into a trailing underline streak partway along its path, so the temporal
/// effect is inspectable from a screenshot. No clock is consulted.
pub fn capture_motion(out_png: &Path, buffer: &Buffer) -> Result<()> {
    pollster::block_on(capture_async(
        out_png,
        buffer,
        CaretMode::Motion,
        &CaptureOpts::default(),
    ))
}

/// Like [`capture_motion`], but a VERTICAL mid-glide: the caret has slid to a thin
/// amber bar on the cell's left edge, trailing up the lines it just travelled.
pub fn capture_motion_vertical(out_png: &Path, buffer: &Buffer) -> Result<()> {
    pollster::block_on(capture_async(
        out_png,
        buffer,
        CaretMode::MotionVertical,
        &CaptureOpts::default(),
    ))
}

/// Like [`capture_motion`], but a DIAGONAL mid-glide: the caret is part-way through
/// a jump between two points on different rows AND columns, so its trail is a true
/// slanted tracer from source to target (not an axis-snapped bar).
pub fn capture_motion_diagonal(out_png: &Path, buffer: &Buffer) -> Result<()> {
    pollster::block_on(capture_async(
        out_png,
        buffer,
        CaretMode::MotionDiagonal,
        &CaptureOpts::default(),
    ))
}

async fn capture_async(
    out_png: &Path,
    buffer: &Buffer,
    caret_mode: CaretMode,
    opts: &CaptureOpts,
) -> Result<()> {
    // --- Device (no surface needed for offscreen) -------------------------
    let (device, queue) = headless_device().await?;

    // PHYSICAL canvas dims for this run: the flagged `--capture-size`, else the
    // byte-stable default. DPI defaults to 1.0 (a `set_dpi` no-op).
    let (width, height) = opts.canvas.unwrap_or((CANVAS_WIDTH, CANVAS_HEIGHT));
    let dpi = opts.dpi.unwrap_or(1.0);

    // --- Offscreen color target ------------------------------------------
    let (texture, view) = offscreen_target(&device, width, height);

    // --- Text pipeline (shared with windowed) ----------------------------
    let cache = Cache::new(&device);
    let mut pipeline = TextPipeline::new(&device, &queue, &cache, FORMAT);
    pipeline.set_size(width as f32, height as f32);
    pipeline.set_pending_crash(opts.pending_crash);
    // DPI AFTER set_size: set_dpi re-wraps at column_width(), which reads window_w
    // (set by set_size). No-op at the default 1.0, so the no-flag path is unchanged.
    pipeline.set_dpi(dpi);

    // Fold the buffer + capture opts into the shaped, scrolled view — the ONE
    // owner shared with the storyboard film stepper (`super::film`).
    let vstate = settled_viewstate(&mut pipeline, buffer, opts, height);
    // Pose the caret deterministically for this capture.
    match caret_mode {
        CaretMode::Rest => pipeline.settle_caret(),
        CaretMode::Motion => pipeline.inject_motion_demo(),
        CaretMode::MotionVertical => pipeline.inject_motion_demo_vertical(),
        CaretMode::MotionDiagonal => pipeline.inject_motion_demo_diagonal(),
    }
    // CARET-STYLE PICKER preview: pin its looping preview caret to its SETTLED look on
    // cell 0 (the loop is live-only, so the capture renders the deterministic resting
    // caret of the highlighted style). No-op when that picker isn't open.
    pipeline.settle_caret_preview();
    // WHICH-KEY panel: summon it with the derived continuation rows when `--whichkey`
    // populated them (`None` otherwise → nothing drawn, byte-identical default).
    pipeline.set_whichkey(opts.whichkey.clone());
    pipeline.prepare(&device, &queue, width, height)?;

    // --- Draw the frame, then read it back via the shared helper ---------
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("awl capture encoder"),
    });
    pipeline.render(&mut encoder, &view)?;
    queue.submit(Some(encoder.finish()));
    let img = read_frame(&device, &queue, &texture, width, height)?;

    // --- Write PNG --------------------------------------------------------
    img.save(out_png)
        .with_context(|| format!("failed to write PNG {}", out_png.display()))?;

    // --- Write JSON sidecar ----------------------------------------------
    write_sidecar(out_png, &vstate, &pipeline, opts, None)?;

    Ok(())
}

/// Fold `buffer` + `opts` into the fully-shaped, scrolled capture [`ViewState`]
/// — the search derivation, every overlay/selection/preedit override, the
/// history live-preview text fold, and the cursor-follow / typewriter scroll —
/// leaving the pipeline shaped by TWO `set_view`s (shape, then scroll), the
/// caret UNPOSED. Lifted VERBATIM out of `capture_async` so the single-frame
/// path and the storyboard film stepper (`super::film`) share ONE owner of
/// "what does this capture state look like"; the caller decides the caret pose
/// (settle / motion inject / the film's free-running spring).
///
/// The returned scroll is always normalized against shaped variable-row geometry.
pub(super) fn settled_viewstate(
    pipeline: &mut TextPipeline,
    buffer: &Buffer,
    opts: &CaptureOpts,
    height: u32,
) -> ViewState {
    let (cursor_line, cursor_col) = buffer.cursor_line_col();
    let zoom = render::clamp_zoom(opts.zoom.unwrap_or(crate::range::ZOOM.default));
    // Spell-check the buffer text for the headless capture too, so `--screenshot`
    // renders the squiggles. Deterministic (fixed text -> fixed spans). If the
    // bundled dictionary fails to parse, report it and render without squiggles.
    let misspelled = super::policy::misspellings(buffer);
    // --- Search panel (deterministic headless isearch) -------------------
    // Compute matches against the loaded buffer, pick current = first match at
    // or after the cursor (Forward, deterministic) else the first match, and
    // move the resting caret onto the current match. capture takes &Buffer
    // (immutable), so we DO NOT set_cursor; we derive sc_line/sc_col locally and
    // feed them into the ViewState so settle_caret lands the caret on the match.
    let (search_matches, search_current, mut sc_line, mut sc_col) = if let Some(q) = &opts.search {
        let cs = opts.search_case_sensitive;
        let raw = crate::search::find_all(&buffer.text(), q, cs);
        let ranges: Vec<((usize, usize), (usize, usize))> = raw
            .iter()
            .map(|m| {
                (
                    buffer.char_to_line_col(m.start),
                    buffer.char_to_line_col(m.end),
                )
            })
            .collect();
        let cur_char = buffer.cursor_char();
        let cur_idx = if raw.is_empty() {
            None
        } else {
            Some(raw.iter().position(|m| m.start >= cur_char).unwrap_or(0))
        };
        let (cl, cc) = match cur_idx {
            Some(i) => buffer.char_to_line_col(raw[i].start),
            None => (cursor_line, cursor_col),
        };
        (ranges, cur_idx, cl, cc)
    } else {
        (Vec::new(), None, cursor_line, cursor_col)
    };
    let search_active = opts.search.is_some();

    // Shape the document first (at zoom 0/no-scroll) so the pipeline can report
    // wrap-aware row counts. Scroll is counted in VISUAL ROWS, so an explicit
    // `--scroll N` is N visual rows clamped to the document's total visual rows,
    // and the cursor-follow default uses the cursor's VISUAL row. Both need the
    // buffer shaped, which a preliminary `set_view` provides.
    // Start from the shared inert-default base (project status + flags filled once),
    // then drive the search / overlay / selection fields this single-frame path
    // verifies. With an active --search the resting caret lands on the current match.
    let mut vstate = base_viewstate(
        buffer,
        &opts.project,
        (sc_line, sc_col),
        zoom,
        misspelled,
        false,
    );
    // THE GUTTER'S LIVE-APP-ONLY FACTS — the persistent affordance and the
    // working set — through their one fold. Both empty on every replay by
    // construction, so an ordinary capture stays byte-identical.
    opts.fold_gutter(&mut vstate);
    // THE CALM NOTICE. `None` on every capture that raises none, which is why the
    // gallery stays byte-identical; a capture that DID raise one now photographs
    // it instead of silently agreeing with a capture that did not.
    if let Some((text, kind)) = &opts.notice {
        vstate.notice = text.clone();
        vstate.notice_kind = *kind;
    }
    vstate.selection = opts.selection;
    vstate.preedit = opts.preedit.clone().unwrap_or_default();
    vstate.search_matches = search_matches;
    vstate.search_current = search_current;
    vstate.search_query = opts.search.clone().unwrap_or_default();
    vstate.search_active = search_active;
    vstate.search_case_sensitive = opts.search_case_sensitive;
    // Search replay and the live window share the same interception seam.
    vstate.search_replace_active = opts.search_replace_active;
    vstate.search_replacement = opts.search_replacement.clone();
    vstate.search_editing_replacement = opts.search_editing_replacement;
    // Synthetic capture options do not carry field carets, so use the end.
    vstate.search_query_caret = vstate.search_query.chars().count();
    vstate.search_replacement_caret = vstate.search_replacement.chars().count();
    vstate.overlay_active = opts.overlay.as_ref().map(|o| o.active).unwrap_or(false);
    // Preserve the alignment frozen when the overlay was summoned.
    vstate.overlay_align = opts.overlay.as_ref().map(|o| o.align);
    // Capture-only force summon for a live mouse gesture; keep the live gates.
    if crate::popover::popover_on()
        && !search_active
        && !vstate.overlay_active
        && (opts.force_popover || std::env::var_os("AWL_POPOVER").is_some())
        && let Some(((l0, c0), (l1, c1))) = vstate.selection
    {
        let a = buffer.line_col_to_char(l0, c0);
        let c = buffer.line_col_to_char(l1, c1);
        vstate.popover =
            crate::actions::popover::plan(&buffer.text(), Some(a), c, buffer.is_markdown());
    }
    // Resolve serialized modes through the same owner as the live path.
    vstate.overlay_crisp = opts
        .overlay
        .as_ref()
        .and_then(|o| crate::overlay::OverlayKind::from_mode(o.mode))
        .is_some_and(|kind| kind.keeps_backdrop_crisp());
    vstate.overlay_query = opts
        .overlay
        .as_ref()
        .map(|o| o.query.clone())
        .unwrap_or_default();
    // `OverlayInfo::query_caret` is the real caret when the overlay came
    // through the live replay path (`capture_fold`); a synthetic override
    // built by hand still defaults to the end, matching `query`'s own default.
    vstate.overlay_query_caret = opts
        .overlay
        .as_ref()
        .map(|o| o.query_caret)
        .unwrap_or_else(|| vstate.overlay_query.chars().count());
    vstate.overlay_query_selection = opts.overlay.as_ref().and_then(|o| o.query_selection);
    // Modal prompts orient via `foot_hint`; unknown modes keep a visible title.
    vstate.overlay_title = opts
        .overlay
        .as_ref()
        .filter(|o| {
            crate::overlay::OverlayKind::from_mode(o.mode).is_none_or(|k| k.draws_title_prefix())
        })
        .map(|o| o.title.clone())
        .unwrap_or_default();
    // Share the live path/URL figure-ground gate; unknown modes stay single-ink.
    vstate.overlay_row_path_splits = opts
        .overlay
        .as_ref()
        .and_then(|o| crate::overlay::OverlayKind::from_mode(o.mode))
        .map(|k| k.row_path_splits())
        .unwrap_or(false);
    vstate.overlay_items = opts
        .overlay
        .as_ref()
        .map(|o| o.items.clone())
        .unwrap_or_default();
    vstate.overlay_hug_roster = opts.overlay_hug_roster.clone();
    vstate.overlay_empty = opts.overlay.as_ref().and_then(|o| o.empty.clone());
    vstate.overlay_bindings = opts
        .overlay
        .as_ref()
        .map(|o| o.bindings.clone())
        .unwrap_or_default();
    // The rail fractions ride the sidecar's own `overlay.ranges` block, so
    // a JSON-driven capture draws the same thumbs the live picker does.
    vstate.overlay_ranges = opts
        .overlay
        .as_ref()
        .map(|o| o.ranges.clone())
        .unwrap_or_default();
    vstate.overlay_git = opts
        .overlay
        .as_ref()
        .map(|o| o.git.clone())
        .unwrap_or_default();
    vstate.overlay_selected = opts.overlay.as_ref().map(|o| o.selected_index).unwrap_or(0);
    // Scroll window: keep the selection visible with the same min-scroll math
    // `OverlayState::scroll_to_selected` uses, so a JSON-driven capture windows a
    // long list identically to the live picker. The pipeline re-clamps to the item
    // count, so this needs no `n_items` here.
    let theme_panel = opts
        .overlay
        .as_ref()
        .and_then(|o| OverlayKind::from_mode(o.mode))
        .is_some_and(|k| k == OverlayKind::Theme);
    // Use the kind-owned row cap for both the window and its scroll hint.
    let win = opts
        .overlay
        .as_ref()
        .and_then(|o| crate::overlay::OverlayKind::from_mode(o.mode))
        .map(|k| k.window_rows())
        .unwrap_or(12);
    vstate.overlay_window_rows = win;
    // The THEME picker's item-space scroll is pinned at 0 (a valid window HINT — the
    // grouped-path geometry converts it to a display line and then slides the display
    // window to keep the selected row visible, bounding the card to the canvas even when
    // a faceted corpus overflows).
    vstate.overlay_scroll = if theme_panel {
        0
    } else {
        vstate.overlay_selected.saturating_sub(win - 1)
    };
    vstate.overlay_hint = opts
        .overlay
        .as_ref()
        .map(|o| o.hint.clone())
        .unwrap_or_default();
    // THEME PICKER: the lens strip + per-row section labels (drives the faceted render).
    vstate.overlay_lens = opts
        .overlay
        .as_ref()
        .map(|o| o.lens_strip.clone())
        .unwrap_or_default();
    // CHIP-VARIATIONS PROBE (capture-only, inert unless `AWL_THEME_LENS_DEMO` is set):
    // the theme picker's runtime lens strip was RETIRED (facets.rs), so a live
    // `--keys "Cmd-T"` capture carries an EMPTY strip and the chip skins have no
    // labels to mark. This dev knob injects a representative strip (one active
    // facet + neighbours) ONLY into the theme picker capture, so the six
    // `AWL_FACET_STYLE_FORCE=chips:<variant>` shots have something to render. No-op
    // unless the env is set; never compiled into any live-app path.
    if theme_panel && vstate.overlay_lens.is_empty() && std::env::var("AWL_THEME_LENS_DEMO").is_ok()
    {
        vstate.overlay_lens = vec![
            ("All".to_string(), false),
            ("Warm".to_string(), true),
            ("Cool".to_string(), false),
            ("Light".to_string(), false),
            ("Dark".to_string(), false),
        ];
    }
    vstate.overlay_sections = opts
        .overlay
        .as_ref()
        .map(|o| o.sections.clone())
        .unwrap_or_default();
    // Rebuilt from the strip this snapshot already carries rather than added as a
    // second serialized fact; held to the live owner's answer by a scheme sweep.
    vstate.overlay_location =
        crate::facets::strip_location(&vstate.overlay_lens).map(std::string::ToString::to_string);
    // The SUMMONED WORKSPACE's presentation + focus stage. Set for
    // every capture that carries an overlay, not only a previewing one: a
    // workspace has two regions whether or not anything is previewed beneath it,
    // and the focus stage is what says which of them is live.
    vstate.overlay_workspace = opts.overlay.as_ref().map(|o| o.workspace).unwrap_or(false);
    // …and WHICH SHAPE it is presented as, derived from the kind's own owner
    // rather than carried as a second sidecar field: the sidecar names the mode,
    // and a capture that re-declared the shape could disagree with the live App.
    // Without this a replayed workspace draws the OTHER shape entirely.
    vstate.overlay_rows_primary = opts
        .overlay
        .as_ref()
        .filter(|o| o.workspace)
        .and_then(|o| crate::overlay::OverlayKind::from_mode(o.mode))
        .and_then(|k| k.workspace_shape())
        .is_some_and(crate::overlay::workspace::WorkspaceShape::rows_are_primary);
    // …and whether that region has prose in it — the capture's own `preview_text`,
    // resolved through the SAME typed request the live App uses.
    vstate.overlay_comparison = opts.preview_text.is_some();
    vstate.overlay_detail_focus = opts
        .overlay
        .as_ref()
        .map(|o| o.detail_focus)
        .unwrap_or(false);
    // Spell and context cards retain their real pointer/text anchors.
    vstate.overlay_spell = opts.overlay.as_ref().and_then(|o| o.spell_target);
    vstate.overlay_table_dims = opts.overlay.as_ref().and_then(|o| o.table_dims);
    vstate.overlay_context_anchor = opts.overlay.as_ref().and_then(|o| o.context_anchor);
    // The Asset Cleaner's live preview panel reads the same still-open
    // overlay's highlighted-row path a `--keys` replay resolved.
    vstate.overlay_asset_preview = opts.overlay.as_ref().and_then(|o| o.asset_preview.clone());
    // CARET-STYLE PICKER preview: when the still-open overlay is the caret picker,
    // map its highlighted row label back to the look so the headless capture renders
    // that look's SETTLED preview caret (the loop is live-only; see settle_caret_preview).
    // WHICH kind that is comes from the same mode->kind door every other per-kind
    // question here uses, never the mode's own spelling: `App::sync_view` gates this
    // field on `o.kind == OverlayKind::Caret`, and two doors that answer one question
    // in two vocabularies are the drift the crisp-backdrop merge was written to end.
    vstate.caret_preview = opts
        .overlay
        .as_ref()
        .filter(|o| OverlayKind::from_mode(o.mode) == Some(OverlayKind::Caret))
        .and_then(|o| {
            o.items
                .get(o.selected_index)
                .and_then(|name| crate::caret::CaretMode::from_label(name))
        });
    // HISTORY TIMELINE live preview: the still-open History overlay's highlighted
    // row previews THAT VERSION in the document itself — override the snapshot's
    // text BEFORE the first `set_view`, so the scroll math below shapes the
    // previewed version (exactly like the live `sync_view` fold), and the sidecar
    // `text` reports it. Mirrors the live geometry safety: the cursor clamps into
    // the previewed text (the shared `clamp_line_col`) and the buffer-indexed
    // spans (selection / squiggles / search) are cleared. `None` (default) leaves
    // a plain `--screenshot` byte-identical.
    if let Some(p) = &opts.preview_text {
        vstate.substitute_text(p.clone()); // the ONE door, as `sync_view` does
        // DIFF-AS-PREVIEW: the previewed text is the writer's-diff TRANSCRIPT —
        // park the caret on its blank line 1 (between `# title` and the first
        // block) so no line's WYSIWYG conceal reveals, mirroring the live
        // `sync_view` park exactly (the ONE reveal-suppression rule).
        let (pl, pc) = crate::history::clamp_line_col(p, 1, 0);
        vstate.cursor_line = pl;
        vstate.cursor_col = pc;
        sc_line = pl;
        sc_col = pc;
        // The focus cue is mirrored from the overlay state.
        vstate.overlay_detail_focus = opts
            .overlay
            .as_ref()
            .map(|o| o.detail_focus)
            .unwrap_or(false);
        vstate.selection = None;
        vstate.misspelled = Vec::new();
        vstate.search_matches = Vec::new();
        vstate.search_current = None;
        vstate.search_query = String::new();
        vstate.search_active = false;
        vstate.search_case_sensitive = false;
        vstate.search_replace_active = false;
        vstate.search_replacement = String::new();
        vstate.search_editing_replacement = false;
    }
    // FOLDS: collapse the folded sections out of the shaped text BEFORE the first
    // `set_view`, so the pipeline shapes the fold-filtered document (a hidden line
    // is never laid out → contributes ZERO height) and the scroll math below counts
    // the filtered rows. The buffer's fold set was built during the `--keys` replay;
    // recorded (unfiltered) for the sidecar. Skipped during a history preview (its
    // transcript owns the text). No-op → byte-identical when nothing is folded.
    vstate.folds = buffer.folds().iter().copied().collect();
    if opts.preview_text.is_none() && buffer.has_folds() {
        let hidden = buffer.hidden_lines();
        // Remap the resting-caret row the scroll-follow below reads into filtered
        // space (the action-seam auto-expand keeps the caret on a visible line).
        let filter = crate::fold::Filter::new(&vstate.text, &hidden);
        if filter.visible(sc_line) {
            sc_line = filter.line(sc_line);
        }
        crate::fold::apply_to_view(&mut vstate, &hidden, &buffer.fold_tails(), buffer.folds());
    }
    pipeline.set_view(&vstate);

    // Normalize semantic scroll only after shaping supplies variable-row geometry.
    let settled_scroll = match opts.scroll {
        // `--scroll N` is N VISUAL rows; 999 etc. clamps to the last reachable row.
        Some(pos) => pipeline.scroll_by_px(pos, 0.0, height as f32),
        None => {
            // Cursor-follow default: scroll so the cursor's VISUAL row is on screen
            // (from the top, since the headless cursor starts at the buffer start
            // unless a selection moved it). Mirrors the windowed cursor-follow,
            // INCLUDING the CENTERED (typewriter) pin: with the sticky TYPEWRITER
            // SCROLL toggle on, the caret row is CENTERED, otherwise it's the
            // minimal-adjust — so a `--keys` capture with typewriter on verifies the
            // centered scroll deterministically.
            super::policy::follow_scroll(pipeline, sc_line, sc_col, height as f32)
        }
    };
    debug_assert!(settled_scroll.px_q >= 0);
    debug_assert!(pipeline.scroll_top_px(settled_scroll) >= 0.0);
    vstate.scroll = settled_scroll;
    pipeline.set_view(&vstate);
    vstate
}
