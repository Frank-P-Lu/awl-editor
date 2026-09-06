//! THE SIDECAR FOLD for a still-open summoned card: the one owner of "what
//! does an open overlay look like in the sidecar", shared by the storyboard
//! runner and the one-shot `--keys` capture so the two can never drift. Lifted
//! verbatim out of `main/run.rs` to keep that file inside its
//! size mark; behaviour is unchanged.
//!
//! [`CaptureSubject`] + [`fold_capture_state`]
//! are now the one owner of the WHOLE per-frame fold (zoom / selection / search /
//! overlay / diff-preview / buffers), because there is now a THIRD driver — a
//! real headless live `App` (`--screenshot-app`, `main/run/live_app.rs`) — and a
//! third hand-written copy of this fold is exactly the defect the file exists to
//! prevent. The `ReplaySession` and the live `App` differ in what drives them,
//! never in what a sidecar says about them.

use crate::buffer::Buffer;
use crate::capture::{self, CaptureOpts};

pub(super) fn apply_replay_accept(
    accept: Option<&(crate::overlay::OverlayKind, String)>,
    buffer: &mut Buffer,
    opts: &mut CaptureOpts,
    workspace: &Option<std::path::PathBuf>,
    default_folder: &std::path::Path,
    config: &crate::config::Config,
) {
    let Some((kind, value)) = accept else {
        return;
    };
    match kind {
        crate::overlay::OverlayKind::Goto => {}
        crate::overlay::OverlayKind::Project => {
            opts.project = Some(super::project_info(
                std::path::Path::new(value),
                workspace,
                Some(default_folder),
                config,
            ));
        }
        crate::overlay::OverlayKind::History => {
            if let Some(path) =
                crate::history::source_path(buffer.path(), buffer.is_unnamed_fresh())
                && let Some(content) = crate::history::load(&path, value)
            {
                buffer.set_text(&content);
            }
        }
        _ => {}
    }
}

/// A DRIVEN EDITOR, as the sidecar fold reads it — the six facts
/// [`fold_capture_state`] needs and nothing else. Implemented by
/// [`super::ReplaySession`] (the shared-core driver behind `--keys`,
/// `--storyboard` and `--capture-timeline`) and by [`crate::app::App`] (the live
/// driver behind `--screenshot-app`). The trait is the seam that lets one fold
/// serve both without either knowing the other exists.
pub(crate) trait CaptureSubject {
    fn buffer(&self) -> Option<&Buffer>;
    fn zoom(&self) -> f32;
    fn search(&self) -> Option<&crate::search::SearchState>;
    fn journey(&self) -> &crate::overlay::Journey;
    fn buffers_open(&self) -> usize;
    /// Is the active document holding an UNRESOLVED external change? The
    /// persistent `changed elsewhere` affordance's one input, and the sixth fact
    /// because it is the first one the shared core genuinely cannot answer: the
    /// conflict is latched on the live `App`'s per-buffer disk baseline, and a
    /// replay never builds one. `ReplaySession` therefore answers `false`
    /// STRUCTURALLY rather than as a default (see its impl), which is what makes
    /// a `driver: "live-app"` sidecar the only one that can ever report `true`.
    fn changed_elsewhere(&self) -> bool;
    /// THE CALM NOTICE on screen, with its kind — the seventh fact, and the one
    /// whose absence made every capture door blind to a channel with ~ten
    /// production callers. Both drivers can answer it: the live `App` off its
    /// frame state, an ordinary replay off the notice its own effect interpreter
    /// latched. `None` when nothing is showing.
    fn notice(&self) -> Option<(String, crate::actions::NoticeKind)>;
    /// **DOES ANY BUFFER BEHIND THE ACTIVE ONE WANT THE MARGIN OUTLINE'S
    /// RAIL?** The eighth fact, and the first one about the buffers a frame is
    /// NOT rendering: the adaptive column reserves the rail's room for the
    /// WORKING SET, so a capture that asked only the photographed buffer would
    /// place the writing column where the reader would never see it. Both
    /// drivers keep a `crate::buffers::BufferRegistry` and answer off its own
    /// per-slot stamps, so the two cannot drift.
    fn set_wants_outline_rail(&self) -> bool;
}

