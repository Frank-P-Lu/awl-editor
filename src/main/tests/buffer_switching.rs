use super::super::*;
use super::{keyspec, replay_keys};
use crate::testscratch::ScratchDir;

#[test]
fn replay_scrolled_deep_then_open_swaps_to_the_short_file() {
    // The SCROLLED-DEEP-THEN-OPEN replay (the open-then-blank-screen hunt): park
    // the cursor at the END of a long document (Cmd-Down — cursor-follow scroll
    // then sits far past a one-line file's end), summon the Goto picker (Cmd-O),
    // filter to the short file, and accept with Enter. The replay must surface
    // the ACCEPT; the RunCapture arm's swap (mirrored here) yields the SHORT
    // file's buffer with the cursor at (0,0) — the capture re-derives its follow
    // scroll from THAT cursor, so the frame can never render past the new
    // document's EOF. Locks the headless half of the hunt (the live half is the
    // App view-text cache across a swap, tested in `app::tests`).
    // Reads the REAL disk through the fs seam → hold the fs TEST_LOCK so a
    // parallel InMemoryFs installation can't swallow the temp files.
    let _fs = crate::testlock::serial();
    let dir =
        ScratchDir::new(std::env::temp_dir().join(format!("awl-goto-swap-{}", std::process::id())));
    let long: String = (0..300).map(|i| format!("line {i}\n")).collect();
    std::fs::write(dir.join("long.txt"), &long).unwrap();
    std::fs::write(dir.join("short.txt"), "just one line\n").unwrap();
    let mut buffer = Buffer::from_file(&dir.join("long.txt"));
    let keys = keyspec::parse_keys("s-Down s-o s h o r t RET").unwrap();
    let corpus = vec!["long.txt".to_string(), "short.txt".to_string()];
    let res = replay_keys(
        &mut buffer,
        &keys,
        &corpus,
        &dir,
        None,
        &Config::empty(),
        None,
    );
    let (kind, val) = res.accept.expect("Enter accepts the filtered picker row");
    assert_eq!(kind, crate::overlay::OverlayKind::Goto);
    assert_eq!(val, "short.txt");
    // The RunCapture arm swaps in the accepted file's buffer; scroll derives
    // from its fresh (0,0) cursor, never the old document's depth.
    let swapped = Buffer::from_file(&crate::index::resolve(&dir, &val));
    assert_eq!(swapped.text(), "just one line\n");
    assert_eq!(swapped.cursor_line_col(), (0, 0));
}

#[test]
fn replay_keys_goto_a_then_b_then_a_preserves_edits_and_cursor() {
    // THE MULTI-BUFFER v1 win, driven entirely through `--keys`: A -> edit ->
    // B -> edit -> A round-trips through the SAME `crate::buffers::BufferRegistry`
    // the live App uses (wired inline inside `replay_keys`, not deferred to the
    // caller), so the FINAL buffer must be A's LIVE edited content — not a fresh
    // disk re-read — with A's own cursor. This is what makes "assert preserved
    // cursor after an A -> B -> A switch" a headless, agent-verifiable capture.
    let _fs = crate::testlock::serial();
    let dir =
        ScratchDir::new(std::env::temp_dir().join(format!("awl-mb-replay-{}", std::process::id())));
    std::fs::write(dir.join("a.txt"), "alpha\n").unwrap();
    std::fs::write(dir.join("b.txt"), "beta\n").unwrap();
    let mut buffer = Buffer::scratch();
    let corpus = vec!["a.txt".to_string(), "b.txt".to_string()];
    let keys =
        keyspec::parse_keys("s-o a . t x t RET X s-o b . t x t RET Y s-o a . t x t RET").unwrap();
    let res = replay_keys(
        &mut buffer,
        &keys,
        &corpus,
        &dir,
        None,
        &Config::empty(),
        None,
    );
    assert_eq!(
        buffer.text(),
        "Xalpha\n",
        "A's live edit survived the A -> B -> A round trip, not a fresh disk read"
    );
    assert_eq!(
        buffer.path(),
        Some(dir.join("a.txt").as_path()),
        "A is active again"
    );
    assert_eq!(
        res.buffers_open, 3,
        "the launch scratch + A (active) + B (backgrounded, still holding its own edit)"
    );
}

