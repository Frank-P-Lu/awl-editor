//! THE SIDECAR FOLD for a still-open summoned card: the one owner of "what
//! does an open overlay look like in the sidecar", shared by the storyboard
//! runner and the one-shot `--keys` capture so the two can never drift. Lifted
//! verbatim out of `main/run.rs` (queue item 173, to keep that file inside its
//! size mark); behaviour is unchanged.
//!
//! ITEM 188 widened it by one level: [`CaptureSubject`] + [`fold_capture_state`]
//! are now the one owner of the WHOLE per-frame fold (zoom / selection / search /
//! overlay / diff-preview / buffers), because there is now a THIRD driver — a
//! real headless live `App` (`--screenshot-app`, `main/run/live_app.rs`) — and a
//! third hand-written copy of this fold is exactly the defect the file exists to
//! prevent. The `ReplaySession` and the live `App` differ in what drives them,
//! never in what a sidecar says about them.

use crate::buffer::Buffer;
use crate::capture::{self, CaptureOpts};

/// A DRIVEN EDITOR, as the sidecar fold reads it — the five facts
/// [`fold_capture_state`] needs and nothing else. Implemented by
/// [`super::ReplaySession`] (the shared-core driver behind `--keys`,
/// `--storyboard` and `--capture-timeline`) and by [`crate::app::App`] (the live
/// driver behind `--screenshot-app`). The trait is the seam that lets one fold
/// serve both without either knowing the other exists.
pub(crate) trait CaptureSubject {
    fn buffer(&self) -> &Buffer;
    fn zoom(&self) -> f32;
    fn search(&self) -> Option<&crate::search::SearchState>;
    fn journey(&self) -> &crate::overlay::Journey;
    fn buffers_open(&self) -> usize;
}

/// THE ONE PER-FRAME FOLD: a driven editor's CURRENT state plus its already-built
/// project block, into the [`CaptureOpts`] the single-frame capture path renders
/// and [`crate::capture::sidecar`]'s ONE writer serializes. Lifted verbatim out
/// of `main/story.rs::step_opts` (item 188), which now delegates here, so the
/// storyboard stepper and the live-`App` capture cannot answer "what does this
/// state look like in the sidecar" two different ways.
///
/// `project` arrives already derived — by `run::project_info`, the one builder
/// (item 183) — rather than being re-derived here, because its inputs (the raw
/// `--workspace` flag, the effective default folder) belong to the caller's door,
/// not to the frame.
pub(crate) fn fold_capture_state(
    subject: &dyn CaptureSubject,
    project: capture::ProjectInfo,
) -> CaptureOpts {
    let buffer = subject.buffer();
    let mut opts = CaptureOpts {
        project: Some(project),
        zoom: (subject.zoom() != 1.0).then(|| subject.zoom()),
        selection: buffer.selection_line_col(),
        ..CaptureOpts::default()
    };
    if let Some(s) = subject.search() {
        opts.search = Some(s.query().to_string());
        opts.search_case_sensitive = s.is_case_sensitive();
        opts.search_replace_active = s.is_replace_active();
        opts.search_replacement = s.replacement().to_string();
        opts.search_editing_replacement = s.is_editing_replacement();
    }
    if let Some((info, preview_text, diff)) = overlay_capture_info(subject.journey(), buffer) {
        opts.overlay = Some(info);
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
    opts.buffers = Some(capture::BuffersInfo {
        open: subject.buffers_open(),
        active: match buffer.path() {
            Some(p) => p.display().to_string(),
            None => "scratch".to_string(),
        },
    });
    opts
}

/// Fold ONE still-open overlay into its sidecar [`capture::OverlayInfo`] block
/// plus the History live-preview TEXT (if that overlay is the History timeline
/// — see [`history_preview_for`]). Extracted from [`capture_screenshot`]
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
    let ov = journey.card()?;
    let preview = history_preview_for(ov, buffer);
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
        items: ov.item_strings(),
        empty: ov.empty_notice(),
        bindings: ov.item_bindings(),
        ranges: ov.item_range_fracs(),
        git: ov.item_git_tags(),
        selected_index: ov.selected,
        hint: ov.foot_hint(),
        browse_dir: ov.browse_dir.clone(),
        return_to: journey.parked().map(|p| p.kind().as_str()),
        spell_target: ov.spell_target,
        context_anchor: ov.context_anchor,
        preview_id: preview.map(|(id, _, _)| id),
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
        title: ov.kind.title(),
    };
    Some((info, preview_text, diff))
}

/// The SHARED-CORE driver's view of itself. Every method already existed as an
/// inherent accessor on the session (`main/run.rs`); this impl only names them as
/// the fold's five facts, so the storyboard stepper reads them through the same
/// seam the live `App` does.
impl CaptureSubject for super::ReplaySession<'_> {
    fn buffer(&self) -> &Buffer {
        super::ReplaySession::buffer(self)
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
}

/// The HISTORY timeline's headless live preview: when the replay left the History
/// overlay OPEN, resolve its highlighted row's restore id to that version's
/// `(id, content)` via [`crate::history::load`] — keyed by the same shared
/// [`crate::history::source_path`] derivation the live App uses — so the capture
/// shows THAT VERSION in the document itself and the sidecar reports which.
/// `None` for every other overlay kind, the empty-state row, or an unresolvable
/// id (the capture then just shows the buffer — the live degrade). Pure over the
/// store, so it is unit-testable with a seeded log.
pub(super) fn history_preview_for(
    ov: &crate::overlay::OverlayState,
    buffer: &Buffer,
) -> Option<(String, String, crate::prosediff::DiffCounts)> {
    // DIFF-AS-PREVIEW: the preview IS the writer's diff of the current buffer vs
    // the highlighted version — built by the SAME one owner the live App renders
    // through (`history::diff_preview`), synchronously (the live debounce is a
    // wall-clock concern the deterministic capture never has).
    crate::history::diff_preview(ov, buffer.path(), buffer.is_unnamed_fresh(), &buffer.text())
}
