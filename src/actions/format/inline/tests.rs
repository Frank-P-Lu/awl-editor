use super::*;

fn inl(kind: InlineKind, text: &str, anchor: Option<usize>, cursor: usize) -> FormatResult {
    inline_toggle(kind, text, anchor, cursor)
}

#[test]
fn inline_active_matches_the_toggle_strip_condition() {
    assert!(!inline_active(
        InlineKind::Bold,
        "the quick fox",
        Some(4),
        9
    ));
    assert!(inline_active(
        InlineKind::Bold,
        "the **quick** fox",
        Some(6),
        11
    ));
    assert!(inline_active(InlineKind::Bold, "a **beta** c", Some(2), 10));
    assert!(inline_active(InlineKind::Italic, "a *word* here", None, 4));
}

/// The lit-I-inside-bold TRUTH TABLE: `**` is bold's fence, not two italic
/// markers. The disambiguator ([`content_is_kind`]) reads the real markdown
/// parse so a bare `*` inside a `**` never lights I.
#[test]
fn inline_active_disambiguates_bold_from_italic() {
    assert!(
        inline_active(InlineKind::Italic, "*i*", None, 1),
        "*i* is italic"
    );
    assert!(
        !inline_active(InlineKind::Bold, "*i*", None, 1),
        "*i* is not bold"
    );
    assert!(
        inline_active(InlineKind::Bold, "**b**", None, 2),
        "**b** is bold"
    );
    assert!(
        !inline_active(InlineKind::Italic, "**b**", None, 2),
        "**b** is NOT italic"
    );
    assert!(
        inline_active(InlineKind::Italic, "***bi***", None, 4),
        "***bi*** is italic too"
    );
    assert!(
        inline_active(InlineKind::Bold, "***bi***", None, 4),
        "***bi*** is bold too"
    );
    assert!(
        inline_active(InlineKind::Italic, "**a *i* b**", None, 5),
        "italic on the nested i"
    );
    assert!(
        !inline_active(InlineKind::Italic, "**a *i* b**", None, 2),
        "I dark on plain-bold a"
    );
    assert!(
        inline_active(InlineKind::Bold, "**b**", Some(0), 5),
        "select-all **b** is bold"
    );
    assert!(
        !inline_active(InlineKind::Italic, "**b**", Some(0), 5),
        "select-all **b** not italic"
    );
    assert!(
        !inline_active(InlineKind::Italic, "*i*", None, 0),
        "caret on the * marker: dark"
    );
}

/// Pressing I inside plain bold WRAPS italic inside it (`***bold***`), never
/// strips the bold — the (b) half of the lit-I bug — and the result renders as
/// bold+italic; a second I strips the italic back to plain bold (round-trip).
#[test]
fn toggling_italic_inside_bold_wraps_then_strips_back() {
    let a = inl(InlineKind::Italic, "**bold**", None, 4);
    assert_eq!(
        a.text, "***bold***",
        "I wraps italic inside bold, no bold→italic degrade"
    );
    let rendered = crate::markdown::spans(&a.text);
    assert!(
        rendered
            .iter()
            .any(|(_, k)| *k == crate::markdown::MdKind::BoldItalic),
        "***bold*** renders bold+italic: {rendered:?}"
    );
    let b = inl(InlineKind::Italic, &a.text, a.anchor, a.cursor);
    assert_eq!(
        b.text, "**bold**",
        "second I strips the italic, bold survives"
    );
}

#[test]
fn bold_wraps_the_selection() {
    let r = inl(InlineKind::Bold, "the quick fox", Some(4), 9);
    assert_eq!(r.text, "the **quick** fox");
    assert_eq!((r.anchor, r.cursor), (Some(6), 11));
}

#[test]
fn italic_wraps_the_selection() {
    let r = inl(InlineKind::Italic, "a word here", Some(2), 6);
    assert_eq!(r.text, "a *word* here");
}

#[test]
fn inline_code_wraps_the_selection() {
    let r = inl(InlineKind::InlineCode, "call foo now", Some(5), 8);
    assert_eq!(r.text, "call `foo` now");
}

#[test]
fn highlight_and_strikethrough_wrap() {
    let h = inl(InlineKind::Highlight, "mark me", Some(0), 4);
    assert_eq!(h.text, "==mark== me");
    let s = inl(InlineKind::Strikethrough, "cut me", Some(0), 3);
    assert_eq!(s.text, "~~cut~~ me");
}

