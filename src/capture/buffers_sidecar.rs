//! The live-App buffer registry and working-set state in the capture sidecar.

use super::CaptureOpts;

/// The sidecar `buffers` block, serialized beside the type it reports rather
/// than in the writer, so the count/identity half and the working-set half stay
/// one owner's business.
///
/// `files` / `active_index` are the VISIBLE WORKING SET — the same rows the
/// margin's bottom identity widened into this frame, read off the `ViewState`
/// the frame was composed from rather than re-derived, so the sidecar cannot
/// report a stack the pixels do not show.
///
/// These are state labels: each row is its full root-relative path, never the
/// width-elided text the margin drew. The drawn line is `gutter`'s business.
/// Which ordinary file row is active stays `active_index` alone; the richer
/// capture-only prototype report is emitted only when its sealed pose exists.
pub(super) fn json(opts: &CaptureOpts, view: &crate::render::ViewState) -> String {
    let files = view
        .gutter_files
        .iter()
        .map(|row| super::sidecar::json_string(&format!("{}{}", row.parent, row.leaf)))
        .collect::<Vec<_>>()
        .join(", ");
    let active_index = view
        .gutter_files
        .iter()
        .position(|row| row.active)
        .map(|at| at.to_string())
        .unwrap_or_else(|| "null".to_string());
    let (open, active) = match &opts.buffers {
        Some(b) => (
            b.open,
            b.active
                .as_ref()
                .map(|active| super::sidecar::json_string(active))
                .unwrap_or_else(|| "null".to_string()),
        ),
        None => (1, super::sidecar::json_string(&view.gutter_name)),
    };
    let prototype = opts
        .working_set_prototype
        .as_ref()
        .map(|report| prototype_json(report, &view.gutter_files))
        .unwrap_or_default();
    format!(
        "{{ \"open\": {open}, \"active\": {active}, \"files\": [{files}], \
         \"active_index\": {active_index}{prototype} }}"
    )
}

fn prototype_json(
    report: &crate::workingset::PrototypeReport,
    rows: &[crate::workingset::StackRow],
) -> String {
    use crate::workingset::StackRowKind;
    let rows = rows
        .iter()
        .map(|row| {
            let label = format!("{}{}", row.parent, row.leaf);
            let (kind, hidden, group_active) = match row.kind {
                StackRowKind::File => ("file", "null".to_string(), "null".to_string()),
                StackRowKind::More { hidden } => ("more", hidden.to_string(), "null".to_string()),
                StackRowKind::Group { active } => ("group", "null".to_string(), active.to_string()),
            };
            format!(
                "{{ \"kind\": \"{kind}\", \"label\": {}, \"active\": {}, \
                 \"hidden\": {hidden}, \"group_active\": {group_active}, \
                 \"hovered\": {} }}",
                super::sidecar::json_string(&label),
                row.active,
                row.prototype_hovered,
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        ", \"prototype\": {{ \"mode\": \"{}\", \"total_open\": {}, \
         \"total_file_rows\": {}, \"visible_file_rows\": {}, \"hidden\": {}, \
         \"scroll\": {}, \"viewport\": {}, \"active_row\": {}, \
         \"hovered_row\": {}, \"rows\": [{rows}] }}",
        report.mode,
        report.total_open,
        report.total_file_rows,
        report.visible_file_rows,
        report.hidden,
        report.scroll,
        report.viewport,
        report
            .active_row
            .map(|at| at.to_string())
            .unwrap_or_else(|| "null".to_string()),
        report
            .hovered_row
            .map(|at| at.to_string())
            .unwrap_or_else(|| "null".to_string()),
    )
}