#[test]
fn replay_keys_reopening_the_active_file_is_a_noop() {
    // Guards the same "already active" short-circuit the live `App::load_path`
    // takes: Goto-ing the file that's ALREADY active must not disturb its edit.
    let _fs = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-mb-replay-noop-{}", std::process::id())),
    );
    std::fs::write(dir.join("a.txt"), "alpha\n").unwrap();
    let mut buffer = Buffer::from_file(&dir.join("a.txt"));
    let corpus = vec!["a.txt".to_string()];
    let keys = keyspec::parse_keys("X s-o a . t x t RET").unwrap();
    let res = replay_keys(
        &mut buffer,
        &keys,
        &corpus,
        &dir,
        None,
        &Config::empty(),
        None,
    );
    assert_eq!(
        buffer.text(),
        "Xalpha\n",
        "the edit survives a no-op reopen of the active file"
    );
    assert_eq!(res.buffers_open, 1, "nothing was ever backgrounded");
}

#[test]
fn replay_keys_goto_recognizes_the_active_file_under_a_differently_spelled_but_equal_path() {
    // REGRESSION (code review): the SAME file reached under two different
    // (but equal-after-normalization) path spellings must resolve to the
    // SAME registry entry, or a later Goto silently re-reads it from disk
    // and discards the live edit, orphaning the first spelling's dirty
    // entry in the registry forever. This is the real report's shape (a
    // CLI file argument that stayed relative vs. the Goto picker's always
    // ROOT-JOINED, absolute spelling) reproduced with a `..`-bearing path
    // instead, so the test is deterministic and independent of the test
    // process's real cwd.
    let _fs = crate::testlock::serial();
    let dir =
        ScratchDir::new(std::env::temp_dir().join(format!("awl-mb-relid-{}", std::process::id())));
    std::fs::create_dir_all(dir.join("sub")).unwrap();
    std::fs::write(dir.join("a.txt"), "alpha\n").unwrap();
    std::fs::write(dir.join("b.txt"), "beta\n").unwrap();
    let messy = dir.join("sub").join("..").join("a.txt");
    let mut buffer = Buffer::from_file(&messy);
    let corpus = vec!["a.txt".to_string(), "b.txt".to_string()];
    let keys = keyspec::parse_keys("X s-o b . t x t RET Y s-o a . t x t RET").unwrap();
    let res = replay_keys(
        &mut buffer,
        &keys,
        &corpus,
        &dir,
        None,
        &Config::empty(),
        None,
    );
    assert_eq!(
        buffer.text(),
        "Xalpha\n",
        "the edit to a.txt (opened via a differently-spelled but identical path) survived \
             the round trip to B and back"
    );
    assert_eq!(
        res.buffers_open, 2,
        "a.txt (active) + b.txt (backgrounded) — no orphaned duplicate entry for the messy \
             spelling"
    );
}

#[test]
fn replay_keys_new_note_parks_the_leaving_buffer_instead_of_discarding_it() {
    // REGRESSION (code review): `Effect::NewDocument` used to reset `buffer` in
    // place with no park at all, so A's live edit was gone for good and a
    // later Goto back to it silently re-read a stale disk copy. `Cmd-N`
    // must park the leaving buffer through the SAME registry a Goto switch
    // uses, mirroring the live `App::new_document` (Cmd-N). (The note itself types
    // content but is never named in headless replay — no autosave engine
    // here to derive its filename — so it stays pathless and correctly
    // has NO stable identity to register; see `BufferKey::of`. Only A's
    // survival is under test.)
    let _fs = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-mb-newnote-{}", std::process::id())),
    );
    std::fs::write(dir.join("a.txt"), "alpha\n").unwrap();
    let mut buffer = Buffer::from_file(&dir.join("a.txt"));
    let corpus = vec!["a.txt".to_string()];
    let keys = keyspec::parse_keys("X s-n Z s-o a . t x t RET").unwrap();
    let res = replay_keys(
        &mut buffer,
        &keys,
        &corpus,
        &dir,
        None,
        &Config::empty(),
        None,
    );
    assert_eq!(
        buffer.text(),
        "Xalpha\n",
        "A's edit survived being left for a new note, not a fresh disk re-read"
    );
    assert_eq!(
        res.buffers_open, 1,
        "A active again; the still-unnamed note was never registered (no stable identity), \
             not lost from anywhere else A could be found"
    );
}
