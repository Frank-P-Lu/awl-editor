//! **THE DRAWN PAGE IS THE AUTHORED PAGE.** The frame's one `LoadOp::Clear`
//! (`render::pipeline_layers::begin_clear_pass`) paints the writing page, and
//! its value skips the fragment stage — so an sRGB-format attachment applies no
//! linear→sRGB encode and consumes it as linear light. Handing it raw sRGB
//! bytes stored their sRGB ENCODE instead of the token: Currawong's authored
//! `#060607` drew as `#2A2A2E`, Potoroo's `#1F0400` as `#622200`, and every
//! world in the roster but the pure-black one drew a page lighter than the one
//! THEMES.md describes. The margin field was never affected — `background.rs`
//! linearises its own gradient endpoints, as does every other colour path in
//! the tree — so what a reader saw was a page visibly LIGHTER than the margin
//! it is supposed to be one seamless plane with.
//!
//! # What this law grades, and why not a single probe pixel
//!
//! The MODE of the page column's interior, per world, at 1× and 2×. A single
//! sampled pixel would have to dodge every ground personality the roster
//! carries — a lava blob, a star, a warped-grid ring under Kite's page veil, a
//! stripe band — and a probe point tuned to miss them on twenty worlds today is
//! a probe point that lands on one tomorrow. The mode is what a reader calls
//! "the page", it is immune to what any ornament paints over it, and it is
//! derived from the roster rather than from a chosen coordinate.
//!
//! Three claims, no one of which is satisfiable by breaking the others:
//!
//!   * **IDENTITY** — the modal interior colour IS `base_100`'s authored bytes,
//!     exactly, no tolerance.
//!   * **PRESENCE** — that mode covers a supermajority of the interior. Without
//!     it, "the mode is the token" stays true of a ground that painted over
//!     almost the whole page and left a sliver of correct clear behind, which is
//!     the same shape as a contrast floor that gets happier as its subject
//!     fades away.
//!   * **BOTH TIERS** — 1× and 2×. Every ordinary capture runs at
//!     `--capture-dpi 1`; a law that never leaves it has run in one
//!     configuration and proved nothing about the other.
//!
//! There is deliberately NO arm here comparing the page against its margin.
//! That comparison reads as the tempting one — it is the shape the defect had —
//! but for a world whose ground starts at its page it is implied by the identity
//! claim already (page mode == `base_100` == `Background::from`), and it would
//! otherwise be grading the background pipeline, which never carried this bug.
//!
//! The pure-arithmetic half of the same law — that the number handed to the
//! clear survives the attachment's encode, swept over all 256 channel values
//! rather than the twenty grounds that happen to be authored today, with the
//! retired rule written out inline for non-vacuity — is `theme::tests::clear`.

use super::super::*;
use super::{headless_dqp, pixeldiff, view};

/// The LOGICAL canvas both tiers render, so the adaptive column makes the same
/// layout decision at 1× and 2× and only the device resolution changes (the
/// `chrome_pixel_space` discipline: a canvas held at one PHYSICAL size
/// across DPI reflows its content, which is a different question).
const LOGICAL: (f32, f32) = (600.0, 400.0);

/// How far inside the page column's own edges the sampled region starts, in
/// LOGICAL px — clear of the hairline page frame some worlds draw on the column
/// boundary, which is ink and not ground.
const INSET: f32 = 12.0;

/// The most common `[u8;4]` in `region`, and the fraction of the region it
/// covers. `None` if the region is empty.
fn modal_colour(
    frame: &[[u8; 4]],
    w: u32,
    h: u32,
    region: pixeldiff::Region,
) -> Option<([u8; 4], f32)> {
    let x0 = region.x.max(0) as u32;
    let y0 = region.y.max(0) as u32;
    let x1 = ((region.x + region.w).max(0) as u32).min(w);
    let y1 = ((region.y + region.h).max(0) as u32).min(h);
    if x1 <= x0 || y1 <= y0 {
        return None;
    }
    let mut counts: std::collections::HashMap<[u8; 4], u32> = std::collections::HashMap::new();
    let mut total = 0u32;
    for y in y0..y1 {
        for x in x0..x1 {
            *counts.entry(frame[(y * w + x) as usize]).or_insert(0) += 1;
            total += 1;
        }
    }
    counts
        .into_iter()
        .max_by_key(|&(_, n)| n)
        .map(|(c, n)| (c, n as f32 / total as f32))
}

/// Render the active world at one `dpi` and report `(modal page-interior
/// colour, its coverage fraction, the sampled pixel count)`.
fn page_interior_mode(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &mut TextPipeline,
    dpi: f32,
) -> ([u8; 4], f32, u32) {
    let (w, h) = ((LOGICAL.0 * dpi) as u32, (LOGICAL.1 * dpi) as u32);
    p.set_dpi(dpi);
    p.set_size(w as f32, h as f32);
    p.sync_theme();
    let v = view("The page is the ground.\n", 0, 0);
    p.set_view(&v);
    p.prepare(device, queue, w, h).unwrap();
    let frame = pixeldiff::render_frame(p, device, queue, w, h);
    let inset = INSET * dpi;
    let region = pixeldiff::Region::new(
        p.column_left() + inset,
        inset,
        (p.column_width() - 2.0 * inset).max(1.0),
        (h as f32 - 2.0 * inset).max(1.0),
    );
    let (c, frac) = modal_colour(&frame, w, h, region).expect("the page column has an interior");
    (c, frac, (region.w * region.h) as u32)
}

/// **THE LAW.** For every world in `theme::THEMES`, at 1× and 2×, the page
/// column's interior reads as that world's own authored `base_100`.
///
/// Enrolment is the roster itself and the failure message names the world, the
/// tier, and both colours — so a twenty-first world is swept the day it lands,
/// and a failure says which page is wrong rather than that one is.
#[test]
fn every_worlds_page_draws_its_authored_base_100_at_both_dpi_tiers() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(LOGICAL.0, LOGICAL.1) else {
        eprintln!(
            "skipping every_worlds_page_draws_its_authored_base_100_at_both_dpi_tiers: \
             no wgpu adapter"
        );
        return;
    };
    let _pin = theme::WorldPin::snapshot();
    for (i, t) in theme::THEMES.iter().enumerate() {
        theme::set_active(i);
        let want = t.base_100.rgba_bytes();
        for dpi in [1.0f32, 2.0f32] {
            let (got, frac, n) = page_interior_mode(&device, &queue, &mut p, dpi);
            assert_eq!(
                got, want,
                "{} at dpi {dpi}: the page column's interior draws \
                 #{:02x}{:02x}{:02x} where the world authors #{:02x}{:02x}{:02x} \
                 — LoadOp::Clear takes LINEAR light, so an sRGB-format target \
                 stores the sRGB ENCODE of any raw sRGB bytes handed to it",
                t.name, got[0], got[1], got[2], want[0], want[1], want[2],
            );
            // PRESENCE: the mode has to BE the page, not a surviving sliver of
            // it. The roster's tightest real value is a lava world's, whose
            // blobs cover roughly a tenth of their own page.
            assert!(
                frac >= 0.60,
                "{} at dpi {dpi}: the modal interior colour covers only \
                 {:.1}% of {n} sampled pixels — too little of the page is \
                 ground for its mode to be a claim about the page at all",
                t.name,
                frac * 100.0
            );
        }
    }
    p.set_dpi(1.0);
}
