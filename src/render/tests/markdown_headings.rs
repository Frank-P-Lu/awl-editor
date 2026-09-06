//! Blockquote pull-quote/hanging-mark conceal + the heading-size variable-row
//! machinery (`md_line_scale`, thematic-break row growth, zoom-on-heading
//! caret alignment) -- split out of the former monolithic `render::tests`
//! (2026-07 code-organization pass). See `markdown` for the rest of the
//! markdown-styling suite.

use super::super::*;
use super::pixeldiff;
use super::{headless_dqp, headless_pipeline, view};

/// The blockquote `>` marker CONCEALS off the caret's line (collapses to
/// near-zero advance, so the quote text starts flush at the column edge) and
/// REVEALS at its real advance when the caret lands on the line — the same
/// reveal-on-cursor contract as the heading/emphasis conceal, now generalized
/// to `ConcealKind::Blockquote`.
#[test]
fn blockquote_marker_conceals_off_caret_and_reveals_on_caret() {
    let _w = crate::testlock::serial();
    crate::markdown::set_wysiwyg_on(true);
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping blockquote_marker_conceals_off_caret_and_reveals_on_caret: no wgpu adapter"
        );
        return;
    };
    // "> quoted": the "> " marker is chars 0..2; "quoted" starts at char col 2.
    let text = "> quoted\nprose\n";

    // Caret on line 1 (a DIFFERENT line): line 0's "> " conceals to near-zero,
    // so "quoted" starts flush at ~0.
    let mut off = view(text, 1, 0);
    off.is_markdown = true;
    p.set_view(&off);
    let xs_off = p.visual_rows(0)[0].xs.clone();
    assert!(
        xs_off[2] < 1.0,
        "concealed '> ' collapses, quote text starts flush off-cursor: {xs_off:?}"
    );

    // Caret ON the blockquote line: the "> " reveals at its real advance.
    let mut on = view(text, 0, 0);
    on.is_markdown = true;
    p.set_view(&on);
    let xs_on = p.visual_rows(0)[0].xs.clone();
    assert!(
        xs_on[2] > 5.0,
        "revealed on-cursor: '> ' keeps its real advance (reflow accepted): {xs_on:?}"
    );

    crate::markdown::set_wysiwyg_on(true);
}

/// ONE hanging pull-quote PAIR per contiguous blockquote BLOCK — not per line.
/// Two separate blockquotes yield two blocks; a nested `>>` line stays part of
/// its contiguous block (the markers coalesce), so it never spawns a third
/// block. The cached span is `(first line, last line)`: the two ends the pair
/// hangs from. Asserted via the page/scroll-independent `quote_block_lines`
/// cache.
#[test]
fn blockquote_hanging_mark_is_one_per_block_nested_coalesces() {
    let _w = crate::testlock::serial();
    crate::markdown::set_wysiwyg_on(true);
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping blockquote_hanging_mark_is_one_per_block_nested_coalesces: no wgpu adapter"
        );
        return;
    };
    // Block A: lines 0-1. A blank + a paragraph break the run. Block B: lines
    // 5-6, whose line 6 is a NESTED `>>` (still one contiguous block).
    //  0: "> a"   1: "> b"   2: ""   3: "para"   4: ""   5: "> c"   6: ">> d"
    let text = "> a\n> b\n\npara\n\n> c\n>> d\n";
    let mut v = view(text, 3, 0); // caret on the plain paragraph
    v.is_markdown = true;
    p.set_view(&v);
    assert_eq!(
        p.quote_block_lines(),
        vec![(0, 1), (5, 6)],
        "one block spanning lines 0-1 (a,b) and one spanning 5-6 (c + nested d)"
    );
}

/// The margin PULL-QUOTE marks are PAGE-MODE only (the text-pad gutters they
/// hang in exist only in page mode) — `quote_marks` yields a PAIR per visible
/// block in page mode and NOTHING edge-to-edge (the documented non-page
/// fallback: the concealed marker alone). Also present regardless of the caret
/// (a block affordance, not reveal-on-cursor).
#[test]
fn blockquote_pull_quote_mark_page_mode_only() {
    let _w = crate::testlock::serial();
    let _g = crate::testlock::serial();
    crate::markdown::set_wysiwyg_on(true);
    let was_page = crate::page::page_on();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping blockquote_pull_quote_mark_page_mode_only: no wgpu adapter");
        crate::page::set_page_on(was_page);
        return;
    };
    let text = "> a\n> b\n\npara\n\n> c\n";
    let mut v = view(text, 0, 0); // caret INSIDE block A — mark still present
    v.is_markdown = true;
    p.set_view(&v);

    crate::page::set_page_on(true);
    let marks = p.quote_marks();
    assert_eq!(
        marks.len(),
        4,
        "page mode: an OPEN + CLOSE pair per visible block (two blocks), present \
         even with the caret in a block: {marks:?}"
    );
    assert_eq!(
        marks
            .iter()
            .filter(|(_, side)| *side == crate::render::rects::QuoteSide::Open)
            .count(),
        2,
        "one opening mark per block, never two: {marks:?}"
    );
    assert_eq!(
        marks
            .iter()
            .filter(|(_, side)| *side == crate::render::rects::QuoteSide::Close)
            .count(),
        2,
        "one closing mark per block — the 66 is followed by its 99: {marks:?}"
    );

    crate::page::set_page_on(false);
    assert!(
        p.quote_marks().is_empty(),
        "edge-to-edge (non-page): no margin, so no hanging mark (concealed marker only)"
    );

    crate::page::set_page_on(was_page);
}

/// DETERMINISM GUARD: a doc with no blockquote produces NO pull-quote marks and
/// NO blockquote conceal spans — nothing here touches a non-blockquote render.
#[test]
fn non_blockquote_doc_has_no_quote_marks() {
    let _w = crate::testlock::serial();
    let _g = crate::testlock::serial();
    crate::markdown::set_wysiwyg_on(true);
    let was_page = crate::page::page_on();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping non_blockquote_doc_has_no_quote_marks: no wgpu adapter");
        crate::page::set_page_on(was_page);
        return;
    };
    let text = "# Title\nplain prose with a > not-a-quote inline\n";
    let mut v = view(text, 0, 0);
    v.is_markdown = true;
    p.set_view(&v);
    crate::page::set_page_on(true);
    assert!(
        p.quote_block_lines().is_empty(),
        "no blockquote blocks in a plain doc"
    );
    assert!(
        p.quote_marks().is_empty(),
        "no pull-quote marks in a plain doc"
    );
    crate::page::set_page_on(was_page);
}

