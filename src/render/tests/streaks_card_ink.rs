//! The Writing-streaks card's own drawn geometry and its ←/→ paging hint.
//!
//! Two claims, proved independently:
//!
//!   1. **GEOMETRY**: `TextPipeline::streaks_card_rect` / `streaks_hint_row_rect`
//!      report the SAME plan the card actually renders from — the throwaway
//!      pipeline's `prepare()` is the identical production `prepare_streaks_card`
//!      the real capture door calls, so the two cannot disagree.
//!   2. **APPEARANCE**: the hint line those rects locate carries real ink, not
//!      just correctly-planned empty space — proved over a REAL captured PNG's
//!      pixels through `capture::capture_with`, the exact path `--screenshot`
//!      renders through (CLAUDE.md's sidecar-is-a-state-oracle tripwire: a
//!      geometry claim alone cannot stand in for an appearance one).
//!
//! The presence floor's own non-vacuity control is built in rather than
//! asserted separately: the SAME region sampled on a CLOSED card (bare page
//! background, no card drawn at all) is required to fall under the floor the
//! open card must clear — so the floor cannot be satisfied by a law that
//! samples the wrong pixels or by a hint that silently stopped drawing.
//!
//! One failure mode this floor CANNOT catch alone: the first drawn line is
//! always SOME text (every card composes non-empty content), so a mutation
//! that deletes the hint span but leaves the next line ("CURRENT STREAK")
//! sliding up into its place still draws real ink at this exact geometry —
//! the law would stay green over the WRONG text. `card::content::tests::
//! the_streaks_hint_is_the_first_line_on_every_page` is the state half of the
//! pairing that closes that gap: it proves line 0's TEXT is the hint,
//! specifically, on both pages. Neither law alone is CLAUDE.md's tripwire-safe
//! ("the law is satisfied by the broken state"); together they are.

use super::{headless_dqp, view};
use crate::capture::CaptureOpts;
use crate::testscratch::ScratchDir;

/// The most common pixel color over a region — its own background, since
/// glyph ink covers a small minority of a text row's area. Mirrors
/// `date_picker_ink.rs::region_mode_color`.
fn region_mode_color(img: &image::RgbaImage, x0: i64, y0: i64, x1: i64, y1: i64) -> [u8; 4] {
    use std::collections::HashMap;
    let mut counts: HashMap<[u8; 4], usize> = HashMap::new();
    for y in y0.max(0)..y1.min(img.height() as i64) {
        for x in x0.max(0)..x1.min(img.width() as i64) {
            let p = img.get_pixel(x as u32, y as u32).0;
            *counts.entry(p).or_insert(0) += 1;
        }
    }
    counts
        .into_iter()
        .max_by_key(|(_, n)| *n)
        .map(|(c, _)| c)
        .unwrap_or([0, 0, 0, 0])
}

/// Count of pixels in the region whose max-channel distance from `bg` clears
/// `threshold` — a low noise floor (24 of 255, matching `date_picker_ink.rs`),
/// not a "solid fill only" bar.
fn solid_ink_count(
    img: &image::RgbaImage,
    x0: i64,
    y0: i64,
    x1: i64,
    y1: i64,
    bg: [u8; 4],
) -> usize {
    let threshold = 24u8;
    let mut n = 0usize;
    for y in y0.max(0)..y1.min(img.height() as i64) {
        for x in x0.max(0)..x1.min(img.width() as i64) {
            let p = img.get_pixel(x as u32, y as u32).0;
            let d = p[0]
                .abs_diff(bg[0])
                .max(p[1].abs_diff(bg[1]))
                .max(p[2].abs_diff(bg[2]));
            if d > threshold {
                n += 1;
            }
        }
    }
    n
}

/// A real capture through the ordinary door (`capture_with`, the exact
/// function `--screenshot` calls), with the streaks card open/closed as
/// asked, on `world`. `page` toggles the card's page before capture.
fn capture_streaks(
    dir: &std::path::Path,
    world: &str,
    open: bool,
    cumulative: bool,
    tag: &str,
) -> image::RgbaImage {
    use crate::capture::capture_with;
    let _g = crate::testlock::serial();
    assert!(
        crate::theme::set_active_by_name(world).is_some(),
        "unknown world {world:?}"
    );
    // `set_open(true)` always resets the page to the heatmap (its own
    // documented contract), so a single conditional toggle from that known
    // starting point reaches whichever page was asked for.
    crate::streaks::set_open(open);
    if open && cumulative {
        crate::streaks::toggle_view();
    }
    let buf = crate::buffer::Buffer::from_str("hello world\n");
    let png = dir.join(format!("{world}_{tag}.png"));
    capture_with(&png, &buf, &CaptureOpts::default()).expect("streaks capture renders");
    let img = image::open(&png).expect("decode streaks png").to_rgba8();
    crate::streaks::set_open(false);
    img
}

