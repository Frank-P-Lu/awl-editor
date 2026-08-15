use super::*;

#[test]
fn cursor_line_col_basic() {
    let mut buf = b("hello\nworld");
    assert_eq!(buf.cursor_line_col(), (0, 0));
    buf.buffer_end();
    assert_eq!(buf.cursor_line_col(), (1, 5));
}

#[test]
fn forward_backward_char() {
    let mut buf = b("ab");
    buf.forward_char();
    assert_eq!(buf.cursor_char(), 1);
    buf.forward_char();
    assert_eq!(buf.cursor_char(), 2);
    buf.forward_char(); // clamp at end
    assert_eq!(buf.cursor_char(), 2);
    buf.backward_char();
    assert_eq!(buf.cursor_char(), 1);
    buf.backward_char();
    buf.backward_char(); // clamp at start
    assert_eq!(buf.cursor_char(), 0);
}

#[test]
fn line_start_end() {
    let mut buf = b("hello\nworld");
    buf.next_line(); // now on line 1
    buf.line_end_motion();
    assert_eq!(buf.cursor_line_col(), (1, 5));
    buf.line_start_motion();
    assert_eq!(buf.cursor_line_col(), (1, 0));
}

#[test]
fn vertical_keeps_goal_column() {
    // line 0 long, line 1 short, line 2 long. Goal column should survive
    // the short middle line.
    let mut buf = b("abcdef\nxy\nABCDEF");
    // move to col 5 on line 0
    for _ in 0..5 {
        buf.forward_char();
    }
    assert_eq!(buf.cursor_line_col(), (0, 5));
    buf.next_line(); // line 1 only has 2 chars -> clamp to col 2
    assert_eq!(buf.cursor_line_col(), (1, 2));
    buf.next_line(); // line 2 long -> restore goal col 5
    assert_eq!(buf.cursor_line_col(), (2, 5));
}

#[test]
fn word_motion_forward() {
    let mut buf = b("foo bar.baz");
    buf.forward_word();
    assert_eq!(buf.cursor_char(), 3); // after "foo"
    buf.forward_word();
    assert_eq!(buf.cursor_char(), 7); // after "bar"
    buf.forward_word();
    assert_eq!(buf.cursor_char(), 11); // after "baz"
}

#[test]
fn word_motion_backward() {
    let mut buf = b("foo bar baz");
    buf.buffer_end();
    buf.backward_word();
    assert_eq!(buf.cursor_char(), 8); // start of "baz"
    buf.backward_word();
    assert_eq!(buf.cursor_char(), 4); // start of "bar"
    buf.backward_word();
    assert_eq!(buf.cursor_char(), 0); // start of "foo"
}

#[test]
fn word_motion_skips_leading_punct() {
    let mut buf = b("  ..foo");
    buf.forward_word();
    assert_eq!(buf.cursor_char(), 7); // jumps over spaces+dots to end of foo
}

/// THE WORD-MOTION CLUSTER LAW, swept over every starting index of every entry
/// in [`crate::grapheme::CLUSTER_CORPUS`]: M-f and M-b can never rest between a
/// letter and a mark drawn on top of it.
///
/// The word RULE is unchanged and is NOT what this asserts — `is_word_char` still
/// answers `char::is_alphanumeric`, which is why the walk stops mid-cluster in the
/// first place. What it asserts is that the resulting index is a position that
/// exists on screen. The classes matter and disagree: a combining acute (`Mn`) and
/// a variation selector are non-word, so a FORWARD walk stops after the base; a
/// Devanagari virama is non-word between two word consonants of ONE conjunct
/// cluster, so a BACKWARD walk stops after it. An emoji ZWJ sequence and a flag
/// pair are non-word THROUGHOUT and were already boundary-correct — they are here
/// to keep it that way, not because they were broken.
#[test]
fn word_motion_never_rests_inside_a_grapheme_cluster() {
    for (label, text) in crate::grapheme::CLUSTER_CORPUS.iter().copied() {
        let bounds = crate::grapheme::boundaries_of(text);
        let len = text.chars().count();
        for start in 0..=len {
            let mut buf = b(text);
            buf.set_cursor(start);
            buf.forward_word();
            assert!(
                bounds.contains(&buf.cursor_char()),
                "{label}: M-f from {start} in {text:?} rests at {} — inside a cluster \
                 (boundaries {bounds:?})",
                buf.cursor_char()
            );
            let mut buf = b(text);
            buf.set_cursor(start);
            buf.backward_word();
            assert!(
                bounds.contains(&buf.cursor_char()),
                "{label}: M-b from {start} in {text:?} rests at {} — inside a cluster \
                 (boundaries {bounds:?})",
                buf.cursor_char()
            );
        }
    }
}

