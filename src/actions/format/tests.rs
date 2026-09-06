use super::*;

fn blk(kind: BlockKind, text: &str, anchor: Option<usize>, cursor: usize) -> FormatResult {
    block_toggle(kind, text, anchor, cursor)
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