/// FIX (2026-07-09): the hanging pull-quote DROP-CAP mark must live INSIDE the
/// writing column (in the quote block's own left text-pad gutter), NOT out in the
/// left margin where it collided with the now-default-on OUTLINE. The pure
/// placement law (`geometry::pull_quote_left`): the mark's RIGHT edge
/// clears the quote text's left edge, and its LEFT edge never spills back out of
/// the page into the margin.
#[test]
fn pull_quote_hangs_in_the_column_gutter_never_the_margin() {
    use geometry::pull_quote_left;
    // Typical page-mode geometry: page column at 240, text inset to 280, a small
    // clearance gap, a narrow mark that fits the gutter.
    let (column_left, text_left, gap, mark_w) = (240.0_f32, 280.0_f32, 4.0_f32, 22.0_f32);
    let x = pull_quote_left(column_left, text_left, gap, mark_w);
    assert!(
        x >= column_left - 1e-4,
        "mark left never past the page edge into the outline's margin: {x} < {column_left}"
    );
    assert!(
        x + mark_w <= text_left - gap + 1e-4,
        "mark right edge clears the quote text (a `gap` shy of `text_left`): {} vs {text_left}",
        x + mark_w
    );
    assert!(
        x > column_left + 1e-4,
        "a mark that fits the gutter hangs shy of the text, not flush at the page edge: {x}"
    );
    // An OVER-WIDE mark (wider than the gutter) clamps to `column_left` — it stays
    // INSIDE the page (out of the margin) rather than spilling left into the
    // outline; the accepted cost is a slight overlap with the text, never a
    // collision with the margin.
    let wide = pull_quote_left(column_left, text_left, gap, 100.0);
    assert!(
        (wide - column_left).abs() < 1e-4,
        "an over-wide mark clamps to the page edge, never the margin: {wide}"
    );
}

/// THE MIRROR LAW: the CLOSING pull-quote mark sits exactly as far from the
/// writing column's RIGHT edge as the opening one sits from its LEFT edge —
/// `geometry::pull_quote_right` is `pull_quote_left` reflected, so the pair
/// reads as one ornament bracketing the block rather than as two marks with
/// unrelated placements. Swept over the whole mark-width axis (including the
/// over-wide regime where BOTH ends clamp flush to their page edge) and over
/// several column geometries and clearances, because a mirror that only holds
/// at one width is not a mirror. Pure, so no GPU is needed.
#[test]
fn pull_quote_close_mirrors_open_about_the_column() {
    use geometry::{pull_quote_left, pull_quote_right};
    let mut checked = 0usize;
    for &(column_left, column_width, pad) in &[
        (240.0_f32, 400.0_f32, 40.0_f32),
        (0.0, 1200.0, 96.0),
        (17.5, 333.25, 12.5),
        (600.0, 220.0, 20.0),
    ] {
        let column_right = column_left + column_width;
        let text_left = column_left + pad;
        let text_right = column_right - pad;
        for gap_step in 0..6 {
            let gap = gap_step as f32 * 2.5;
            for w_step in 1..=60 {
                let mark_w = w_step as f32 * 2.5; // 2.5 .. 150, past `pad` on every geometry
                let l = pull_quote_left(column_left, text_left, gap, mark_w);
                let r = pull_quote_right(column_right, text_right, gap, mark_w);
                let inset_left = l - column_left;
                let inset_right = column_right - (r + mark_w);
                assert!(
                    (inset_left - inset_right).abs() < 1e-3,
                    "asymmetric pair: column {column_left}..{column_right}, pad {pad}, \
                     gap {gap}, mark_w {mark_w} — opening inset {inset_left}, \
                     closing inset {inset_right}"
                );
                assert!(
                    r + mark_w <= column_right + 1e-3,
                    "closing mark spills out of the page into the right margin: \
                     right edge {} past {column_right} (mark_w {mark_w})",
                    r + mark_w
                );
                if mark_w + gap <= pad {
                    assert!(
                        r >= text_right + gap - 1e-3,
                        "a mark that FITS the gutter must clear the text's own wrap \
                         edge: {r} < {text_right} + {gap} (mark_w {mark_w})"
                    );
                }
                checked += 1;
            }
        }
    }
    assert!(
        checked >= 4 * 6 * 60,
        "the mirror sweep visited only {checked} cells — the loop bounds moved"
    );
}

/// The CLOSING mark hangs from the block's LAST VISUAL row, not from the last
/// logical line's FIRST row. A one-logical-line blockquote long enough to soft
/// wrap is the case that separates the two: its opening mark sits on row 0 and
/// its closing mark a whole row-height lower, on the wrapped tail.
#[test]
fn pull_quote_close_hangs_from_the_last_wrapped_row() {
    let _w = crate::testlock::serial();
    let was_page = crate::page::page_on();
    crate::markdown::set_wysiwyg_on(true);
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping pull_quote_close_hangs_from_the_last_wrapped_row: no wgpu adapter");
        crate::page::set_page_on(was_page);
        return;
    };
    crate::page::set_page_on(true);
    let long = format!("> {}\n\ntail\n", "wrapping quote body ".repeat(40));
    let mut v = view(&long, 2, 0); // caret off the quote line
    v.is_markdown = true;
    p.set_view(&v);
    let rows = p.visual_rows(0).len();
    assert!(
        rows > 1,
        "fixture failed to soft-wrap: the blockquote occupies {rows} visual row(s), \
         so this law cannot see the difference it names"
    );
    let marks = p.quote_marks();
    assert_eq!(marks.len(), 2, "one pair for the one block: {marks:?}");
    let open = marks
        .iter()
        .find(|(_, s)| *s == crate::render::rects::QuoteSide::Open)
        .expect("an opening mark")
        .0;
    let close = marks
        .iter()
        .find(|(_, s)| *s == crate::render::rects::QuoteSide::Close)
        .expect("a closing mark")
        .0;
    assert!(
        close > open,
        "the closing mark sits on the block's LAST wrapped row, below the opening \
         one: open {open}, close {close} over {rows} rows"
    );
    let last_row_top = p.doc_top() + p.visual_rows(0).last().expect("rows").line_top;
    assert!(
        (close - last_row_top).abs() < 0.5,
        "the closing mark hangs from the last visual row's top {last_row_top}, not {close}"
    );
    crate::page::set_page_on(was_page);
}

