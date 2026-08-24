//! **HOVER GRAMMAR** — hover feedback is drawn only where WHICH
//! control fires or THAT a control exists is genuinely ambiguous: the format
//! popover's buttons, and inline-image resize handles. Every other clickable
//! surface stays visually still under hover.
//!
//! The calm is the law too, and it is the axis a presence floor alone would
//! never catch — a version that lit up a margin outline row by accident would
//! still pass "the popover button acknowledges hover". So this file carries
//! three claims: the two NEW acknowledgements are real, non-vacuous pixels
//! (presence floor, swept over the whole theme roster); a SIBLING button /
//! the image's own interior draws nothing extra; and hovering both new
//! surfaces at once leaves the margin outline column and the drawn menu
//! bar's own band byte-identical.

use super::super::*;
use super::pixeldiff::{
    DistinguishFloor, Region, assert_identical, assert_perceptibly_different, render_frame,
};
use super::{headless_dqp, view};

const W: u32 = 1400;
const H: u32 = 900;

/// Scan `p.popover_hit` across the card's own width at `y` and return the
/// FULL run of x that `button` claims — the SAME midpoint-split hit region
/// [`crate::render::plan::PopoverGeom::hit_span_x`] names and the ring paints,
/// read off the real production hit-test rather than a re-derived formula
/// (mirrors `gutter_stack_pixels.rs::find_row_right_edge`).
fn button_hit_span(
    p: &TextPipeline,
    button: crate::popover::PopoverButton,
    card: [f32; 4],
    y: f32,
) -> (f32, f32) {
    let [card_x, _, card_w, _] = card;
    let mut lo: Option<f32> = None;
    let mut hi = card_x;
    let mut x = card_x;
    while x <= card_x + card_w {
        if p.popover_hit(x, y) == Some(button) {
            if lo.is_none() {
                lo = Some(x);
            }
            hi = x;
        }
        x += 0.5;
    }
    let lo = lo.unwrap_or_else(|| panic!("{button:?} claims no span on this card"));
    (lo, hi)
}

fn popover_view() -> ViewState {
    let text = "select this word\n".to_string();
    let mut v = view(&text, 0, 11);
    v.selection = Some(((0, 7), (0, 11)));
    v.popover = crate::actions::popover::plan(&text, Some(7), 11, true);
    v
}

/// **THE HOVERED POPOVER BUTTON CARRIES A VISIBLE RING; ITS SIBLINGS DO NOT
/// — on every world.**
///
/// Two bounds, and the second is the one a naive "just draw a wash everywhere
/// hover is engaged" shortcut fails: hovering button 1 (Italic) must move
/// pixels in ITS OWN hit-region (a presence floor, non-vacuous) and must
/// leave button 4 (Strike)'s own hit-region byte-identical.
#[test]
fn the_hovered_popover_button_carries_a_ring_and_its_sibling_does_not_on_every_world() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!(
            "skipping the_hovered_popover_button_carries_a_ring_and_its_sibling_does_not_\
             on_every_world: no wgpu adapter"
        );
        return;
    };
    p.set_dpi(1.0);
    let _pin = theme::WorldPin::snapshot();
    let v = popover_view();

    let hovered_button = crate::popover::PopoverButton::ALL[1]; // Italic
    let sibling_button = crate::popover::PopoverButton::ALL[4]; // Strike

    let mut judged = Vec::new();
    for (index, world) in theme::THEMES.iter().enumerate() {
        theme::set_active(index);
        let world = world.name;
        p.set_view(&v);
        p.clear_popover_hover();
        p.prepare(&device, &queue, W, H).unwrap();
        let (card, buttons) = p.popover_report().expect("popover lays out");
        let mid_y = card[1] + card[3] * 0.5;
        assert_eq!(buttons.len(), 7, "{world}: the locked seven-button roster");
        let resting = render_frame(&mut p, &device, &queue, W, H);

        let [hx0, hx1] = buttons[1].2;
        let changed = p.resolve_popover_hover((hx0 + hx1) * 0.5, mid_y);
        assert!(
            changed,
            "{world}: hovering a real button must change hover state"
        );
        p.prepare(&device, &queue, W, H).unwrap();
        let hovered = render_frame(&mut p, &device, &queue, W, H);

        let (lo, hi) = button_hit_span(&p, hovered_button, card, mid_y);
        assert_perceptibly_different(
            &resting,
            &hovered,
            W as i64,
            H as i64,
            Region::new(lo, card[1], hi - lo, card[3]),
            DistinguishFloor::DEFAULT,
            &format!("{world}: hovered popover button ({hovered_button:?})"),
        );

        let (slo, shi) = button_hit_span(&p, sibling_button, card, mid_y);
        assert_identical(
            &resting,
            &hovered,
            W as i64,
            H as i64,
            Region::new(slo, card[1], shi - slo, card[3]),
            &format!("{world}: non-hovered sibling popover button ({sibling_button:?})"),
        );
        judged.push(world);
    }
    assert_eq!(
        judged.len(),
        theme::THEMES.len(),
        "only {} of {} worlds judged: {judged:?}",
        judged.len(),
        theme::THEMES.len()
    );
}

