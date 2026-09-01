//! `--keys` replay journeys for ⌥↑/⌥↓ (`Action::MoveLineUp`/`MoveLineDown`) --
//! proves the whole path (chord -> keymap resolution -> `apply_transition`
//! -> the buffer engine) end to end, exactly as a real `--keys "..."`
//! invocation would drive it and a sidecar would report it.

use super::super::*;
use super::{keyspec, replay_keys};

#[test]
fn keys_option_down_then_option_up_round_trip_a_single_line_move() {
    let mut buffer = Buffer::from_str("alpha\nbeta\ngamma\n");
    let root = PathBuf::from("/tmp");

    let keys = keyspec::parse_keys("Option-Down").unwrap();
    replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert_eq!(buffer.text(), "beta\nalpha\ngamma\n");
    assert_eq!(
        buffer.cursor_line_col(),
        (1, 0),
        "the caret rides \"alpha\" down to its new line"
    );

    let keys = keyspec::parse_keys("Option-Up").unwrap();
    replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert_eq!(
        buffer.text(),
        "alpha\nbeta\ngamma\n",
        "Option-Up swaps it right back to the original order"
    );
}

#[test]
fn keys_journey_selects_two_lines_then_moves_the_block_down_as_one() {
    // Shift+Down twice builds a real ("whole line" shaped) selection over
    // "alpha" + "beta"; Option-Down then moves that BLOCK past its one
    // neighbor ("gamma") as a single unit, selection riding along.
    let mut buffer = Buffer::from_str("alpha\nbeta\ngamma\ndelta\n");
    let keys = keyspec::parse_keys("S-Down S-Down Option-Down").unwrap();
    let root = PathBuf::from("/tmp");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);

    assert_eq!(
        buffer.text(),
        "gamma\nalpha\nbeta\ndelta\n",
        "the two-line block swapped past its single neighbor, moved as one"
    );
    assert_eq!(
        res.selection,
        Some(((1, 0), (3, 0))),
        "the selection still spans exactly the moved block, riding the text"
    );
}

#[test]
fn keys_option_up_at_the_first_line_is_a_calm_no_op() {
    let mut buffer = Buffer::from_str("alpha\nbeta\n");
    let keys = keyspec::parse_keys("Option-Up").unwrap();
    let root = PathBuf::from("/tmp");
    replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert_eq!(
        buffer.text(),
        "alpha\nbeta\n",
        "nothing above the first line"
    );
    assert!(!buffer.can_undo(), "a no-op records nothing to undo");
}