/// THE DEGENERATE CASE: a ONE-LINE blockquote still shows a pair. Both marks
/// share the row top — they are told apart by x (opposite gutters), never by y
/// — so a law that distinguishes them by row would report a single mark here.
#[test]
fn one_line_blockquote_still_shows_a_pair() {
    let _w = crate::testlock::serial();
    let was_page = crate::page::page_on();
    crate::markdown::set_wysiwyg_on(true);
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping one_line_blockquote_still_shows_a_pair: no wgpu adapter");
        crate::page::set_page_on(was_page);
        return;
    };
    crate::page::set_page_on(true);
    let mut v = view("> alone\n\ntail\n", 2, 0);
    v.is_markdown = true;
    p.set_view(&v);
    assert_eq!(
        p.quote_block_lines(),
        vec![(0, 0)],
        "a one-line block spans one line: its first and last line coincide"
    );
    let marks = p.quote_marks();
    assert_eq!(
        marks.len(),
        2,
        "a one-line block still draws BOTH marks: {marks:?}"
    );
    assert_eq!(
        marks[0].0, marks[1].0,
        "both marks hang from the one row: {marks:?}"
    );
    assert_ne!(marks[0].1, marks[1].1, "one open, one close: {marks:?}");
    crate::page::set_page_on(was_page);
}

/// THE TWO ENDS ARE CULLED INDEPENDENTLY. A block taller than the viewport has
/// one end on screen and one far off it, and the pair is emitted per END, not
/// per block — so scrolling from the head of such a quote to its foot trades an
/// opening mark for a closing one rather than showing both or neither. A cull
/// that keyed the CLOSING mark off the OPENING one's visibility (the natural
/// shape when a block is treated as a single ornament) would draw nothing at the
/// foot of a long quote, which is exactly where the reader needs the close.
#[test]
fn long_block_culls_its_two_ends_independently() {
    let _w = crate::testlock::serial();
    let was_page = crate::page::page_on();
    crate::markdown::set_wysiwyg_on(true);
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping long_block_culls_its_two_ends_independently: no wgpu adapter");
        crate::page::set_page_on(was_page);
        return;
    };
    crate::page::set_page_on(true);
    // Far taller than the 800px viewport plus its generous cull margin, so each
    // end is unambiguously outside the other's band.
    const QUOTED: usize = 120;
    let doc = format!("{}\ntail\n", "> a quoted line\n".repeat(QUOTED));
    let mut v = view(&doc, QUOTED + 1, 0);
    v.is_markdown = true;
    p.set_view(&v);
    assert_eq!(
        p.quote_block_lines(),
        vec![(0, QUOTED - 1)],
        "the fixture is ONE block spanning every quoted line"
    );

    let sides = |p: &crate::render::TextPipeline| -> Vec<crate::render::rects::QuoteSide> {
        p.quote_marks().iter().map(|&(_, s)| s).collect()
    };
    assert_eq!(
        sides(&p),
        vec![crate::render::rects::QuoteSide::Open],
        "at the HEAD of a viewport-tall quote only the opening mark is on screen"
    );

    // Park the viewport on the block's foot; the head is now far above it.
    v.scroll = crate::render::ScrollPos::at_row(QUOTED - 2);
    p.set_view(&v);
    assert_eq!(
        sides(&p),
        vec![crate::render::rects::QuoteSide::Close],
        "at the FOOT of the same quote only the closing mark is on screen — the \
         close is culled on its OWN row, not on the opening mark's"
    );
    crate::page::set_page_on(was_page);
}

#[test]
fn md_line_scale_keys_off_leading_hash_count() {
    use crate::markdown::heading_scale;
    // Non-markdown buffer: always body size, whatever the text.
    assert_eq!(md_line_scale("# heading", false, true), 1.0);
    // Size by the leading-hash COUNT (valid ATX or not). `confirmed_rule` is
    // irrelevant here — the heading branch always wins first.
    assert_eq!(md_line_scale("# h1", true, true), heading_scale(1));
    assert_eq!(md_line_scale("## h2", true, true), heading_scale(2));
    assert_eq!(md_line_scale("### h3", true, true), heading_scale(3));
    assert_eq!(md_line_scale("###### deep", true, true), heading_scale(3)); // 4+ clamps
    // Grows the instant you type `#`, before the space + title.
    assert_eq!(md_line_scale("#", true, true), heading_scale(1));
    assert_eq!(md_line_scale("#nospace", true, true), heading_scale(1));
    assert_eq!(md_line_scale("  ## indented", true, true), heading_scale(2));
    // A `#` that is NOT the line's leading run is ignored (body size).
    assert_eq!(md_line_scale("not a #heading", true, true), 1.0);
    assert_eq!(md_line_scale("plain prose", true, true), 1.0);
}

/// **THE RENDER SIZE HALF AND THE FOLD HALF AGREE — BY CONSTRUCTION.**
/// `md_line_heading_level` delegates to `crate::fold::heading_level` (its one
/// owner) rather than keeping a second copy, so a divergence is impossible
/// without deliberately undoing that delegation. This law exists to catch
/// exactly that regression: if a future edit reintroduces an independent
/// body here, this goes red on the first line where the two heuristics
/// disagree.
#[test]
fn heading_level_agrees_with_folds_own_heading_level_over_a_line_corpus() {
    let corpus = [
        ("# h1", true),
        ("## h2", true),
        ("### h3", true),
        ("###### deep", true),
        ("#", true),
        ("#nospace", true),
        ("  ## indented", true),
        ("not a #heading", true),
        ("plain prose", true),
        ("", true),
        ("# h1", false),
        ("#nospace", false),
    ];
    for (line, md) in corpus {
        assert_eq!(
            crate::render::md_line_heading_level(line, md),
            crate::fold::heading_level(line, md),
            "{line:?} (md={md}): render size half and fold half must agree"
        );
    }
}