/// **HOVERING AN IMAGE'S RESIZE HANDLE DRAWS ITS GRIP; THE IMAGE'S OWN
/// INTERIOR DOES NOT — on every world.**
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn hovering_an_image_handle_draws_its_grip_and_the_interior_stays_still_on_every_world() {
    let _g = crate::testlock::serial();
    std::fs::metadata("samples/tiny.png")
        .expect("tracked samples/tiny.png fixture must be present");
    let prev = crate::markdown::inline_images_on();
    let prevw = crate::markdown::wysiwyg_on();
    crate::markdown::set_inline_images_on(true);
    crate::markdown::set_wysiwyg_on(true);
    let restore = || {
        crate::markdown::set_inline_images_on(prev);
        crate::markdown::set_wysiwyg_on(prevw);
    };
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!(
            "skipping hovering_an_image_handle_draws_its_grip_and_the_interior_stays_still_\
             on_every_world: no wgpu adapter"
        );
        restore();
        return;
    };
    p.set_dpi(1.0);
    let _pin = theme::WorldPin::snapshot();
    let text = "![pic](samples/tiny.png)\nprose below\n";
    let mut v = view(text, 1, 0);
    v.is_markdown = true;

    let mut judged = Vec::new();
    for (index, world) in theme::THEMES.iter().enumerate() {
        theme::set_active(index);
        let world = world.name;
        p.set_view(&v);
        p.clear_image_hover();
        p.prepare(&device, &queue, W, H).unwrap();
        let rects = p.image_hit_rects();
        assert_eq!(
            rects.len(),
            1,
            "{world}: fixture must lay out exactly one image"
        );
        let (_, rect) = rects[0];
        let resting = render_frame(&mut p, &device, &queue, W, H);

        // The right edge's own midpoint — a real edge handle, not a corner.
        let (hx, hy) = (rect[0] + rect[2], rect[1] + rect[3] * 0.5);
        let changed = p.resolve_image_hover(hx, hy);
        assert!(
            changed,
            "{world}: hovering the image's right edge must change hover state"
        );
        p.prepare(&device, &queue, W, H).unwrap();
        let hovered = render_frame(&mut p, &device, &queue, W, H);

        let mark = p
            .image_hover_mark_rect()
            .expect("hover engaged, so a mark rect must exist");
        assert_perceptibly_different(
            &resting,
            &hovered,
            W as i64,
            H as i64,
            Region::new(mark[0] - 1.0, mark[1] - 1.0, mark[2] + 2.0, mark[3] + 2.0),
            DistinguishFloor::DEFAULT,
            &format!("{world}: image resize-handle grip"),
        );

        // A generous interior chunk of the image, well clear of any edge —
        // hovering one handle must not tint the picture itself.
        let interior = Region::new(
            rect[0] + rect[2] * 0.35,
            rect[1] + rect[3] * 0.35,
            rect[2] * 0.3,
            rect[3] * 0.3,
        );
        assert_identical(
            &resting,
            &hovered,
            W as i64,
            H as i64,
            interior,
            &format!("{world}: image interior, away from the hovered handle"),
        );
        judged.push(world);
    }
    restore();
    assert_eq!(
        judged.len(),
        theme::THEMES.len(),
        "only {} of {} worlds judged: {judged:?}",
        judged.len(),
        theme::THEMES.len()
    );
}

