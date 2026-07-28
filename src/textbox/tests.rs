//! Unit tests for the shared single-line textbox — the 7-field roster law,
//! the Buffer/TextBox parity table (char step, word motion, word delete) and
//! the multibyte-splice coverage. Carved out of `textbox.rs` into the module
//! root's `#[cfg(test)] mod tests;`; `use super::*` reaches every item exactly
//! as it did inline.

use super::*;
use crate::buffer::Buffer;

// --- A. NO-WILDCARD 7-FIELD ROSTER --------------------------------------

/// THE 7-FIELD ROSTER LAW: every [`TextField::ALL`] member has a home in
/// this NO-WILDCARD match — an 8th field variant fails to COMPILE this
/// test until it is added here, so a future single-line surface can never
/// dodge the "route it through TextBox" sweep silently. Mirrors
/// `OverlayKind::ALL`'s own exhaustive-match law.
#[test]
fn all_seven_fields_have_a_home_no_wildcard() {
    for f in TextField::ALL {
        match f {
            TextField::PickerQuery => {}
            TextField::Rename => {}
            TextField::InsertLink => {}
            TextField::KeepVersion => {}
            TextField::SettingsValue => {}
            TextField::FindQuery => {}
            TextField::ReplaceText => {}
        }
    }
    assert_eq!(
        TextField::ALL.len(),
        7,
        "the roster is exactly the 7 fields item 10 names"
    );
}

// --- B. UNICODE / BUFFER PARITY -----------------------------------------

/// One (text, description) fixture per Unicode class the parity table
/// sweeps: plain ASCII, CJK (multibyte, no combining), a DECOMPOSED
/// combining cluster (base + U+0301 COMBINING ACUTE — two Rust `char`s but
/// ONE visual glyph, so ONE caret step: the char step is a GRAPHEME step,
/// never a scalar step), its PRECOMPOSED twin (U+00E9 — the same `é` as a
/// single scalar, which must behave identically), an emoji ZWJ SEQUENCE and
/// a REGIONAL-INDICATOR flag pair (multi-scalar clusters that fall out of
/// the same rule rather than being special-cased), a lone BMP emoji, and a
/// PUNCTUATION-ADJACENT fixture ending "word, " — its trailing char is
/// whitespace immediately preceded by punctuation, the ONE shape where
/// word MOTION (`word_backward_boundary`: collapse ALL non-word chars —
/// space AND punctuation — before hitting a word char) and word DELETE
/// (`word_delete_backward_boundary`: collapse whitespace only, then ONE
/// token of the resulting class) actually disagree. `"hello world foo"`
/// etc. never place punctuation next to a boundary, so the motion/delete
/// rules coincide on them — a `word_left` mis-wired to the delete
/// boundary would pass this whole table without this fixture (see the
/// module doc's "two word rules" trap).
fn fixtures() -> Vec<(&'static str, &'static str)> {
    vec![
        ("ascii", "hello world foo"),
        ("cjk", "日本語 text 二つ目"),
        ("combining", "cafe\u{0301} au lait\u{0301} noir"),
        ("precomposed", "caf\u{00e9} au lait\u{00e9} noir"),
        ("emoji", "hi 🎉 there 🚀 world"),
        ("zwj", "hi 👨‍👩‍👧‍👦 there 🧑🏽‍🚀 world"),
        ("flags", "go 🇯🇵 then 🇺🇸 home"),
        // First and last cluster are BOTH multi-scalar: an end-only
        // assertion is only as good as the fixture it runs on.
        ("cluster edges", "e\u{0301} mid 🇯🇵"),
        ("punct", "abc, "),
    ]
}

/// PARITY: starting both a [`TextBox`] and a [`Buffer`] on the SAME text
/// with the caret at the SAME char index, an identical sequence of
/// char-motion / word-motion / word-delete ops must land the SAME char
/// index in both — `TextBox` is not a second, silently-diverging
/// implementation of the document's own rules.
#[test]
fn textbox_char_and_word_motion_match_buffer_char_indices() {
    for (label, text) in fixtures() {
        let start = text.chars().count() / 2;
        let mut tb = TextBox::seeded(text);
        tb.set_caret(start);
        let mut buf = Buffer::from_str(text);
        buf.set_cursor(start);
        assert_eq!(
            tb.caret(),
            buf.cursor_char(),
            "{label}: seeded caret parity"
        );

        // Walk forward by word twice, then back by word once, then char-step
        // in both directions — the SAME ops on both models.
        tb.word_right();
        buf.forward_word();
        assert_eq!(tb.caret(), buf.cursor_char(), "{label}: word_right #1");

        tb.word_right();
        buf.forward_word();
        assert_eq!(tb.caret(), buf.cursor_char(), "{label}: word_right #2");

        tb.word_left();
        buf.backward_word();
        assert_eq!(tb.caret(), buf.cursor_char(), "{label}: word_left");

        tb.char_right();
        buf.forward_char();
        assert_eq!(tb.caret(), buf.cursor_char(), "{label}: char_right");

        tb.char_left();
        buf.backward_char();
        assert_eq!(tb.caret(), buf.cursor_char(), "{label}: char_left");
    }
}

