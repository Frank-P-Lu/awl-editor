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
/// Which ordinary file row is active stays `active_index` alone.
pub(in crate::capture) fn json(opts: &CaptureOpts, view: &crate::render::ViewState) -> String {
    let files = view
        .gutter_files
        .iter()
        .map(|row| super::super::sidecar::json_string(&format!("{}{}", row.parent, row.leaf)))
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
                .map(|active| super::super::sidecar::json_string(active))
                .unwrap_or_else(|| "null".to_string()),
        ),
        None => (1, super::super::sidecar::json_string(&view.gutter_name)),
    };
    format!(
        "{{ \"open\": {open}, \"active\": {active}, \"files\": [{files}], \
         \"active_index\": {active_index} }}"
    )
}
