//! Sentence motion at the `apply_transition` seam — shift-extension (via
//! `Action::is_motion`, exactly [`picker_misc_smoke`]'s word-motion tests'
//! own shape) and the sentence deletes' selection/kill-ring behavior. The
//! boundary RULE itself (terminators, quotes, ellipsis, CJK, buffer edges,
//! soft-wrap independence) is exhaustively swept at the purer
//! `buffer::sentence` seam (`buffer/tests/sentence_boundary.rs`); this file
//! only proves the seeded `Action`s wire into the shared editor state
//! correctly. The full keys-to-buffer path (a real `--keys` chord through
//! `KeymapState` resolution) is the `main/tests/sentence_motion.rs` journey.

use super::super::*;
use super::{drive_act, drive_shift};

#[test]
fn shift_sentence_motion_sets_mark_extends_then_unshifted_motion_collapses() {
    let mut b = Buffer::from_str("Hello world. Second one. Third.");
    let mut sel = false;
    drive_shift(&mut b, &mut sel, &Action::ForwardSentence, true);
    assert_eq!(b.anchor_char(), Some(0), "first shift-motion sets the mark");
    assert_eq!(
        b.selection_range(),
        Some((0, 13)),
        "shift-M-e extends through 'Hello world. '"
    );
    assert!(sel, "the transient shift flag arms");
    drive_shift(&mut b, &mut sel, &Action::ForwardSentence, true);
    assert_eq!(b.anchor_char(), Some(0), "the anchor never moves during the run");
    assert_eq!(
        b.selection_range(),
        Some((0, 25)),
        "a second shift-M-e extends onto 'Third.'"
    );
    // Shift released + a plain motion: the transient selection collapses,
    // exactly the non-sentence motions' own rule.
    drive_shift(&mut b, &mut sel, &Action::ForwardSentence, false);
    assert!(
        !b.has_selection(),
        "unshifted sentence motion collapses the shift-selection"
    );
    assert_eq!(b.anchor_char(), None);
    assert!(!sel, "the transient flag disarms");
}

#[test]
fn shift_backward_sentence_extends_backwards_too() {
    let mut b = Buffer::from_str("Hello world. Second one.");
    let mut sel = false;
    b.buffer_end();
    drive_shift(&mut b, &mut sel, &Action::BackwardSentence, true);
    assert_eq!(b.anchor_char(), Some(24));
    assert_eq!(
        b.selection_range(),
        Some((13, 24)),
        "shift-M-a selects 'Second one.'"
    );
    drive_shift(&mut b, &mut sel, &Action::BackwardSentence, true);
    assert_eq!(
        b.selection_range(),
        Some((0, 24)),
        "a second shift-M-a extends to the buffer start"
    );
}

#[test]
fn delete_sentence_forward_replaces_an_active_selection_instead() {
    let mut b = Buffer::from_str("Hello world. Second one.");
    b.set_cursor(0);
    b.set_mark();
    b.set_cursor(5); // selects "Hello"
    drive_act(&mut b, &Action::DeleteSentenceForward);
    assert_eq!(
        b.text(),
        " world. Second one.",
        "an active selection deletes instead of killing to the sentence end"
    );
}

#[test]
fn delete_sentence_backward_replaces_an_active_selection_instead() {
    let mut b = Buffer::from_str("Hello world. Second one.");
    b.set_cursor(0);
    b.set_mark();
    b.set_cursor(5);
    drive_act(&mut b, &Action::DeleteSentenceBackward);
    assert_eq!(
        b.text(),
        " world. Second one.",
        "an active selection deletes instead of killing to the sentence start"
    );
}

#[test]
fn delete_sentence_forward_at_buffer_end_is_a_calm_no_op() {
    let mut b = Buffer::from_str("Only sentence.");
    b.buffer_end();
    drive_act(&mut b, &Action::DeleteSentenceForward);
    assert_eq!(b.text(), "Only sentence.", "nothing follows the caret");
}

#[test]
fn delete_sentence_backward_at_buffer_start_is_a_calm_no_op() {
    let mut b = Buffer::from_str("Only sentence.");
    b.buffer_start();
    drive_act(&mut b, &Action::DeleteSentenceBackward);
    assert_eq!(b.text(), "Only sentence.", "nothing precedes the caret");
}