#[test]
fn md_line_scale_grows_thematic_break_rows_to_the_active_worlds_ornament_scale() {
    // A thematic break grows its row to the ACTIVE WORLD'S per-world ornament scale
    // (no longer a single global rung), so the tall row centers the bigger fleuron
    // — and by the SAME value `prepare_ornaments` shapes the glyph at. md_line_scale
    // reads `theme::active().ornament_scale`, so hold the theme lock while flipping.
    let _t = crate::testlock::serial();

    // A world with its own measured ornament scale (Currawong): every break
    // syntax grows to ITS scale, GIVEN a confirmed real Rule (the ground-truth
    // gate `md_line_scale` requires on top of the raw single-line scan — see
    // its doc comment). Each world now carries its own measured ink-height-
    // equalized value rather than a shared tier constant, so the sanity pin
    // here is just
    // that `md_line_scale` reads back exactly the active world's own value.
    crate::theme::set_active_by_name("Currawong").unwrap();
    let geo = crate::theme::active().ornament_scale;
    assert_eq!(md_line_scale("---", true, true), geo);
    assert_eq!(md_line_scale("***", true, true), geo);
    assert_eq!(md_line_scale("___", true, true), geo);
    assert_eq!(md_line_scale("- - -", true, true), geo);

    // A DIFFERENT world (Mopoke): the SAME break lines grow to ITS OWN
    // measured scale — proof the row height is per-world, not a fixed rung.
    crate::theme::set_active_by_name("Mopoke").unwrap();
    let ornate = crate::theme::active().ornament_scale;
    assert_ne!(
        ornate, geo,
        "two different worlds must carry two different measured scales"
    );
    assert_eq!(md_line_scale("---", true, true), ornate);
    assert_eq!(md_line_scale("***", true, true), ornate);

    // Gated to markdown; a non-md buffer keeps the break at body size (per-world
    // scale never applies), and a dash LIST item (not a break) stays body size.
    assert_eq!(md_line_scale("---", false, true), 1.0);
    assert_eq!(md_line_scale("- item", true, true), 1.0);

    crate::theme::set_active(crate::theme::DEFAULT_THEME);
}

/// `is_thematic_break("---")` is a single-line scan: it cannot see whether THIS
/// `---` sits somewhere the real parse would never call a break — e.g. inside a
/// fenced code block's body, where the bytes are `MdKind::Code`, never
/// `MdKind::Rule`, regardless of what they look like. `md_line_scale` requires
/// `confirmed_rule` (the REAL parse's ground truth, via `line_has_rule_span`) on
/// TOP of the raw scan before it grows the row, so a bare-scan false positive can
/// never reserve space for an ornament the real pipeline will not draw. (A dash
/// underline directly under a paragraph is NOT such a case any more — see
/// `setext_dash_underline_draws_rule_real_pipeline`
/// below: that one now confirms too, and grows.)
#[test]
fn md_line_scale_does_not_grow_an_unconfirmed_dash_line() {
    let _t = crate::testlock::serial();
    crate::theme::set_active_by_name("Currawong").unwrap();
    let geo = crate::theme::active().ornament_scale;
    // The raw scan alone says "grow" (matches the SAME `---` line the previous
    // test proves DOES grow when confirmed) — but unconfirmed (e.g. this exact
    // line living inside a fenced code block), it must not.
    assert!(
        crate::markdown::is_thematic_break("---"),
        "sanity: the raw scan reads '---' as a break regardless of context"
    );
    assert_eq!(
        md_line_scale("---", true, false),
        1.0,
        "an UNCONFIRMED dash line (e.g. one living inside a fenced code block) must \
         stay body size, not {geo}"
    );
    crate::theme::set_active(crate::theme::DEFAULT_THEME);
}

#[test]
fn heading_rows_are_taller_and_gated_to_markdown() {
    // The row-count assertion assumes NOTHING wraps, which folds the page
    // globals (column width); hold the page lock (page.rs:95-99).
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping heading_rows_are_taller_and_gated_to_markdown: no wgpu adapter");
        return;
    };
    // line0 = h1, line1 blank, line2/3 body, line4 trailing empty.
    let text = "# Big\n\nbody one\nbody two\n";

    // MARKDOWN: the heading row (row 0) is taller than a body row (row 2) by
    // ~heading_scale(1) * heading_row_lead(1) — the SIZE ladder's own factor
    // further widened by the theme-QA round's row-height lead (the extra
    // breathing room decoupled from font size; see that fn's doc comment).
    let mut md = view(text, 0, 0);
    md.is_markdown = true;
    p.set_view(&md);
    assert_eq!(
        p.total_visual_rows(),
        5,
        "no wrap => one row per logical line"
    );
    let h1 = p.row_height_px(0);
    let body = p.row_height_px(2);
    assert!(body > 0.0);
    let ratio = h1 / body;
    let want = crate::markdown::heading_scale(1) * crate::markdown::heading_row_lead(1);
    assert!(
        (ratio - want).abs() < 0.05,
        "h1 row should be ~{want}x a body row, got {ratio} ({h1}/{body})"
    );
    // Body rows are uniform among themselves.
    assert!((p.row_height_px(2) - p.row_height_px(3)).abs() < 0.01);
    let md_doc_h = p.total_doc_height();

    // NON-MARKDOWN: the SAME text shapes with uniform rows (no heading growth),
    // proving the size is gated like every other md effect.
    let mut plain = view(text, 0, 0);
    plain.is_markdown = false;
    p.set_view(&plain);
    assert!(
        (p.row_height_px(0) - p.row_height_px(2)).abs() < 0.01,
        "a non-markdown buffer must keep every row a uniform height"
    );
    assert!(
        md_doc_h > p.total_doc_height(),
        "the heading must make the markdown document taller in pixels"
    );

    // Non-wrapped: visual_row_of still equals the logical line, so cursor-follow
    // is unchanged when nothing wraps even though rows differ in height.
    p.set_view(&md);
    assert_eq!(p.visual_row_of(2, 0), 2);
}

