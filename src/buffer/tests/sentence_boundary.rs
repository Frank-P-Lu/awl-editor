//! Exhaustive boundary-rule sweep for `buffer::sentence` (M-e / M-a / M-k),
//! at the purest reachable seam — direct `Buffer::forward_sentence`/
//! `backward_sentence` calls, no keymap involved. Every case here was
//! recorded from the REAL `unicode-segmentation` UAX #29 output (never
//! hand-guessed), per the module's own "record it" directive. Two tests
//! (`documented_gap_title_abbreviation_before_a_capitalized_name` and
//! `adjacent_initials_stay_one_token_but_the_sentence_after_them_still_breaks`)
//! pin a DOCUMENTED GAP rather than a win — see their own comments.
use super::*;

/// `forward_sentence`/`backward_sentence` from every listed cursor, as char
/// indices — the exhaustive-probe shape `word_delete_boundary.rs` already
/// uses, extended with both directions since sentence motion, unlike a
/// word-delete boundary, is symmetric and reversible.
fn assert_forward(text: &str, cases: &[(usize, usize)]) {
    for &(start, want) in cases {
        let mut buf = b(text);
        buf.set_cursor(start);
        buf.forward_sentence();
        assert_eq!(
            buf.cursor_char(),
            want,
            "forward_sentence({start}) over {text:?}"
        );
    }
}

fn assert_backward(text: &str, cases: &[(usize, usize)]) {
    for &(start, want) in cases {
        let mut buf = b(text);
        buf.set_cursor(start);
        buf.backward_sentence();
        assert_eq!(
            buf.cursor_char(),
            want,
            "backward_sentence({start}) over {text:?}"
        );
    }
}

#[test]
fn ordinary_terminators_period_bang_question() {
    // "Wait, really? Yes! Okay." — three sentences, three terminator kinds.
    let text = "Wait, really? Yes! Okay.";
    assert_forward(
        text,
        &[(0, 14), (13, 14), (14, 19), (18, 19), (19, 24), (24, 24)],
    );
    assert_backward(text, &[(24, 19), (19, 14), (14, 0), (0, 0)]);
}

#[test]
fn closing_quote_rides_along_with_its_terminator() {
    // SB9/SB9a: `Close*` after the terminator doesn't itself break, so the
    // closing `"` right after `.` stays part of the FIRST sentence — the
    // boundary lands after "she said. " too (the whole attribution clause
    // stays glued to the quote), not right after the closing quote.
    assert_forward(
        "\"Sentence.\" she said. Next.",
        &[(0, 22), (10, 22), (22, 27), (27, 27)],
    );
}

#[test]
fn closing_paren_rides_along_with_its_terminator() {
    assert_forward("(Parenthetical.) Next.", &[(0, 17), (17, 22)]);
}

#[test]
fn ellipsis_three_dot_run_does_not_break_mid_run_or_early() {
    // The `...` run awl's own smart-punct concealment paints over (see the
    // module doc) is stored as three literal periods in the rope — this is
    // exactly what motion walks. None of the three periods individually
    // ends the sentence (SB998 default: consecutive ATerm just extends the
    // run), so a caret anywhere in or after the ellipsis still lands on the
    // REAL terminator that follows.
    assert_forward("Wait... really? Yes.", &[(0, 16), (4, 16), (7, 16)]);
    assert_forward(
        "He trailed off... then stopped. Next.",
        &[(0, 32), (16, 32)],
    );
}

#[test]
fn numeric_decimal_point_does_not_break() {
    // SB6: ATerm × Numeric — the `.` in "$3.50" sits between two digits and
    // never counts as a sentence terminator.
    assert_forward("This costs $3.50 today. Next one.", &[(0, 24), (14, 24)]);
}

#[test]
fn cjk_ideographic_full_stop() {
    // U+3002 IDEOGRAPHIC FULL STOP is Sentence_Terminator (SC_STerm) in the
    // crate's own derived tables — no special-casing needed on our side.
    assert_forward("第一句。第二句。", &[(0, 4), (4, 8), (8, 8)]);
    assert_backward("第一句。第二句。", &[(8, 4), (4, 0), (0, 0)]);
}

#[test]
fn buffer_edges_saturate_instead_of_panicking() {
    // No terminator anywhere: the whole buffer is one sentence, so forward
    // saturates at `len` and backward at `0` from every position (SB2 makes
    // both ends of the text a boundary in their own right).
    let text = "One sentence only";
    let len = text.chars().count();
    assert_forward(text, &[(0, len), (len / 2, len), (len, len)]);
    assert_backward(text, &[(len, 0), (len / 2, 0), (0, 0)]);
    // The empty buffer is its own edge case (`cursor >= len` is `0 >= 0`).
    assert_forward("", &[(0, 0)]);
    assert_backward("", &[(0, 0)]);
}