/// THE CHARACTER-STEP LAW: walking a fixture end to end one char step at a
/// time visits EXACTLY the extended-grapheme-cluster boundaries — in the
/// document [`Buffer`] and in a [`TextBox`], forward and backward, with no
/// stop inside a cluster and none skipped. A scalar step lands between a
/// base and its combining mark (and inside every emoji ZWJ sequence and
/// flag pair), so it fails here on the decomposed, zwj and flags fixtures
/// while still passing on ascii/cjk/precomposed — which is exactly the
/// shape of the defect: the two spellings of `é` behaved differently.
///
/// The expectation is derived from UAX #29 over the fixture text rather
/// than hand-listed, so a fixture added above is swept without anyone
/// re-deriving its boundaries by eye.
#[test]
fn char_steps_visit_exactly_the_grapheme_boundaries_in_both_models() {
    use unicode_segmentation::UnicodeSegmentation;
    for (label, text) in fixtures() {
        let expected: Vec<usize> = std::iter::once(0)
            .chain(text.graphemes(true).scan(0usize, |acc, g| {
                *acc += g.chars().count();
                Some(*acc)
            }))
            .collect();
        let len = text.chars().count();

        let mut tb = TextBox::seeded(text);
        tb.set_caret(0);
        let mut buf = Buffer::from_str(text);
        buf.set_cursor(0);
        let mut visited = vec![0];
        while tb.caret() < len {
            let from = tb.caret();
            tb.char_right();
            buf.forward_char();
            assert_eq!(tb.caret(), buf.cursor_char(), "{label}: char_right parity");
            assert!(tb.caret() > from, "{label}: char_right stalled at {from}");
            visited.push(tb.caret());
        }
        assert_eq!(visited, expected, "{label}: forward char steps");

        let mut back = vec![len];
        while tb.caret() > 0 {
            let from = tb.caret();
            tb.char_left();
            buf.backward_char();
            assert_eq!(tb.caret(), buf.cursor_char(), "{label}: char_left parity");
            assert!(tb.caret() < from, "{label}: char_left stalled at {from}");
            back.push(tb.caret());
        }
        back.reverse();
        assert_eq!(back, expected, "{label}: backward char steps");
    }
}

/// PARITY: CHARACTER delete removes exactly ONE CLUSTER and lands the same
/// caret in both models — swept over EVERY cluster boundary of every
/// fixture, in both directions.
///
/// The sweep is the point. Deleting only at the ends would prove nothing:
/// every fixture above happens to begin and end on a plain ASCII char, so
/// an end-only version of this test stays green under scalar stepping while
/// the interior combining pairs and ZWJ sequences it names are being cut in
/// half.
#[test]
fn textbox_char_delete_matches_buffer_char_delete() {
    use unicode_segmentation::UnicodeSegmentation;
    for (label, text) in fixtures() {
        // (byte offset, char offset) of every cluster boundary, plus the end.
        let mut bounds = vec![(0usize, 0usize)];
        for (b, g) in text.grapheme_indices(true) {
            bounds.push((b + g.len(), bounds.last().unwrap().1 + g.chars().count()));
        }

        for w in bounds.windows(2) {
            let ((lb, lc), (rb, rc)) = (w[0], w[1]);
            let without = format!("{}{}", &text[..lb], &text[rb..]);

            // Backspace from the cluster's END takes the whole cluster.
            let mut tb = TextBox::seeded(text);
            tb.set_caret(rc);
            let mut buf = Buffer::from_str(text);
            buf.set_cursor(rc);
            tb.delete_back();
            buf.delete_backward();
            assert_eq!(tb.text(), buf.text(), "{label}@{rc}: delete_back parity");
            assert_eq!(
                tb.caret(),
                buf.cursor_char(),
                "{label}@{rc}: delete_back caret parity"
            );
            assert_eq!(
                tb.text(),
                without,
                "{label}@{rc}: backspace took one cluster"
            );
            assert_eq!(tb.caret(), lc, "{label}@{rc}: backspace caret");

            // Forward-delete from its START takes the same whole cluster.
            let mut tb = TextBox::seeded(text);
            tb.set_caret(lc);
            let mut buf = Buffer::from_str(text);
            buf.set_cursor(lc);
            tb.delete_forward();
            buf.delete_forward();
            assert_eq!(tb.text(), buf.text(), "{label}@{lc}: delete_forward parity");
            assert_eq!(
                tb.caret(),
                buf.cursor_char(),
                "{label}@{lc}: delete_forward caret parity"
            );
            assert_eq!(
                tb.text(),
                without,
                "{label}@{lc}: forward-delete took one cluster"
            );
            assert_eq!(tb.caret(), lc, "{label}@{lc}: forward-delete caret");
        }
    }
}