/// GEOMETRY: the throwaway pipeline's `prepare()` is the identical
/// `prepare_streaks_card` the real capture renders through, so its reported
/// rects describe where the card and the hint's own line actually sit — well
/// inside the canvas, the hint row inside the card, and the hint row planted
/// at the TOP of the card's text flow (directly under the page dots, above
/// "CURRENT STREAK").
#[test]
fn streaks_geometry_places_the_hint_row_inside_the_card_above_the_figures() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping streaks_geometry_places_the_hint_row_inside_the_card: no wgpu adapter");
        return;
    };
    crate::streaks::set_open(true);
    let v = view("hello world\n", 0, 0);
    p.set_view(&v);
    p.prepare(&device, &queue, 1200, 800).unwrap();

    let [cx, cy, cw, ch] = p.streaks_card_rect().expect("the streaks card is open");
    assert!(cw > 0.0 && ch > 0.0, "the card has real extent");
    assert!(cx >= 0.0 && cy >= 0.0 && cx + cw <= 1200.0 && cy + ch <= 800.0);

    let [hx, hy, hw, hh] = p
        .streaks_hint_row_rect()
        .expect("the hint is the first drawn line");
    assert!(hw > 0.0 && hh > 0.0, "the hint row has real extent");
    assert!(
        hx >= cx && hy >= cy && hx + hw <= cx + cw + 0.5 && hy + hh <= cy + ch + 0.5,
        "the hint row [{hx},{hy},{hw},{hh}] must sit inside the card [{cx},{cy},{cw},{ch}]"
    );
    // The FIRST line of the text flow sits in the card's upper portion — well
    // above its vertical midpoint, where the heatmap/chart and the page dots
    // live above it.
    assert!(
        hy < cy + ch * 0.6,
        "the hint row must sit in the card's upper region, not among the figures"
    );

    crate::streaks::set_open(false);
}

/// APPEARANCE + PRESENCE FLOOR: the hint row carries real ink in a REAL
/// capture, on both pages (it is not part of either page's own content) and
/// across a dark Pane world, the shipped light default, and the one-bit
/// (Wagtail) world — the tofu/legibility risk a new glyph in an unfamiliar
/// font ladder always carries. The floor's own non-vacuity control: the
/// IDENTICAL region on a CLOSED card (bare page background) must fall well
/// under it, so the floor cannot be cleared by noise or by a silently empty
/// hint.
#[test]
fn streaks_hint_row_carries_real_ink_on_both_pages_across_worlds() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping streaks_hint_row_carries_real_ink_on_both_pages: no wgpu adapter");
        return;
    };
    let orig_theme = crate::theme::active_index();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl-streaks-hint-ink_{}", std::process::id())),
    );

    // FLOOR: chosen well under every observed open-card count and well over
    // every observed closed-card (background-only) count — see the eprintln
    // this test emits per cell.
    const FLOOR: usize = 60;

    for world in ["Tawny", "Saltpan", "Wagtail"] {
        crate::streaks::set_open(true);
        let v = view("hello world\n", 0, 0);
        p.set_view(&v);
        crate::theme::set_active_by_name(world);
        p.prepare(&device, &queue, 1200, 800).unwrap();
        let [hx, hy, hw, hh] = p
            .streaks_hint_row_rect()
            .expect("the hint row is drawn while the card is open");
        crate::streaks::set_open(false);

        // A small margin around the measured row so antialiased edges are not
        // clipped out of the sampled region.
        let (x0, y0, x1, y1) = (
            (hx - 2.0) as i64,
            (hy - 2.0) as i64,
            (hx + hw + 2.0) as i64,
            (hy + hh + 2.0) as i64,
        );

        let closed = capture_streaks(&dir, world, false, false, "closed");
        let closed_bg = region_mode_color(&closed, x0, y0, x1, y1);
        let closed_ink = solid_ink_count(&closed, x0, y0, x1, y1, closed_bg);

        for cumulative in [false, true] {
            let tag = if cumulative { "cumulative" } else { "heatmap" };
            let img = capture_streaks(&dir, world, true, cumulative, tag);
            let bg = region_mode_color(&img, x0, y0, x1, y1);
            let ink = solid_ink_count(&img, x0, y0, x1, y1, bg);
            eprintln!(
                "streaks hint ink: world={world} page={tag} ink={ink} closed_ink={closed_ink} \
                 region=[{x0},{y0},{x1},{y1}]"
            );
            assert!(
                ink >= FLOOR,
                "{world} {tag}: only {ink} ink pixels over the hint row \
                 [{hx},{hy},{hw},{hh}] — the ←/→ hint must draw real, legible ink \
                 (floor {FLOOR})"
            );
        }
        assert!(
            closed_ink < FLOOR,
            "{world}: a CLOSED card already shows {closed_ink} ink pixels in the same \
             region — the floor above would be vacuous (region drawn something\
             regardless of the hint)"
        );
    }

    crate::theme::set_active(orig_theme);
}