#[test]
fn bold_round_trips_via_surrounding_delimiters() {
    let src = "the quick fox";
    let a = inl(InlineKind::Bold, src, Some(4), 9);
    let b = inl(InlineKind::Bold, &a.text, a.anchor, a.cursor);
    assert_eq!(b.text, src, "apply then strip restores the original text");
    assert_eq!(
        (b.anchor, b.cursor),
        (Some(4), 9),
        "selection back over the same text"
    );
}

#[test]
fn every_inline_kind_round_trips() {
    for kind in [
        InlineKind::Bold,
        InlineKind::Italic,
        InlineKind::InlineCode,
        InlineKind::Highlight,
        InlineKind::Strikethrough,
    ] {
        let src = "alpha beta gamma";
        let a = inl(kind, src, Some(6), 10); // "beta"
        let b = inl(kind, &a.text, a.anchor, a.cursor);
        assert_eq!(b.text, src, "{kind:?} must round-trip");
    }
}

#[test]
fn stripping_a_fully_selected_wrapped_span() {
    let text = "a **beta** c";
    let r = inl(InlineKind::Bold, text, Some(2), 10);
    assert_eq!(r.text, "a beta c");
}

#[test]
fn no_selection_wraps_the_word_under_the_caret() {
    let r = inl(InlineKind::Bold, "the quick fox", None, 6);
    assert_eq!(r.text, "the **quick** fox");
    assert_eq!(
        (r.anchor, r.cursor),
        (Some(6), 11),
        "selection over the wrapped word"
    );
}

#[test]
fn no_selection_no_word_inserts_empty_delimiters_with_caret_between() {
    let r = inl(InlineKind::Bold, "one\n\ntwo\n", None, 4);
    assert_eq!(r.text, "one\n****\ntwo\n");
    assert_eq!(r.anchor, None, "empty delimiters leave a bare caret");
    assert_eq!(r.cursor, 6, "caret sits between the two delimiters");
}

#[test]
fn no_selection_empty_delimiters_round_trip() {
    let a = inl(InlineKind::Italic, "one\n\ntwo\n", None, 4);
    assert_eq!(a.text, "one\n**\ntwo\n");
    assert_eq!(a.cursor, 5, "caret sits between the two delimiters");
    let b = inl(InlineKind::Italic, &a.text, a.anchor, a.cursor);
    assert_eq!(
        b.text, "one\n\ntwo\n",
        "toggling empty delimiters off restores the text"
    );
    assert_eq!(b.cursor, 4, "caret lands where the delimiters were");
}

/// The item's own reproduction, and the axis it names an existing regression
/// guard for: a PLAIN two-line selection (no block boundary between the
/// lines) already round-tripped before this fix, because pulldown-cmark
/// parses a raw `\n` inside a code span within one paragraph just fine —
/// only a genuine block boundary breaks the parse. Kept as a named
/// regression guard so a future change to the fallback gate can't silently
/// stop covering the common case via the POSITIVE path.
#[test]
fn inline_code_round_trips_across_a_plain_two_line_selection() {
    let src = "call foo\nbar now"; // "foo\nbar" is chars 5..12
    let a = inl(InlineKind::InlineCode, src, Some(5), 12);
    assert_eq!(a.text, "call `foo\nbar` now", "first press wraps");
    let b = inl(InlineKind::InlineCode, &a.text, a.anchor, a.cursor);
    assert_eq!(
        b.text, src,
        "second press strips back to the exact original"
    );
    assert_eq!((b.anchor, b.cursor), (Some(5), 12), "selection restored");
}

/// "selections spanning two complete lines" + "a selection ending at
/// column zero of the next line" (the item's own phrasing): the selection
/// covers whole lines including the trailing newline.
#[test]
fn inline_code_round_trips_across_two_complete_lines() {
    let src = "alpha\nbeta\ngamma\n"; // "alpha\nbeta\n" is chars 0..11
    let a = inl(InlineKind::InlineCode, src, Some(0), 11);
    assert_eq!(a.text, "`alpha\nbeta\n`gamma\n");
    let b = inl(InlineKind::InlineCode, &a.text, a.anchor, a.cursor);
    assert_eq!(b.text, src, "round-trips to the exact original bytes");
    assert_eq!((b.anchor, b.cursor), (Some(0), 11));
}

