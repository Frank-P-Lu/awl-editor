//! THE SIDECAR FOLD for a still-open summoned card: the one owner of "what
//! does an open overlay look like in the sidecar", shared by the storyboard
//! runner and the one-shot `--keys` capture so the two can never drift. Lifted
//! verbatim out of `main/run.rs` (queue item 173, to keep that file inside its
//! size mark); behaviour is unchanged.

use crate::buffer::Buffer;
use crate::capture;

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
        preview_id: preview.map(|(id, _, _)| id),
        workspace: ov.workspace_shell(),
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
