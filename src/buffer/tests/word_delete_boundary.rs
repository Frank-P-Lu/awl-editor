use super::*;

#[test]
fn delete_word_backward_boundary_sweep() {
    // Exhaustive boundary probe for ⌥⌫ (M-Backspace / DeleteWordBackward):
    // from the caret, consume any trailing WHITESPACE run, then delete exactly
    // ONE token — a WORD run if a word char now precedes the caret, else a
    // PUNCTUATION run. Punctuation and a word are DISTINCT classes that never
    // delete together (native macOS Option-Delete, verified 2026-07-22): a
    // caret after a punctuation run keeps the word before it. This pins both
    // the classic `abc def ghi|` -> only "ghi" AND the previously-buggy
    // `abc def...|` -> keeps "abc def" (the word "def" survives the "..." kill;
    // the old rule over-deleted it to "abc "). Each tuple: (text, caret
    // char-index, expected remaining text).
    let cases: &[(&str, usize, &str)] = &[
        ("abc def ghi", 11, "abc def "), // caret after a word -> only "ghi"
        ("abc def ghi   ", 14, "abc def "), // caret after trailing spaces -> ws + word
        ("abc def...", 10, "abc def"),   // caret after a punct run -> ONLY "..." (word survives)
        ("abc ...", 7, "abc "),          // the reported bug: `abc ...|` keeps "abc "
        ("abc", 0, "abc"),               // caret at buffer start -> no-op
        ("abc def", 2, "c def"),         // mid-word "ab|c" -> only "ab"
        ("abc\ndef", 4, "def"),          // caret at start of line 2 (after \n)
        ("abc def  ", 9, "abc "),        // multiple consecutive spaces + word
        ("hello", 5, ""),                // lone word -> empties, not a crash
        ("  hello", 7, "  "),            // leading whitespace preserved
        ("word ", 5, ""),                // word + one trailing space
    ];
    for (text, cur, want) in cases {
        let mut buf = b(text);
        buf.set_cursor(*cur);
        buf.delete_word_backward();
        assert_eq!(
            &buf.text(),
            want,
            "delete_word_backward over-/under-deleted: text={text:?} caret={cur}"
        );
    }
}

#[test]
fn delete_word_forward_boundary_sweep() {
    // Exhaustive boundary probe for ⌥+forward-Delete (DeleteWordForward), the
    // exact mirror of `delete_word_backward_boundary_sweep`: from the caret,
    // consume any LEADING whitespace run, then delete exactly ONE token — a
    // WORD run if a word char now sits AT the caret, else a PUNCTUATION run.
    // Punctuation and a word are DISTINCT classes that never delete together.
    //
    // NON-VACUOUS: the punct-then-word cases (`⎸... abc` -> ` abc`,
    // `⎸...abc` -> `abc`) FAIL under the retired skip-nonword-then-word rule,
    // which skipped the whole "..." (and any spaces) then ate the word too,
    // over-deleting to "" — the exact forward twin of the original backward bug.
    // Each tuple: (text, caret char-index, expected remaining text).
    let cases: &[(&str, usize, &str)] = &[
        ("abc def ghi", 0, " def ghi"), // caret before a word -> only "abc"
        ("   abc def", 0, " def"),      // caret before leading spaces -> ws + word
        ("... abc", 0, " abc"),         // THE REPRO: `⎸... abc` -> keeps " abc"
        ("...abc", 0, "abc"),           // punct run abutting a word -> ONLY "..."
        ("abc ...", 0, " ..."),         // caret before a word -> only "abc" (punct survives)
        ("abc", 3, "abc"),              // caret at buffer end -> no-op
        ("abc def", 1, "a def"),        // mid-word "a|bc" -> only "bc"
        ("abc\ndef", 3, "abc"),         // caret before \n -> newline(ws) + "def"
        ("abc   def", 3, "abc"),        // multiple spaces + word
        ("hello", 0, ""),               // lone word -> empties, not a crash
        ("word   ", 4, "word"),         // caret before pure trailing whitespace -> ws alone
    ];
    for (text, cur, want) in cases {
        let mut buf = b(text);
        buf.set_cursor(*cur);
        buf.delete_word_forward();
        assert_eq!(
            &buf.text(),
            want,
            "delete_word_forward over-/under-deleted: text={text:?} caret={cur}"
        );
        assert_eq!(
            buf.cursor_char(),
            *cur,
            "delete_word_forward moved the caret: text={text:?} caret={cur}"
        );
    }
}

/// The forward/backward twins delete the SAME token count per stroke on a
/// shared fixture — one class per stroke, both directions. `⎸... abc` forward
/// removes only `...`; `... abc⎸` backward removes only `abc`. Pins the
/// symmetry the item-28 fix restored (the old forward rule broke it).
#[test]
fn word_delete_directions_are_symmetric_on_punct() {
    let mut fwd = b("... abc");
    fwd.set_cursor(0);
    fwd.delete_word_forward();
    assert_eq!(fwd.text(), " abc", "forward eats only the punct run");

    let mut bwd = b("... abc");
    bwd.buffer_end();
    bwd.delete_word_backward();
    assert_eq!(bwd.text(), "... ", "backward eats only the word");
}