/// THE BUG: a wrapped selection whose byte range crosses a real block
/// boundary — a blank line (paragraph break) or a line a list/heading
/// marker turns into its own block — never confirms as a genuine
/// CommonMark code span (the grammar forbids a code span crossing one),
/// so the naive "ask the parser" check used for every other inline kind
/// can't recognize what THIS command itself wrapped. Sweeps the axis (the
/// author's first fix draft only covered the blank-line case) so a
/// different block-interrupting construct can't reopen the same bug.
#[test]
fn inline_code_round_trips_across_a_selection_a_block_boundary_interrupts() {
    let cases: &[(&str, Option<usize>, usize)] = &[
        ("one\n\ntwo\n", Some(0), 8), // blank middle line
        ("one\n- two\n", Some(0), 7), // list line interrupts, no blank line
        ("one\n# two\n", Some(0), 7), // heading line interrupts, no blank line
    ];
    for &(src, anchor, cursor) in cases {
        let a = inl(InlineKind::InlineCode, src, anchor, cursor);
        assert!(
            a.text.starts_with('`') && a.text.contains('`'),
            "{src:?}: first press wraps: {a:?}"
        );
        let b = inl(InlineKind::InlineCode, &a.text, a.anchor, a.cursor);
        assert_eq!(
            b.text, src,
            "{src:?}: second press must strip back to the exact original, not \
             re-wrap (the reported bug: the first press's backticks survive \
             the second press untouched)"
        );
        assert_eq!(
            (b.anchor, b.cursor),
            (anchor, cursor),
            "{src:?}: selection restored to the same logical content"
        );
    }
}

/// Multibyte content spanning a block boundary: char/byte offset
/// conversion must not corrupt the wrap or the strip.
#[test]
fn inline_code_round_trips_multibyte_across_a_blank_line() {
    let src = "héllo\n\nwörld\n"; // "héllo\n\nwörld" is chars 0..12
    let a = inl(InlineKind::InlineCode, src, Some(0), 12);
    assert_eq!(a.text, "`héllo\n\nwörld`\n");
    let b = inl(InlineKind::InlineCode, &a.text, a.anchor, a.cursor);
    assert_eq!(
        b.text, src,
        "multibyte round-trips to the exact original bytes"
    );
    assert_eq!((b.anchor, b.cursor), (Some(0), 12));
}

/// The fallback that makes the block-boundary case strip must stay
/// disjoint from same-line content merely FLANKED by two independent code
/// spans: selecting `" or "` between `` `a` `` and `` `b` `` is
/// structurally Surrounding (a backtick sits immediately on each side),
/// but the plain text between them is not itself code and pressing Code
/// must WRAP it, never merge the two spans into `` `a or b` ``. No `\n`
/// in the span, so the fallback never engages — this stays on the
/// existing positive-match path.
#[test]
fn inline_code_does_not_merge_two_flanking_spans_on_a_single_line() {
    let src = "`a` or `b`";
    let r = inl(InlineKind::InlineCode, src, Some(3), 7); // " or "
    assert_eq!(
        r.text, "`a`` or ``b`",
        "wraps the flanked text in its own pair, never strips/merges"
    );
    assert!(
        !inline_active(InlineKind::InlineCode, src, Some(3), 7),
        "the popover's code button must stay DARK on flanked plain text"
    );
}

/// The fallback's own protective boundary: a literal backtick that is
/// SOURCE TEXT inside a real fenced code block must never be mistaken for
/// this command's own markup, even when the selection crosses a line
/// inside that block.
#[test]
fn inline_code_never_strips_literal_backticks_inside_a_fenced_block() {
    let src = "```\n`raw`\nmore\n```\n";
    assert!(
        !inline_active(InlineKind::InlineCode, src, Some(4), 9),
        "backticks that are fenced-block SOURCE are not this command's markup"
    );
}

// --- The real parser as the oracle ------------------------------------------
//
// Marker presence proves nothing: `**hello world **` carries both fences and is
// not bold, and `` `a `tick` b` `` carries both backticks and is two spans.
// Every law below asks `markdown::spans` — the same parse the renderer draws
// from — what the emitted source actually MEANS.

fn parsed(kind: InlineKind, text: &str) -> Vec<std::ops::Range<usize>> {
    crate::markdown::spans(text)
        .into_iter()
        .filter(|(_, k)| kind_matches_span(kind, *k))
        .map(|(r, _)| r)
        .collect()
}

