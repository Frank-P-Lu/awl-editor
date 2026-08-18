use super::*;

/// A three-file, one-heading fixture with `line_count` attached -- the same
/// shape [`unified_goto::unified`] uses, plus Go to Line's own fact.
fn goto_with_lines(line_count: usize) -> OverlayState {
    let mut ov = OverlayState::new(
        OverlayKind::Goto,
        vec!["notes/alpha.md".into(), "zebra.txt".into()],
        vec![],
        vec![],
    );
    ov.attach_headings(vec![("Alpha heading".into(), 7)]);
    ov.attach_line_jump(line_count);
    ov
}

#[test]
fn a_digit_query_synthesizes_exactly_one_line_jump_row_on_all() {
    let mut ov = goto_with_lines(500);
    assert_eq!(ov.active_facet_id(), Some("all"));
    let before = ov.item_strings();
    assert!(
        !before.iter().any(|s| s.starts_with("Go to line")),
        "no line-jump row before any digit is typed: {before:?}"
    );
    for c in "42".chars() {
        ov.push(c);
    }
    let items = ov.item_strings();
    assert_eq!(
        items.iter().filter(|s| s.starts_with("Go to line")).count(),
        1,
        "exactly one line-jump row: {items:?}"
    );
    assert!(items.contains(&"Go to line 42".to_string()), "{items:?}");
    assert!(ov.selected_is_line_jump());
    assert_eq!(ov.selected_line(), Some(41), "0-based, one-based 42 -> 41");
}

#[test]
fn a_non_digit_query_never_shows_the_line_jump_row() {
    let mut ov = goto_with_lines(500);
    for c in "alpha".chars() {
        ov.push(c);
    }
    let items = ov.item_strings();
    assert!(
        !items.iter().any(|s| s.starts_with("Go to line")),
        "a name query is not a line number: {items:?}"
    );
    // Nor a query that MIXES digits with letters.
    while !ov.query.is_empty() {
        ov.pop();
    }
    for c in "4a".chars() {
        ov.push(c);
    }
    let items = ov.item_strings();
    assert!(
        !items.iter().any(|s| s.starts_with("Go to line")),
        "a mixed query is not a bare line number: {items:?}"
    );
}

#[test]
fn the_line_jump_row_is_scoped_to_the_all_lens_only() {
    // Files/Headings/Folders/Recent buckets don't claim the line-jump row --
    // it is the flat All home's own numeric companion to Headings, not a
    // member of any of the other typed destination lenses.
    let mut ov = goto_with_lines(500);
    for c in "42".chars() {
        ov.push(c);
    }
    assert!(ov.item_strings().iter().any(|s| s == "Go to line 42"));
    for lens in ["files", "headings", "folders", "recent"] {
        ov.focus_facet_id(lens);
        assert!(
            !ov.item_strings()
                .iter()
                .any(|s| s.starts_with("Go to line")),
            "lens {lens:?} must not show the line-jump row: {:?}",
            ov.item_strings()
        );
        ov.focus_facet_id("all");
    }
}

#[test]
fn no_buffer_known_never_offers_a_line_jump_row() {
    // `attach_line_jump(0)` (or never calling it at all, the bare `new()`
    // default) mirrors `attach_headings`/`attach_folders`'s own opt-in shape:
    // nothing attached, so nothing shows, regardless of what's typed.
    let mut ov = OverlayState::new(OverlayKind::Goto, vec!["a.md".to_string()], vec![], vec![]);
    for c in "42".chars() {
        ov.push(c);
    }
    assert!(
        !ov.item_strings()
            .iter()
            .any(|s| s.starts_with("Go to line")),
        "goto_line_count defaults to 0 -- no buffer, no line-jump row: {:?}",
        ov.item_strings()
    );
}

/// CLAMP LAW: Go to Line clamps out-of-range input to `[1, line_count]`
/// rather than refusing it -- matching `Buffer::line_col_to_char`'s own
/// clamp, the shared jump owner every OTHER caller already gets silently
/// clamped by. The row's own LABEL always names the line actually reached,
/// so a wild input is never a silent surprise: what it promises is what
/// pressing Enter reaches.
#[test]
fn out_of_range_input_clamps_to_the_buffers_own_line_count() {
    let cases: [(usize, &str, usize); 5] = [
        // (line_count, typed query, expected clamped one-based target)
        (100, "1", 1),        // first line
        (100, "50", 50),      // middle
        (100, "100", 100),    // last line
        (100, "0", 1),        // too low -> clamps up to the first line
        (100, "999999", 100), // too high -> clamps down to the last line
    ];
    for (line_count, query, expected) in cases {
        let mut ov = goto_with_lines(line_count);
        for c in query.chars() {
            ov.push(c);
        }
        let items = ov.item_strings();
        let want = format!("Go to line {expected}");
        assert!(
            items.contains(&want),
            "line_count={line_count} query={query:?}: expected {want:?} among {items:?}"
        );
        assert_eq!(
            ov.selected_line(),
            Some(expected - 1),
            "line_count={line_count} query={query:?}: 0-based target"
        );
    }
    // A pathologically huge digit string (overflows u64) must still clamp to
    // the last line rather than panicking or vanishing.
    let mut ov = goto_with_lines(100);
    for c in "99999999999999999999999999999999".chars() {
        ov.push(c);
    }
    assert_eq!(
        ov.selected_line(),
        Some(99),
        "overflow still clamps to the last line"
    );
}

/// PRESERVATION LAW, the line-jump row's own counterpart to
/// `row_meta_laws::goto_heading_rows_keep_their_line_across_refilter`: the
/// row's `line` stays correct across repeated refilters as the query keeps
/// changing (typing more digits, backspacing).
#[test]
fn the_line_jump_row_tracks_the_query_across_repeated_edits() {
    let mut ov = goto_with_lines(500);
    ov.push('1');
    assert_eq!(ov.selected_line(), Some(0));
    ov.push('2');
    assert_eq!(ov.selected_line(), Some(11), "line 12, 0-based 11");
    ov.pop();
    assert_eq!(
        ov.selected_line(),
        Some(0),
        "back to line 1 after backspace"
    );
}

#[test]
fn typed_line_query_emits_jump_to_line_through_the_shared_accept_path() {
    use crate::actions::{ActionCtx, Effect, apply_transition};
    use crate::keymap::Action;

    let run = |mut ov: OverlayState, query: &str| {
        for c in query.chars() {
            ov.push(c);
        }
        let mut journey = Journey::seeded(Some(ov));
        let mut buffer = crate::buffer::Buffer::scratch();
        let mut shift = false;
        let mut zoom = 1.0;
        let mut search = None;
        let mut make_overlay = |_| None;
        let mut browse_to = |_, _| None;
        let mut ctx = ActionCtx {
            buffer: &mut buffer,
            shift_selecting: &mut shift,
            zoom: &mut zoom,
            search: &mut search,
            scroll_page_lines: 1,
            journey: &mut journey,
            make_overlay: &mut make_overlay,
            browse_to: &mut browse_to,
            oracle: None,
        };
        apply_transition(&mut ctx, &Action::Newline, false).primary()
    };

    // A query that names no real file/heading/folder but IS a bare number
    // resolves ONLY to the line-jump row -- the same shared `Effect::JumpToLine`
    // the Headings lens's own accept path emits (unified_goto.rs's
    // `run(unified(), "heading") == Effect::JumpToLine(7)`).
    assert_eq!(run(goto_with_lines(500), "123"), Effect::JumpToLine(122));
}