/// **THE CALM IS THE LAW TOO — engaging BOTH new hover mechanisms at once
/// leaves the margin outline column and the drawn menu bar's own band
/// byte-identical, on every world.**
///
/// The fixture puts a heading (an outline row), a popover-summoning
/// selection, and an image all in the SAME document, with the menu bar
/// forced on — the one frame shape where every new hover mechanism this item
/// adds can be engaged simultaneously. Non-vacuity comes for free: the other
/// two tests in this file already prove the popover/image regions genuinely
/// move under this exact mechanism, so a law that only checked "nothing
/// changed anywhere" could not be satisfied by silently disabling the new
/// features — this one checks two SPECIFIC regions the new features must
/// never reach, while the mechanism is provably live elsewhere in the frame.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn engaging_new_hover_leaves_the_outline_and_menu_bar_untouched_on_every_world() {
    let _g = crate::testlock::serial();
    std::fs::metadata("samples/tiny.png")
        .expect("tracked samples/tiny.png fixture must be present");
    let prev = crate::markdown::inline_images_on();
    let prevw = crate::markdown::wysiwyg_on();
    crate::markdown::set_inline_images_on(true);
    crate::markdown::set_wysiwyg_on(true);
    let ambient_menu_bar = crate::menubar::menu_bar_on();
    crate::menubar::set_menu_bar_on(true);
    let restore = || {
        crate::markdown::set_inline_images_on(prev);
        crate::markdown::set_wysiwyg_on(prevw);
        crate::menubar::set_menu_bar_on(ambient_menu_bar);
    };
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!(
            "skipping engaging_new_hover_leaves_the_outline_and_menu_bar_untouched_on_every_world: \
             no wgpu adapter"
        );
        restore();
        return;
    };
    p.set_dpi(1.0);
    let _pin = theme::WorldPin::snapshot();
    // The selected word sits deep inside a long line (not near its start) so
    // the popover card — anchored at the selection's own x, half its width
    // either side — settles comfortably inside the writing column on every
    // world's panel face, never spilling into the margin this law reads.
    let text = "# A heading\n\nprose to give the outline a row\n\n\
padding padding padding padding select this word here\n\n\
![pic](samples/tiny.png)\nmore prose\n";
    let sel_line = 4usize; // "padding … select this word here"
    let sel_start = 39; // byte/char offset of "this" on that line
    let sel_end = 43;
    let mut v = view(text, sel_line, sel_start);
    v.is_markdown = true;
    v.selection = Some(((sel_line, sel_start), (sel_line, sel_end)));
    v.popover = crate::actions::popover::plan(text, Some(sel_start), sel_end, true);

    let mut judged = Vec::new();
    for (index, world) in theme::THEMES.iter().enumerate() {
        theme::set_active(index);
        let world = world.name;
        p.set_view(&v);
        p.clear_popover_hover();
        p.clear_image_hover();
        p.prepare(&device, &queue, W, H).unwrap();
        let (card, buttons) = p.popover_report().expect("{world}: popover lays out");
        let mid_y = card[1] + card[3] * 0.5;
        let column_left = p.column_left();
        assert!(
            card[0] >= column_left,
            "{world}: fixture bug — the popover card (x={}) must sit inside the writing \
             column (column_left={column_left}), or this law's own margin region would \
             overlap the card it deliberately excludes",
            card[0]
        );
        let rects = p.image_hit_rects();
        assert_eq!(
            rects.len(),
            1,
            "{world}: fixture must lay out exactly one image"
        );
        let (_, rect) = rects[0];
        let resting = render_frame(&mut p, &device, &queue, W, H);

        let [hx0, hx1] = buttons[1].2;
        let popover_changed = p.resolve_popover_hover((hx0 + hx1) * 0.5, mid_y);
        let (ihx, ihy) = (rect[0] + rect[2], rect[1] + rect[3] * 0.5);
        let image_changed = p.resolve_image_hover(ihx, ihy);
        assert!(
            popover_changed && image_changed,
            "{world}: the fixture must engage both new hover mechanisms at once \
             (popover={popover_changed}, image={image_changed})"
        );
        p.prepare(&device, &queue, W, H).unwrap();
        let hovered = render_frame(&mut p, &device, &queue, W, H);

        // Non-vacuity: the mechanisms are genuinely live in THIS fixture.
        let (lo, hi) = button_hit_span(&p, crate::popover::PopoverButton::ALL[1], card, mid_y);
        assert_perceptibly_different(
            &resting,
            &hovered,
            W as i64,
            H as i64,
            Region::new(lo, card[1], hi - lo, card[3]),
            DistinguishFloor::DEFAULT,
            &format!("{world}: sanity — the popover ring must be live in this fixture"),
        );
        let mark = p
            .image_hover_mark_rect()
            .unwrap_or_else(|| panic!("{world}: hover engaged, so a mark rect must exist"));
        assert_perceptibly_different(
            &resting,
            &hovered,
            W as i64,
            H as i64,
            Region::new(mark[0] - 1.0, mark[1] - 1.0, mark[2] + 2.0, mark[3] + 2.0),
            DistinguishFloor::DEFAULT,
            &format!("{world}: sanity — the image grip must be live in this fixture"),
        );

        // THE CLAIM: the margin column (outline rows, the fold-chevron pad,
        // the working-set stack) never moves — the whole strip left of the
        // writing column, so a future surface added to that margin inherits
        // the same proof without a new region to name.
        assert_identical(
            &resting,
            &hovered,
            W as i64,
            H as i64,
            Region::new(0.0, 0.0, column_left, H as f32),
            &format!("{world}: margin column (outline rows and neighbours)"),
        );

        // THE DRAWN MENU BAR's own band, full width.
        assert_identical(
            &resting,
            &hovered,
            W as i64,
            H as i64,
            Region::new(0.0, 0.0, W as f32, p.menubar_bar_h),
            &format!("{world}: the drawn menu bar's own band"),
        );
        judged.push(world);
    }
    restore();
    assert_eq!(
        judged.len(),
        theme::THEMES.len(),
        "only {} of {} worlds judged: {judged:?}",
        judged.len(),
        theme::THEMES.len()
    );
}

