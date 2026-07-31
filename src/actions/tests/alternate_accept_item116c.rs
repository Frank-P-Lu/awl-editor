//! ITEM 116c — THE BYTE-IDENTITY LAW.
//!
//! `Action::AcceptAlternate` (⇧↵) is resolved as its own keymap `Action` (never
//! a bare alias applied at the keymap layer), but `apply_buffer_action`'s
//! `Newline` arm — the ONE place the editor's smart-Enter decision lives —
//! matches `Action::Newline | Action::AcceptAlternate` on the SAME arm. This
//! file is the law that proves the delegation over real buffer bytes
//! (`Buffer::disk_bytes`), not by inspecting the source: for every shape
//! `smart_newline` can decide (the axis a bare "same code" claim could still
//! get wrong if a future edit split the arm back apart), driving `Newline` and
//! driving `AcceptAlternate` from an identical starting buffer must produce
//! byte-identical disk contents.
//!
//! MUTATION-PROVED: temporarily routing `AcceptAlternate` to a bare
//! `ctx.buffer.insert_newline()` (skipping `smart_newline` entirely) turned
//! `accept_alternate_is_byte_identical_to_newline_across_every_smart_newline_shape`
//! red by name on the list-continuation and empty-list-item fixtures (the ones
//! `smart_newline` actually changes); see the round's commit for the verbatim
//! panic text. Restored before landing.

use super::super::*;
use crate::buffer::Buffer;

/// Drive one `action` against a FRESH buffer `build` produces, and return the
/// resulting on-disk bytes. A fresh `ActionCtx` per call (mirroring
/// `drive_act_effect`), so comparing two calls' results can never be polluted
/// by shared state.
fn disk_bytes_after(build: fn() -> Buffer, action: &Action) -> Vec<u8> {
    let mut buffer = build();
    let mut shift = false;
    let mut zoom = 1.0;
    let mut search = None;
    let mut journey = crate::overlay::Journey::default();
    let mut make_overlay = |_k: OverlayKind| -> Option<OverlayState> { None };
    let mut browse_to = |_k: OverlayKind, _r: Option<String>| -> Option<OverlayState> { None };
    let mut ctx = ActionCtx {
        buffer: &mut buffer,
        shift_selecting: &mut shift,
        zoom: &mut zoom,
        search: &mut search,
        scroll_page_lines: 1,
        journey: &mut journey,
        make_overlay: &mut make_overlay,
        browse_to: &mut browse_to,
        oracle: None,
    };
    let _ = apply_transition(&mut ctx, action, false);
    buffer.disk_bytes()
}

/// Like [`disk_bytes_after`], but drives a WHOLE SEQUENCE on one buffer — the
/// only way to exercise item 78's short-lived list-continuation provenance
/// flag, which lives on the buffer across successive actions.
fn disk_bytes_after_seq(build: fn() -> Buffer, actions: &[Action]) -> Vec<u8> {
    let mut buffer = build();
    let mut shift = false;
    let mut zoom = 1.0;
    let mut search = None;
    let mut journey = crate::overlay::Journey::default();
    let mut make_overlay = |_k: OverlayKind| -> Option<OverlayState> { None };
    let mut browse_to = |_k: OverlayKind, _r: Option<String>| -> Option<OverlayState> { None };
    let mut ctx = ActionCtx {
        buffer: &mut buffer,
        shift_selecting: &mut shift,
        zoom: &mut zoom,
        search: &mut search,
        scroll_page_lines: 1,
        journey: &mut journey,
        make_overlay: &mut make_overlay,
        browse_to: &mut browse_to,
        oracle: None,
    };
    for action in actions {
        let _ = apply_transition(&mut ctx, action, false);
    }
    buffer.disk_bytes()
}

// ── FIXTURES — one per `smart_newline_for` shape, plus the two gates
//    (`is_markdown`, `has_selection`) that skip it entirely. ────────────────

fn plain_prose() -> Buffer {
    let mut b = Buffer::from_str("hello world");
    b.set_cursor(5); // between "hello" and " world" — no marker to continue
    b
}

fn bullet_continue() -> Buffer {
    let mut b = Buffer::from_str("- alpha\n- beta");
    b.set_cursor(7); // end of "- alpha"
    b
}