/// The parse wears `kind` over `payload` and nowhere else. Asked at the payload's
/// two ENDS plus containment rather than as exact coverage, because a nested
/// construct legitimately owns bytes in the middle: `**a `tick` b**` is one bold
/// run reported as two spans with the code span between them.
#[track_caller]
fn wears_exactly(kind: InlineKind, text: &str, payload: std::ops::Range<usize>) {
    let got = parsed(kind, text);
    assert!(
        !got.is_empty(),
        "{kind:?}: the parser reports NO {kind:?} span anywhere in {text:?} — \
         the delimiters are literal characters, not markup"
    );
    assert!(
        got.iter().any(|r| r.contains(&payload.start)),
        "{kind:?}: payload's first byte is outside every {kind:?} span in {text:?}: {got:?}"
    );
    assert!(
        got.iter().any(|r| r.contains(&(payload.end - 1))),
        "{kind:?}: payload's last byte is outside every {kind:?} span in {text:?}: {got:?}"
    );
    assert!(
        got.iter()
            .all(|r| r.start >= payload.start && r.end <= payload.end),
        "{kind:?}: a {kind:?} span escapes the payload {payload:?} in {text:?}: {got:?}"
    );
}

const PROSE_KINDS: [InlineKind; 4] = [
    InlineKind::Bold,
    InlineKind::Italic,
    InlineKind::Highlight,
    InlineKind::Strikethrough,
];

fn char_range(text: &str, from: usize, to: usize) -> std::ops::Range<usize> {
    let byte = |i: usize| {
        text.char_indices()
            .nth(i)
            .map(|(b, _)| b)
            .unwrap_or(text.len())
    };
    byte(from)..byte(to)
}

/// THE 586 BUG. A selection carrying edge whitespace produced `**hello world **`
/// — which is not emphasis at all (a CommonMark closing run flanked by whitespace
/// on its inner side is not right-flanking), so the line showed four literal
/// asterisks around ordinary text. The whitespace must stay in the document,
/// OUTSIDE the delimiters.
///
/// Swept along the axis the report did not name: leading as well as trailing,
/// tabs, a selection running through the line end, both edges at once, a reversed
/// selection, and every prose kind rather than only the reported Bold. Each cell
/// asserts the exact resulting text, the parse over the payload, the selection,
/// and the toggle-back round trip.
#[test]
fn prose_kinds_push_edge_whitespace_outside_their_delimiters() {
    // (document, selection start, selection end, payload start, payload end) —
    // all char indices. The payload is what the delimiters must end up carrying.
    let cases: &[(&str, usize, usize, usize, usize)] = &[
        ("hello world ", 0, 12, 0, 11),            // the item's own fixture
        (" hello world", 0, 12, 1, 12),            // leading
        (" hello world ", 0, 13, 1, 12),           // both
        ("say hello world  again", 3, 17, 4, 15),  // two trailing spaces
        ("alpha\thello world\t\nz", 5, 18, 6, 17), // tabs on both edges
        ("hello world \nnext", 0, 13, 0, 11),      // selection through the line end
        ("a\nhello world \nz", 2, 14, 2, 13),      // payload on an interior line
    ];
    for kind in PROSE_KINDS {
        let Grammar::Prose(d) = kind.grammar() else {
            unreachable!("PROSE_KINDS holds only prose grammars")
        };
        let dl = d.chars().count();
        for &(doc, sa, sc, ps, pe) in cases {
            let take = |from: usize, to: usize| -> String {
                doc.chars().skip(from).take(to - from).collect()
            };
            let (head, payload, tail) = (take(0, ps), take(ps, pe), take(pe, doc.chars().count()));
            let want = format!("{head}{d}{payload}{d}{tail}");

            for (label, a, c) in [("forward", Some(sa), sc), ("reversed", Some(sc), sa)] {
                let r = inl(kind, doc, a, c);
                assert_eq!(
                    r.text, want,
                    "{kind:?} {label} {doc:?}: the whitespace must survive OUTSIDE the delimiters"
                );
                wears_exactly(kind, &r.text, char_range(&r.text, ps + dl, pe + dl));
                assert_eq!(
                    (r.anchor, r.cursor),
                    (Some(ps + dl), pe + dl),
                    "{kind:?} {label} {doc:?}: selection lands on the payload"
                );
                let back = inl(kind, &r.text, r.anchor, r.cursor);
                assert_eq!(
                    back.text, doc,
                    "{kind:?} {label} {doc:?}: toggling off restores the exact original bytes"
                );
                assert_eq!(
                    (back.anchor, back.cursor),
                    (Some(ps), pe),
                    "{kind:?} {label} {doc:?}: selection restored over the same payload"
                );
            }
        }
    }
}

