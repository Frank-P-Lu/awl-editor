//! **GO TO LINE, DRIVEN ENTIRELY THROUGH `--keys`.** Real chords open the
//! Command Palette, filter to "Go to…" (`OpenGoto` carries no default
//! binding of its own), type a line number, and accept — the exact route a
//! live user takes (`crate::run::ReplaySession::apply_chord`, the seam
//! `--keys` itself runs). The sidecar is read back for BOTH the caret
//! (`cursor.line`/`cursor.col`) and the document scroll (`scroll_lines`,
//! derived from the buffer's resting cursor at capture time, the same
//! `follow_scroll` door the live App's own `sync_view(true)` uses) — a jump
//! that moves the caret but leaves the destination scrolled off-screen is
//! exactly the bug this file's laws must catch.

use super::super::*;
use super::adapter_available;
use crate::buffer::Buffer;
use crate::config::Config;
use crate::testscratch::ScratchDir;

/// Open the unified Go-to picker through the palette, exactly as a live user
/// would (`OpenGoto` has no direct default binding) — mirrors
/// `diagonal_transition_geometry.rs`'s own `open_goto` helper.
fn open_goto(session: &mut crate::run::ReplaySession) {
    let chords = crate::keyspec::parse_chords("s-p g o Space t o Enter").expect("chords");
    for c in &chords {
        session.apply_chord(c).expect("chord applies");
    }
}

/// Type `text`'s characters as individual real chords — what `--keys` itself
/// replays for a plain, unmodified sequence.
fn type_chars(session: &mut crate::run::ReplaySession, text: &str) {
    let spec: String = text
        .chars()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join(" ");
    let chords = crate::keyspec::parse_chords(&spec).expect("chords");
    for c in &chords {
        session.apply_chord(c).expect("chord applies");
    }
}

fn press_enter(session: &mut crate::run::ReplaySession) {
    let chords = crate::keyspec::parse_chords("Enter").expect("chords");
    for c in &chords {
        session.apply_chord(c).expect("chord applies");
    }
}

/// A real, keymap-bound picker navigation (`End` -> `Action::LineEnd` ->
/// `OverlayState::select_last`) that deterministically reaches the line-jump
/// row: it is always PARKED LAST (`RowMeta::terminal`), after any file/folder
/// row the digit query happens to fuzzy-match too (the go-to picker's
/// workspace root is itself always a candidate row, and this session's
/// workspace lives under the host's REAL, ambient scratch/temp path -- whose
/// own name is not under this test's control and may coincidentally share
/// digits with a query). Pressing End is exactly what a real user would do if
/// the row weren't already the obvious top hit; it does not change WHICH
/// effect fires or how, only which of several already-produced rows is
/// selected before Enter.
fn press_end(session: &mut crate::run::ReplaySession) {
    let chords = crate::keyspec::parse_chords("End").expect("chords");
    for c in &chords {
        session.apply_chord(c).expect("chord applies");
    }
}

/// One full "open Go to…, type a line number, accept" round through real
/// chords, over a fresh session on `text`. Returns the settled sidecar JSON
/// plus the driven `Buffer` (owned by the caller so fold state is
/// inspectable after the call).
fn run_goto_line(
    dir: &std::path::Path,
    label: &str,
    text: &str,
    query: &str,
    fold_first_line: bool,
) -> (serde_json::Value, Buffer) {
    let mut buffer = Buffer::from_str(text);
    if fold_first_line {
        let folded = buffer.toggle_fold_at_cursor();
        assert!(folded.is_some(), "{label}: fixture must actually fold");
    }
    let corpus: Vec<String> = Vec::new();
    let root = dir.to_path_buf();
    let config = Config::empty();
    let mut km =
        crate::keymap::KeymapState::new_with_convention(crate::convention::Convention::Mac);
    // An EXPLICIT, isolated workspace (the empty scratch dir itself) rather
    // than `None` -- `None` falls back to the real ambient workspace, whose
    // (real, populated) child folders would otherwise fuzzy-rank alongside a
    // digit query and steal the default selection away from the parked
    // line-jump row (CLAUDE.md's "capture a file picker against a seeded
    // root, never the ambient one").
    let mut session = crate::run::ReplaySession::new(
        crate::run::ReplayPolicy::ordinary(),
        &mut buffer,
        &corpus,
        &root,
        Some(root.as_path()),
        &config,
        None,
        &mut km,
    );
    open_goto(&mut session);
    type_chars(&mut session, query);
    press_end(&mut session);
    assert!(
        session
            .journey()
            .card()
            .is_some_and(|o| o.selected_is_line_jump()),
        "{label}: End must land on the line-jump row, the row `Enter` is about to accept"
    );
    press_enter(&mut session);
    assert!(
        session.journey().card().is_none(),
        "{label}: accepting the line-jump row must close the overlay back to the document"
    );

    let project = crate::run::project_info(&root, &None, None, &config);
    let opts = crate::run::fold_capture_state(&session, project);
    let out = dir.join(format!("{label}.png"));
    capture_with(&out, session.buffer(), &opts).expect("capture succeeds");
    let json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(out.with_extension("json")).unwrap())
            .expect("sidecar parses");
    drop(session);
    (json, buffer)
}