fn numbered_continue() -> Buffer {
    let mut b = Buffer::from_str("1. first\n2. second");
    b.set_cursor(8); // end of "1. first" — the next marker must increment
    b
}

fn task_list_continue() -> Buffer {
    let mut b = Buffer::from_str("- [x] done item");
    b.set_cursor(15); // end of line — continuation must open UNCHECKED
    b
}

fn empty_bullet_preserves_marker() -> Buffer {
    let mut b = Buffer::from_str("- ");
    b.set_cursor(2); // an empty item with NO generated-provenance flag set
    b
}

fn blockquote_continue() -> Buffer {
    let mut b = Buffer::from_str("> quoted text");
    b.set_cursor(13);
    b
}

fn blockquote_ends_when_empty() -> Buffer {
    let mut b = Buffer::from_str("> ");
    b.set_cursor(2);
    b
}

fn bare_indent_carries() -> Buffer {
    let mut b = Buffer::from_str("    indented text");
    b.set_cursor(17);
    b
}

fn non_markdown_ignores_markers() -> Buffer {
    // A `.rs` path takes `is_markdown()` false — `smart_newline` must bail
    // before ever reading the (list-marker-shaped) line text.
    let mut b = Buffer::from_str("- alpha");
    b.set_path(std::path::PathBuf::from("src/scratch.rs"));
    b.set_cursor(7);
    b
}

fn active_selection_is_overwritten() -> Buffer {
    let mut b = Buffer::from_str("- alpha");
    b.set_anchor(0);
    b.set_cursor(7); // a real, non-empty selection over the whole marker line
    b
}

/// Every single-step fixture above, named for assertion failures.
fn single_step_fixtures() -> Vec<(&'static str, fn() -> Buffer)> {
    vec![
        ("plain_prose", plain_prose),
        ("bullet_continue", bullet_continue),
        ("numbered_continue", numbered_continue),
        ("task_list_continue", task_list_continue),
        (
            "empty_bullet_preserves_marker",
            empty_bullet_preserves_marker,
        ),
        ("blockquote_continue", blockquote_continue),
        ("blockquote_ends_when_empty", blockquote_ends_when_empty),
        ("bare_indent_carries", bare_indent_carries),
        ("non_markdown_ignores_markers", non_markdown_ignores_markers),
        (
            "active_selection_is_overwritten",
            active_selection_is_overwritten,
        ),
    ]
}

/// THE LAW: for every shape above, `Newline` and `AcceptAlternate` from an
/// identical starting buffer land on byte-identical disk contents. Swept
/// over every `smart_newline_for` branch (not just the reported "plain
/// prose" case) — the axis a narrower, single-fixture law would miss.
#[test]
fn accept_alternate_is_byte_identical_to_newline_across_every_smart_newline_shape() {
    for (name, build) in single_step_fixtures() {
        let via_newline = disk_bytes_after(build, &Action::Newline);
        let via_alternate = disk_bytes_after(build, &Action::AcceptAlternate);
        assert_eq!(
            via_newline, via_alternate,
            "{name}: Shift+Enter must produce the exact bytes plain Enter does"
        );
    }
}

/// THE PROVENANCE SHAPE (item 78): a list item's SECOND, now-empty Enter reads
/// a flag the FIRST Enter left on the buffer. Proven three ways so the flag's
/// behavior can never depend on which of the two actions set it or read it:
/// both steps `Newline`, both `AcceptAlternate`, and each MIXED order.
#[test]
fn accept_alternate_carries_the_list_continuation_provenance_flag_identically() {
    let sequences: [[Action; 2]; 4] = [
        [Action::Newline, Action::Newline],
        [Action::AcceptAlternate, Action::AcceptAlternate],
        [Action::Newline, Action::AcceptAlternate],
        [Action::AcceptAlternate, Action::Newline],
    ];
    let baseline = disk_bytes_after_seq(bullet_continue, &sequences[0]);
    for seq in &sequences[1..] {
        assert_eq!(
            disk_bytes_after_seq(bullet_continue, seq),
            baseline,
            "{seq:?}: the provenance-gated second Enter must land identically \
             regardless of which accept action fired at either step"
        );
    }
}