/// The reported line, spelled out once with no table between the reader and the
/// bug: one trailing space, ⌘B, and the result is real bold followed by the
/// space the document already had.
#[test]
fn bold_on_a_line_with_one_trailing_space_is_bold_not_literal_asterisks() {
    let r = inl(InlineKind::Bold, "hello world ", Some(0), 12);
    assert_eq!(r.text, "**hello world** ");
    wears_exactly(InlineKind::Bold, &r.text, 2..13);
}

/// A selection holding nothing but whitespace has no content any emphasis run can
/// wrap, so the command is a calm no-op — never a producer of `** **`, which is
/// four literal asterisks around a space.
#[test]
fn a_whitespace_only_selection_leaves_every_prose_kind_a_calm_no_op() {
    for kind in PROSE_KINDS {
        for doc in [" ", "   ", "\t", " \t ", "a\n \nb"] {
            let (s, e) = if doc.contains('\n') {
                (2, 3)
            } else {
                (0, doc.chars().count())
            };
            let r = inl(kind, doc, Some(s), e);
            assert_eq!(
                r.text, doc,
                "{kind:?} {doc:?}: whitespace-only selection must change nothing"
            );
            assert!(
                !inline_active(kind, doc, Some(s), e),
                "{kind:?} {doc:?}: the popover button stays dark on it too"
            );
        }
    }
}

/// The sibling grammar, and the reason the trim is NOT applied roster-wide: a
/// code span's payload is LITERAL, so its edge whitespace is content and stays
/// inside the fence. Imposing the emphasis rule here would silently drop
/// characters the selection held.
#[test]
fn inline_code_keeps_edge_whitespace_inside_its_own_fence() {
    for (doc, s, e) in [
        ("hello world ", 0usize, 12usize),
        (" hello world", 0, 12),
        (" x ", 0, 3),
        ("say\thello\t", 3, 10),
    ] {
        let payload: String = doc.chars().skip(s).take(e - s).collect();
        let r = inl(InlineKind::InlineCode, doc, Some(s), e);
        let got = parsed(InlineKind::InlineCode, &r.text);
        assert_eq!(
            got.len(),
            1,
            "{doc:?}: exactly one code span: {r:?} {got:?}"
        );
        assert_eq!(
            &r.text[got[0].clone()],
            payload,
            "{doc:?}: the code span's source IS the selected literal, spaces and all"
        );
        let back = inl(InlineKind::InlineCode, &r.text, r.anchor, r.cursor);
        assert_eq!(back.text, doc, "{doc:?}: round-trips to the exact original");
    }
}

/// THE 587 BUG, spelled out: wrapping a selection that already holds a code span
/// in SINGLE backticks collides with the inner pair — the parse came back as two
/// fragments (`a ` and ` b`) with `tick` outside both.
#[test]
fn inline_code_over_an_existing_span_is_one_span_not_two_fragments() {
    let r = inl(InlineKind::InlineCode, "a `tick` b", Some(0), 10);
    assert_eq!(r.text, "``a `tick` b``");
    let got = parsed(InlineKind::InlineCode, &r.text);
    assert_eq!(
        got.len(),
        1,
        "one code span, not two fragments: {:?}",
        got.iter().map(|g| &r.text[g.clone()]).collect::<Vec<_>>()
    );
    assert_eq!(&r.text[got[0].clone()], "a `tick` b");
}

