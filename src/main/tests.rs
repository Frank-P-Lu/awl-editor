use super::*;

// CONVENTION-PROOF SHADOWS: this whole file's `--keys` replay tests hardcode
// MAC-form literal specs ("Cmd-S-h", "s-p", a bare "C-n"/"C-x" whose letter
// Linux's collision table displaces, …) — pinning resolution to
// `Convention::Mac` is the honest fix (these tests document specifically
// what a MAC-convention chord does; Linux's own displacement/collision
// behavior is separately, exhaustively law-tested in `keymap.rs`). Chord
// PARSING is now convention-free (`parse_chords` never touches the keymap),
// so the pinning moved WITH resolution into the replay loop: these local
// `replay_keys`/`replay_keys_mode` wrappers SHADOW the module-level fns
// (a local item wins over a glob import) and supply a Mac-pinned
// `KeymapState`, so none of the ~60 call sites below needed rewriting. The
// local `keyspec` module keeps the old call shape for the same reason.
mod keyspec {
    pub fn parse_keys(spec: &str) -> anyhow::Result<Vec<crate::keyspec::Chord>> {
        crate::keyspec::parse_chords(spec)
    }
}

#[allow(clippy::too_many_arguments)]
fn replay_keys_mode(
    mode: crate::replay::Mode,
    buffer: &mut Buffer,
    keys: &[crate::keyspec::Chord],
    corpus: &[String],
    root: &std::path::Path,
    workspace: Option<&std::path::Path>,
    config: &Config,
    oracle: Option<&mut capture::OraclePipeline>,
) -> Result<ReplayResult> {
    let mut km =
        crate::keymap::KeymapState::new_with_convention(crate::convention::Convention::Mac);
    super::replay_keys_mode(
        mode, buffer, keys, corpus, root, workspace, config, oracle, &mut km,
    )
}

#[allow(clippy::too_many_arguments)]
fn replay_keys(
    buffer: &mut Buffer,
    keys: &[crate::keyspec::Chord],
    corpus: &[String],
    root: &std::path::Path,
    workspace: Option<&std::path::Path>,
    config: &Config,
    oracle: Option<&mut capture::OraclePipeline>,
) -> ReplayResult {
    match replay_keys_mode(
        crate::replay::Mode::Permissive,
        buffer,
        keys,
        corpus,
        root,
        workspace,
        config,
        oracle,
    ) {
        Ok(res) => res,
        Err(e) => unreachable!("permissive replay never aborts: {e}"),
    }
}

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
// replay derives its `apply_core` shift flag through the ONE owner
// (`crate::app::motion_honors_shift_select`), so these laws pin the
// OUTCOME: a spec's `S-` chord builds the same selection live Shift+motion
// does, and the documented non-movers stay non-movers. ──

/// The shared fixture the shift-select laws replay over: three lines, so
/// every catalog motion has somewhere to go from the middle.
const SHIFT_FIXTURE: &str = "alpha beta\ngamma delta\nepsilon zeta\n";

