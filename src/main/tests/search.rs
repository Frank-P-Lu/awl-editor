use super::super::*;
use super::{keyspec, replay_keys, replay_keys_mode};

// ── SHARED SEARCH/REPLACE INPUT ROUTING: the replay-side search guard ──
//
// While the isearch panel is open the replay loop consumes EVERY chord
// through the ONE interception seam the live window uses
// (`crate::search::keys::intercept`), BEFORE keymap resolution — so the
// whole in-panel operation set is `--keys`-drivable. The seam itself is
// unit-tested in `search::keys::tests`; these pin the replay WIRING.

#[test]
fn replay_search_typing_extends_the_query_never_the_buffer() {
    let mut buffer = Buffer::from_str("say hi twice: hi");
    let keys = keyspec::parse_keys("C-s h i").unwrap();
    let root = PathBuf::from("/tmp");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert_eq!(
        buffer.text(),
        "say hi twice: hi",
        "the document is untouched"
    );
    assert_eq!(res.search_query.as_deref(), Some("hi"));
    assert_eq!(buffer.cursor_char(), 4, "the caret sits on the first match");
}

#[test]
fn query_input_live_delegate_and_headless_guard_have_identical_outcomes() {
    let _guard = crate::testlock::serial();
    crate::search::clear_last_query();
    let original = "alpha beta alpha";
    let root = PathBuf::from("/tmp");

    let mut replay_buffer = Buffer::from_str(original);
    let keys = keyspec::parse_keys("C-s a l p h a Tab X Enter").unwrap();
    let replay = replay_keys(
        &mut replay_buffer,
        &keys,
        &[],
        &root,
        None,
        &Config::empty(),
        None,
    );

    // `App::handle_search_key` is deliberately a thin delegate to this exact
    // seam. Drive the same post-summon chords directly to pin the live
    // delegate's model outcome against ReplaySession's search guard.
    let mut live_buffer = Buffer::from_str(original);
    let mut live_search = Some(crate::search::SearchState::start(
        0,
        crate::search::Direction::Forward,
    ));
    for chord in keyspec::parse_keys("a l p h a Tab X Enter").unwrap() {
        let _ = crate::search::keys::intercept(
            &mut live_search,
            &mut live_buffer,
            &chord.key,
            chord.mods.state(),
        );
    }
    let live = live_search
        .as_ref()
        .expect("replace-one keeps the panel open");
    assert_eq!(replay_buffer.text(), live_buffer.text());
    assert_eq!(replay_buffer.cursor_char(), live_buffer.cursor_char());
    assert_eq!(replay.search_query.as_deref(), Some(live.query()));
    assert_eq!(replay.replacement, live.replacement());
    assert_eq!(replay.editing_replacement, live.is_editing_replacement());
    crate::search::clear_last_query();
}

#[test]
fn replay_search_steps_case_toggle_and_prefix_chords_stay_in_the_panel() {
    let mut buffer = Buffer::from_str("x.x.x");
    let keys = keyspec::parse_keys("C-s x Down C-s").unwrap();
    let root = PathBuf::from("/tmp");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert_eq!(res.search_query.as_deref(), Some("x"));
    assert_eq!(buffer.cursor_char(), 4, "two steps advanced 0 -> 2 -> 4");

    let mut buffer = Buffer::from_str("Hello HELLO hello");
    let keys = keyspec::parse_keys("C-s h e l l o M-c").unwrap();
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert!(
        res.search_case,
        "M-c toggled case sensitivity inside the panel"
    );
    assert_eq!(res.search_query.as_deref(), Some("hello"));

    // A `C-x` while searching is CONSUMED (a no-op), never a prefix arm —
    // the following `r` extends the QUERY instead of resolving to the
    // `C-x r` ToggleDebug sequence. (The old parse-time resolution got
    // this wrong by construction.)
    let _g = crate::testlock::serial();
    let debug_before = crate::debug::debug_on();
    let mut buffer = Buffer::from_str("xr marks");
    let keys = keyspec::parse_keys("C-s x C-x r").unwrap();
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert_eq!(
        res.search_query.as_deref(),
        Some("xr"),
        "C-x was eaten; r joined the query"
    );
    assert_eq!(
        crate::debug::debug_on(),
        debug_before,
        "C-x r never reached the keymap"
    );
}