/// A setext heading's TITLE (a paragraph underlined by `===`/`---`) must read as
/// PLAIN BODY TEXT — same size (ink), same row height as the surrounding prose —
/// while an ATX `#` heading still grows, over the REAL render pipeline (not a
/// mirror of the arithmetic `md_line_scale` already proves in isolation above).
/// The setext UNDERLINE is a DIFFERENT question: DECIDED, `---` always draws as
/// the rule whatever precedes it (awl has no setext headings), so a qualifying
/// dash underline (3-or-more run) must grow and draw the ornament exactly like a
/// REAL thematic break — never fall back to a suppressed setext title's
/// body-height row. Row HEIGHT is the direct proxy for both claims; for the
/// title's SIZE, it and the body line share the EXACT SAME TEXT ("Same Words
/// Here") so their shaped `xs` (per-glyph pixel positions) are comparable
/// glyph-for-glyph even under a PROPORTIONAL font (an `xs`-step diff alone would
/// just measure "S" vs "B", not scale).
#[test]
fn setext_dash_underline_draws_rule_real_pipeline() {
    let _t = crate::testlock::serial();
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping setext_dash_underline_draws_rule_real_pipeline: no wgpu adapter");
        return;
    };
    // line: 0 ATX h1, 1 blank, 2 body ("Same Words Here"), 3 blank, 4 setext
    // title (the IDENTICAL text "Same Words Here"), 5 setext underline, 6
    // blank, 7 body, 8 blank, 9 REAL rule, 10 blank, 11 body.
    let text = "# ATX Heading\n\nSame Words Here\n\nSame Words Here\n------------\n\n\
                Body two\n\n---\n\nBody three\n";
    let mut v = view(text, 0, 0);
    v.is_markdown = true;
    p.set_view(&v);

    let body_h = p.row_height_px(2);
    assert!(body_h > 0.0);
    let row_width = |p: &TextPipeline, row: usize| {
        let xs = p.visual_rows(row)[0].xs.clone();
        *xs.last().unwrap()
    };
    let body_w = row_width(&p, 2);

    // The ATX heading still grows: taller row, and its (differently-worded, but
    // longer) row is wider than body's identical-length comparison would allow
    // at body scale — checked precisely via row height, the unconfounded proxy.
    assert!(
        p.row_height_px(0) > body_h + 1.0,
        "an ATX heading's row must still grow: {} vs body {body_h}",
        p.row_height_px(0)
    );

    // The setext TITLE (row 4) matches body exactly — no promoted size. Same
    // text on both rows, so the shaped LAST-glyph x-position (proportional to
    // the whole run's width) must match glyph-for-glyph, not just in row height.
    assert!(
        (p.row_height_px(4) - body_h).abs() < 0.01,
        "a setext title's row must equal a body row: {} vs {body_h}",
        p.row_height_px(4)
    );
    assert!(
        (row_width(&p, 4) - body_w).abs() < 0.01,
        "a setext title's shaped width (identical text to the body row) must \
         equal body's: {} vs {body_w}",
        row_width(&p, 4)
    );

    // The setext UNDERLINE (row 5, "------------") now DRAWS AS THE RULE: DECIDED,
    // `---` always renders as the rule whatever precedes it — `spans()`'
    // `Tag::Heading` arm promotes a qualifying dash underline to a real
    // `MdKind::Rule`, so it must grow exactly like the standalone break below,
    // never fall back to a suppressed setext title's body-height row.
    assert!(
        p.row_height_px(5) > body_h + 1.0,
        "a setext dash underline must draw as the rule and grow its row: {} vs body {body_h}",
        p.row_height_px(5)
    );

    // The REAL thematic break (row 9, "---" alone with blank lines on both
    // sides) still grows — the ornament genuinely draws there.
    assert!(
        p.row_height_px(9) > body_h + 1.0,
        "a REAL thematic break's row must still grow: {} vs body {body_h}",
        p.row_height_px(9)
    );
}

/// PER-WORLD HEADING WEIGHT — the DISTINGUISHABILITY law: in EVERY world,
/// under its OWN proposed `heading_bold` bit (no force, the shipped data),
/// each heading level stays measurably distinct from body at the shaped-pixel
/// outcome — the Ladder-J rungs (1.6/1.3/1.15) must survive as three strictly
/// descending row heights above body, whatever the world's face or weight bit
/// does. Outcome, not mechanism: the assertion is over the real pipeline's
/// per-row pixel heights, never the consts (a future ladder retune that
/// collapses two rungs — or a face whose metrics swallow a step — fails here,
/// not in a mirror of the arithmetic).
#[test]
fn heading_levels_stay_measurably_distinct_from_body_in_every_world() {
    // Row-height math folds the page wrap globals AND the active theme —
    // hold both locks (theme, then page).
    let _t = crate::testlock::serial();
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!(
            "skipping heading_levels_stay_measurably_distinct_from_body_in_every_world: no wgpu adapter"
        );
        return;
    };
    // h1 / h2 / h3 / body, one per line; caret parked on the body line so the
    // heading rows sit in their settled (marker-concealed) state.
    let text = "# word\n## word\n### word\nword\n";
    for t in crate::theme::THEMES.iter() {
        crate::theme::set_active_by_name(t.name).unwrap();
        p.sync_theme();
        let mut md = view(text, 3, 0);
        md.is_markdown = true;
        p.set_view(&md);
        let (h1, h2, h3, body) = (
            p.row_height_px(0),
            p.row_height_px(1),
            p.row_height_px(2),
            p.row_height_px(3),
        );
        assert!(body > 0.0, "{}: body row must have height", t.name);
        for (name, h) in [("h1", h1), ("h2", h2), ("h3", h3)] {
            assert!(
                h > body + 1.0,
                "{}: {name} row ({h}px) must read measurably taller than body ({body}px)",
                t.name
            );
        }
        assert!(
            h1 > h2 + 1.0 && h2 > h3 + 1.0,
            "{}: the ladder must descend strictly — h1 {h1} > h2 {h2} > h3 {h3}",
            t.name
        );
    }
    crate::theme::set_active(crate::theme::DEFAULT_THEME);
    p.sync_theme();
}

