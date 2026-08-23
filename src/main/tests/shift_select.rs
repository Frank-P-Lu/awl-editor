use super::super::*;
use super::{keyspec, replay_keys};

#[test]
fn replay_keys_builds_selection_from_mark_and_motion() {
    // replay_keys is pure (Buffer + actions, no GPU) but was only reached
    // through the adapter-gated capture tests. Drive it directly: type "abc",
    // mark with C-Space at the end, then move left twice — the post-replay
    // ReplayResult must carry the ordered region.
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys("a b c C-Space Left Left").unwrap();
    let root = PathBuf::from("/tmp");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert_eq!(
        res.selection,
        Some(((0, 1), (0, 3))),
        "mark@3 + two Lefts -> [1,3)"
    );
}

// ── REPLAY SHIFT-SELECT LAWS: `S-` on a motion is select-intent, exactly
// as a live held Shift (the retired "replay is unshifted" hole). The
// replay derives its `apply_transition` shift flag through the ONE owner
// (`crate::app::motion_honors_shift_select`), so these laws pin the
// OUTCOME: a spec's `S-` chord builds the same selection live Shift+motion
// does, and the documented non-movers stay non-movers. ──

/// The shared fixture the shift-select laws replay over: three lines, so
/// every catalog motion has somewhere to go from the middle.
const SHIFT_FIXTURE: &str = "alpha beta\ngamma delta\nepsilon zeta\n";

/// [`shift_replay`]'s result: the replayed selection (as `(anchor,
/// cursor)` line-col pairs, if any), and the final cursor line-col.
type ShiftReplayResult = (Option<((usize, usize), (usize, usize))>, (usize, usize));

fn shift_replay(spec: &str) -> ShiftReplayResult {
    let mut buffer = Buffer::from_str(SHIFT_FIXTURE);
    let keys = keyspec::parse_keys(spec).unwrap();
    let root = PathBuf::from("/tmp");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    (res.selection, buffer.cursor_line_col())
}

#[test]
fn replay_shift_arrow_extends_a_real_selection_then_unshifted_motion_collapses() {
    let (sel, cursor) = shift_replay("S-Right S-Right");
    assert_eq!(cursor, (0, 2), "the motion itself still runs");
    assert_eq!(
        sel,
        Some(((0, 0), (0, 2))),
        "S-Right S-Right spans exactly the two chars the live Shift+Right pair selects"
    );
    let (sel, cursor) = shift_replay("S-Right S-Right Right");
    assert_eq!(cursor, (0, 3));
    assert_eq!(
        sel, None,
        "an unshifted motion collapses the transient shift-selection"
    );
}

#[test]
fn replay_shift_extends_every_catalog_motion_exactly_as_live() {
    // Enumerate the keymap's own motion roster — every catalog command whose
    // action `is_motion()`, over BOTH binding slots — rather than a hand
    // list that can drift: a new motion command is swept automatically.
    // For each, replay its chord with an `S-` prefix from mid-document and
    // assert the LIVE equivalence through the one owner: the selection
    // extends iff `motion_honors_shift_select` — keyed on the pressed CHORD's
    // KEY, not the Action alone — says the Shift is select-intent. So
    // Shift+Cmd-Up/Down (a NAMED nav key reaching BufferStart/BufferEnd) DO
    // extend, GUI-style; only an incidental-Shift printable glyph (`M-<`/`M->`,
    // a `Key::Character`, pinned at the pure-fn seam in `app.rs`) would not.
    // When it extends it spans exactly (pre-cursor, post-cursor).
    // Setup: plain arrows walk to (1,3) — unshifted motions build no selection.
    const SETUP: &str = "Down Right Right Right";
    let (pre_sel, pre_cursor) = shift_replay(SETUP);
    assert_eq!(pre_sel, None);
    assert_eq!(pre_cursor, (1, 3), "setup parks the cursor mid-document");
    let mut swept = 0usize;
    let mut extended_a_named_endpoint = false;
    for cmd in crate::commands::COMMANDS
        .iter()
        .filter(|c| c.action.is_motion())
    {
        for chord in [cmd.native, cmd.emacs] {
            if chord.is_empty() {
                continue;
            }
            swept += 1;
            let spec = format!("S-{chord}");
            let key = keyspec::parse_keys(&spec)
                .unwrap()
                .last()
                .expect("one chord")
                .key
                .clone();
            let (sel, cursor) = shift_replay(&format!("{SETUP} {spec}"));
            assert_ne!(
                cursor, pre_cursor,
                "{} (S-{chord}): the motion must actually move (witness)",
                cmd.name
            );
            if crate::app::motion_honors_shift_select(&cmd.action, &key) {
                let expected = (pre_cursor.min(cursor), pre_cursor.max(cursor));
                assert_eq!(
                    sel,
                    Some(expected),
                    "{} (S-{chord}): Shift+motion extends exactly like live",
                    cmd.name
                );
                if matches!(cmd.action, Action::BufferStart | Action::BufferEnd) {
                    extended_a_named_endpoint = true;
                }
            } else {
                assert_eq!(
                    sel, None,
                    "{} (S-{chord}): incidental Shift stays pure motion, like live",
                    cmd.name
                );
            }
        }
    }
    assert!(
        swept >= 10,
        "the catalog motion roster shrank? swept only {swept} chords"
    );
    assert!(
        extended_a_named_endpoint,
        "a named-key BufferStart/BufferEnd chord must extend now (the shift-select fix)"
    );
}

