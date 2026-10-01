//! Workspace continuation owns its fixed footer composition. Keeping the row
//! reservation beside the shaped strings prevents a scroll-position-specific
//! edge count from changing the card, rail, or content origins.

use super::edge_cue_text;

/// A windowed workspace keeps one fixed footer shape at every scroll position:
/// a separator plus one slot for each edge cue. An edge with no hidden items
/// contributes a blank slot rather than changing the footer's height under the
/// reader. Returning the row count with the strings makes the reservation and
/// the shaper two reads of one composition.
pub(super) fn workspace_continuation_footer(
    windowed: bool,
    cue_above: Option<usize>,
    cue_below: Option<usize>,
) -> (Vec<String>, usize) {
    if !windowed {
        return (Vec::new(), 0);
    }
    let footer = vec![
        cue_above.map_or_else(|| " ".into(), |n| edge_cue_text(true, n)),
        cue_below.map_or_else(|| " ".into(), |n| edge_cue_text(false, n)),
    ];
    let rows = footer.len() + 1;
    (footer, rows)
}