/// The destination row must sit within the CAPTURE'S OWN reported visible
/// window — `scroll_lines <= line < scroll_lines + visible_rows` — read off
/// THIS capture's own geometry (`text_origin.top`, `font.line_height`,
/// `canvas.height`), never a hand-guessed constant. Catches exactly the bug
/// the Verify clause names: a caret that moved but was left scrolled off
/// the canvas.
fn assert_line_visible(ctx: &str, json: &serde_json::Value, line: usize) {
    let text_top = json["text_origin"]["top"].as_f64().unwrap();
    let line_h = json["font"]["line_height"].as_f64().unwrap();
    let canvas_h = json["canvas"]["height"].as_f64().unwrap();
    let scroll_lines = json["scroll_lines"].as_u64().unwrap() as usize;
    // CEIL, not floor: a row whose top is on-canvas but whose bottom is
    // clipped still reads as "shown" by the product's own scroll strategy
    // (`scroll_to_show_row_pos`) -- flooring undercounts capacity by one row
    // at the exact boundary this law's "last line" case lands on.
    let visible_rows = ((canvas_h - text_top) / line_h).ceil() as usize;
    assert!(
        line >= scroll_lines && line < scroll_lines + visible_rows,
        "{ctx}: destination line {line} is outside the visible window \
         [{scroll_lines}, {}) -- scrolled off-screen (canvas_h={canvas_h}, \
         text_top={text_top}, line_h={line_h})",
        scroll_lines + visible_rows
    );
}

fn assert_cursor(ctx: &str, json: &serde_json::Value, line: usize, col: usize) {
    assert_eq!(
        json["cursor"]["line"].as_u64(),
        Some(line as u64),
        "{ctx}: cursor.line"
    );
    assert_eq!(
        json["cursor"]["col"].as_u64(),
        Some(col as u64),
        "{ctx}: cursor.col"
    );
}

/// **THE CORE SWEEP: first / middle / last / out-of-range (both directions),
/// with the sidecar proving caret AND scroll.** A 200-line plain-text
/// buffer, one full palette-driven round-trip per case.
#[test]
fn goto_line_sweeps_first_middle_last_and_out_of_range_with_caret_and_scroll_proof() {
    let _g = crate::testlock::serial();
    if !adapter_available() {
        eprintln!("skipping goto_line_sweeps_...caret_and_scroll_proof: no wgpu adapter");
        return;
    }
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl_goto_line_sweep_{}", std::process::id())),
    );
    let n = 200usize;
    let text: String = (0..n).map(|i| format!("line {i}\n")).collect();

    // Establish this fixture's REAL line count from the buffer itself
    // (ropey's own line-counting convention), never a hand-derived guess.
    let probe = Buffer::from_str(&text);
    let line_count = probe.line_count();
    assert!(
        line_count >= n,
        "fixture has at least {n} lines: {line_count}"
    );

    let cases: [(&str, &str, usize); 5] = [
        ("first", "1", 0),
        ("middle", "100", 99),
        ("last", &line_count.to_string(), line_count - 1),
        ("too_low", "0", 0),
        ("too_high", "999999", line_count - 1),
    ];
    for (label, query, expected_line) in cases {
        let (json, _buf) = run_goto_line(&dir, label, &text, query, false);
        let ctx = format!("query={query:?} label={label}");
        assert_cursor(&ctx, &json, expected_line, 0);
        assert_line_visible(&ctx, &json, expected_line);
    }
}

/// **WRAPPED TEXT: the destination still resolves to the correct LOGICAL
/// line**, not thrown off by the wrapped line's inflated VISUAL row count.
/// A single very long first line wraps into several visual rows; the target
/// is a short LOGICAL line well past it.
#[test]
fn goto_line_resolves_the_correct_logical_line_past_a_wrapped_line() {
    let _g = crate::testlock::serial();
    if !adapter_available() {
        eprintln!("skipping goto_line_resolves_...past_a_wrapped_line: no wgpu adapter");
        return;
    }
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl_goto_line_wrap_{}", std::process::id())),
    );
    let long_line = "word ".repeat(200); // wraps into multiple visual rows at any reasonable width
    let mut text = format!("{}\n", long_line.trim_end());
    for i in 1..20 {
        text.push_str(&format!("short line {i}\n"));
    }
    let probe = Buffer::from_str(&text);
    let line_count = probe.line_count();
    // Target logical line 10 (0-based 9), well past the wrapped line 0.
    let target_one_based = 10usize;
    let (json, _buf) = run_goto_line(&dir, "wrapped", &text, &target_one_based.to_string(), false);
    assert_cursor("wrapped", &json, target_one_based - 1, 0);
    // Sanity: the sidecar's OWN line count agrees with the fixture (the
    // wrapped line is still exactly ONE logical line, however many visual
    // rows it occupies).
    assert_eq!(
        json["line_count"].as_u64(),
        Some(line_count as u64),
        "wrapping must not change the LOGICAL line count"
    );
    assert_line_visible("wrapped", &json, target_one_based - 1);
}