/// PARITY: word DELETE lands the SAME char index (and removes the SAME
/// text) in both models — the DISTINCT rule from word motion above.
#[test]
fn textbox_word_delete_matches_buffer_word_delete() {
    for (label, text) in fixtures() {
        let start = text.chars().count();
        let mut tb = TextBox::seeded(text);
        let mut buf = Buffer::from_str(text);
        buf.set_cursor(start);

        tb.delete_word_back();
        buf.delete_word_backward();
        assert_eq!(
            tb.caret(),
            buf.cursor_char(),
            "{label}: delete_word_back caret"
        );
        assert_eq!(tb.text(), buf.text(), "{label}: delete_word_back text");

        // Forward word-delete from the START of what remains.
        let mut tb2 = TextBox::seeded(tb.text());
        tb2.set_caret(0);
        let mut buf2 = Buffer::from_str(buf.text().as_str());
        buf2.set_cursor(0);
        tb2.delete_word_forward();
        buf2.delete_word_forward();
        assert_eq!(
            tb2.caret(),
            buf2.cursor_char(),
            "{label}: delete_word_forward caret"
        );
        assert_eq!(tb2.text(), buf2.text(), "{label}: delete_word_forward text");
    }
}

/// A multibyte splice never panics and never splits a char: inserting /
/// backspacing / forward-deleting mid-string around CJK, a combining
/// mark, and an emoji all leave `text` valid UTF-8 with the expected
/// content — the CHAR-index-not-byte-offset discipline the module doc
/// names, exercised at every splice site.
#[test]
fn multibyte_splices_never_panic_and_stay_char_correct() {
    // CJK: insert a char between two multibyte glyphs.
    let mut tb = TextBox::seeded("日本語");
    tb.set_caret(1); // between 日 and 本
    tb.insert('X');
    assert_eq!(tb.text(), "日X本語");
    assert_eq!(tb.caret(), 2);

    // Combining mark: backspace removes the WHOLE cluster — the two
    // scalars the reader sees as one `é` — leaving no bare base behind.
    let mut tb = TextBox::seeded("ae\u{0301}"); // 'a', then e + combining acute
    assert_eq!(tb.caret(), 3, "seeded at the end: three scalars");
    tb.delete_back();
    assert_eq!(tb.text(), "a");
    assert_eq!(tb.caret(), 1);

    // Emoji: forward-delete mid-string.
    let mut tb = TextBox::seeded("a🚀b");
    tb.set_caret(1); // just after 'a', before the emoji
    tb.delete_forward();
    assert_eq!(tb.text(), "ab");
    assert_eq!(tb.caret(), 1);
}

// --- misc TextBox unit coverage ------------------------------------------

#[test]
fn new_is_empty_caret_zero() {
    let tb = TextBox::new();
    assert!(tb.is_empty());
    assert_eq!(tb.caret(), 0);
}

#[test]
fn seeded_puts_caret_at_the_end() {
    let tb = TextBox::seeded("abc");
    assert_eq!(tb.caret(), 3);
    assert_eq!(tb.text(), "abc");
}

#[test]
fn set_caret_clamps_never_panics() {
    let mut tb = TextBox::seeded("abc");
    tb.set_caret(999);
    assert_eq!(tb.caret(), 3);
}

#[test]
fn insert_at_mid_caret_splices_not_appends() {
    let mut tb = TextBox::seeded("ac");
    tb.set_caret(1);
    tb.insert('b');
    assert_eq!(tb.text(), "abc");
    assert_eq!(tb.caret(), 2);
}

#[test]
fn delete_back_and_forward_are_boundary_safe() {
    let mut tb = TextBox::new();
    tb.delete_back(); // no-op, no panic
    tb.delete_forward(); // no-op, no panic
    assert_eq!(tb.text(), "");
}

#[test]
fn eq_str_impl_matches_plain_string_compare() {
    let tb = TextBox::seeded("hello");
    assert_eq!(tb, "hello");
    assert_ne!(tb, "world");
}