/// The same sweep over `samples/mixed-cjk.md`, the real mixed-script fixture.
/// Every cluster in that file is a single char, so this arm is the IDENTITY
/// guard — it proves the snap changed nothing for ordinary CJK/Latin prose over
/// a whole document's worth of positions. The defect classes live in
/// [`crate::grapheme::CLUSTER_CORPUS`] above, which is where non-vacuity comes
/// from.
#[test]
fn word_motion_over_the_mixed_cjk_sample_stays_on_boundaries() {
    let text = std::fs::read_to_string("samples/mixed-cjk.md")
        .expect("tracked samples/mixed-cjk.md must be present");
    let bounds = crate::grapheme::boundaries_of(&text);
    let len = text.chars().count();
    assert!(len > 100, "the sample must be substantial: {len} chars");
    for start in 0..=len {
        let mut buf = b(&text);
        buf.set_cursor(start);
        buf.forward_word();
        let f = buf.cursor_char();
        buf.set_cursor(start);
        buf.backward_word();
        let bk = buf.cursor_char();
        assert!(bounds.contains(&f), "M-f from {start} rests at {f}");
        assert!(bounds.contains(&bk), "M-b from {start} rests at {bk}");
    }
}

/// The reported defect as arithmetic: one M-f over `café` spelled DECOMPOSED
/// lands past the accent, not between the `e` and it — and a second M-f is not
/// needed to escape a position the caret should never have held. The precomposed
/// spelling is the control and must answer identically; the two spellings of the
/// same word cannot behave differently.
#[test]
fn word_motion_crosses_a_decomposed_accent_whole() {
    let mut buf = b("cafe\u{0301} x");
    buf.forward_word();
    assert_eq!(buf.cursor_char(), 5, "M-f lands after the accent");
    let mut buf = b("caf\u{00e9} x");
    buf.forward_word();
    assert_eq!(buf.cursor_char(), 4, "the precomposed control: after the é");

    // A BACKWARD walk stops mid-cluster in a different place: a Devanagari
    // conjunct is one cluster whose virama is non-word between two word
    // consonants, so the char walk halts at the second consonant.
    let mut buf = b("a\u{0915}\u{094d}\u{0915}b");
    buf.set_cursor(5);
    buf.backward_word();
    assert_eq!(buf.cursor_char(), 1, "M-b clears the whole conjunct");
}

/// A DOUBLE-CLICK's selection ends where a caret must be able to sit: the word
/// bounds around a decomposed `café` include the accent, so the caret that lands
/// on the selection's end is not left inside the `é`. Both ends are swept over
/// the whole cluster corpus, from every index — the class walk that finds a word
/// is unchanged, only where it is allowed to stop.
#[test]
fn word_bounds_snap_to_cluster_boundaries() {
    let buf = b("cafe\u{0301} x");
    assert_eq!(buf.word_bounds(2), (0, 5), "the accent belongs to the word");
    for (label, text) in crate::grapheme::CLUSTER_CORPUS.iter().copied() {
        let bounds = crate::grapheme::boundaries_of(text);
        let buf = b(text);
        for idx in 0..=text.chars().count() {
            let (s, e) = buf.word_bounds(idx);
            assert!(
                bounds.contains(&s) && bounds.contains(&e),
                "{label}: word_bounds({idx}) = ({s},{e}) in {text:?} ends inside a cluster \
                 (boundaries {bounds:?})"
            );
            assert!(
                s <= idx.min(e) && idx <= e.max(s),
                "{label}: ({s},{e}) must hold {idx}"
            );
        }
    }
}

/// THE POINTER SEAM, at the buffer's own resolution owner: a hit-tested column
/// interior to a cluster resolves to the NEAREST boundary — the left half of the
/// `é` to its start, the right half to its end — and an ordinary ASCII column is
/// untouched.
#[test]
fn hit_char_snaps_to_the_nearest_cluster_boundary() {
    let buf = b("xe\u{0301}y\nplain");
    assert_eq!(buf.hit_char(0, 0), 0);
    assert_eq!(
        buf.hit_char(0, 1),
        1,
        "the cluster's own start is a boundary"
    );
    assert_eq!(buf.hit_char(0, 2), 3, "a tie inside the é resolves forward");
    assert_eq!(buf.hit_char(0, 3), 3);
    assert_eq!(buf.hit_char(1, 3), 8, "an ASCII column is byte-identical");
    // A three-char cluster has a real nearer side: col 2 is one char past the
    // start and two before the end.
    let buf = b("a\u{0915}\u{094d}\u{0915}b");
    assert_eq!(buf.hit_char(0, 2), 1, "nearer the conjunct's start");
    assert_eq!(buf.hit_char(0, 3), 4, "nearer the conjunct's end");
}

/// A VERTICAL step aims at a pixel column, and the oracle's landing column can
/// name an interior position (a cluster's chars each own a slice of its ink), so
/// the one sink every Up/Down passes through refuses to park there.
#[test]
fn set_cursor_visual_never_parks_inside_a_cluster() {
    let mut buf = b("xe\u{0301}y");
    buf.set_cursor_visual(2, 7.0);
    assert_eq!(
        buf.cursor_char(),
        3,
        "landing inside the é resolves to a boundary"
    );
    buf.set_cursor_visual(1, 3.0);
    assert_eq!(buf.cursor_char(), 1, "a boundary landing is unchanged");
    buf.set_cursor_visual(99, 0.0);
    assert_eq!(buf.cursor_char(), 4, "past the end still clamps to the end");
}