#[test]
fn a_logical_newline_always_forces_a_boundary() {
    // SB4: unconditional break right after Sep/CR/LF — a real rope `\n`
    // (a markdown paragraph break) always ends whatever sentence preceded
    // it, terminator or not.
    assert_forward("One.\nTwo starts after a newline.", &[(0, 5)]);
}

#[test]
fn sentence_spanning_a_soft_wrapped_line_moves_on_logical_text() {
    // `Buffer::forward_sentence`/`backward_sentence` take no `LayoutOracle`
    // (contrast `vertical_motion`/`line_edge_motion` in `actions/motion.rs`,
    // which do) — there is structurally no visual-row concept for them to
    // consult. This buffer is ONE long logical line (no `\n` at all, well
    // past any real page width), so at render time it would soft-wrap
    // across several visual rows; motion must still cross that non-existent
    // "row boundary" as if it weren't there.
    // Capitalized so the run AFTER the real terminator starts with an
    // uppercase letter — SB8 only suppresses a break before a LOWERCASE
    // continuation (see `documented_gap_title_abbreviation_...` below), and
    // a lowercase filler here would trigger exactly that exception, gluing
    // the whole fixture into one sentence and defeating the test's point.
    let filler = "Supercalifragilisticexpialidocious ".repeat(6); // 210 chars, no terminator
    let text = format!("{filler}First real sentence ends here. {filler}Second one.");
    let mut buf = b(&text);
    let first_end = filler.chars().count() + "First real sentence ends here. ".chars().count();
    buf.forward_sentence();
    assert_eq!(
        buf.cursor_char(),
        first_end,
        "forward_sentence crosses the ~216-char run exactly once, landing on the real terminator \
         past it — nothing about the (nonexistent) visual wrap interrupts it"
    );
    buf.buffer_end();
    buf.backward_sentence();
    assert_eq!(
        buf.cursor_char(),
        first_end,
        "backward_sentence from the end walks the ENTIRE trailing filler+'Second one.' run in \
         one hop, past the same ~210-char no-terminator stretch — there is no boundary between \
         the filler and 'Second one.' (no terminator separates them), so both are one sentence, \
         and backward lands on the one real boundary that precedes them"
    );
}

#[test]
fn documented_gap_title_abbreviation_before_a_capitalized_name() {
    // THE HONEST LIMIT of bare UAX #29 (no locale abbreviation dictionary,
    // which `unicode-segmentation` doesn't carry): SB8 only suppresses a
    // break when the text AFTER the terminator's whitespace starts with a
    // LOWERCASE letter (`match_sb8` in the crate returns true only on
    // `SC_Lower`) — so "e.g. the second case" (lowercase "the") stays glued
    // together, correctly, but "Dr. Smith" (capitalized "Smith") does NOT:
    // nothing in the default algorithm distinguishes a title abbreviation
    // before a proper noun from a genuine sentence boundary before one, so
    // it breaks right after "Dr. ". This is UAX #29's own documented
    // tailoring gap, not a bug in this module — pinned here so the gap is
    // recorded rather than silently assumed away.
    assert_forward(
        "Dr. Smith left the building. Second.",
        &[(0, 4)], // breaks after "Dr. " — the known-imperfect case
    );
    // The CONTRASTING case that DOES work, same abbreviation shape, only
    // the case of the following letter differs — proof this is a real rule
    // axis and not a blanket failure:
    assert_forward(
        "e.g. the second case matters. Next.",
        &[(0, 30)], // stays glued through the lowercase continuation
    );
}

#[test]
fn adjacent_initials_stay_one_token_but_the_sentence_after_them_still_breaks() {
    // SB7 (Upper ATerm × Upper, no whitespace between) keeps "U" and "S"
    // from fragmenting into two false one-letter "sentences" — so a caret
    // right after the FIRST period of "U.S." does not stop there, it
    // carries through to the real break past "U.S. " itself. That break
    // still lands before "Government" (capitalized) — the same documented
    // gap as `Dr. Smith` above, just with the internal-fragmentation half of
    // the rule demonstrably working.
    assert_forward(
        "U.S. Government policy changed. Next.",
        &[(0, 5), (1, 5), (2, 5)],
    );
}