#[allow(clippy::type_complexity)]
fn shift_replay(spec: &str) -> (Option<((usize, usize), (usize, usize))>, (usize, usize)) {
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

#[test]
fn strict_replay_aborts_on_an_unsupported_effect_naming_action_and_effect() {
    // Cmd-Q's `Effect::Quit` is classified Unsupported (live exits the
    // event loop; a replay would keep applying keys past it) — the strict
    // door must refuse it, naming the exact action AND effect.
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys("s-q").unwrap();
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
    .expect("strict replay aborts on an unsupported effect")
    .to_string();
    assert!(err.contains("`quit`"), "effect named: {err}");
    assert!(err.contains("Quit"), "action named: {err}");
    assert!(err.starts_with("strict replay:"), "{err}");
}

#[test]
fn strict_replay_records_intercepted_handoffs_without_aborting() {
    // C-c C-o on a link produces `Effect::FollowLink(url)` — an EXTERNAL
    // handoff the replay observes and records but never performs. Strict
    // must PASS it (that's the intercept contract, not a violation) and the
    // recorded intercept must carry the observed URL — the phase-5 trace seam.
    let mut buffer = Buffer::from_str("[a](https://awl.example/doc) tail");
    buffer.set_cursor(1); // inside the link
    let root = PathBuf::from("/tmp");
    let keys = keyspec::parse_keys("C-c C-o").unwrap();
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
    .expect("intercepted handoffs are legal under strict");
    assert_eq!(
        res.intercepts,
        vec![crate::replay::Intercept {
            effect: "follow_link",
            detail: "https://awl.example/doc".into()
        }]
    );
    assert!(
        res.warnings.is_empty(),
        "strict records silently, never warns"
    );
}

#[test]
fn permissive_replay_never_aborts_and_warns_on_both_non_applied_seams() {
    let mut buffer = Buffer::from_str("[a](https://awl.example/x) tail");
    buffer.set_cursor(1);
    let keys = keyspec::parse_keys("s-q C-c C-o s-Down").unwrap();
    let root = PathBuf::from("/tmp");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert_eq!(
        res.warnings.len(),
        2,
        "one warning per crossing: {:?}",
        res.warnings
    );
    assert!(
        res.warnings[0].contains("skipped unsupported effect `quit`"),
        "{}",
        res.warnings[0]
    );
    assert!(
        res.warnings[1].contains("intercepted `follow_link`")
            && res.warnings[1].contains("https://awl.example/x"),
        "{}",
        res.warnings[1]
    );
    assert_eq!(
        res.intercepts.len(),
        1,
        "the handoff is recorded permissively too"
    );
    let (line, col) = buffer.cursor_line_col();
    assert!(
        line > 0 || col > 0,
        "the key after Quit still applied (BufferEnd moved)"
    );
}

#[test]
fn a_fully_applied_replay_stays_warning_and_intercept_free() {
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys("a b c C-a C-e").unwrap();
    let root = PathBuf::from("/tmp");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert!(res.warnings.is_empty(), "{:?}", res.warnings);
    assert!(res.intercepts.is_empty());
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn hermetic_scenario_save_lands_in_the_sandbox_never_on_real_disk() {
    let dir = std::env::temp_dir().join(format!("awl-hermetic-save-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let input = dir.join("doc.md");
    std::fs::write(&input, "alpha\n").unwrap();
    {
        let _restore = crate::fs::FsGuard::capture();
        crate::scenario::install_hermetic_fs(Some(&input), None, Some(&dir));
        let mut buffer = load_buffer(&Some(input.clone()));
        assert_eq!(
            buffer.text(),
            "alpha\n",
            "the sandbox seeded the real input's bytes"
        );
        let keys = keyspec::parse_keys("X s-s").unwrap();
        let res = replay_keys_mode(
            crate::replay::Mode::Strict,
            &mut buffer,
            &keys,
            &[],
            &dir,
            None,
            &Config::empty(),
            None,
        )
        .expect("an edit + save crosses no unsupported seam");
        assert!(res.intercepts.is_empty());
        assert_eq!(
            crate::fs::active().read_to_string(&input).unwrap(),
            "Xalpha\n",
            "the replayed save landed in the sandbox"
        );
    }
    assert_eq!(
        std::fs::read_to_string(&input).unwrap(),
        "alpha\n",
        "the REAL file keeps every byte a hermetic scenario 'saved'"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn hermetic_scenario_witnesses_the_url_handoff_as_an_intercept() {
    let dir = std::env::temp_dir().join(format!("awl-hermetic-link-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let input = dir.join("linked.md");
    let body = "[a](https://awl.example/doc) tail\n";
    std::fs::write(&input, body).unwrap();
    {
        let _restore = crate::fs::FsGuard::capture();
        crate::scenario::install_hermetic_fs(Some(&input), None, Some(&dir));
        let mut buffer = load_buffer(&Some(input.clone()));
        let keys = keyspec::parse_keys("Right C-c C-o").unwrap();
        let res = replay_keys_mode(
            crate::replay::Mode::Strict,
            &mut buffer,
            &keys,
            &[],
            &dir,
            None,
            &Config::empty(),
            None,
        )
        .expect("an intercepted handoff is legal under strict");
        assert_eq!(
            res.intercepts,
            vec![crate::replay::Intercept {
                effect: "follow_link",
                detail: "https://awl.example/doc".into()
            }],
            "the handoff was observed and recorded, not performed"
        );
        assert_eq!(
            crate::fs::active().read_to_string(&input).unwrap(),
            body,
            "the sandbox copy is untouched (following a link edits nothing)"
        );
    }
    assert_eq!(
        std::fs::read_to_string(&input).unwrap(),
        body,
        "the real file too"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

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

#[test]
fn replay_keys_cmd_s_on_scratch_buffer_converts_it_into_a_document_under_the_active_root() {
    use crate::fs::{FileSystem, InMemoryFs};
    let mem = InMemoryFs::new();
    let _g = crate::fs::FsGuard::install(std::sync::Arc::new(mem.clone()));
    let mut buffer = Buffer::scratch();
    assert!(buffer.path().is_none() && !buffer.is_unnamed_fresh());
    let keys = keyspec::parse_keys("m e a d o w s-s").unwrap();
    let root = PathBuf::from("/tmp");
    let _res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert!(
        !buffer.is_unnamed_fresh(),
        "one-shot naming: an ordinary file immediately, not a lasting note identity"
    );
    let p = buffer.path().expect("a real path was derived");
    assert!(
        p.starts_with(&root),
        "landed under the replay's ACTIVE ROOT: {p:?}"
    );
    assert_eq!(mem.read_to_string(p).unwrap(), "meadow");
}

#[test]
fn replay_keys_cmd_s_on_an_already_pathed_buffer_is_a_plain_save() {
    // The contrast case: an already-pathed buffer's Cmd-S is a PLAIN save
    // (the pre-existing behavior) — never routed through the scratch
    // conversion, never re-homed.
    use crate::fs::{FileSystem, InMemoryFs};
    let mem = InMemoryFs::new().with_dir("/proj");
    let _g = crate::fs::FsGuard::install(std::sync::Arc::new(mem.clone()));
    let mut buffer = Buffer::scratch();
    buffer.set_path(PathBuf::from("/proj/a.md"));
    let keys = keyspec::parse_keys("h i s-s").unwrap();
    let root = PathBuf::from("/proj");
    let _res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert!(
        !buffer.is_unnamed_fresh(),
        "an already-pathed buffer never becomes a fresh document"
    );
    assert_eq!(buffer.path(), Some(std::path::Path::new("/proj/a.md")));
    assert_eq!(
        mem.read_to_string(std::path::Path::new("/proj/a.md"))
            .unwrap(),
        "hi"
    );
}

#[test]
fn replay_keys_drives_the_rename_minibuffer_prompt_and_sidecar_reflects_typing() {
    // Cmd-P → "rename" → Enter opens the Rename overlay pre-filled with the
    // current filename; typing MORE characters extends it live — all through
    // the shared core, so both the overlay STATE and its sidecar-facing
    // `foot_hint()` (the same seam the Keybindings capture prompt rides)
    // reflect the in-progress edit with zero live App involved.
    let mut buffer = Buffer::scratch();
    buffer.set_path(PathBuf::from("/proj/old.md"));
    let keys = keyspec::parse_keys("s-p r e n a m e RET 2").unwrap();
    let root = PathBuf::from("/proj");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    let ov = res
        .overlay
        .expect("Rename note… opens the minibuffer overlay");
    assert_eq!(ov.kind, crate::overlay::OverlayKind::Rename);
    assert_eq!(
        ov.accepts(),
        vec!["old.md2"],
        "typing extends the seeded name"
    );
    assert_eq!(
        ov.foot_hint(),
        "rename to: old.md2   Enter commit   Esc cancel",
        "the live prompt is sidecar-visible via the same foot_hint seam Keybindings uses"
    );
}

#[test]
fn replay_keys_rename_minibuffer_esc_cancels_with_no_overlay_left() {
    let mut buffer = Buffer::scratch();
    buffer.set_path(PathBuf::from("/proj/old.md"));
    let keys = keyspec::parse_keys("s-p r e n a m e RET x Esc").unwrap();
    let root = PathBuf::from("/proj");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert!(
        res.overlay.is_none(),
        "Esc closes the minibuffer outright, no breadcrumb pop"
    );
    assert_eq!(
        buffer.path(),
        Some(std::path::Path::new("/proj/old.md")),
        "no disk rename happened"
    );
}

#[test]
fn replay_keys_rename_minibuffer_does_not_open_on_a_pathless_buffer() {
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys("s-p r e n a m e RET").unwrap();
    let root = PathBuf::from("/proj");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert!(
        res.overlay.is_none(),
        "nothing to rename on a pathless buffer"
    );
}

#[test]
fn replay_keys_drives_the_keep_version_minibuffer_prompt_and_sidecar_reflects_typing() {
    // Cmd-P → "keep" → Enter opens the naming minibuffer (empty — a fresh
    // point has no old name); typing builds the optional name live — all
    // through the shared core, so both the overlay STATE and its
    // sidecar-facing `foot_hint()` (the same seam Rename/InsertLink ride)
    // reflect the in-progress edit with zero live App involved.
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys("s-p k e e p RET d r a f t").unwrap();
    let root = PathBuf::from("/proj");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    let ov = res
        .overlay
        .expect("Keep version… opens the naming minibuffer");
    assert_eq!(ov.kind, crate::overlay::OverlayKind::KeepName);
    assert_eq!(
        ov.accepts(),
        vec!["draft"],
        "typing builds the name from empty"
    );
    assert_eq!(
        ov.foot_hint(),
        "name this version: draft   Enter keep   Esc cancel",
        "the live prompt is sidecar-visible via the same foot_hint seam Rename uses"
    );
}

#[test]
fn replay_keys_keep_version_minibuffer_esc_cancels_with_no_overlay_left() {
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys("s-p k e e p RET x Esc").unwrap();
    let root = PathBuf::from("/proj");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert!(
        res.overlay.is_none(),
        "Esc closes the minibuffer outright, nothing kept"
    );
}

#[test]
fn replay_keys_keep_version_commit_closes_and_defers_the_store_write() {
    // Enter commits through the REAL keymap: the overlay closes and the
    // deferred Effect::KeepVersion { name } is the documented headless no-op
    // (the history determinism gate — a capture never touches the store), so
    // the buffer and fs stay untouched.
    use crate::fs::InMemoryFs;
    let _g = crate::fs::FsGuard::install(std::sync::Arc::new(InMemoryFs::new()));
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys("h i s-p k e e p RET d r a f t RET").unwrap();
    let root = PathBuf::from("/proj");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert!(res.overlay.is_none(), "commit closes the minibuffer");
    assert_eq!(buffer.text(), "hi", "the keep never edits the buffer");
}

// ── LINKS V2: Cmd-K stays --keys-drivable through the shared core ──

#[test]
fn replay_keys_cmd_k_wraps_a_selection_as_a_markdown_link() {
    // Type "hello", mark it with C-Space + move left across it, Cmd-K opens
    // the URL minibuffer pre-filled empty (WithText mode wrapping "hello"),
    // type a URL, RET commits — one atomic edit, fully sidecar-drivable.
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys(
        "h e l l o C-Space Left Left Left Left Left s-k h t t p s : / / x . t e s t RET",
    )
    .unwrap();
    let root = PathBuf::from("/proj");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert!(res.overlay.is_none(), "commit closes the minibuffer");
    assert_eq!(buffer.text(), "[hello](https://x.test)");
}

#[test]
fn replay_keys_cmd_k_prompt_is_sidecar_visible_while_typing() {
    let mut buffer = Buffer::scratch();
    let keys =
        keyspec::parse_keys("h e l l o C-Space Left Left Left Left Left s-k h t t p s : / /")
            .unwrap();
    let root = PathBuf::from("/proj");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    let ov = res.overlay.expect("Cmd-K opens the link minibuffer");
    assert_eq!(ov.kind, crate::overlay::OverlayKind::InsertLink);
    assert_eq!(ov.accepts(), vec!["https://"]);
    assert_eq!(
        ov.foot_hint(),
        "link to: https://   Enter commit   Esc cancel",
        "the live prompt is sidecar-visible via the same foot_hint seam Rename/Keybindings use"
    );
    assert_eq!(buffer.text(), "hello");
}

#[test]
fn replay_keys_cmd_k_esc_cancels_with_no_buffer_change() {
    let mut buffer = Buffer::scratch();
    let keys =
        keyspec::parse_keys("h e l l o C-Space Left Left Left Left Left s-k x x x Esc").unwrap();
    let root = PathBuf::from("/proj");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert!(res.overlay.is_none(), "Esc closes the minibuffer outright");
    assert_eq!(buffer.text(), "hello", "cancel never edits the buffer");
}

#[test]
fn replay_keys_cmd_k_no_selection_inserts_empty_markup_caret_between_brackets() {
    // No selection, no existing link under the caret: Cmd-K inserts empty
    // `[](url)` markup; committing an empty URL is still a harmless, one-shot
    // edit (never a silent cancel the user didn't ask for).
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys("h i s-k RET").unwrap();
    let root = PathBuf::from("/proj");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert!(res.overlay.is_none());
    assert_eq!(buffer.text(), "hi[]()");
}

#[test]
fn replay_keys_cmd_k_on_a_non_markdown_buffer_is_a_calm_no_op() {
    use std::path::PathBuf as PB;
    let mut buffer = Buffer::scratch();
    buffer.set_path(PB::from("/proj/main.rs"));
    buffer.insert_char('a');
    assert!(!buffer.is_markdown());
    let keys = keyspec::parse_keys("s-k").unwrap();
    let root = PathBuf::from("/proj");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert!(
        res.overlay.is_none(),
        "Cmd-K is a calm no-op on a non-markdown buffer"
    );
    assert_eq!(buffer.text(), "a");
}

#[test]
fn replay_keys_runs_palette_chain_into_overlay() {
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys("s-p g o t o RET").unwrap();
    let root = PathBuf::from("/tmp");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert_eq!(
        res.overlay.map(|o| o.kind),
        Some(crate::overlay::OverlayKind::Goto),
        "palette Enter on 'Go to file' chains into the Goto overlay",
    );
}

#[test]
fn replay_keys_drives_palette_guide_and_opens_the_guide_buffer() {
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys("s-p g u i d e RET").unwrap();
    let root = PathBuf::from("/tmp");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert!(
        res.overlay.is_none(),
        "the palette closed itself on accept, no overlay left open"
    );
    let expected = crate::guide::render(
        crate::convention::Convention::current(),
        crate::commands::Platform::current(),
    );
    assert_eq!(
        buffer.text(),
        expected,
        "the buffer now holds the token-rendered guide text"
    );
    assert!(
        !buffer.text().contains("{{key:"),
        "no raw chord token survives in the opened guide"
    );
    assert!(
        buffer.path().is_none(),
        "headless replay never writes/loads a real on-disk guide.md"
    );
}

#[test]
fn replay_keys_palette_filter_surfaces_the_marked_settings_row() {
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys("s-p k e y m a p").unwrap();
    let root = PathBuf::from("/tmp");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    let ov = res.overlay.expect("the palette is still open");
    assert_eq!(ov.kind, crate::overlay::OverlayKind::Command);
    assert!(
        ov.item_strings().iter().any(|s| s == "§ Keymap"),
        "the union corpus surfaces the marked settings row: {:?}",
        ov.item_strings()
    );
}

#[test]
fn replay_keys_palette_filters_to_a_settings_row_and_toggles_it() {
    // THE UNION ROUND: Cmd-P → "keymap" filters to the SETTINGS row "Keymap"
    // (the union palette's marked settings corpus, `§ Keymap`) → Enter signals
    // the SAME `Effect::SettingToggle{key:"keymap"}` the Settings menu's own
    // accept would, and CLOSES the palette (the palette's "activation closes
    // it" convention). Note the honest scope boundary: `Effect::SettingToggle`
    // is a documented headless no-op (see the `Effect` match above) — flipping
    // + persisting the live keymap flavor is the live App's job
    // (`App::toggle_keymap_flavor`, unit-tested there); this replay proves the
    // dispatch reaches the toggle EFFECT end-to-end through the real keymap +
    // fuzzy filter + accept seam, not that the flavor value itself flips in a
    // capture (which the architecture never claims for any settings toggle).
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys("s-p k e y m a p RET").unwrap();
    let root = PathBuf::from("/tmp");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert!(
        res.overlay.is_none(),
        "activating a settings row closes the palette"
    );
}

#[test]
fn replay_keys_palette_sub_picker_stamps_command_breadcrumb() {
    // Cmd-P → "theme" filters to "Switch theme…" → Enter runs OpenThemeMenu, which
    // the worklist re-dispatches into the Theme picker STAMPED return_to = Command
    // (the palette re-dispatch breadcrumb seam). Serialize on the theme lock: the
    // picker reads/reverts the process-global active theme.
    let _g = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys("s-p t h e m e RET").unwrap();
    let root = PathBuf::from("/tmp");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    let ov = res.overlay.expect("palette chained into the theme picker");
    assert_eq!(ov.kind, crate::overlay::OverlayKind::Theme);
    assert_eq!(
        ov.return_to,
        Some(crate::overlay::OverlayKind::Command),
        "a palette-opened sub-picker remembers its way back to the palette",
    );
    crate::theme::set_active(0);
}

#[test]
fn replay_keys_palette_theme_esc_pops_back_to_palette() {
    let _g = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys("s-p t h e m e RET Esc").unwrap();
    let root = PathBuf::from("/tmp");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    let ov = res
        .overlay
        .expect("Esc pops back to the palette, not the buffer");
    assert_eq!(
        ov.kind,
        crate::overlay::OverlayKind::Command,
        "back at the command palette"
    );
    assert_eq!(
        ov.return_to, None,
        "single-level: the palette carries no breadcrumb"
    );
    crate::theme::set_active(0);
}

#[test]
fn replay_keys_palette_theme_keep_closes_to_buffer_not_a_recent_menu() {
    let _g = crate::testlock::serial();
    let _world = crate::theme::WorldPin::snapshot();
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys("s-p t h e m e RET RET").unwrap();
    let root = PathBuf::from("/tmp");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert!(
        res.overlay.is_none(),
        "keeping a palette-launched theme lands in the buffer"
    );
    assert!(
        matches!(res.accept, Some((crate::overlay::OverlayKind::Theme, _))),
        "the theme keep still committed, got {:?}",
        res.accept
    );
    crate::theme::set_active(0);
}

/// THE BUG this round fixes, end-to-end through the REAL `--keys` replay
/// (the reported symptom's actual repro, not just the pure `apply_core`
/// unit — see `actions::tests::overlay_drive::
/// caret_picker_cancel_from_auto_restores_auto_not_a_pin` for that
/// purer-seam sibling). Riding AUTO on a PROPORTIONAL world (Gumtree ->
/// Morph), merely OPENING the Caret-style picker from the palette and
/// backing out with Esc (no pick made) must be a true no-op: a LATER
/// switch to a MONO world must still resolve Block, exactly as auto
/// always would. Before the fix, the Cancel silently pinned the caret at
/// Morph (auto's momentary resolution on Gumtree), so Potoroo (mono)
/// stayed wrongly Morph.
#[test]
fn replay_keys_caret_picker_cancel_from_auto_does_not_pin_it() {
    let _g = crate::testlock::serial();
    let _t = crate::testlock::serial();
    crate::caret::clear_override();
    crate::theme::set_active_by_name("Gumtree").unwrap();
    assert!(crate::caret::is_auto());
    assert_eq!(crate::caret::mode(), crate::caret::CaretMode::Morph);

    let mut buffer = Buffer::scratch();
    let keys =
        keyspec::parse_keys("s-p C a r e t Space s t y l e RET Esc Esc s-t P o t o r o o RET")
            .unwrap();
    let root = PathBuf::from("/tmp");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    assert!(
        res.overlay.is_none(),
        "the whole journey lands back in the buffer"
    );
    assert_eq!(
        crate::theme::active().name,
        "Potoroo",
        "the theme switch landed"
    );

    assert!(
        crate::caret::is_auto(),
        "cancelling the caret picker must not pin auto"
    );
    assert_eq!(
        crate::caret::mode(),
        crate::caret::CaretMode::Block,
        "auto correctly resolves Block on the now-active mono world"
    );

    crate::caret::clear_override();
    crate::theme::set_active(crate::theme::DEFAULT_THEME);
}

#[test]
fn replay_keys_goto_open_file_closes_all_no_overlay() {
    let _tg = crate::testlock::serial();
    let mut buffer = Buffer::scratch();
    let corpus = vec!["doc-fixture.md".to_string()];
    let root = PathBuf::from("/tmp");
    let keys = keyspec::parse_keys("s-o RET").unwrap();
    let res = replay_keys(
        &mut buffer,
        &keys,
        &corpus,
        &root,
        None,
        &Config::empty(),
        None,
    );
    assert!(
        res.overlay.is_none(),
        "opening a file closes the overlay to the buffer"
    );
    assert_eq!(
        res.accept,
        Some((
            crate::overlay::OverlayKind::Goto,
            "doc-fixture.md".to_string()
        )),
        "the file open still fired",
    );
}

#[test]
fn replay_keys_goto_hides_dotfiles_until_file_visibility_is_all() {
    let _g = crate::testlock::serial();
    let saved = crate::file_visibility::all_on();
    crate::file_visibility::set_all_on(false);
    let mut buffer = Buffer::scratch();
    let corpus = vec![
        ".gitignore".to_string(),
        ".env".to_string(),
        "doc-fixture.md".to_string(),
        "src/main.rs".to_string(),
    ];
    let root = PathBuf::from("/tmp");
    let keys = keyspec::parse_keys("s-o").unwrap();
    let res = replay_keys(
        &mut buffer,
        &keys,
        &corpus,
        &root,
        None,
        &Config::empty(),
        None,
    );
    let ov = res.overlay.expect("goto overlay open");
    assert_eq!(ov.kind, crate::overlay::OverlayKind::Goto);
    assert!(!crate::file_visibility::all_on());
    let shown = ov.item_strings();
    assert!(
        !shown.iter().any(|s| s == ".gitignore"),
        "dotfile hidden by default: {shown:?}"
    );
    assert!(
        shown.iter().any(|s| s == ".env"),
        ".env stays visible: {shown:?}"
    );
    assert!(shown.iter().any(|s| s == "doc-fixture.md"));
    crate::file_visibility::set_all_on(true);
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys("s-o").unwrap();
    let res = replay_keys(
        &mut buffer,
        &keys,
        &corpus,
        &root,
        None,
        &Config::empty(),
        None,
    );
    let ov = res.overlay.expect("goto overlay open under All");
    assert!(
        crate::file_visibility::all_on(),
        "File visibility All reveals dotfiles"
    );
    assert!(
        ov.item_strings().iter().any(|s| s == ".gitignore"),
        "dotfile shown under All: {:?}",
        ov.item_strings()
    );
    crate::file_visibility::set_all_on(saved);
}

#[test]
fn replay_keys_asset_cleaner_lists_only_the_orphans_from_the_scan() {
    use std::sync::Arc;
    let root = PathBuf::from("/proj");
    let mem = crate::fs::InMemoryFs::new()
        .with_file("/proj/doc.md", "text\n![a](assets/used.png)\n")
        .with_file("/proj/assets/used.png", "U")
        .with_file("/proj/assets/orphan.png", "OO");
    let corpus = vec![
        "assets/orphan.png".to_string(),
        "assets/used.png".to_string(),
        "doc.md".to_string(),
    ];
    crate::fs::with_fs(Arc::new(mem), || {
        let mut buffer = Buffer::scratch();
        let keys = keyspec::parse_keys("s-p c l e a n RET").unwrap();
        let res = replay_keys(
            &mut buffer,
            &keys,
            &corpus,
            &root,
            None,
            &Config::empty(),
            None,
        );
        let ov = res
            .overlay
            .expect("asset cleaner open after the palette chain");
        assert_eq!(ov.kind, crate::overlay::OverlayKind::Assets);
        assert_eq!(ov.item_strings(), vec!["orphan.png"]);
        assert_eq!(ov.item_bindings(), vec!["2 B · assets"]);
        assert_eq!(ov.kind.as_str(), "assets");
    });
}

#[test]
fn replay_keys_project_hides_dotfolders_marks_git_tag() {
    use std::sync::Arc;
    let _g = crate::testlock::serial();
    let saved = crate::file_visibility::all_on();
    crate::file_visibility::set_all_on(false);
    let ws = PathBuf::from("/ws");
    let mem = crate::fs::InMemoryFs::new()
        .with_dir("/ws/.claude")
        .with_dir("/ws/.git") // junk-filtered before the overlay ever sees it
        .with_dir("/ws/plain")
        .with_dir("/ws/repo")
        .with_dir("/ws/repo/.git"); // marks `repo` a git repo
    crate::fs::with_fs(Arc::new(mem), || {
        let mut buffer = Buffer::scratch();
        let keys = keyspec::parse_keys("s-S-p").unwrap();
        let res = replay_keys(
            &mut buffer,
            &keys,
            &[],
            &ws,
            Some(ws.as_path()),
            &Config::empty(),
            None,
        );
        let ov = res.overlay.expect("switch-project overlay open");
        assert_eq!(ov.kind, crate::overlay::OverlayKind::Project);
        assert!(!crate::file_visibility::all_on());
        let shown = ov.item_strings();
        assert!(
            shown.iter().any(|s| s == "."),
            "'.' accept row kept: {shown:?}"
        );
        assert!(
            !shown.iter().any(|s| s.starts_with(".claude")),
            "dotfolder hidden: {shown:?}"
        );
        assert!(
            !shown.iter().any(|s| s.starts_with(".git")),
            "junk .git hidden: {shown:?}"
        );
        assert!(
            shown.iter().any(|s| s.starts_with("plain")),
            "plain shown: {shown:?}"
        );
        assert!(
            shown.iter().any(|s| s.starts_with("repo")),
            "repo shown: {shown:?}"
        );
        assert!(
            shown.iter().all(|s| !s.contains('•')),
            "no name bullet: {shown:?}"
        );
        let tags = ov.item_git_tags();
        let ipos = |name: &str| shown.iter().position(|s| s.starts_with(name)).unwrap();
        assert_eq!(tags[ipos("repo")], "git", "repo is git-tagged");
        assert_eq!(tags[ipos("plain")], "", "plain folder has no tag");

        // File visibility All reveals the overlay-hidden dotfolder (`.claude`);
        // junk `.git` stays hidden (it never reaches the overlay corpus).
        crate::file_visibility::set_all_on(true);
        let mut buffer = Buffer::scratch();
        let keys = keyspec::parse_keys("s-S-p").unwrap();
        let res = replay_keys(
            &mut buffer,
            &keys,
            &[],
            &ws,
            Some(ws.as_path()),
            &Config::empty(),
            None,
        );
        let ov = res.overlay.expect("project overlay open under All");
        assert!(
            crate::file_visibility::all_on(),
            "File visibility All reveals dotfolders"
        );
        let revealed = ov.item_strings();
        assert!(
            revealed.iter().any(|s| s.starts_with(".claude")),
            "revealed: {revealed:?}"
        );
        assert!(
            revealed.iter().any(|s| s == "."),
            "'.' still present after reveal"
        );
    });
    crate::file_visibility::set_all_on(saved);
}

#[test]
fn replay_keys_drives_rebind_menu_capture() {
    // The GAME-STYLE REBIND MENU, driven entirely through the headless replay:
    // Cmd-P → "keyb" → Enter opens the Keybindings menu, "undo" filters to Undo,
    // Enter starts a capture (ChooseMode), Enter begins recording (KEY), and a
    // plain 'q' is captured → committed (the menu's NOTICE reflects the binding).
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys("s-p k e y b RET u n d o RET RET q").unwrap();
    let root = PathBuf::from("/tmp");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    let ov = res
        .overlay
        .expect("the rebind menu stays open after a commit");
    assert_eq!(ov.kind, crate::overlay::OverlayKind::Keybindings);
    assert!(
        ov.capture.is_none(),
        "capture closed after committing the key"
    );
    assert_eq!(
        ov.notice, "bound undo -> q",
        "notice reflects the captured binding"
    );
}

#[test]
fn replay_keys_rebind_menu_recording_state_visible() {
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys("s-p k e y b RET s a v e RET Down RET").unwrap();
    let root = PathBuf::from("/tmp");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
    let ov = res.overlay.expect("menu open");
    let cap = ov.capture.expect("a capture is in progress");
    assert_eq!(cap.cmd_name, "Save");
    assert_eq!(cap.stage, crate::overlay::CaptureStage::Recording);
    assert!(
        cap.chord_mode,
        "Down selected the CHORD row before recording"
    );
    assert!(cap.captured.is_empty(), "no combo pressed yet");
}

#[test]
fn replay_keys_settings_cjk_picker_round_trips_headlessly() {
    let _g = crate::testlock::serial();
    crate::frontmatter::set_cjk_priority(&crate::frontmatter::DEFAULT_CJK_PRIORITY);
    let mut buffer = Buffer::scratch();
    let keys =
        keyspec::parse_keys("s-p s e t t i n g s RET a m b i g u o u s RET Down Down Down RET")
            .unwrap();
    let root = PathBuf::from("/tmp");
    let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);

    assert_eq!(
        crate::frontmatter::cjk_priority(),
        vec![
            crate::frontmatter::Lang::Ko,
            crate::frontmatter::Lang::Ja,
            crate::frontmatter::Lang::ZhHans,
            crate::frontmatter::Lang::ZhHant,
        ],
    );

    let ov = res
        .overlay
        .expect("popped back to the Settings menu, not closed");
    assert_eq!(
        ov.kind,
        crate::overlay::OverlayKind::Settings,
        "back at Settings"
    );
    assert_eq!(ov.return_to, None, "single-level: no N-deep stack");
    let row_idx = crate::settings::SETTINGS
        .iter()
        .position(|r| r.name == "Ambiguous CJK reads as")
        .unwrap();
    assert_eq!(
        ov.rows[row_idx].secondary, "Korean",
        "the re-summoned Settings menu's value cell is FRESH, in writer-words"
    );

    crate::frontmatter::set_cjk_priority(&crate::frontmatter::DEFAULT_CJK_PRIORITY);
}

#[test]
fn replay_keys_page_reset_restores_default_measure() {
    let _pg = crate::testlock::serial();
    crate::page::set_measure(40);
    let mut buffer = Buffer::scratch();
    let root = PathBuf::from("/tmp");
    let keys = keyspec::parse_keys("C-j").unwrap();
    let mut km = crate::keymap::KeymapState::with_overrides_and_convention(
        &[("reset_page_width".into(), vec!["C-j".into()])],
        crate::convention::Convention::Mac,
    );
    let _ = super::replay_keys_mode(
        crate::replay::Mode::Permissive,
        &mut buffer,
        &keys,
        &[],
        &root,
        None,
        &Config::empty(),
        None,
        &mut km,
    )
    .unwrap();
    assert_eq!(
        crate::page::measure(),
        crate::page::DEFAULT_MEASURE,
        "PageReset snaps the measure back to the built-in default"
    );
    crate::page::set_measure(crate::page::DEFAULT_MEASURE); // leave as found
}

#[test]
fn replay_keys_page_reset_restores_the_code_default_for_a_code_buffer() {
    // The prose/code page-width split: PageReset on a CODE buffer (a `.rs`
    // path) must snap to DEFAULT_MEASURE_CODE (100), never the prose default
    // (70) — `Action::PageReset` resolves via `ctx.buffer.page_class()` on
    // the shared `apply_core` seam, so this is byte-identical to the live
    // App's own reset.
    let _pg = crate::testlock::serial();
    crate::page::set_measure(40);
    let mut buffer = Buffer::from_str("fn main() {}\n");
    buffer.set_path(PathBuf::from("/tmp/main.rs"));
    let root = PathBuf::from("/tmp");
    let keys = keyspec::parse_keys("C-j").unwrap();
    let mut km = crate::keymap::KeymapState::with_overrides_and_convention(
        &[("reset_page_width".into(), vec!["C-j".into()])],
        crate::convention::Convention::Mac,
    );
    let _ = super::replay_keys_mode(
        crate::replay::Mode::Permissive,
        &mut buffer,
        &keys,
        &[],
        &root,
        None,
        &Config::empty(),
        None,
        &mut km,
    )
    .unwrap();
    assert_eq!(
        crate::page::measure(),
        crate::page::DEFAULT_MEASURE_CODE,
        "PageReset on a code buffer snaps to the CODE default, not the prose one"
    );
    crate::page::set_measure(crate::page::DEFAULT_MEASURE); // leave as found
}

#[test]
fn replay_keys_goto_switch_reapplies_measure_per_buffer_kind() {
    let _fs = crate::testlock::serial();
    let _pg = crate::testlock::serial();
    let measure0 = crate::page::measure();
    let dir = std::env::temp_dir().join(format!("awl-mb-measure-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("a.md"), "# hello\n").unwrap();
    std::fs::write(dir.join("b.rs"), "fn main() {}\n").unwrap();
    let cfg = Config {
        page_width_prose: Some(55),
        page_width_code: Some(120),
        ..Config::empty()
    };
    let mut buffer = Buffer::scratch();
    let corpus = vec!["a.md".to_string(), "b.rs".to_string()];
    crate::page::set_measure(1); // deliberately wrong, so the switch below can't coincide

    let keys_to_b = keyspec::parse_keys("s-o b . r s RET").unwrap();
    let _ = replay_keys(&mut buffer, &keys_to_b, &corpus, &dir, None, &cfg, None);
    assert_eq!(
        crate::page::measure(),
        120,
        "b.rs (code) picks up the configured code measure"
    );

    let keys_to_a = keyspec::parse_keys("s-o a . m d RET").unwrap();
    let _ = replay_keys(&mut buffer, &keys_to_a, &corpus, &dir, None, &cfg, None);
    assert_eq!(
        crate::page::measure(),
        55,
        "back to a.md (prose) picks up the configured prose measure"
    );

    crate::page::set_measure(measure0);
    let _ = std::fs::remove_dir_all(&dir);
}

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
    let dir = std::env::temp_dir().join(format!("awl-goto-swap-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
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
    let _ = std::fs::remove_dir_all(&dir);
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
    let dir = std::env::temp_dir().join(format!("awl-mb-replay-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
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
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn replay_keys_reopening_the_active_file_is_a_noop() {
    // Guards the same "already active" short-circuit the live `App::load_path`
    // takes: Goto-ing the file that's ALREADY active must not disturb its edit.
    let _fs = crate::testlock::serial();
    let dir = std::env::temp_dir().join(format!("awl-mb-replay-noop-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
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
    let _ = std::fs::remove_dir_all(&dir);
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
    let dir = std::env::temp_dir().join(format!("awl-mb-relid-{}", std::process::id()));
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
    let _ = std::fs::remove_dir_all(&dir);
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
    let dir = std::env::temp_dir().join(format!("awl-mb-newnote-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
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
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn headless_replay_never_arms_autosave_or_stashes_scratch() {
    use std::sync::Arc;
    crate::fs::with_fs(Arc::new(crate::fs::InMemoryFs::new()), || {
        let mut buffer = Buffer::scratch();
        let keys = keyspec::parse_keys("h i RET t h e r e").unwrap();
        let root = PathBuf::from("/tmp");
        let _ = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
        assert_eq!(buffer.text(), "hi\nthere", "the edits themselves landed");
        assert!(
            crate::fs::active()
                .read(&crate::fs::scratch_stash_path())
                .is_err(),
            "no scratch stash is ever written headlessly"
        );
        let hist = crate::fs::data_root().join("history");
        assert!(
            crate::fs::active()
                .read_dir(&hist)
                .map(|v| v.is_empty())
                .unwrap_or(true),
            "no history log is ever written headlessly"
        );
    });
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn headless_screenshot_never_installs_the_crash_hook() {
    // The CRASH-VISIBILITY CAPTURE GATE as the same tripwire shape:
    // `crashlog::install_hook` is called from exactly ONE door,
    // `crate::app::run`'s native branch — never reached by any headless
    // `--screenshot`/`--keys`/`--bench-*` mode, every one of which drives a
    // bare `Buffer` straight through `replay_keys` (this file's own shared
    // seam) and never constructs a live `App` or calls `crate::app::run`.
    // The witness global stays false across a whole replay.
    use std::sync::Arc;
    crate::fs::with_fs(Arc::new(crate::fs::InMemoryFs::new()), || {
        let mut buffer = Buffer::scratch();
        let keys = keyspec::parse_keys("h i").unwrap();
        let root = PathBuf::from("/tmp");
        let _ = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
        assert!(
            !crate::crashlog::hook_installed_for_test(),
            "a headless replay must never install the panic hook"
        );
    });
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn headless_replay_never_touches_the_session_file() {
    // The SESSION RESTORE determinism law as the same tripwire shape:
    // `session_flush`/`apply_session_restore` live only on the live App
    // (`app/session.rs`), which `replay_keys` never constructs — so a
    // `--keys` replay against a bare `Buffer` must never create
    // `session.toml`, even after edits + a save.
    use std::sync::Arc;
    crate::fs::with_fs(Arc::new(crate::fs::InMemoryFs::new()), || {
        let mut buffer = Buffer::scratch();
        let keys = keyspec::parse_keys("h i s-s").unwrap();
        let root = PathBuf::from("/tmp");
        let _ = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
        assert!(
            crate::fs::active()
                .read(&crate::session::session_path())
                .is_err(),
            "no session file is ever written headlessly"
        );
    });
}

#[test]
fn headless_replay_never_touches_the_recent_files_store() {
    // The RECENTLY-OPENED FILES determinism law as the same tripwire shape:
    // `push_recent_file` (and the `recent_files` load) live only on the live
    // `App` (`app/files/`), which `replay_keys` never constructs — so a
    // `--keys` replay against a bare `Buffer` must never create
    // `recent-files.toml`, even after edits + a save.
    use std::sync::Arc;
    crate::fs::with_fs(Arc::new(crate::fs::InMemoryFs::new()), || {
        let mut buffer = Buffer::scratch();
        let keys = keyspec::parse_keys("h i s-s").unwrap();
        let root = PathBuf::from("/tmp");
        let _ = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
        assert!(
            crate::fs::active()
                .read(&crate::recent_files::recent_files_path())
                .is_err(),
            "no recent-files store is ever written headlessly"
        );
    });
}

#[test]
fn headless_replay_never_touches_reduced_motion() {
    // ACCESSIBILITY TIER 1's determinism law: `motion::apply_at_startup` (the
    // ONLY function that ever consults OS/browser detection OR reads
    // `Config::reduce_motion`) lives exclusively on the live App's own
    // startup path (`App::new`), which `replay_keys` never constructs — so a
    // `--keys` replay must leave `motion::reduced()` at its default `false`
    // EVEN WHEN the passed config explicitly names `reduce_motion: true`
    // (proving the config value itself is never read here, not merely that
    // the OS call is skipped).
    let _g = crate::testlock::serial();
    let saved = crate::motion::reduced();
    crate::motion::set_reduced(false);
    let mut buffer = Buffer::scratch();
    let keys = keyspec::parse_keys("h i s-s").unwrap();
    let root = PathBuf::from("/tmp");
    let cfg = Config {
        reduce_motion: Some(true),
        ..Config::empty()
    };
    let _ = replay_keys(&mut buffer, &keys, &[], &root, None, &cfg, None);
    assert!(
        !crate::motion::reduced(),
        "a headless --keys replay must never apply the config's reduce_motion pref"
    );
    crate::motion::set_reduced(saved);
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn headless_replay_never_touches_the_stats_file() {
    // The LIFETIME STATS determinism law as the same tripwire shape: every
    // stats tracking hook + `stats_flush` lives only on the live `App`
    // (`app/stats.rs`), which `replay_keys` never constructs — so a `--keys`
    // replay against a bare `Buffer` must never create `stats.toml`, even
    // after edits + a save. The SILENT USAGE LEDGER (`command_usage`) rides
    // the SAME `stats.toml`, recorded only in `App::apply` (never the headless
    // core), so this one tripwire covers it too — no capture can attribute a
    // command dispatch to any door.
    use std::sync::Arc;
    crate::fs::with_fs(Arc::new(crate::fs::InMemoryFs::new()), || {
        let mut buffer = Buffer::scratch();
        let keys = keyspec::parse_keys("h i s-s").unwrap();
        let root = PathBuf::from("/tmp");
        let _ = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
        assert!(
            crate::fs::active()
                .read(&crate::stats::stats_path())
                .is_err(),
            "no stats file is ever written headlessly"
        );
    });
}

#[test]
fn headless_load_buffer_never_writes_back_frontmatter() {
    // The i18n round's DETERMINISM LAW as a tripwire (mirrors the autosave
    // one above): `load_buffer` is the headless capture's ONLY file-load
    // door, and the write-back-once tagger lives exclusively on the live
    // `App` (`App::new` / `App::load_path`), never here — so an untagged
    // Japanese fixture loads byte-identically, with NO frontmatter block
    // ever appearing headlessly.
    use std::sync::Arc;
    crate::fs::with_fs(Arc::new(crate::fs::InMemoryFs::new()), || {
        let p = PathBuf::from("/notes/japanese.md");
        let original = "これは日本語の文章です。\n";
        crate::fs::active().write(&p, original.as_bytes()).unwrap();
        let buffer = load_buffer(&Some(p));
        assert_eq!(
            buffer.text(),
            original,
            "no frontmatter ever appears headlessly"
        );
    });
}

/// ITEM 77 FOLLOW-UP — DOOR 3: `load_buffer` (the `--screenshot` /
/// `--screenshot-motion[-v|-d]` / timeline / held / frames / storyboard
/// headless capture door — the LAUNCH-ARGUMENT analog of `App::new`,
/// `src/app/tests/openable.rs`'s DOOR 1) asks the SAME
/// `crate::openable::classify` capability owner before it ever reaches
/// `Buffer::from_file`.
///
/// NON-VACUOUS — the exact real-world repro (`cargo run --
/// --screenshot out.png --keys "s-s" logo.png` truncating a real PNG to
/// zero bytes) reproduced headlessly: revert `load_buffer`'s gating back
/// to `Some(p) => Buffer::from_file(p)` (leaving `crate::openable` itself
/// untouched) and this test FAILS at the FIRST assertion —
/// `buffer.path()` comes back `Some("/proj/logo.png")` instead of `None`,
/// because `Buffer::from_file`'s UTF-8-decode-error fallback returns an
/// EMPTY buffer STILL BOUND to the binary path (see `crate::openable`'s
/// module doc) — and the end-to-end save assertion at the bottom fails
/// too: the replayed `s-s` truncates `logo.png` to `b""`.
#[test]
fn headless_capture_door_refuses_binary_and_never_lets_save_truncate_it() {
    use crate::fs::FileSystem;
    use std::sync::Arc;

    let png = PathBuf::from("/proj/logo.png");
    let png_bytes: &[u8] = &[0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00];
    let xyzzy = PathBuf::from("/proj/notes.xyzzy");

    let mem = crate::fs::InMemoryFs::new();
    mem.write(&png, png_bytes).unwrap();
    mem.write(&xyzzy, b"plain prose, odd extension\n").unwrap();

    crate::fs::with_fs(Arc::new(mem), || {
        let buffer = load_buffer(&Some(png.clone()));
        assert_eq!(
            buffer.path(),
            None,
            "a refused binary file never produces a buffer bound to its path"
        );
        assert_eq!(
            buffer.text(),
            "",
            "the refusal degrades to an ordinary empty scratch buffer"
        );

        let text_buffer = load_buffer(&Some(xyzzy.clone()));
        assert_eq!(
            text_buffer.path(),
            Some(xyzzy.as_path()),
            "a supported unusual-extension text file still opens headlessly"
        );
        assert_eq!(text_buffer.text(), "plain prose, odd extension\n");

        let mut buffer = load_buffer(&Some(png.clone()));
        let keys = keyspec::parse_keys("s-s").unwrap();
        let root = PathBuf::from("/proj");
        let _ = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
        let after = crate::fs::active().read(&png).unwrap();
        assert_eq!(
            after.as_slice(),
            png_bytes,
            "a replayed save can never truncate a file the capture refused to open"
        );
    });
}

fn with_seeded_history(body: impl FnOnce(PathBuf)) {
    use std::sync::Arc;
    let p = PathBuf::from("/notes/draft.md");
    let mem = crate::fs::InMemoryFs::new().with_file(&p, "v2\n");
    crate::fs::with_fs(Arc::new(mem), || {
        crate::history::record(&p, "v1\n", &Config::empty());
        crate::history::record(&p, "v2\n", &Config::empty());
        body(p.clone());
    });
}

#[test]
fn history_preview_for_resolves_selected_row() {
    // DIFF-AS-PREVIEW: the capture-side preview resolver — the still-open
    // History overlay's highlighted row resolves to (id, TRANSCRIPT, counts):
    // the writer's diff of the current buffer vs that version, exactly what
    // the live App renders (the shared `history::diff_preview` owner). The
    // buffer here is "v2\n", so row 0 (v2, identical) is a titled folds-only
    // transcript with NO change marks; row 1 (v1, older) carries them.
    with_seeded_history(|p| {
        let buffer = Buffer::from_file(&p);
        let rows = crate::history::timeline_rows(&p, &buffer.text(), crate::history::now_millis());
        assert_eq!(rows.len(), 2, "two seeded versions");
        let mut ov = crate::overlay::OverlayState::new_history(rows, None, None);
        let (id, transcript, _counts) =
            history_preview_for(&ov, &buffer).expect("the newest row resolves");
        assert!(
            transcript.starts_with("# Comparing with "),
            "a titled diff transcript: {transcript}"
        );
        assert!(
            !transcript.contains("~~") && !transcript.contains("=="),
            "row 0 is identical to the buffer → no change marks: {transcript}"
        );
        assert_eq!(Some(id.as_str()), ov.selected_history_id());
        ov.move_sel(1);
        let (_, older, _) = history_preview_for(&ov, &buffer).expect("row 1 resolves");
        assert!(
            older.contains("~~") || older.contains("=="),
            "the highlighted row's diff carries change marks: {older}"
        );
        // A non-history overlay never previews.
        let goto = crate::overlay::OverlayState::new(
            crate::overlay::OverlayKind::Goto,
            vec!["a.md".into()],
            Vec::new(),
            Vec::new(),
        );
        assert!(history_preview_for(&goto, &buffer).is_none());
    });
}

#[test]
fn replay_history_esc_leaves_buffer_text_exact() {
    // The Esc-reverts-exactly proof at the replay seam: summon the timeline
    // (Cmd-S-h), arrow to the older version (C-n), Esc — the overlay is gone,
    // NOTHING was accepted, and the buffer text is byte-for-byte what it was
    // (the preview is a ViewState-level derivation; the buffer never moved).
    with_seeded_history(|p| {
        let mut buffer = Buffer::from_file(&p);
        let before = buffer.text();
        let keys = keyspec::parse_keys("Cmd-S-h C-n Esc").unwrap();
        let root = PathBuf::from("/notes");
        let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
        assert!(res.overlay.is_none(), "Esc closed the timeline");
        assert!(res.accept.is_none(), "nothing was accepted");
        assert_eq!(buffer.text(), before, "Esc leaves the buffer text exact");
    });
}

#[test]
fn replay_history_enter_restores_undoably() {
    with_seeded_history(|p| {
        let mut buffer = Buffer::from_file(&p);
        let keys = keyspec::parse_keys("Cmd-S-h C-n RET").unwrap();
        let root = PathBuf::from("/notes");
        let res = replay_keys(&mut buffer, &keys, &[], &root, None, &Config::empty(), None);
        let (kind, id) = res.accept.expect("Enter accepts the highlighted version");
        assert_eq!(kind, crate::overlay::OverlayKind::History);
        // The capture arm's restore (same shared source_path + load + set_text).
        let path = crate::history::source_path(buffer.path(), buffer.is_unnamed_fresh())
            .expect("a pathed buffer keys under itself");
        let content = crate::history::load(&path, &id).expect("the id round-trips");
        buffer.set_text(&content);
        assert_eq!(buffer.text(), "v1\n", "the older version restored");
        // UNDO through the real keymap: one sealed edit, back to now.
        let undo = keyspec::parse_keys("s-z").unwrap();
        replay_keys(&mut buffer, &undo, &[], &root, None, &Config::empty(), None);
        assert_eq!(buffer.text(), "v2\n", "the restore is one undoable edit");
    });
}

#[test]
fn resolve_root_explicit_flag_wins_over_file() {
    let flag = PathBuf::from("/flag/root");
    let file = PathBuf::from("/some/file.txt");
    assert_eq!(resolve_root(&Some(flag.clone()), &Some(file)), flag);
}

#[test]
fn resolve_root_file_argument_resolves_from_its_own_directory() {
    let _tg = crate::testlock::serial();
    let dir = std::env::temp_dir().join(format!("awl-resolve-root-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("note.txt");
    std::fs::write(&file, "hi").unwrap();
    assert_eq!(resolve_root(&None, &Some(file)), dir);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn resolve_root_bare_falls_to_cwd() {
    // `resolve_root` alone (the explicit-only half) never consults a
    // remembered folder or a default — that's `resolve_launch_context`'s
    // job. Its own bare fallback stays cwd, unchanged.
    //
    // TWO reads of the process-CWD global (ours + `resolve_root`'s own), so
    // the guard is what makes them the SAME cwd — a `CwdGuard` landing
    // between them would otherwise compare two different directories
    // (queue item 101).
    let _tg = crate::testlock::serial();
    let cwd = crate::fs::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    assert_eq!(resolve_root(&None, &None), cwd);
}

#[test]
fn resolve_launch_context_explicit_root_wins_over_remembered_and_file() {
    let flag = PathBuf::from("/flag/root");
    let remembered = PathBuf::from("/remembered/root");
    let default_folder = PathBuf::from("/home/me/notes");
    let file = PathBuf::from("/some/file.txt");
    assert_eq!(
        resolve_launch_context(
            &Some(flag.clone()),
            &Some(file),
            Some(&remembered),
            &default_folder
        ),
        flag
    );
}

#[test]
fn resolve_launch_context_file_argument_wins_over_remembered() {
    let _tg = crate::testlock::serial();
    let remembered = PathBuf::from("/remembered/root");
    let default_folder = PathBuf::from("/home/me/notes");
    let dir = std::env::temp_dir().join(format!("awl-launch-ctx-file-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("note.txt");
    std::fs::write(&file, "hi").unwrap();
    assert_eq!(
        resolve_launch_context(&None, &Some(file), Some(&remembered), &default_folder),
        dir
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn resolve_launch_context_dir_argument_awl_dot_is_explicit_not_remembered() {
    // `awl .` — a DIR argument — is door 1 (explicit), the crisp
    // bare-vs-dot distinction: it must win over whatever is remembered,
    // exactly like a file argument does.
    //
    // THE VICTIM OF QUEUE ITEM 101: `resolve_root` decides "is this
    // argument a directory?" through `fs::active().is_dir(f)`. Without
    // this guard the probe could land on a sibling test's `InMemoryFs`,
    // which knows nothing of this real temp dir — `is_dir` came back
    // false, the dir argument decayed to its PARENT (`/tmp`), and the
    // assertion below failed under parallel load.
    let _tg = crate::testlock::serial();
    let dir = std::env::temp_dir().join(format!("awl-launch-ctx-dot-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let remembered = PathBuf::from("/remembered/root");
    let default_folder = PathBuf::from("/home/me/notes");
    assert_eq!(
        resolve_launch_context(
            &None,
            &Some(dir.clone()),
            Some(&remembered),
            &default_folder
        ),
        dir
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn resolve_launch_context_bare_launch_restores_remembered() {
    let remembered = PathBuf::from("/home/me/work/repo-a");
    let default_folder = PathBuf::from("/home/me/notes");
    assert_eq!(
        resolve_launch_context(&None, &None, Some(&remembered), &default_folder),
        remembered
    );
}

#[test]
fn resolve_launch_context_first_run_falls_to_default_folder() {
    // Law point 3: bare launch, NOTHING remembered (a fresh install, or
    // session_restore off) — opens the configured default folder, never
    // cwd (the behavior item 76 replaces).
    let default_folder = PathBuf::from("/home/me/notes");
    assert_eq!(
        resolve_launch_context(&None, &None, None, &default_folder),
        default_folder
    );
    let _tg = crate::testlock::serial();
    let cwd = crate::fs::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    assert_ne!(default_folder, cwd);
}

#[test]
fn capture_mode_bare_invocation_never_restores_a_remembered_folder() {
    // The CAPTURE-GATE LAW: headless capture is structurally free of
    // session state (item 76 folded the old sticky `config.project_root`
    // into the live-App-only session) — a bare `--screenshot` (no file,
    // no --root) always falls to cwd via the explicit-only `resolve_root`,
    // never a remembered/default folder, regardless of what the config
    // carries. Reads the REAL disk (Project::resolve / build_index walk
    // it) -> hold the fs TEST_LOCK like the other real-fs test in this
    // module.
    let _fs = crate::testlock::serial();
    let dir = std::env::temp_dir().join(format!("awl-capture-bare-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let _cwd_guard = crate::fs::CwdGuard::enter(&dir);
    let cwd = crate::fs::current_dir().unwrap();
    let config = Config {
        default_folder: Some(PathBuf::from("/should/never/be/read")),
        ..Config::empty()
    };
    let out = dir.join("cap.png");
    let default_folder = dir.join("notes");
    let result = capture_screenshot(
        out.clone(),
        None, // no file argument: a bare capture
        CaptureOpts::default(),
        Vec::new(),
        crate::keymap::KeymapState::new(),
        None, // no explicit --root
        None,
        default_folder,
        config,
        false, // permissive (the legacy default)
    );
    result.expect("capture succeeds");
    let json = std::fs::read_to_string(out.with_extension("json")).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(
        v["project"]["root"].as_str().unwrap(),
        cwd.to_string_lossy(),
        "sidecar project.root reflects cwd, never the config default_folder"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn capture_scenario_search_replace_replay_lands_in_the_sidecar_search_block() {
    // SIDECAR EVIDENCE for the shared search/replace routing: one `--keys`
    // spec drives open + query typing + replace-field reveal + replacement
    // typing + replace-one through the REAL `capture_screenshot` seam, and
    // every operation's outcome is assertable from the sidecar `search`
    // block + `text` — the round's done-criteria witness. Real disk +
    // capture -> hold the fs TEST_LOCK like the sticky-root test above.
    let _fs = crate::testlock::serial();
    let dir = std::env::temp_dir().join(format!("awl-search-replay-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let fixture = dir.join("doc.txt");
    std::fs::write(&fixture, "line one\nline two\nline three\n").unwrap();
    let out = dir.join("cap.png");
    let keys = keyspec::parse_keys("C-s l i n e Tab r o w Enter").unwrap();
    capture_screenshot(
        out.clone(),
        Some(fixture),
        CaptureOpts::default(),
        keys,
        crate::keymap::KeymapState::new_with_convention(crate::convention::Convention::Mac),
        Some(dir.clone()),
        None,
        dir.join("notes"),
        Config::empty(),
        true, // strict: the whole spec must ride real seams
    )
    .expect("capture succeeds");
    let json = std::fs::read_to_string(out.with_extension("json")).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    let s = &v["search"];
    assert_eq!(s["query"].as_str().unwrap(), "line", "typed query");
    assert!(s["active"].as_bool().unwrap());
    assert!(
        s["replace_active"].as_bool().unwrap(),
        "Tab revealed the row"
    );
    assert_eq!(
        s["replacement"].as_str().unwrap(),
        "row",
        "typed replacement"
    );
    assert!(
        s["editing_replacement"].as_bool().unwrap(),
        "focus is in the field"
    );
    assert_eq!(
        s["hit_count"].as_u64().unwrap(),
        2,
        "one of three matches was replaced"
    );
    assert!(
        v["text"].as_str().unwrap().starts_with("row one\nline two"),
        "replace-one swapped exactly the first match"
    );
    assert_eq!(
        v["cursor"]["line"].as_u64().unwrap(),
        1,
        "caret advanced to the next match"
    );
    assert_eq!(v["cursor"]["col"].as_u64().unwrap(), 0);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn capture_sidecar_traces_permissive_replay_skips_and_strict_writes_nothing() {
    // The sidecar, not stderr, is the capture verifier's state oracle. Drive a
    // real Move accept through the same screenshot door users invoke: its
    // settled overlay otherwise looks exactly like a successful live move.
    let _fs = crate::testlock::serial();
    let dir = std::env::temp_dir().join(format!("awl-replay-skips-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("archive")).unwrap();
    let fixture = dir.join("note.md");
    std::fs::write(&fixture, "note\n").unwrap();
    if capture::build_oracle(&Buffer::from_file(&fixture), &CaptureOpts::default()).is_none() {
        eprintln!("skipping replay-skip sidecar capture: no wgpu adapter");
        let _ = std::fs::remove_dir_all(&dir);
        return;
    }
    let keys = keyspec::parse_keys("s-p m o v e Enter Enter").unwrap();
    let keymap =
        || crate::keymap::KeymapState::new_with_convention(crate::convention::Convention::Mac);

    let permissive = dir.join("permissive.png");
    capture_screenshot(
        permissive.clone(),
        Some(fixture.clone()),
        CaptureOpts::default(),
        keys.clone(),
        keymap(),
        Some(dir.clone()),
        None,
        dir.join("notes"),
        Config::empty(),
        false,
    )
    .expect("permissive replay captures past a live-only move");
    let sidecar: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(permissive.with_extension("json")).unwrap())
            .unwrap();
    assert_eq!(
        sidecar["replay_skips"],
        serde_json::json!([{ "effect": "overlay_accept", "action": "Newline" }]),
        "the settled sidecar names the skipped move and its originating action"
    );
    assert!(
        fixture.exists() && !dir.join("archive/note.md").exists(),
        "the test fixture proves the move was really skipped"
    );

    let ordinary = dir.join("ordinary.png");
    capture_screenshot(
        ordinary.clone(),
        Some(fixture.clone()),
        CaptureOpts::default(),
        vec![],
        keymap(),
        Some(dir.clone()),
        None,
        dir.join("notes"),
        Config::empty(),
        false,
    )
    .expect("ordinary capture succeeds");
    let ordinary_sidecar: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(ordinary.with_extension("json")).unwrap())
            .unwrap();
    assert_eq!(ordinary_sidecar["replay_skips"], serde_json::json!([]));

    let strict = dir.join("strict.png");
    let err = capture_screenshot(
        strict.clone(),
        Some(fixture),
        CaptureOpts::default(),
        keys,
        keymap(),
        Some(dir.clone()),
        None,
        dir.join("notes"),
        Config::empty(),
        true,
    )
    .expect_err("strict replay aborts at the same move seam");
    assert!(err.to_string().contains("`overlay_accept`"), "{err}");
    assert!(!strict.exists() && !strict.with_extension("json").exists());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn permissive_skip_sweeps_the_four_known_live_only_effects() {
    let cases = [
        (
            Action::Newline,
            actions::Effect::OverlayAccept(crate::overlay::OverlayKind::MoveDest, "archive".into()),
            "overlay_accept",
        ),
        (
            Action::Newline,
            actions::Effect::RenameNoteCommit {
                new_name: "renamed.md".into(),
            },
            "rename_note_commit",
        ),
        (
            Action::DuplicateNote,
            actions::Effect::DuplicateNote,
            "duplicate_note",
        ),
        (
            Action::Newline,
            actions::Effect::SettingPathPick {
                key: "default_folder".into(),
                path: "/notes".into(),
            },
            "setting_path_pick",
        ),
    ];
    for (action, effect, expected) in cases {
        let skip = crate::replay::permissive_skip(&action, &crate::replay::classify(&effect))
            .expect("known live-only effect records a skip");
        assert_eq!(skip.effect, expected);
        assert_eq!(skip.action, format!("{action:?}"));
    }
    assert!(
        crate::replay::permissive_skip(
            &Action::InsertChar('x'),
            &crate::replay::classify(&actions::Effect::None),
        )
        .is_none()
    );
}

/// USER-BUG LAW: changing the page measure must only reveal/occlude ONE
/// already-authored lava backdrop. The pixels that remain exposed at both
/// widths are therefore byte-identical, while pixels well inside the page
/// stay one flat color. This renders through the real shader/uniform path;
/// the pure `lava::tests` sibling alone cannot catch a bad upload or mask.
#[test]
fn lava_backdrop_pixels_are_page_width_invariant_and_page_interior_is_flat() {
    let _g = crate::testlock::serial();
    let old_theme = crate::theme::active_index();
    let old_measure = crate::page::measure();
    let old_page = crate::page::page_on();
    crate::theme::set_active_by_name("Mangrove").unwrap();
    crate::page::set_page_on(true);

    let dir = std::env::temp_dir().join(format!("awl-lava-width-law-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let render = |measure, stem: &str| {
        crate::page::set_measure(measure);
        let out = dir.join(format!("{stem}.png"));
        let opts = CaptureOpts {
            canvas: Some((1200, 800)),
            ..CaptureOpts::default()
        };
        capture_screenshot(
            out.clone(),
            None,
            opts,
            Vec::new(),
            crate::keymap::KeymapState::new(),
            Some(dir.clone()),
            Some(dir.clone()),
            dir.clone(),
            Config::empty(),
            false, // permissive (the legacy default)
        )
        .expect("lava width-law capture succeeds");
        let json: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(out.with_extension("json")).unwrap())
                .unwrap();
        let left = json["page"]["column"]["left"].as_f64().unwrap() as u32;
        let width = json["page"]["column"]["width"].as_f64().unwrap() as u32;
        (image::open(out).unwrap().to_rgba8(), left, left + width)
    };
    let (narrow, narrow_l, narrow_r) = render(40, "narrow");
    let (wide, wide_l, wide_r) = render(70, "wide");

    let left_full = narrow_l
        .min(wide_l)
        .saturating_sub(crate::lava::MARGIN_GAP_PX as u32);
    let right_full = (narrow_r.max(wide_r) + crate::lava::MARGIN_GAP_PX as u32).min(1200);
    let mut compared = 0usize;
    for y in 80..720 {
        for x in (0..left_full).chain(right_full..1200) {
            assert_eq!(
                narrow.get_pixel(x, y),
                wide.get_pixel(x, y),
                "common exposed backdrop changed at ({x},{y})"
            );
            compared += 1;
        }
    }
    assert!(
        compared > 50_000,
        "width law sampled a substantial common margin"
    );

    let x0 = narrow_l.max(wide_l) + 64;
    let x1 = narrow_r.min(wide_r).saturating_sub(64);
    for (label, frame) in [("narrow", &narrow), ("wide", &wide)] {
        let flat = *frame.get_pixel(600, 650);
        for y in 600..720 {
            for x in x0..x1 {
                assert_eq!(
                    *frame.get_pixel(x, y),
                    flat,
                    "{label} page is not flat at ({x},{y})"
                );
            }
        }
    }

    crate::theme::set_active(old_theme);
    crate::page::set_measure(old_measure);
    crate::page::set_page_on(old_page);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn workspace_defaults_to_root_parent_when_unset() {
    let root = PathBuf::from("/home/me/work/repos/some-repo");
    assert_eq!(
        resolve_workspace(&None, &root),
        PathBuf::from("/home/me/work/repos")
    );
}

#[test]
fn explicit_workspace_overrides_the_default() {
    let root = PathBuf::from("/home/me/work/repos/some-repo");
    let ws = PathBuf::from("/elsewhere/projects");
    assert_eq!(resolve_workspace(&Some(ws.clone()), &root), ws);
}

#[test]
fn workspace_falls_back_to_root_when_no_parent() {
    let root = PathBuf::from("/");
    assert_eq!(resolve_workspace(&None, &root), root);
}

// ---- VISUAL-LINE MOVEMENT (Phase 2) ----------------------------------
//
// These drive the REAL keymap through `replay_keys` with a layout oracle
// shaped at a NARROW measure, exactly as the live window / `--keys --measure`
// CLI do, so a long line soft-wraps and the motions must follow the VISUAL
// rows. The page globals are process-wide, so each test holds `page::test_lock()`
// and restores the default measure. On a GPU-less host the oracle is `None`,
// motion falls back to logical, and the test SKIPS (prints + returns).

/// Build a narrow-measure oracle, replay `keys` through the real keymap, and
/// return the resulting (line, col) — or `None` when no wgpu adapter exists
/// (skip). Holds the page lock for the whole replay and restores the measure.
fn replay_visual(text: &str, measure: usize, keys: &str) -> Option<(usize, usize)> {
    let _g = crate::testlock::serial();
    crate::page::set_page_on(true);
    crate::page::set_measure(measure);
    let mut buffer = Buffer::from_str(text);
    let opts = CaptureOpts::default();
    let out = capture::build_oracle(&buffer, &opts).map(|mut op| {
        let keys = keyspec::parse_keys(keys).unwrap();
        let root = PathBuf::from("/tmp");
        replay_keys(
            &mut buffer,
            &keys,
            &[],
            &root,
            None,
            &Config::empty(),
            Some(&mut op),
        );
        buffer.cursor_line_col()
    });
    crate::page::set_measure(crate::page::DEFAULT_MEASURE);
    out
}

const LONG: &str = "the quick brown fox jumps over the lazy dog today\nNEXT\n";
const LONG_LINE0_LEN: usize = 49; // chars before the first '\n'

#[test]
fn visual_c_n_lands_on_next_visual_row_not_next_paragraph() {
    let Some((line, col)) = replay_visual(LONG, 15, "C-n") else {
        eprintln!("skipping visual_c_n_lands_on_next_visual_row: no wgpu adapter");
        return;
    };
    assert_eq!(
        line, 0,
        "C-n stays on the wrapped logical line, not paragraph 2"
    );
    assert!(
        col > 0,
        "C-n moved off col 0 onto the next visual row, got {col}"
    );
    assert!(
        col < LONG_LINE0_LEN,
        "the landing is a wrap boundary mid-line, not the logical end ({col})"
    );
}

#[test]
fn visual_c_e_stops_at_visual_row_end_not_logical_line_end() {
    let Some((line, col)) = replay_visual(LONG, 15, "C-e") else {
        eprintln!("skipping visual_c_e_stops_at_visual_row_end: no wgpu adapter");
        return;
    };
    assert_eq!(line, 0);
    assert!(col > 0, "C-e moved to the visual row end");
    assert!(
        col < LONG_LINE0_LEN,
        "C-e stopped at the VISUAL row end ({col}), not the logical line end ({LONG_LINE0_LEN})"
    );
}

#[test]
fn visual_goal_x_is_preserved_across_c_n_then_c_p() {
    let down_up = replay_visual(LONG, 15, "C-f C-f C-f C-f C-f C-n C-p");
    let just_right = replay_visual(LONG, 15, "C-f C-f C-f C-f C-f");
    let (Some(down_up), Some(just_right)) = (down_up, just_right) else {
        eprintln!("skipping visual_goal_x_preserved: no wgpu adapter");
        return;
    };
    assert_eq!(just_right, (0, 5), "five C-f land at col 5");
    assert_eq!(
        down_up, just_right,
        "C-n then C-p returns to the starting column via the sticky goal-x"
    );
}

#[test]
fn visual_c_a_goes_to_visual_row_start() {
    let start = replay_visual(LONG, 15, "C-n");
    let from_mid = replay_visual(LONG, 15, "C-n C-f C-f C-a");
    let (Some(start), Some(from_mid)) = (start, from_mid) else {
        eprintln!("skipping visual_c_a_goes_to_visual_row_start: no wgpu adapter");
        return;
    };
    assert_eq!(start.0, 0);
    assert!(start.1 > 0, "C-n reached a wrapped row start > 0");
    assert_eq!(
        from_mid, start,
        "C-a snaps back to the VISUAL row start, not the logical line start (col 0)"
    );
}

#[test]
fn visual_c_n_at_last_visual_row_crosses_to_next_logical_line() {
    let _g = crate::testlock::serial();
    crate::page::set_page_on(true);
    crate::page::set_measure(15);
    let probe = Buffer::from_str(LONG);
    let opts = CaptureOpts::default();
    let result = capture::build_oracle(&probe, &opts).map(|mut op| {
        let mut steps = 0usize;
        {
            let oracle = op.as_oracle();
            let (mut l, mut c) = (0usize, 0usize);
            loop {
                let (nl, nc) =
                    oracle.visual_line_down(l, c, 0.0, crate::caret::Affinity::Downstream);
                steps += 1;
                if nl != 0 {
                    break;
                }
                assert!(steps < 100, "line 0 never ended");
                l = nl;
                c = nc;
            }
        }
        assert!(steps >= 2, "line 0 should wrap into multiple visual rows");
        let root = PathBuf::from("/tmp");
        let mut b0 = Buffer::from_str(LONG);
        let keys_stay = keyspec::parse_keys(&"C-n ".repeat(steps - 1)).unwrap();
        replay_keys(
            &mut b0,
            &keys_stay,
            &[],
            &root,
            None,
            &Config::empty(),
            Some(&mut op),
        );
        let stay = b0.cursor_line_col();
        let mut b1 = Buffer::from_str(LONG);
        let keys_cross = keyspec::parse_keys(&"C-n ".repeat(steps)).unwrap();
        replay_keys(
            &mut b1,
            &keys_cross,
            &[],
            &root,
            None,
            &Config::empty(),
            Some(&mut op),
        );
        (stay, b1.cursor_line_col())
    });
    crate::page::set_measure(crate::page::DEFAULT_MEASURE);
    let Some((stay, cross)) = result else {
        eprintln!("skipping visual_c_n_crosses_to_next_logical_line: no wgpu adapter");
        return;
    };
    assert_eq!(
        stay.0, 0,
        "one C-n short keeps us on line 0's last visual row"
    );
    assert_eq!(
        cross.0, 1,
        "the last-row C-n crosses into the next logical line"
    );
    assert_eq!(cross.1, 0, "we land on line 1's FIRST visual row");
}

#[test]
fn regression_non_wrapped_doc_visual_equals_logical_byte_identical() {
    // REGRESSION GUARD: on a NON-wrapped document (every logical line fits in
    // one visual row) visual motion == logical motion. Identical-content lines
    // make the vertical goal-x round-trip exact even on a proportional font.
    // Replay the SAME keys with the oracle (visual) and without it (logical);
    // the resulting cursors — and the rendered PNGs — must be IDENTICAL.
    let _g = crate::testlock::serial();
    crate::page::set_page_on(true);
    crate::page::set_measure(crate::page::DEFAULT_MEASURE);
    let text = "hello world foo\nhello world foo\nhello world foo\n";
    let keys = keyspec::parse_keys("C-f C-f C-f C-f C-f C-n C-n C-e C-a C-p C-k").unwrap();
    let root = PathBuf::from("/tmp");
    let opts = CaptureOpts::default();

    let mut logical = Buffer::from_str(text);
    replay_keys(
        &mut logical,
        &keys,
        &[],
        &root,
        None,
        &Config::empty(),
        None,
    );

    let mut visual = Buffer::from_str(text);
    let Some(mut op) = capture::build_oracle(&visual, &opts) else {
        crate::page::set_measure(crate::page::DEFAULT_MEASURE);
        eprintln!("skipping regression_non_wrapped byte-identical: no wgpu adapter");
        return;
    };
    replay_keys(
        &mut visual,
        &keys,
        &[],
        &root,
        None,
        &Config::empty(),
        Some(&mut op),
    );

    assert_eq!(
        visual.cursor_line_col(),
        logical.cursor_line_col(),
        "non-wrapped: visual motion must equal logical motion"
    );

    // Byte-identical captures: render both buffers and diff the PNG bytes.
    // PID-suffixed (not just `serial()`-guarded): `serial()` is a per-process
    // reentrant lock, so a SECOND concurrent `cargo test` process (e.g. a
    // parallel native + AWL_CONVENTION_FORCE=linux run) can't be excluded by
    // it — only a unique path can (mirrors every other temp-file test).
    let dir = std::env::temp_dir();
    let pid = std::process::id();
    let pv = dir.join(format!("awl_vl_visual_{pid}.png"));
    let pl = dir.join(format!("awl_vl_logical_{pid}.png"));
    capture::capture_with(&pv, &visual, &opts).expect("render visual");
    capture::capture_with(&pl, &logical, &opts).expect("render logical");
    let bv = std::fs::read(&pv).expect("read visual png");
    let bl = std::fs::read(&pl).expect("read logical png");
    crate::page::set_measure(crate::page::DEFAULT_MEASURE);
    assert_eq!(
        bv, bl,
        "non-wrapped short-line doc: visual + logical captures are byte-identical"
    );
    let _ = std::fs::remove_file(&pv);
    let _ = std::fs::remove_file(&pl);
}

#[test]
fn regression_edit_then_wrapped_motion_sees_fresh_wrap_geometry() {
    // THE known stale case this round retires: a spec that EDITS (wrapping
    // line 0) and then moves DOWN. The pre-phase oracle still held the
    // pre-replay shape (line 0 short, unwrapped), so C-n stepped straight
    // into logical line 1 at (1, 0); fresh per-action geometry lands on
    // line 0's SECOND visual row instead.
    let _g = crate::testlock::serial();
    crate::page::set_page_on(true);
    crate::page::set_measure(15);
    let mut buffer = Buffer::from_str("ab\ntail\n");
    let opts = CaptureOpts::default();
    let Some(mut op) = capture::build_oracle(&buffer, &opts) else {
        crate::page::set_measure(crate::page::DEFAULT_MEASURE);
        eprintln!("skipping regression_edit_then_wrapped_motion: no wgpu adapter");
        return;
    };
    let mut spec: Vec<String> = "the quick brown fox jumps over"
        .chars()
        .map(|c| {
            if c == ' ' {
                "Space".to_string()
            } else {
                c.to_string()
            }
        })
        .collect();
    spec.push("s-Up".to_string()); // BufferStart (mac native)
    spec.push("C-n".to_string()); // NextLine
    let keys = keyspec::parse_keys(&spec.join(" ")).unwrap();
    let root = PathBuf::from("/tmp");
    replay_keys(
        &mut buffer,
        &keys,
        &[],
        &root,
        None,
        &Config::empty(),
        Some(&mut op),
    );
    let (line, col) = buffer.cursor_line_col();
    crate::page::set_measure(crate::page::DEFAULT_MEASURE);
    assert_eq!(
        line, 0,
        "Down follows the freshly-wrapped line 0 (stale geometry crossed into line 1)"
    );
    assert!(
        col > 0,
        "landing on line 0's second visual row, got col {col}"
    );
}

#[test]
fn zoom_change_mid_replay_re_wraps_the_oracle_for_later_motion() {
    // With the column capped by the WINDOW (MAX_MEASURE), a bigger zoom
    // fits fewer chars per visual row — so Down after a replayed Cmd-+
    // must land at a strictly SMALLER column than the same Down at zoom
    // 1.0. The pre-phase oracle kept its build-time zoom, landing the two
    // replays identically.
    let _g = crate::testlock::serial();
    crate::page::set_page_on(true);
    crate::page::set_measure(crate::page::MAX_MEASURE);
    let text = format!("{}\ntail\n", "word ".repeat(80));
    let root = PathBuf::from("/tmp");
    let opts = CaptureOpts::default();

    let mut plain = Buffer::from_str(&text);
    let Some(mut op1) = capture::build_oracle(&plain, &opts) else {
        crate::page::set_measure(crate::page::DEFAULT_MEASURE);
        eprintln!("skipping zoom_change_mid_replay_re_wraps_the_oracle: no wgpu adapter");
        return;
    };
    let keys_plain = keyspec::parse_keys("C-n").unwrap();
    replay_keys(
        &mut plain,
        &keys_plain,
        &[],
        &root,
        None,
        &Config::empty(),
        Some(&mut op1),
    );
    let (l1, c1) = plain.cursor_line_col();

    let mut zoomed = Buffer::from_str(&text);
    let Some(mut op2) = capture::build_oracle(&zoomed, &opts) else {
        crate::page::set_measure(crate::page::DEFAULT_MEASURE);
        eprintln!("skipping zoom_change_mid_replay_re_wraps_the_oracle: no wgpu adapter");
        return;
    };
    let keys_zoom = keyspec::parse_keys("s-= C-n").unwrap();
    replay_keys(
        &mut zoomed,
        &keys_zoom,
        &[],
        &root,
        None,
        &Config::empty(),
        Some(&mut op2),
    );
    let (l2, c2) = zoomed.cursor_line_col();

    crate::page::set_measure(crate::page::DEFAULT_MEASURE);
    assert_eq!((l1, l2), (0, 0), "both Downs stay on the wrapped line 0");
    assert!(
        c1 > 0 && c2 > 0,
        "both landed on a second visual row: {c1}, {c2}"
    );
    assert!(
        c2 < c1,
        "the zoomed row holds fewer chars, so its wrap boundary is earlier: {c2} < {c1}"
    );
}

#[test]
fn goto_switch_mid_replay_reshapes_the_oracle_to_the_arriving_buffer() {
    // The Goto arm swaps the ACTIVE buffer (and re-applies its sticky page
    // measure) mid-replay; a following Down must read the ARRIVING
    // buffer's wrap geometry. Launched on a CODE file (configured measure
    // 100 — b.md's long line would NOT wrap there), the switch to the
    // prose b.md re-applies measure 15 and swaps the text: both must reach
    // the oracle for Down to stay on b.md's wrapped line 0. The pre-phase
    // oracle stayed shaped on a.rs, so Down crossed into line 1 at (1, 0).
    let _fs = crate::testlock::serial();
    let dir = std::env::temp_dir().join(format!("awl-oracle-goto-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("a.rs"), "fn main() {}\n").unwrap();
    std::fs::write(dir.join("b.md"), "the quick brown fox jumps over\ntail\n").unwrap();
    let cfg = Config {
        page_width_prose: Some(15),
        page_width_code: Some(100),
        ..Config::empty()
    };
    crate::page::set_page_on(true);
    crate::page::set_measure(100); // the launch file's own (code) measure
    let mut buffer = Buffer::from_file(&dir.join("a.rs"));
    let corpus = vec!["a.rs".to_string(), "b.md".to_string()];
    let opts = CaptureOpts::default();
    let Some(mut op) = capture::build_oracle(&buffer, &opts) else {
        crate::page::set_measure(crate::page::DEFAULT_MEASURE);
        let _ = std::fs::remove_dir_all(&dir);
        eprintln!("skipping goto_switch_mid_replay_reshapes_the_oracle: no wgpu adapter");
        return;
    };
    let keys = keyspec::parse_keys("s-o b . m d RET C-n").unwrap();
    replay_keys(&mut buffer, &keys, &corpus, &dir, None, &cfg, Some(&mut op));
    let (line, col) = buffer.cursor_line_col();
    crate::page::set_measure(crate::page::DEFAULT_MEASURE);
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(
        buffer.path(),
        Some(dir.join("b.md").as_path()),
        "the Goto switch landed on b.md"
    );
    assert_eq!(
        line, 0,
        "Down follows b.md's line 0, wrapped at ITS re-applied measure"
    );
    assert!(
        col > 0,
        "landing on line 0's second visual row, got col {col}"
    );
}

/// LAW: the caret-MODE preference (an explicit pin, OR auto) must never be
/// mutated by mere THEME movement — a COMMITTED round-trip switch through a
/// one-bit world (Wagtail), or a theme-picker PREVIEW-and-Esc of one — is a
/// true no-op on the caret global. Covers both suspects the caret-style-change
/// bug report named: the 1-bit round's render-time override (`prepare_caret_
/// layer` reads `crate::caret::mode()` but never writes it — this is the
/// sticky round-trip proof of that) and auto-by-design (auto is legitimately
/// theme-dependent, but a journey that ENDS back on the same world must
/// resolve identically to never having left).
#[test]
fn caret_mode_survives_theme_journeys_committed_and_preview_esc() {
    let _g = crate::testlock::serial();
    let _t = crate::testlock::serial();
    let root = PathBuf::from("/tmp");
    let keys =
        keyspec::parse_keys("s-t W a g t a i l RET s-t G u m t r e e RET s-t W a g t a i l Esc")
            .unwrap();

    crate::theme::set_active_by_name("Gumtree").unwrap();
    crate::caret::set_mode(crate::caret::CaretMode::Block);
    let mut buf = Buffer::scratch();
    replay_keys(&mut buf, &keys, &[], &root, None, &Config::empty(), None);
    assert_eq!(
        crate::theme::active().name,
        "Gumtree",
        "the journey lands back on Gumtree"
    );
    assert!(
        !crate::caret::is_auto(),
        "an explicit pin is never cleared by a theme journey"
    );
    assert_eq!(crate::caret::mode(), crate::caret::CaretMode::Block);

    crate::caret::clear_override();
    crate::theme::set_active_by_name("Gumtree").unwrap();
    let mut buf2 = Buffer::scratch();
    replay_keys(&mut buf2, &keys, &[], &root, None, &Config::empty(), None);
    assert_eq!(crate::theme::active().name, "Gumtree");
    assert!(
        crate::caret::is_auto(),
        "a theme-only journey never pins auto"
    );
    assert_eq!(
        crate::caret::mode(),
        crate::caret::CaretMode::Morph,
        "Gumtree (proportional) resolves Morph, exactly as if never visiting Wagtail"
    );

    crate::theme::set_active(crate::theme::DEFAULT_THEME);
    crate::caret::clear_override();
}

/// STATELESSNESS LAW: the DRAWN caret for mode `M` in world `W` is a pure
/// function of `(M, W)` — never of the journey that got there. Proves it by
/// rendering the identical settled `(mode, world)` twice — once landed on
/// directly, once after a full COMMITTED Wagtail (one-bit) detour plus a
/// theme-picker preview-and-Esc of Wagtail — and diffing the PNG bytes. This
/// is the capture-level regression guard for suspect #1 (the 1-bit round's
/// `prepare_caret_layer` Morph->Block override must stay a pure per-frame
/// render decision, never leaking into the mode global or any pipeline
/// Globals left set from Wagtail's own frame).
#[test]
fn caret_render_is_a_pure_function_of_mode_and_world_across_a_wagtail_detour() {
    let _g = crate::testlock::serial();
    let _t = crate::testlock::serial();
    let root = PathBuf::from("/tmp");
    let opts = CaptureOpts::default();
    let text = "hello frame\n";
    let detour_keys =
        keyspec::parse_keys("s-t W a g t a i l RET s-t G u m t r e e RET s-t W a g t a i l Esc")
            .unwrap();

    for mode in [
        crate::caret::CaretMode::Block,
        crate::caret::CaretMode::Morph,
        crate::caret::CaretMode::Ibeam,
    ] {
        crate::theme::set_active_by_name("Gumtree").unwrap();
        crate::caret::set_mode(mode);
        let base_buf = Buffer::from_str(text);
        let Some(_op) = capture::build_oracle(&base_buf, &opts) else {
            eprintln!(
                "skipping caret_render_is_a_pure_function_of_mode_and_world_across_a_wagtail_detour: no wgpu adapter"
            );
            crate::theme::set_active(crate::theme::DEFAULT_THEME);
            crate::caret::clear_override();
            return;
        };
        let dir = std::env::temp_dir();
        let pid = std::process::id();
        let base_png = dir.join(format!("awl_caret_stateless_base_{mode:?}_{pid}.png"));
        capture::capture_with(&base_png, &base_buf, &opts).expect("baseline capture");

        crate::theme::set_active_by_name("Gumtree").unwrap();
        crate::caret::set_mode(mode);
        let mut detour_buf = Buffer::from_str(text);
        replay_keys(
            &mut detour_buf,
            &detour_keys,
            &[],
            &root,
            None,
            &Config::empty(),
            None,
        );
        assert_eq!(
            crate::theme::active().name,
            "Gumtree",
            "the detour lands back on Gumtree"
        );
        assert_eq!(
            crate::caret::mode(),
            mode,
            "the detour never touched the pinned mode"
        );
        let detour_png = dir.join(format!("awl_caret_stateless_detour_{mode:?}_{pid}.png"));
        capture::capture_with(&detour_png, &detour_buf, &opts).expect("detour capture");

        let b1 = std::fs::read(&base_png).expect("read baseline png");
        let b2 = std::fs::read(&detour_png).expect("read detour png");
        assert_eq!(
            b1, b2,
            "mode {mode:?}: caret pixels must be byte-identical whether or not Wagtail was visited in between"
        );
        let _ = std::fs::remove_file(&base_png);
        let _ = std::fs::remove_file(&detour_png);
    }

    crate::theme::set_active(crate::theme::DEFAULT_THEME);
    crate::caret::clear_override();
}

/// ITEM 106 — THE POINTER-REPLAY SEAM, end to end through the REAL
/// headless `--keys` engine (`ReplaySession`, the exact type
/// `--screenshot --keys` constructs) — not a pure `OverlayState`
/// simulation. Opens a real 40-row Goto picker, hovers a real row via a
/// REAL hit-test against the oracle's own pipeline, drives a real
/// keyboard scroll past the candidate window, then re-checks the SAME
/// physical pixel: the row now under it (per the SAME real hit-test)
/// must not steal the keyboard's selection. Proves the seam the scout
/// named (`ReplaySession::cursor_px` + `apply_move`, sharing
/// `TextPipeline::resolve_overlay_hover` with the live
/// `App::overlay_hover`) actually reproduces the item's own named live
/// hazard deterministically and headlessly, through the sidecar-adjacent
/// `ReplaySession::overlay()` state oracle.
#[test]
fn item_106_pointer_replay_seam_reproduces_a_keyboard_scroll_stealing_a_stationary_pointer_check() {
    let _g = crate::testlock::serial();
    let mut buffer = Buffer::scratch();
    let corpus: Vec<String> = (0..40).map(|i| format!("row{i}.md")).collect();
    let root = PathBuf::from("/tmp");
    let opts = CaptureOpts::default();
    let Some(mut oracle) = capture::build_oracle(&buffer, &opts) else {
        eprintln!(
            "skipping item_106_pointer_replay_seam_reproduces_a_keyboard_scroll_stealing_a_stationary_pointer_check: \
                 no wgpu adapter"
        );
        return;
    };
    let config = Config::empty();
    let mut km =
        crate::keymap::KeymapState::new_with_convention(crate::convention::Convention::Mac);
    let mut session = ReplaySession::new(
        crate::replay::Mode::Permissive,
        &mut buffer,
        &corpus,
        &root,
        None,
        &config,
        Some(&mut oracle),
        &mut km,
    );

    for chord in keyspec::parse_keys("s-o").unwrap() {
        session.apply_chord(&chord).unwrap();
    }
    assert!(session.overlay().is_some(), "Goto must be open");

    session.sync_oracle_overlay();
    let card = session
        .oracle()
        .expect("oracle present")
        .overlay_card_rect()
        .expect("goto card rect");
    let px = card[0] + card[2] * 0.5;
    let (py_top, py_bot) = (card[1], card[1] + card[3]);
    let find_row = |session: &ReplaySession, target: Option<usize>| -> Option<f32> {
        let op = session.oracle().expect("oracle present");
        let mut y = py_top;
        while y < py_bot {
            let hit = op.overlay_row_at(px, y);
            if target.is_none() && hit.is_some() {
                return Some(y);
            }
            if hit == target && target.is_some() {
                return Some(y);
            }
            y += 1.0;
        }
        None
    };
    let py = find_row(&session, Some(3)).expect("row 3 must be found within the card");

    session.apply_move(px, py);
    assert_eq!(
        session.overlay().unwrap().selected,
        3,
        "the real hover selected row 3"
    );

    for chord in keyspec::parse_keys(&"Down ".repeat(22)).unwrap() {
        session.apply_chord(&chord).unwrap();
    }
    assert_eq!(
        session.overlay().unwrap().selected,
        25,
        "keyboard nav landed on row 25"
    );
    assert!(session.overlay().unwrap().scroll > 0, "the window scrolled");

    // Re-locate whatever is NOW at the SAME physical pixel (px, py) — the
    // hazard's premise: the window scrolled, so a DIFFERENT item sits
    // there now, even though the pointer never moved a device pixel.
    session.sync_oracle_overlay();
    let hit_now = session
        .oracle()
        .expect("oracle present")
        .overlay_row_at(px, py);
    assert!(
        hit_now.is_some(),
        "the scrolled card still draws SOME row at that pixel"
    );
    assert_ne!(
        hit_now,
        Some(25),
        "a different item now sits under the stationary pixel"
    );

    // THE LAW: a stray re-check with a REAL 1px jitter off the parked
    // pixel — not the exact same coordinate (item 85's own
    // exact-equality gate already refused a bare duplicate; this law's
    // own regression needs genuine, if tiny, travel) — through the exact
    // production seam a spurious `CursorMoved` would drive — must not
    // steal the keyboard's selection.
    session.apply_move(px + 1.0, py);
    assert_eq!(
        session.overlay().unwrap().selected,
        25,
        "the keyboard's selection survives a 1px-jittered stationary pointer re-check"
    );

    session.sync_oracle_overlay();
    let py0 = find_row(&session, None).expect("display row 0 must be found");
    let hit0 = session
        .oracle()
        .expect("oracle present")
        .overlay_row_at(px, py0);
    session.apply_move(px, py0);
    assert_eq!(
        session.overlay().unwrap().selected,
        hit0.unwrap(),
        "a genuine pointer move to a different row takes over on the first event"
    );
}