/// THE ONE PER-FRAME FOLD: a driven editor's CURRENT state plus its already-built
/// project block, into the [`CaptureOpts`] the single-frame capture path renders
/// and [`crate::capture::sidecar`]'s ONE writer serializes. Lifted verbatim out
/// of `main/story.rs::step_opts`, which now delegates here, so the
/// storyboard stepper and the live-`App` capture cannot answer "what does this
/// state look like in the sidecar" two different ways.
///
/// `project` arrives already derived by the one builder, `run::project_info`,
/// rather than being re-derived here, because its inputs (the raw
/// `--workspace` flag, the effective default folder) belong to the caller's door,
/// not to the frame.
pub(crate) fn fold_capture_state(
    subject: &dyn CaptureSubject,
    project: capture::ProjectInfo,
) -> CaptureOpts {
    let buffer = subject.buffer();
    let mut opts = CaptureOpts {
        project: Some(project),
        zoom: (subject.zoom() != crate::range::ZOOM.default).then(|| subject.zoom()),
        selection: buffer.and_then(Buffer::selection_line_col),
        document_absent: buffer.is_none(),
        ..CaptureOpts::default()
    };
    if let Some(s) = subject.search() {
        opts.search = Some(s.query().to_string());
        opts.search_case_sensitive = s.is_case_sensitive();
        opts.search_replace_active = s.is_replace_active();
        opts.search_replacement = s.replacement().to_string();
        opts.search_editing_replacement = s.is_editing_replacement();
    }
    if let Some((info, preview_text, diff)) =
        overlay_capture_info_optional(subject.journey(), buffer)
    {
        opts.overlay = Some(info);
        opts.overlay_hug_roster = subject
            .journey()
            .card()
            .and_then(crate::overlay::OverlayState::hug_roster);
        opts.preview_text = preview_text;
        // DIFF-AS-PREVIEW: mirror the one-shot capture's fold (diff state block
        // + the overlay-owned diff scroll), so a stepped/live frame reports the
        // same preview the single-frame path would.
        if opts.diff.is_none() {
            opts.diff = diff;
        }
        if opts.scroll.is_none() && opts.preview_text.is_some() {
            let diff_scroll = subject.journey().card().map(|o| o.diff_scroll).unwrap_or(0);
            opts.scroll = Some(crate::render::ScrollPos::at_row(diff_scroll));
        }
    }
    opts.gutter_changed = subject.changed_elsewhere();
    opts.notice = subject.notice();
    opts.buffers = Some(buffers_info(subject.buffers_open(), buffer));
    opts.set_wants_outline_rail = subject.set_wants_outline_rail();
    opts
}

/// One identity fold for both ordinary capture doors. `BufferKey::of` is total:
/// a pathless buffer may be the durable scratch singleton or a provisional
/// Fresh document, and the sidecar must not collapse those identities.
pub(super) fn buffers_info(open: usize, buffer: Option<&Buffer>) -> capture::BuffersInfo {
    capture::BuffersInfo {
        open,
        active: buffer.map(|buffer| crate::buffers::BufferKey::of(buffer).sidecar_label()),
    }
}

pub(super) fn apply_replay_tail(
    opts: &mut capture::CaptureOpts,
    buffers_open: usize,
    buffer: &Buffer,
    replay_skips: Vec<crate::replay::SkippedEffect>,
) {
    opts.buffers = Some(buffers_info(buffers_open, Some(buffer)));
    opts.replay_skips = replay_skips;
}

/// Fold ONE still-open overlay into its sidecar [`capture::OverlayInfo`] block
/// plus the read-only COMPARISON TEXT (if that overlay shows one — see
/// [`comparison_preview_for`]). Extracted from [`capture_screenshot`]
/// VERBATIM so the storyboard runner's per-step render (`crate::story`) and the
/// one-shot `--keys` capture share ONE owner of "what does an open overlay
/// report" — the two can never drift.
pub(crate) fn overlay_capture_info(
    journey: &crate::overlay::Journey,
    buffer: &Buffer,
) -> Option<(
    capture::OverlayInfo,
    Option<String>,
    Option<capture::DiffInfo>,
)> {
    overlay_capture_info_optional(journey, Some(buffer))
}