#[test]
fn thematic_break_row_grows_by_the_active_worlds_ornament_scale_and_refits_on_theme_switch() {
    // Row-height math folds the page wrap globals AND reads the active theme's
    // per-world ornament scale — hold both locks (order: theme, then page).
    let _t = crate::testlock::serial();
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping thematic_break_row_ornament_scale: no wgpu adapter");
        return;
    };
    // A thematic break (row 0) over a plain body line (row 2).
    let text = "---\n\nbody line\n";
    let mut md = view(text, 2, 0); // caret on the body line (logical line 2), NOT the break
    md.is_markdown = true;

    // Currawong: the break row grows to ~ITS OWN measured ornament scale (item
    // 561 equalized ink height per-world; each world now carries its own
    // measured value rather than a shared tier constant).
    crate::theme::set_active_by_name("Currawong").unwrap();
    let currawong_scale = crate::theme::active().ornament_scale;
    p.set_view(&md);
    let body = p.row_height_px(2);
    assert!(body > 0.0);
    let geo_break = p.row_height_px(0);
    let geo_ratio = geo_break / body;
    assert!(
        (geo_ratio - currawong_scale).abs() < 0.05,
        "Currawong break row should be ~{currawong_scale}x a body row, got {geo_ratio}"
    );

    // Switch to a DIFFERENT world (Mopoke) and RESHAPE via the same theme-font
    // seam a live theme switch rides: the break row must RE-FIT to Mopoke's OWN
    // measured scale (proof the row-height ↔ glyph-box coupling is per-world,
    // picked up on switch — not that one named world is taller than another).
    crate::theme::set_active_by_name("Mopoke").unwrap();
    let mopoke_scale = crate::theme::active().ornament_scale;
    p.sync_theme_font(crate::render::ShapeReach::Whole);
    let body2 = p.row_height_px(2);
    let ornate_break = p.row_height_px(0);
    let ornate_ratio = ornate_break / body2;
    assert!(
        (ornate_ratio - mopoke_scale).abs() < 0.05,
        "Mopoke break row should be ~{mopoke_scale}x a body row, got {ornate_ratio}"
    );
    assert!(
        (ornate_break - geo_break).abs() > 0.5,
        "two different worlds' measured scales must produce two different \
         break-row heights ({ornate_break} vs {geo_break})"
    );

    crate::theme::set_active(crate::theme::DEFAULT_THEME);
}

#[test]
fn heading_size_survives_theme_switch() {
    // Shaping folds the theme font AND the page wrap globals; hold both
    // (theme → page order, page.rs:95-99).
    let _t = crate::testlock::serial();
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping heading_size_survives_theme_switch: no wgpu adapter");
        return;
    };
    theme::set_active_by_name("Tawny").unwrap();
    p.sync_theme();
    let text = "# Big\n\nbody one\nbody two\n";
    let mut md = view(text, 0, 0);
    md.is_markdown = true;
    p.set_view(&md);
    let ratio_before = p.row_height_px(0) / p.row_height_px(2);
    assert!(
        ratio_before > 1.4,
        "sanity: heading taller before switch ({ratio_before})"
    );

    // Switch to a DIFFERENT-font world: the heading must STAY bigger. The bug was
    // `sync_theme` rebuilding CJK-only attrs, which dropped the markdown styling
    // and shrank headings back to body size on a live theme switch.
    theme::set_active_by_name("Gumtree").unwrap();
    p.sync_theme();
    let ratio_after = p.row_height_px(0) / p.row_height_px(2);
    assert!(
        ratio_after > 1.4,
        "heading must stay larger than body after a theme/font switch ({ratio_after})"
    );

    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
}

/// BUG regression (user screenshot 2026-07-04): zooming with the caret ON a
/// heading line left the amber block caret floating ~half a row above the
/// glyphs while the text itself re-laid correctly. Root cause: `set_view`
/// called `set_caret_target` (which reads the cursor's row geometry via
/// `cursor_row_height`/`caret_cell_top`) BEFORE the zoom-triggered
/// `restyle_all_lines` — so on a doc with headings, a zoom step reshaped body
/// text at the new metrics while the heading line's ABSOLUTE per-span pixel
/// metrics (set by the PREVIOUS restyle) were still stale until
/// `restyle_all_lines` ran, moments later, with no caret-target recompute
/// after it. The caret spring latched a target built from the transient,
/// pre-restyle row geometry — and nothing ever asked it to recompute once the
/// geometry settled.
#[test]
fn zoom_on_heading_line_keeps_caret_target_aligned() {
    // Shaping folds the theme font AND the page wrap globals; hold both
    // (theme -> page order, page.rs:95-99).
    let _t = crate::testlock::serial();
    let _g = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping zoom_on_heading_line_keeps_caret_target_aligned: no wgpu adapter");
        return;
    };
    let text = "## h2\n\nbody one\nbody two\n";

    // 1) Open the markdown doc with the caret on a BODY line at zoom 1.0. The
    // md-flip restyle fires here, but the cursor's own row is a body row
    // (unaffected by heading scale), so this establishes a clean baseline.
    let mut v = view(text, 2, 0);
    v.is_markdown = true;
    v.zoom = 1.0;
    p.set_view(&v);

    // 2) Move the caret ONTO the heading line, zoom unchanged: a plain
    // cursor-move target update against already-settled heading geometry.
    let mut v2 = view(text, 0, 3);
    v2.is_markdown = true;
    v2.zoom = 1.0;
    p.set_view(&v2);
    let (_, target_before_zoom, _, _) = p.caret_snapshot();

    // 3) Zoom, caret still on the heading line. This is the exact repro: the
    // zoom step both rescales body metrics AND (because the doc has a
    // heading) triggers `restyle_all_lines` to rescale the heading's
    // absolute pixel metrics to match.
    let row0_h_before = p.row_height_px(0);
    let mut v3 = view(text, 0, 3);
    v3.is_markdown = true;
    v3.zoom = 1.6;
    p.set_view(&v3);
    let (_, target_after_zoom, _, _) = p.caret_snapshot();

    // Sanity: the heading row itself really did grow with the zoom (the
    // "text re-lays correctly" half of the bug report) — read fresh from the
    // settled row-geometry table, not the caret.
    let row0_h_after = p.row_height_px(0);
    assert!(
        row0_h_after > row0_h_before * 1.3,
        "sanity: a 1.6x zoom must actually grow the heading row's height \
         (before={row0_h_before} after={row0_h_after})"
    );
    let _ = target_before_zoom;

    // The pipeline's state is fully settled after `set_view` returns (the
    // conditional restyle, if any, has already run), so a FRESH read of the
    // pure `caret_target_xy()` reflects the true, post-restyle geometry —
    // independent of whatever order `set_view` computed things in. The
    // caret's LATCHED spring target must agree with it.
    let (correct_x, correct_y) = p.caret_target_xy();
    assert!(
        (target_after_zoom.0 - correct_x).abs() < 0.5,
        "caret target x must match the settled heading-row geometry \
         (latched={:?}, correct=({correct_x}, {correct_y}))",
        target_after_zoom
    );
    assert!(
        (target_after_zoom.1 - correct_y).abs() < 0.5,
        "caret target y must match the settled heading-row geometry, not a \
         stale pre-restyle row height (latched={:?}, correct=({correct_x}, {correct_y}))",
        target_after_zoom
    );
}