#[test]
fn replay_search_replacement_typing_replace_one_and_replace_all() {
    let mut buffer = Buffer::from_str("line one\nline two\nline three");
    let keys = keyspec::parse_keys("C-s l i n e Tab r o w Enter").unwrap();
    let root = PathBuf::from("/tmp");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert_eq!(buffer.text(), "row one\nline two\nline three");
    assert_eq!(res.search_query.as_deref(), Some("line"));
    assert!(res.replace_active);
    assert_eq!(res.replacement, "row");
    assert!(res.editing_replacement, "focus stayed in the replace field");
    assert_eq!(
        buffer.cursor_char(),
        8,
        "the caret advanced to the next match"
    );

    let mut buffer = Buffer::from_str("line one\nline two\nline three");
    let keys = keyspec::parse_keys("C-s l i n e Tab r o w s-Enter").unwrap();
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert_eq!(buffer.text(), "row one\nrow two\nrow three");
    assert!(
        res.replace_active,
        "the panel is still open after replace-all"
    );
}

#[test]
fn replay_search_enter_accepts_and_esc_restores_origin() {
    let _g = crate::testlock::serial();
    crate::search::clear_last_query();
    let mut buffer = Buffer::from_str("alpha beta alpha");
    let keys = keyspec::parse_keys("C-s b e t a Enter").unwrap();
    let root = PathBuf::from("/tmp");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert_eq!(res.search_query, None, "Enter closed the panel");
    assert_eq!(
        buffer.cursor_char(),
        6,
        "the cursor stays on the accepted match"
    );
    assert_eq!(crate::search::last_query(), "beta");

    // ESC aborts: the panel closes AND the origin cursor is restored —
    // live behavior (the old headless-only `Cancel` close skipped the
    // origin restore; that divergence is gone with the shared seam).
    let mut buffer = Buffer::from_str("alpha beta alpha");
    let keys = keyspec::parse_keys("C-f C-f C-s b e t a Esc").unwrap();
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert_eq!(res.search_query, None, "Esc closed the panel");
    assert_eq!(buffer.cursor_char(), 2, "the origin cursor is restored");
    assert_eq!(buffer.text(), "alpha beta alpha");
    crate::search::clear_last_query();
}

#[test]
fn strict_replay_allows_panel_consumed_chords_but_rejects_them_outside() {
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys("s-l").unwrap();
    let root = PathBuf::from("/tmp");
    let err = replay_keys_mode(
        crate::replay::Mode::Strict,
        &mut buffer,
        &keys,
        &[],
        &root,
        None,
        &Config::empty(),
        None,
    )
    .err()
    .expect("strict refuses an unbound chord outside the panel")
    .to_string();
    assert!(err.contains("\"s-l\"") && err.contains("unbound"), "{err}");

    // …but INSIDE the panel the guard owns every chord (`s-l` is a
    // consumed no-op, `M-c` a case toggle, `C-x` never arms the prefix),
    // so the same spec is legal under strict — the reason strictness had
    // to move from parse time to replay time.
    let mut buffer = Buffer::from_str("Hello HELLO hello");
    let keys = keyspec::parse_keys("C-s h s-l M-c C-x").unwrap();
    let res = replay_keys_mode(
        crate::replay::Mode::Strict,
        &mut buffer,
        &keys,
        &[],
        &root,
        None,
        &Config::empty(),
        None,
    )
    .expect("panel-consumed chords are legal under strict");
    assert!(res.search_case, "the M-c actually toggled case");
    assert_eq!(res.search_query.as_deref(), Some("h"));
}