#[test]
fn replay_shift_named_key_arms_extend_like_live() {
    // The KEYMAP-ONLY named-key arms (plain arrows, Home/End, and the
    // convention-free Ctrl-arrow word aliases live as hand-written input
    // policy in `resolve_named` — no data table exists to enumerate, so
    // these pins mirror `keymap.rs`'s own arm-by-arm style). Each replays
    // with `S-` from mid-document and must extend, spanning exactly
    // (pre, post) — including Shift COMPOSED with M-/C- (the shifted-
    // variant fill / Ctrl-arrow alias, resolving identically to live).
    const SETUP: &str = "Down Right Right Right";
    let (_, pre) = shift_replay(SETUP);
    for chord in [
        "S-Left",
        "S-Right",
        "S-Up",
        "S-Down",
        "S-Home",
        "S-End",
        "S-M-Right",
        "S-M-Left",
        "S-C-Right",
        "S-C-Left",
    ] {
        let (sel, cursor) = shift_replay(&format!("{SETUP} {chord}"));
        assert_ne!(
            cursor, pre,
            "{chord}: the motion must actually move (witness)"
        );
        assert_eq!(
            sel,
            Some((pre.min(cursor), pre.max(cursor))),
            "{chord}: Shift extends the selection exactly like live"
        );
    }
}

#[test]
fn replay_shift_cmd_up_down_extend_to_document_bounds() {
    const SETUP: &str = "Down Right Right Right";
    let (_, pre) = shift_replay(SETUP);
    assert_eq!(pre, (1, 3), "setup parks the cursor mid-document");
    let (sel, cursor) = shift_replay(&format!("{SETUP} S-s-Up"));
    assert_eq!(cursor, (0, 0), "Cmd-Up still lands on document start");
    assert_eq!(
        sel,
        Some(((0, 0), (1, 3))),
        "S-s-Up extends the selection from mid-document to the start"
    );
    let (sel, cursor) = shift_replay(&format!("{SETUP} S-s-Down"));
    assert_ne!(cursor, pre, "Cmd-Down moves the caret to the document end");
    assert_eq!(
        sel,
        Some((pre.min(cursor), pre.max(cursor))),
        "S-s-Down extends the selection from mid-document to the end"
    );
}

#[test]
fn replay_shift_page_scroll_stays_a_documented_non_mover() {
    // Shift-PageDown/PageUp deliberately do NOT extend a selection (the
    // documented divergence — `is_motion` excludes PageScroll*, so the
    // shift-select block never arms). Pin it so the replay-shift fix can
    // never silently promote them; promoting is a conscious follow-up.
    let (sel, cursor) = shift_replay("S-PageDown");
    assert_ne!(cursor, (0, 0), "the page scroll still moves the cursor");
    assert_eq!(sel, None, "Shift-PageDown stays a non-extending non-mover");
    let (sel, _) = shift_replay("S-PageDown S-PageUp");
    assert_eq!(sel, None, "Shift-PageUp stays a non-extending non-mover");
}