/// **STRUCTURAL: the six still-surfaces' own drawing code never reads either
/// new hover field.** Not a rendering claim — a grep-shaped one, proving
/// nobody wired `popover_hover`/`image_hover` into the margin outline, the
/// workspace rail, the settings range rail, the find/replace panel, the
/// start screen, or the drawn menu bar, which is the shape a "just reuse the
/// popover ring everywhere" shortcut would take. No-wildcard: every file
/// named in the item's own roster is checked by name.
#[test]
fn the_six_still_surfaces_own_draw_code_never_reads_the_new_hover_fields() {
    let files: [(&str, &str); 6] = [
        (
            "chrome/outline.rs (margin outline rows)",
            include_str!("../chrome/outline.rs"),
        ),
        (
            "chrome/workspace_rail.rs (workspace rail entries)",
            include_str!("../chrome/workspace_rail.rs"),
        ),
        (
            "chrome/overlay_rows.rs (settings range rail draw home)",
            include_str!("../chrome/overlay_rows.rs"),
        ),
        (
            "chrome/panel.rs (find/replace panel cells)",
            include_str!("../chrome/panel.rs"),
        ),
        (
            "chrome/start.rs (start-screen action rows)",
            include_str!("../chrome/start.rs"),
        ),
        (
            "chrome/menubar.rs (drawn web/Linux menu bar)",
            include_str!("../chrome/menubar.rs"),
        ),
    ];
    for (label, src) in files {
        assert!(
            !src.contains("popover_hover"),
            "{label} must never read the new popover-hover field"
        );
        assert!(
            !src.contains("image_hover"),
            "{label} must never read the new image-hover field"
        );
    }
}