/// LAW (theme-QA round, the reported cell "no-bold worlds: h3 reads as body —
/// headings need vertical spacing"): on a NO-BOLD world
/// (`Theme::heading_bold == false` — Bombora, Mulga, …), the BLANK GAP BEFORE
/// a heading (tested at the WEAKEST rung, SUBHEAD `###`, since it has the
/// least size lead over body to begin with) must read MEASURABLY TALLER at
/// REAL GPU PIXELS than the gap between two ordinary body paragraphs in the
/// SAME document — real GPU pixels, not the row-geometry accessors alone
/// (the Wagtail lesson, CLAUDE.md's harness section: appearance is proven
/// over bytes, never inferred from state — a mechanism could report a taller
/// row while the extra height painted no visible gap at all). Before
/// `heading_row_lead` existed, this was the bug in one sentence: a no-bold
/// world's `###` grew ONLY with its own font size (SUBHEAD's modest 1.15x) —
/// the blank-line gap around it was pixel-identical to a plain paragraph
/// break, so the heading read as body with a slightly bigger font, no rhythm
/// to it.
///
/// GAP BEFORE only, not after — a real rendering-mechanics finding from this
/// round's own instrumentation, not an assumption: cosmic-text places a
/// heading's enlarged line-box leading ABOVE its baseline (measured: the
/// row's own leading-before-ink offset grows with `heading_row_lead`, while
/// its trailing overflow past the row's nominal bottom stays roughly
/// constant across a plain body row and a heading row alike). That happens to
/// be the RIGHT typographic convention anyway (space-before a heading reads
/// bigger than space-after — a heading separates from what came before and
/// hugs what it introduces), so the law asserts the axis this mechanism
/// actually moves, and the gap AFTER is asserted merely NOT SHRUNK (never
/// worse than the plain baseline), never claimed to grow.
///
/// NO-WILDCARD-ish sweep: every world whose OWN `heading_bold` bit is `false`
/// in the roster (not a hand-picked pair), so a future no-bold world is
/// enrolled automatically; also asserts the sweep actually ran (guards
/// against every world someday flipping to `heading_bold: true` and this law
/// going vacuous).
#[test]
fn no_bold_worlds_get_more_gap_before_a_heading_than_between_paragraphs_at_real_pixels() {
    let _t = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!(
            "skipping no_bold_worlds_get_more_gap_before_a_heading_than_between_paragraphs_at_real_pixels: no wgpu adapter"
        );
        return;
    };
    let w = 1200u32;
    let h = 800u32;
    // line0/2: a plain paragraph pair (line1 blank) — the BASELINE gap.
    // line4: `### Heading Three` (line3 blank before it, line5 blank after) —
    // the HEADING gap on both sides. Short lines, so no wrap: logical line
    // index == visual row index (matches the sibling heading tests' idiom).
    let text = "Body paragraph one, the plain-gap baseline.\n\nBody paragraph two, still plain prose.\n\n### Heading Three\n\nBody paragraph three, right after the heading.\n";
    let mut checked = 0usize;
    for t in crate::theme::THEMES.iter().filter(|t| !t.heading_bold) {
        theme::set_active_by_name(t.name).unwrap();
        p.sync_theme();
        let mut v = view(text, 6, 0); // caret on the trailing line, off every measured line
        v.is_markdown = true;
        p.set_view(&v);
        p.prepare(&device, &queue, w, h).unwrap();
        let pixels = pixeldiff::render_frame(&mut p, &device, &queue, w, h);

        let text_left = p.text_left() as i64;
        let x0 = text_left.max(0);
        let x1 = (text_left + 700).min(w as i64);
        // Background reference: a row well below every measured line, still
        // page column (never the margin).
        let bg = pixels[((h as i64 - 10) * w as i64 + (x0 + 5)) as usize];

        let scan_top = (p.row_top_px(0) as i64).max(0);
        let scan_bot = ((p.row_top_px(6) + p.row_height_px(6)) as i64).min(h as i64);
        let bands = pixeldiff::ink_row_bands(
            &pixels, w as i64, h as i64, x0, x1, scan_top, scan_bot, bg, 18,
        );
        let ink_idx: Vec<usize> = bands
            .iter()
            .enumerate()
            .filter(|(_, b)| b.ink)
            .map(|(i, _)| i)
            .collect();
        assert_eq!(
            ink_idx.len(),
            4,
            "{}: expected 4 ink bands (para1, para2, heading, para3), got {bands:?}",
            t.name
        );
        // The gap BAND sitting between two consecutive ink bands is the band
        // at the index right after the first one's — `bands` alternates
        // ink/gap/ink/gap/…, so `ink_idx[i]+1` is always a gap (never past
        // the vec's end, since the scan starts and ends mid-content here).
        let gap_h = |after_ink: usize| -> i64 {
            let b = bands[ink_idx[after_ink] + 1];
            assert!(!b.ink, "{}: expected a gap band, got {b:?}", t.name);
            b.x1 - b.x0 + 1
        };
        let baseline_gap = gap_h(0); // between para1 and para2 — plain
        let pre_heading_gap = gap_h(1); // between para2 and the heading
        let post_heading_gap = gap_h(2); // between the heading and para3

        assert!(
            pre_heading_gap > baseline_gap + 3,
            "{}: gap BEFORE the heading ({pre_heading_gap}px) must read MEASURABLY \
             taller than the plain paragraph gap ({baseline_gap}px) — the reported \
             \"h3 reads as body\" bug",
            t.name
        );
        assert!(
            post_heading_gap + 2 >= baseline_gap,
            "{}: gap AFTER the heading ({post_heading_gap}px) must never read SMALLER \
             than the plain paragraph gap ({baseline_gap}px)",
            t.name
        );
        checked += 1;
    }
    assert!(
        checked >= 1,
        "no no-bold world ran — the sweep's filter matched nothing"
    );

    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
}

