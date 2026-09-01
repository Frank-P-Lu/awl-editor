//! Action-level (`apply_transition`-seam) tests for `Action::MoveLineUp` /
//! `Action::MoveLineDown`: the table row-leave re-pad interaction and the
//! numbered-list renumbering question. The pure line-permutation engine
//! itself (`Buffer::move_line_up`/`move_line_down`) is exhaustively tested
//! at the buffer seam in `buffer/reorder.rs`; these prove the ACTION wiring
//! (dispatch, the landed row-leave re-pad hook, and the existing numbered-
//! list toggle) rather than re-deriving that coverage.

use super::super::*;
use super::{drive_act, md};

#[test]
fn moving_a_table_row_triggers_the_existing_row_leave_repad() {
    // The table starts RAGGED (unaligned) -- moving a row is a plain source
    // edit like any other, so the landed row-leave re-pad (item 542's
    // `auto_align_table_on_row_leave`) should snap the WHOLE table into
    // Prettier alignment as a side effect of the caret leaving its row,
    // proving the EXISTING mechanism fires for a move rather than this
    // action needing its own re-pad logic.
    let mut buffer = md("|Name|V|\n|---|---|\n|a|100|\n|b|2|\n", 0);
    buffer.set_cursor(buffer.line_col_to_char(2, 1)); // inside "|a|100|"
    drive_act(&mut buffer, &Action::MoveLineDown);

    let expected = crate::markdown::align_table("|Name|V|\n|---|---|\n|b|2|\n|a|100|");
    assert_eq!(
        buffer.text(),
        format!("{expected}\n"),
        "the row-leave re-pad snapped the whole table into alignment after the move"
    );

    // The grid itself isn't corrupted: still exactly 4 rows (header,
    // separator, two body rows), body rows genuinely swapped, header/
    // separator untouched and still first.
    let text = buffer.text();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 4, "the move reordered rows, not the row count");
    assert!(lines[0].contains("Name"), "header stayed first");
    assert!(
        lines[1].trim_start_matches('|').trim().starts_with("---"),
        "separator stayed second"
    );
    assert!(lines[2].contains('b'), "the moved-up row is now third");
    assert!(lines[3].contains('a'), "the moved-down row is now fourth");
}

#[test]
fn moving_a_numbered_list_line_does_not_itself_renumber() {
    // No hook fires "renumber a numbered list" on an arbitrary structural
    // edit -- `tab_indents_an_ordered_list_without_renumbering`
    // (`format_editing.rs`) already documents the identical absence for
    // Tab/Shift-Tab. A move relocates the literal ordinal text WITH its
    // line, same as any other action that doesn't touch a list's own
    // markers; it does not re-implement the renumbering rule.
    let mut buffer = md("1. one\n2. two\n3. three\n", 0);
    buffer.set_cursor(buffer.line_col_to_char(0, 0));
    drive_act(&mut buffer, &Action::MoveLineDown);
    assert_eq!(
        buffer.text(),
        "2. two\n1. one\n3. three\n",
        "the ordinals moved WITH their lines, unrenumbered by the move itself"
    );
}

#[test]
fn the_existing_numbered_list_toggle_resequences_a_moved_list() {
    // Proves the EXISTING renumbering rule (`ToggleNumberedList`'s own
    // `block_toggle` sequencing) still correctly fixes a moved list's now-
    // stale ordinals when the user re-invokes it -- reusing the mechanism
    // rather than this action re-implementing it. `block_toggle` is a
    // TOGGLE like every other formatting command: over an ALREADY-numbered
    // selection it strips first (off), so resequencing is off-then-on,
    // exactly the same two-press fixup a user already has for any other
    // numbered-list drift (e.g. deleting a middle item).
    let mut buffer = md("1. one\n2. two\n3. three\n", 0);
    buffer.set_cursor(buffer.line_col_to_char(0, 0));
    drive_act(&mut buffer, &Action::MoveLineDown); // "2. two\n1. one\n3. three\n"

    let select_all = |b: &mut Buffer| {
        b.set_cursor(0);
        b.set_mark();
        let end = b.text().chars().count();
        b.set_cursor(end);
    };

    select_all(&mut buffer);
    drive_act(&mut buffer, &Action::ToggleNumberedList); // already all-prefixed -> strips
    assert_eq!(
        buffer.text(),
        "two\none\nthree\n",
        "toggle OFF strips the stale markers"
    );

    select_all(&mut buffer);
    drive_act(&mut buffer, &Action::ToggleNumberedList); // toggle back ON -> fresh sequence
    assert_eq!(
        buffer.text(),
        "1. two\n2. one\n3. three\n",
        "the existing renumbering rule resequences the moved list correctly"
    );
}