/// The 587 sweep. The fence is a backtick RUN, chosen against what the payload
/// actually contains, and padded where the payload's own edge is a backtick.
/// Every cell asserts the parse: EXACTLY ONE code span, holding the selected
/// literal verbatim and adding nothing but the at-most-one space per side the
/// grammar forces. Marker counting would pass on the broken output.
#[test]
fn inline_code_picks_a_fence_that_clears_every_backtick_run_in_the_selection() {
    let payloads: &[&str] = &[
        "a `tick` b",       // the item's fixture: an interior run of 1
        "a ``t`` b",        // interior run of 2
        "a ```t``` b",      // interior run of 3
        "a ` b `` c ``` d", // runs of 1, 2 and 3 at once
        "`x",               // backtick at the LEADING edge
        "x`",               // backtick at the TRAILING edge
        "`tick` b",         // a whole span sitting at the leading edge
        "a `tick`",         // a whole span sitting at the trailing edge
        "`",                // nothing but a backtick
        "``",
        "a`b",   // interior, no run boundary at an edge
        "plain", // the un-special control
    ];
    for payload in payloads {
        let n = payload.chars().count();
        let r = inl(InlineKind::InlineCode, payload, Some(0), n);
        let got = parsed(InlineKind::InlineCode, &r.text);
        assert_eq!(
            got.len(),
            1,
            "{payload:?} -> {:?}: expected exactly ONE code span, got {:?}",
            r.text,
            got.iter().map(|g| &r.text[g.clone()]).collect::<Vec<_>>()
        );
        let src = &r.text[got[0].clone()];
        assert!(
            src.contains(payload),
            "{payload:?} -> {:?}: the code span's source {src:?} does not hold the selected \
             literal verbatim",
            r.text
        );
        assert!(
            src.len() <= payload.len() + 2
                && src.trim_start_matches(' ').trim_end_matches(' ')
                    == payload.trim_start_matches(' ').trim_end_matches(' '),
            "{payload:?} -> {:?}: the span adds more than the grammar's one space per side: \
             {src:?}",
            r.text
        );
        assert!(
            r.text.ends_with('`'),
            "{payload:?} -> {:?}: the fence closes the line",
            r.text
        );
        let back = inl(InlineKind::InlineCode, &r.text, r.anchor, r.cursor);
        assert_eq!(
            back.text, *payload,
            "{payload:?} -> {:?}: a second press must strip back to the exact original",
            r.text
        );
        assert_eq!(
            (back.anchor, back.cursor),
            if n == 0 { (None, 0) } else { (Some(0), n) },
            "{payload:?}: selection restored over the same literal"
        );
        assert!(
            inline_active(InlineKind::InlineCode, &r.text, r.anchor, r.cursor),
            "{payload:?} -> {:?}: the popover's Code button lights on what the toggle strips",
            r.text
        );
    }
}

/// An EMPTY code selection is the caret case, not a payload case: two backticks
/// with the caret between them, and a second press takes them away again.
#[test]
fn inline_code_on_an_empty_selection_inserts_a_bare_pair() {
    let a = inl(InlineKind::InlineCode, "one\n\ntwo\n", None, 4);
    assert_eq!(a.text, "one\n``\ntwo\n");
    assert_eq!((a.anchor, a.cursor), (None, 5));
    let b = inl(InlineKind::InlineCode, &a.text, a.anchor, a.cursor);
    assert_eq!(b.text, "one\n\ntwo\n");
    assert_eq!((b.anchor, b.cursor), (None, 4));
}

/// A selection that IS an existing code span — fence and all — toggles off in one
/// press, whatever the fence's length. Recognizing only a single backtick left
/// the outer pair of a `` ``y`` `` behind.
#[test]
fn inline_code_strips_a_fully_selected_span_of_any_fence_length() {
    for (doc, want) in [
        ("`y`", "y"),
        ("``y``", "y"),
        ("```y```", "y"),
        ("`` `y` ``", "`y`"),
        ("` `` `", "``"),
    ] {
        let n = doc.chars().count();
        assert!(
            inline_active(InlineKind::InlineCode, doc, Some(0), n),
            "{doc:?}: the Code button lights on a fully selected span"
        );
        let r = inl(InlineKind::InlineCode, doc, Some(0), n);
        assert_eq!(r.text, want, "{doc:?}: one press takes the whole fence off");
    }
}

/// Formatting a selection that already holds a code span, then formatting it
/// again, must land back on the exact original — for every prose kind as well as
/// for code. The nested span owns the bytes in the middle, which is why the
/// "already formatted?" question is asked at the payload's ends.
#[test]
fn every_kind_round_trips_a_selection_holding_a_nested_code_span() {
    for kind in [
        InlineKind::Bold,
        InlineKind::Italic,
        InlineKind::InlineCode,
        InlineKind::Strikethrough,
    ] {
        for doc in ["a `tick` b", "a ``t`` b"] {
            let n = doc.chars().count();
            let a = inl(kind, doc, Some(0), n);
            assert_ne!(a.text, doc, "{kind:?} {doc:?}: the first press wraps");
            let b = inl(kind, &a.text, a.anchor, a.cursor);
            assert_eq!(
                b.text, doc,
                "{kind:?} {doc:?} -> {:?}: the second press must strip, not wrap again",
                a.text
            );
        }
    }
}