fn overlay_capture_info_optional(
    journey: &crate::overlay::Journey,
    buffer: Option<&Buffer>,
) -> Option<(
    capture::OverlayInfo,
    Option<String>,
    Option<capture::DiffInfo>,
)> {
    let ov = journey.card()?;
    // The REQUEST is read once here and reported beside its answer, so the
    // sidecar names the view it is showing rather than leaving a reader to infer
    // it from the selected row.
    let request = ov.comparison_request();
    let preview = buffer.and_then(|buffer| comparison_preview_for(ov, buffer));
    let preview_text = preview
        .as_ref()
        .map(|(_, transcript, _)| transcript.clone());
    let diff = preview.as_ref().map(|(_, _, c)| capture::DiffInfo {
        active: true,
        label: ov
            .selected_value()
            .unwrap_or("an earlier version")
            .to_string(),
        struck: c.struck,
        washed: c.washed,
        modified: c.modified,
        moved: c.moved,
        folds: c.folds,
    });
    let info = capture::OverlayInfo {
        active: true,
        mode: ov.kind.as_str(),
        align: ov.align,
        query: ov.query.text().to_string(),
        query_caret: ov.query.caret(),
        query_selection: ov.query.selection_range(),
        items: ov.item_strings(),
        empty: ov.empty_notice(),
        bindings: ov.item_bindings(),
        ranges: ov.item_range_fracs(),
        git: ov.item_git_tags(),
        selected_index: ov.selected,
        hint: journey.foot_hint(),
        browse_dir: ov.browse_dir.clone(),
        return_to: journey.parked().map(|p| p.kind().as_str()),
        spell_target: ov.spell_target,
        table_dims: ov.table_dims_target(),
        context_anchor: ov.context_anchor,
        asset_preview: ov.selected_asset_path().map(std::path::Path::to_path_buf),
        preview_id: preview.map(|(id, _, _)| id),
        preview_view: request.as_ref().map(|r| r.view.tag()),
        workspace: ov.workspace_shape().is_some(),
        detail_focus: ov.detail_focus,
        diff_scroll: ov.diff_scroll,
        show_hidden: ov.kind.hides_dotfiles() && crate::file_visibility::all_on(),
        capture: ov.capture.as_ref().map(|c| capture::CaptureInfo {
            command: c.cmd_name.clone(),
            stage: match c.stage {
                crate::overlay::CaptureStage::ChooseMode => "choose",
                crate::overlay::CaptureStage::Recording => "recording",
                crate::overlay::CaptureStage::Confirm => "confirm",
            },
            chord_mode: c.chord_mode,
            captured: c.captured.clone(),
            prompt: c.prompt(),
        }),
        notice: ov.notice.clone(),
        lens: ov.active_facet_id(),
        lens_strip: ov.lens_strip(),
        sections: ov.item_sections(),
        title: ov.title(),
    };
    Some((info, preview_text, diff))
}

/// The SHARED-CORE driver's view of itself. Every method already existed as an
/// inherent accessor on the session (`main/run.rs`); this impl only names them as
/// the fold's five facts, so the storyboard stepper reads them through the same
/// seam the live `App` does.
impl CaptureSubject for super::ReplaySession<'_> {
    fn buffer(&self) -> Option<&Buffer> {
        Some(super::ReplaySession::buffer(self))
    }
    fn zoom(&self) -> f32 {
        super::ReplaySession::zoom(self)
    }
    fn search(&self) -> Option<&crate::search::SearchState> {
        super::ReplaySession::search(self)
    }
    fn journey(&self) -> &crate::overlay::Journey {
        super::ReplaySession::journey(self)
    }
    fn buffers_open(&self) -> usize {
        super::ReplaySession::buffers_open(self)
    }
    /// STRUCTURALLY false: an ordinary replay holds no per-buffer disk baseline,
    /// so it has nothing that could be in conflict. Not a "not implemented yet"
    /// — there is no state here to read.
    fn changed_elsewhere(&self) -> bool {
        false
    }
    /// A replay DOES hold this one — `Effect::Notice` is interpreted rather than
    /// swallowed (`main/replay_effects.rs`), so an ordinary `--keys` capture of a
    /// notice-raising action photographs the notice. Headless has no clock, so a
    /// Toast never expires here, exactly as it never expires in a GPU-less live
    /// `App` (`App::set_toast_notice` arms no deadline without a surface).
    fn notice(&self) -> Option<(String, crate::actions::NoticeKind)> {
        super::ReplaySession::notice(self)
    }
    /// A replay keeps the SAME `crate::buffers::BufferRegistry` the live App
    /// does, so it answers this off the same per-slot stamps — a `--keys`
    /// capture that opened a second file places its column exactly where the
    /// running editor would.
    fn set_wants_outline_rail(&self) -> bool {
        super::ReplaySession::set_wants_outline_rail(self)
    }
}

/// The headless side of the READ-ONLY COMPARISON: when the replay left a
/// comparison surface OPEN, resolve the view it is asking for into prose, so the
/// capture shows THAT text in the document itself and the sidecar reports which
/// subject and which view. `None` for every overlay kind that shows no
/// comparison, an empty-state row, or an unresolvable subject (the capture then
/// just shows the buffer — the live degrade). Pure over its inputs, so it is
/// unit-testable with a seeded store or a seeded conflict card.
pub(super) fn comparison_preview_for(
    ov: &crate::overlay::OverlayState,
    buffer: &Buffer,
) -> Option<(String, String, crate::prosediff::DiffCounts)> {
    // THE SAME TYPED REQUEST through THE SAME DISPATCH the live App resolves
    // (`OverlayState::comparison_request` -> `comparison::prose_for`),
    // synchronously — the live debounce is a wall-clock concern the
    // deterministic capture never has. Routing the capture through the one
    // dispatch rather than a parallel per-kind lookup is what keeps live and
    // `--keys` replay unable to disagree, and it is why a SECOND comparison
    // surface needed no second capture path at all.
    let request = ov.comparison_request()?;
    crate::comparison::prose_for(
        ov,
        &request,
        buffer.path(),
        buffer.is_unnamed_fresh(),
        &buffer.text(),
    )
}
