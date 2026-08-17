//! THE CREDITS VIEWER, DRIVEN BY REAL `--keys` CHORDS THROUGH THE COMMAND
//! PALETTE — the end-to-end proof that `Action::OpenCredits` reaches
//! `OverlayKind::Credits` through the real keymap + palette fuzzy filter, not
//! only through a hand-built `ActionCtx` (`actions::tests::credits` owns
//! that purer-seam half of the same law). `replay_keys` is the exact seam
//! `history.rs`'s `replay_history_esc_leaves_buffer_text_exact` already uses
//! for the sibling workspace.

use super::super::*;
use super::{keyspec, replay_keys};

fn credits_buffer() -> Buffer {
    let mut b = Buffer::from_str("# My Notes\n\nSome real prose the user is editing.\n");
    b.set_path(PathBuf::from("/notes/draft.md"));
    b
}

/// Cmd-P, type "credits", Enter: the palette's fuzzy filter lands on the
/// Credits row and accepting it summons `OverlayKind::Credits` standing on
/// its CONTENT stage already — there is no row to choose, so PageDown must
/// scroll immediately rather than stepping an inert one-row rail.
#[test]
fn replay_credits_opens_via_the_palette_onto_the_content_stage_and_scrolls() {
    let mut buffer = credits_buffer();
    let keys = keyspec::parse_keys("s-p c r e d i t s Enter").unwrap();
    let root = PathBuf::from("/notes");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    let card = res
        .journey
        .card()
        .expect("Cmd-P -> credits -> Enter summons the viewer");
    assert_eq!(card.kind, crate::overlay::OverlayKind::Credits);
    assert!(
        card.detail_focus,
        "Credits opens with the content stage already focused"
    );
    assert_eq!(card.diff_scroll, 0);
    assert!(
        res.replay_skips.is_empty(),
        "opening Credits is fully replay-supported, not a live-App-only degrade: {:?}",
        res.replay_skips
    );

    // A second replay, one PageDown further, moves the scroll — the SAME
    // `diff_scroll` field History/Conflict already drive, proving the
    // universal workspace scroll keys reach Credits through the real keymap.
    let keys2 = keyspec::parse_keys("s-p c r e d i t s Enter PageDown").unwrap();
    let mut buffer2 = credits_buffer();
    let res2 = replay_keys(
        &mut buffer2,
        &keys2,
        &[],
        &root,
        None,
        &Config::empty(),
        None,
    );
    assert!(
        res2.journey.card().unwrap().diff_scroll > 0,
        "PageDown must move diff_scroll, or the byte-identity law below would be \
         vacuously true of a viewer that never actually opened"
    );
}

/// **THE REGRESSION LAW.** Credits used to swap the editor to a real editable
/// buffer (`App::open_credits` -> `open_bundled_doc` -> `load_path`). Driven
/// through the real `--keys` seam end to end — palette open, scroll, dismiss —
/// the active buffer's path and text must be byte-for-byte what they were
/// before any of it happened.
///
/// TWO `Esc`, not one: a palette-launched action PARKS the palette as the
/// summon's parent (`Effect::RunAction`'s `pending_return_to`, the same
/// breadcrumb every other palette-launched picker uses — `palette.rs`'s own
/// `"...RET Esc Esc s-t..."` is the precedent), so the first `Esc` returns to
/// the parked Command palette and the second leaves it. Neither Esc, nor the
/// parked palette, ever touches the buffer.
#[test]
fn replay_credits_open_scroll_and_esc_leave_the_buffer_exact() {
    let mut buffer = credits_buffer();
    let before_path = buffer.path().map(|p| p.to_path_buf());
    let before_text = buffer.text();
    let keys = keyspec::parse_keys("s-p c r e d i t s Enter PageDown PageDown Esc Esc").unwrap();
    let root = PathBuf::from("/notes");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert!(
        res.journey.card().is_none(),
        "two Esc closes the parked palette and the Credits viewer beneath it: {:?}",
        res.journey.card().map(|c| c.kind)
    );
    assert!(
        res.accept.is_none(),
        "the read-only viewer never accepts anything"
    );
    assert_eq!(
        buffer.path().map(|p| p.to_path_buf()),
        before_path,
        "the active buffer's path must not change across open/scroll/dismiss"
    );
    assert_eq!(
        buffer.text(),
        before_text,
        "the active buffer's text must not change across open/scroll/dismiss"
    );
}
