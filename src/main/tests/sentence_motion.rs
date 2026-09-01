//! THE `--keys` SIDECAR JOURNEY for sentence motion — the full path
//! (keys → `Action` → `apply_transition` → visible buffer state) exercised
//! through the SAME `KeymapState` resolution live editing uses, not a
//! direct `Action` drive. Unlike every other subject in this directory, the
//! chords under test (M-e / M-a / M-k) have NO default binding on
//! `Convention::Mac` — see `buffer::sentence`'s module doc and
//! `keymap::platform::LINUX_EMACS_META_SEED` — so this file builds its own
//! `Convention::Linux` + `keymap = "emacs"` [`KeymapState`] rather than
//! reusing this directory's Mac-pinned `replay_keys` shadow.

use super::super::*;
use crate::convention::Convention;
use crate::keymap::KeymapState;

/// The fixture the whole file replays over: two ordinary sentences, so
/// M-e/M-a/M-k each have somewhere real to land.
const FIXTURE: &str = "Hello world. Second one.";

/// Replay `spec` against `buffer` through a REAL `Convention::Linux`,
/// `keymap = "emacs"` resolver — the production door
/// (`KeymapState::set_linux_emacs_meta`) that seeds M-e/M-a/M-k, called the
/// same way `App::apply_keymap_flavor`/config reload call it live.
fn replay_linux_emacs(buffer: &mut Buffer, spec: &str) -> ReplayResult {
    let mut km = KeymapState::new_with_convention(Convention::Linux);
    km.set_linux_emacs_meta(true);
    let keys = crate::keyspec::parse_chords(spec).unwrap();
    let root = PathBuf::from("/tmp");
    match replay_keys_mode(
        crate::replay::Mode::Permissive,
        crate::replay::FilesystemCapability::None,
        buffer,
        &keys,
        &[],
        &root,
        None,
        &Config::empty(),
        None,
        &mut km,
    ) {
        Ok(res) => res,
        Err(e) => unreachable!("permissive replay never aborts: {e}"),
    }
}

#[test]
fn m_e_walks_forward_sentence_by_sentence() {
    let mut buffer = Buffer::from_str(FIXTURE);
    replay_linux_emacs(&mut buffer, "M-e");
    assert_eq!(
        buffer.cursor_line_col(),
        (0, 13),
        "M-e from the start lands on 'Second', past 'Hello world. '"
    );
    replay_linux_emacs(&mut buffer, "M-e");
    assert_eq!(
        buffer.cursor_line_col(),
        (0, FIXTURE.chars().count()),
        "a second M-e reaches the buffer end — the last sentence has no follower"
    );
}

#[test]
fn m_a_walks_backward_sentence_by_sentence() {
    let mut buffer = Buffer::from_str(FIXTURE);
    buffer.buffer_end();
    replay_linux_emacs(&mut buffer, "M-a");
    assert_eq!(
        buffer.cursor_line_col(),
        (0, 13),
        "M-a from the end lands on the start of 'Second one.'"
    );
    replay_linux_emacs(&mut buffer, "M-a");
    assert_eq!(
        buffer.cursor_line_col(),
        (0, 0),
        "a second M-a reaches the buffer start"
    );
}

#[test]
fn shift_m_e_is_the_seeded_layers_own_documented_gap_not_a_regression() {
    // `Action::is_motion` enrollment is what makes shift-extension possible
    // (see `actions::tests::sentence_motion` for that proof, driven at the
    // pure `apply_transition` seam) — but the CHORD path here doesn't reach
    // it: `KeymapState::seed_defaults` seeds the Meta layer LAST, deliberately
    // AFTER the "give every default chord an automatic Shift companion" pass
    // (see that fn's own "seeded LAST" comment), so a seeded entry never
    // grows one. `S-M-e` therefore resolves the SAME way `S-M-f` (word
    // motion, seeded by the identical table) already does — not to
    // `ForwardSentence` at all. Pinned here so a future change to that
    // seeding order is a conscious choice, not a silent regression either
    // way.
    let mut buffer = Buffer::from_str(FIXTURE);
    let res = replay_linux_emacs(&mut buffer, "S-M-e");
    assert_ne!(
        buffer.cursor_line_col(),
        (0, 13),
        "S-M-e does not resolve to ForwardSentence — no seeded chord carries an auto Shift companion"
    );
    assert_eq!(
        res.selection, None,
        "and so builds no selection through the chord path either"
    );
}

#[test]
fn m_k_kills_to_sentence_end_and_the_kill_ring_yanks_it_back() {
    let mut buffer = Buffer::from_str(FIXTURE);
    replay_linux_emacs(&mut buffer, "M-k");
    assert_eq!(
        buffer.text(),
        "Second one.",
        "M-k removes 'Hello world. ' up to the following sentence"
    );
    // C-y is the emacs paste slot (assets/keymap-defaults.toml `paste`); it
    // fires on Convention::Linux without the emacs Meta layer, so a plain
    // Linux replay proves the kill actually landed in the kill ring, not
    // just off the visible buffer.
    replay_linux_emacs(&mut buffer, "C-y");
    assert_eq!(
        buffer.text(),
        FIXTURE,
        "C-y yanks the killed sentence back, restoring the original text"
    );
}
