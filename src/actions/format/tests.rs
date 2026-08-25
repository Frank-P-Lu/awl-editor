use super::*;

fn blk(kind: BlockKind, text: &str, anchor: Option<usize>, cursor: usize) -> FormatResult {
    block_toggle(kind, text, anchor, cursor)
}
fn inl(kind: InlineKind, text: &str, anchor: Option<usize>, cursor: usize) -> FormatResult {
    inline_toggle(kind, text, anchor, cursor)
}

#[test]
fn blockquote_applies_to_the_caret_line() {
    let r = blk(BlockKind::Blockquote, "hello\nworld\n", None, 2);
    assert_eq!(r.text, "> hello\nworld\n");
    assert_eq!(r.anchor, None);
    assert_eq!(r.cursor, 4);
}

#[test]
fn bullet_prefixes_every_selected_line() {
    let r = blk(BlockKind::Bullet, "a\nb\nc\n", Some(0), 3);
    assert_eq!(r.text, "- a\n- b\nc\n");
}

#[test]
fn numbered_list_renumbers_on_apply() {
    let r = blk(BlockKind::Numbered, "one\ntwo\nthree\n", Some(0), 11);
    assert_eq!(r.text, "1. one\n2. two\n3. three\n");
}

#[test]
fn task_list_applies_open_checkbox() {
    let r = blk(BlockKind::Task, "todo\n", None, 0);
    assert_eq!(r.text, "- [ ] todo\n");
}

#[test]
fn heading_toggles_one_hash() {
    let r = blk(BlockKind::Heading, "Title\n", None, 0);
    assert_eq!(r.text, "# Title\n");
}

#[test]
fn heading_cycle_walks_off_1_2_3_off() {
    let a = heading_cycle("Title\n", None, 0);
    assert_eq!(a.text, "# Title\n");
    assert_eq!(heading_level(&a.text, a.anchor, a.cursor), 1);
    let b = heading_cycle(&a.text, a.anchor, a.cursor);
    assert_eq!(b.text, "## Title\n");
    let c = heading_cycle(&b.text, b.anchor, b.cursor);
    assert_eq!(c.text, "### Title\n");
    let d = heading_cycle(&c.text, c.anchor, c.cursor);
    assert_eq!(d.text, "Title\n");
    assert_eq!(heading_level(&d.text, d.anchor, d.cursor), 0);
}

#[test]
fn heading_cycle_caret_rides_the_prefix() {
    let a = heading_cycle("Title\n", None, 0);
    assert_eq!(a.cursor, 2, "caret rode the inserted `# ` prefix");
    let b = heading_cycle("Title\n", None, 3);
    assert_eq!(&b.text[..], "# Title\n");
    assert_eq!(b.cursor, 5);
}

#[test]
fn heading_cycle_applies_one_level_to_all_selected_lines() {
    let src = "# one\ntwo\n"; // first line is H1, second is plain
    let r = heading_cycle(src, Some(0), 9);
    assert_eq!(r.text, "## one\n## two\n");
}

#[test]
fn heading_level_reads_the_first_nonempty_line() {
    assert_eq!(heading_level("plain\n", None, 0), 0);
    assert_eq!(heading_level("## sec\n", None, 3), 2);
    assert_eq!(heading_level("#notation\n", None, 2), 0);
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
fn block_prefix_lands_after_indentation() {
    let r = blk(BlockKind::Bullet, "  item\n", None, 4);
    assert_eq!(r.text, "  - item\n");
}

#[test]
fn blockquote_round_trips() {
    let src = "hello\nworld\n";
    let a = blk(BlockKind::Blockquote, src, None, 2);
    let b = blk(BlockKind::Blockquote, &a.text, a.anchor, a.cursor);
    assert_eq!(b.text, src, "apply then strip restores the original text");
}

#[test]
fn bullet_multiline_round_trips() {
    let src = "a\nb\nc\n";
    let a = blk(BlockKind::Bullet, src, Some(0), 3);
    assert_eq!(a.text, "- a\n- b\nc\n");
    let b = blk(BlockKind::Bullet, &a.text, a.anchor, a.cursor);
    assert_eq!(b.text, src);
}

#[test]
fn numbered_list_round_trips() {
    let src = "one\ntwo\nthree\n";
    let a = blk(BlockKind::Numbered, src, Some(0), 11);
    let b = blk(BlockKind::Numbered, &a.text, a.anchor, a.cursor);
    assert_eq!(b.text, src);
}

#[test]
fn task_list_round_trips_and_strips_a_checked_box() {
    let src = "todo\n";
    let a = blk(BlockKind::Task, src, None, 0);
    let b = blk(BlockKind::Task, &a.text, a.anchor, a.cursor);
    assert_eq!(b.text, src);
    let checked = blk(BlockKind::Task, "- [x] done\n", None, 8);
    assert_eq!(checked.text, "done\n");
}

#[test]
fn heading_round_trips() {
    let src = "Title\n";
    let a = blk(BlockKind::Heading, src, None, 0);
    let b = blk(BlockKind::Heading, &a.text, a.anchor, a.cursor);
    assert_eq!(b.text, src);
}

#[test]
fn blank_lines_in_a_selection_are_left_untouched() {
    let src = "a\n\nb\n";
    let r = blk(BlockKind::Bullet, src, Some(0), 4);
    assert_eq!(r.text, "- a\n\n- b\n");
}

#[test]
fn selection_ending_at_col_zero_excludes_the_trailing_line() {
    let r = blk(BlockKind::Bullet, "a\nb\n", Some(0), 2);
    assert_eq!(r.text, "- a\nb\n");
}

#[test]
fn code_block_wraps_then_unwraps() {
    let src = "let x = 1;\nlet y = 2;\n";
    let a = blk(BlockKind::CodeBlock, src, Some(0), 21);
    assert_eq!(a.text, "```\nlet x = 1;\nlet y = 2;\n```\n");
    let b = blk(BlockKind::CodeBlock, &a.text, a.anchor, a.cursor);
    assert_eq!(b.text, src);
}

#[test]
fn code_block_wraps_a_single_line_with_no_selection() {
    let r = blk(BlockKind::CodeBlock, "code\n", None, 2);
    assert_eq!(r.text, "```\ncode\n```\n");
}

#[test]
fn code_block_toggle_recognizes_an_already_tilde_fenced_selection() {
    // `is_fence` (the already-fenced judgment) must recognize `~~~`
    // fences, not just backtick ones -- the renderer treats a tilde
    // fence as a real fence, so the toggle must unwrap it in one step
    // rather than nesting a second (backtick) fence around it.
    let src = "~~~\nlet x = 1;\nlet y = 2;\n~~~\n";
    let r = blk(BlockKind::CodeBlock, src, Some(0), src.chars().count());
    assert_eq!(r.text, "let x = 1;\nlet y = 2;\n");
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
