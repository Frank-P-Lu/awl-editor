//! Capture-local wiring for shared view policies.

use crate::buffer::Buffer;
use crate::render::{ScrollPos, TextPipeline};

/// Resolve a bare capture's initial cursor-follow through the same pure policy
/// the live App consumes. Timeline and held captures keep this result fixed after
/// initialization so their frame sequence moves only the caret.
pub(super) fn follow_scroll(
    pipeline: &TextPipeline,
    line: usize,
    col: usize,
    height: f32,
) -> ScrollPos {
    let row = pipeline.visual_row_of(line, col);
    match crate::view_policy::follow_scroll_strategy(crate::typewriter::typewriter_on(), false) {
        crate::view_policy::FollowScroll::ShowRow => {
            pipeline.scroll_to_show_row_pos(row, ScrollPos::default(), height)
        }
        crate::view_policy::FollowScroll::CenterRow => {
            pipeline.scroll_to_center_row_pos(row, height)
        }
        crate::view_policy::FollowScroll::Deferred => {
            unreachable!("a bare capture has no primary-button drag; it still shares the policy")
        }
    }
}

/// Compute capture spell verdicts through the shared version trigger. Capture has
/// no persistent cache, so every new bare pipeline starts at the same `None` state
/// a newly activated live buffer does; checker construction remains capture-local.
pub(super) fn misspellings(buffer: &Buffer) -> Vec<crate::spell::Misspelling> {
    let checked_version = None;
    if crate::view_policy::spell_recompute_needed(checked_version, buffer.version()) {
        match crate::spell::SpellChecker::new(crate::spell::active_variant()) {
            Ok(sc) => sc.misspellings_for(&buffer.text(), buffer.syntax_lang()),
            Err(e) => {
                eprintln!("spell-check disabled for capture: {e}");
                Vec::new()
            }
        }
    } else {
        Vec::new()
    }
}