/// **UNICODE: multi-byte / grapheme-cluster content on and around the
/// target line must not perturb line counting.** CJK text, combining marks,
/// and multi-codepoint emoji all precede the destination; jumping still
/// lands on the correct LOGICAL line at column 0 (a byte/char/grapheme
/// counting bug would land the cursor on the wrong line or mid-character).
#[test]
fn goto_line_resolves_correctly_amid_multibyte_unicode_content() {
    let _g = crate::testlock::serial();
    if !adapter_available() {
        eprintln!(
            "skipping goto_line_resolves_correctly_amid_multibyte_unicode_content: no wgpu adapter"
        );
        return;
    }
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl_goto_line_unicode_{}", std::process::id())),
    );
    let lines = [
        "日本語のテキスト一行目",          // Japanese, multi-byte
        "emoji line: 👨‍👩‍👧‍👦 family + 🇦🇺 flag", // ZWJ sequences + regional indicators
        "combining: e\u{0301}\u{0301}\u{0301} triple-accented", // stacked combining marks
        "한국어 텍스트",                   // Korean
        "target line — plain ASCII",       // the actual jump destination
        "中文文本在这里",                  // Chinese, after the target
    ];
    let text: String = lines.iter().map(|l| format!("{l}\n")).collect();
    // "target line" is line 5 (1-based).
    let target_one_based = 5usize;
    let (json, _buf) = run_goto_line(&dir, "unicode", &text, &target_one_based.to_string(), false);
    assert_cursor("unicode", &json, target_one_based - 1, 0);
    assert_eq!(
        json["first_lines"][target_one_based - 1].as_str(),
        Some("target line — plain ASCII"),
        "the sidecar's own line readout confirms which LOGICAL line was reached: {json}"
    );
    assert_line_visible("unicode", &json, target_one_based - 1);
}

/// **FOLDED DESTINATION: jumping into a folded region actually unfolds it.**
/// The first heading ("# Alpha") is folded at the cursor's natural start
/// position (0,0); the target line lives inside its hidden body. After the
/// jump, that specific line must no longer be hidden — proven off the real
/// `Buffer::hidden_lines()` state, not merely off the sidecar (a state
/// oracle, not a fold-geometry one).
#[test]
fn goto_line_reveals_the_enclosing_fold() {
    let _g = crate::testlock::serial();
    if !adapter_available() {
        eprintln!("skipping goto_line_reveals_the_enclosing_fold: no wgpu adapter");
        return;
    }
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl_goto_line_fold_{}", std::process::id())),
    );
    // Lines (0-based): 0 "# Alpha", 1 "", 2 alpha one, 3 alpha two,
    // 4 alpha three, 5 "", 6 "# Beta", 7 beta body.
    let text = "# Alpha\n\nalpha one\nalpha two\nalpha three\n\n# Beta\nbeta body\n";
    // Target line 4 (1-based) = "alpha two" (0-based line 3), inside Alpha's
    // folded body.
    let target_one_based = 4usize;
    let target_zero_based = target_one_based - 1;

    // NON-VACUITY: prove the target line is genuinely hidden by the fold
    // BEFORE the jump — else "revealed after" would be trivially true.
    let mut probe = Buffer::from_str(text);
    assert!(probe.toggle_fold_at_cursor().is_some(), "fixture must fold");
    assert!(
        probe.hidden_lines()[target_zero_based],
        "fixture sanity: the target line must start HIDDEN by the fold, \
         else the reveal claim below proves nothing: {:?}",
        probe.hidden_lines()
    );

    let (json, buf) = run_goto_line(&dir, "folded", text, &target_one_based.to_string(), true);
    let hidden = buf.hidden_lines();
    // `hidden_lines()` returns EMPTY once nothing at all is folded (`Buffer::
    // has_folds() == false`) -- not a full false-vector -- so "revealed" is
    // either an empty mask (the whole fold went away) or a non-empty one
    // whose OWN entry for this line reads false.
    let revealed = hidden.is_empty() || !hidden[target_zero_based];
    assert!(
        revealed,
        "the destination line must be REVEALED after the jump: hidden={hidden:?}"
    );
    assert_cursor("folded", &json, target_zero_based, 0);
    assert_line_visible("folded", &json, target_zero_based);
}