/// LAW — no line with content ever renders as EMPTY PIXELS: a sidecar can
/// report a row's geometry while the row paints nothing (the Wagtail
/// invisible-picker-row shape — CLAUDE.md's harness tripwire: `selected_index:
/// 2` while the row rendered fully invisible), so this asserts PRESENCE over a
/// REAL rendered frame's pixels, never row geometry / `md_spans` state alone.
///
/// Swept over the axis a `---`/`***`/`___` line actually varies on: the three
/// thematic-break syntaxes, each BOTH directly under a paragraph (the
/// setext-ambiguous position pulldown could otherwise swallow into a heading
/// it never draws) AND separated by a blank line (the unambiguous position),
/// with the caret BOTH off the line (concealed dashes, ornament drawn) and on
/// it (revealed dashes, no ornament) — 3 x 2 x 2 = 12 cells, every one a
/// presence floor over differing-pixel COUNT in exactly that row's own pixel
/// band, never "not byte-identical to blank" (a floor satisfiable by a single
/// stray antialiased pixel proves nothing; `MIN_INK_PIXELS` sits an order of
/// magnitude under a real glyph's coverage at this canvas size, but two orders
/// over antialiasing noise).
///
/// The scan region is inset well inside the page COLUMN (never the raw
/// canvas edges): the active theme's own page background is a top-to-bottom
/// GRADIENT (Saltpan: `#fbf3de` to `#f2e6c7`) plus a themed margin band right
/// at the column's outer edges — a single background sample compared across
/// the WHOLE canvas width, or across a wide y-gap, reads that gradient/band
/// as "ink" and the floor would pass on paint that is not the glyph's at all.
/// The background reference is sampled two pixels ABOVE the measured row, at
/// the SAME x used for the scan, so it rides the identical local band of the
/// gradient (a ~2px y-drift is negligible against a ~30px total canvas span).
#[test]
fn no_thematic_break_line_ever_renders_as_empty_pixels() {
    let _t = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping no_thematic_break_line_ever_renders_as_empty_pixels: no wgpu adapter");
        return;
    };
    let w = 1200u32;
    let h = 800u32;
    const CHANNEL_DELTA_THRESHOLD: u8 = 24;
    const MIN_INK_PIXELS: usize = 20;
    const MARGIN_INSET: f32 = 20.0;

    let mut cells_checked = 0usize;
    for syntax in ["---", "***", "___"] {
        for (position, text, break_line) in [
            ("direct-under-paragraph", format!("a\n{syntax}\n"), 1usize),
            ("blank-line-separated", format!("a\n\n{syntax}\n"), 2usize),
        ] {
            for (caret_state, caret_line) in [("off", 0usize), ("on", break_line)] {
                let mut v = view(&text, caret_line, 0);
                v.is_markdown = true;
                p.set_view(&v);
                p.prepare(&device, &queue, w, h).unwrap();
                let pixels = pixeldiff::render_frame(&mut p, &device, &queue, w, h);

                // Scan the page COLUMN interior only — inset past the themed
                // margin band on both edges — so both a left-aligned revealed
                // dash run and a CENTERED ornament fleuron fall inside it.
                let x0 = ((p.column_left() + MARGIN_INSET) as i64).clamp(0, w as i64);
                let x1 = ((p.column_left() + p.column_width() - MARGIN_INSET) as i64)
                    .clamp(x0, w as i64);
                let top = (p.doc_top() + p.row_top_px(break_line)).round() as i64;
                let bot = (p.doc_top() + p.row_top_px(break_line) + p.row_height_px(break_line))
                    .round() as i64;
                let (top, bot) = (top.max(1), bot.min(h as i64));

                // Background reference: the SAME x0, 2px above this row's own
                // top — same local gradient band, guaranteed blank (either the
                // previous row's trailing space or a blank line).
                let bg = pixels[((top - 2).max(0) * w as i64 + x0) as usize];

                let mut ink = 0usize;
                for y in top..bot {
                    for x in x0..x1 {
                        let px = pixels[(y * w as i64 + x) as usize];
                        let d = px[0]
                            .abs_diff(bg[0])
                            .max(px[1].abs_diff(bg[1]))
                            .max(px[2].abs_diff(bg[2]));
                        if d > CHANNEL_DELTA_THRESHOLD {
                            ink += 1;
                        }
                    }
                }
                assert!(
                    ink >= MIN_INK_PIXELS,
                    "{syntax:?} {position}, caret {caret_state}: line {break_line}'s row \
                     ({top}..{bot}px) painted only {ink} ink pixel(s) (floor \
                     {MIN_INK_PIXELS}) — content that renders as (near) empty pixels"
                );
                cells_checked += 1;
            }
        }
    }
    assert_eq!(
        cells_checked, 12,
        "the full 3-syntax x 2-position x 2-caret-state sweep must run every cell \
         (12), not a subset — a smaller count means the enrolment silently dropped one"
    );
}
