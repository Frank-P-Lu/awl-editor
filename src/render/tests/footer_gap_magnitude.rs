//! THE FOOTER GAP'S OWN MAGNITUDE, over the bare-plate roster, and every
//! candidate row's own label pixels against the picker's own giant title
//! placard — the busiest ground in the room.
//!
//! Two separate claims, because they have two separate subjects:
//!
//! * **Law A** — `OVERLAY_HINT_GAP_ROW` widened the blank row ahead of the
//!   foot hint, but the row's own PLATE still starts flush at
//!   `OverlayRowPlan::footer_top()` by design (`footer_plate_rect`'s doc: the
//!   footer reads as one composed unit with the last row, never a raw gap
//!   floating over a pinned chin). So "the hint sits flush under the last
//!   row" was never about the plate's own top edge — it is about how little
//!   room the hint's own TEXT got before it, inside that plate. This law
//!   reads the drawn gap (`overlay_hint_gap_probe`, the same oracle
//!   `hint_gap.rs` already grades for mere presence) against a floor set
//!   ABOVE what the retired dial could ever produce, so a revert of the
//!   magnitude bump is caught here even on a world/shape pair where the
//!   presence-only law next door stays green regardless.
//! * **Law B** — the picker's own title, shaped as a canvas-anchored
//!   wordmark (`TitleStyle::Placard`), draws SHARP and UNMASKED behind the
//!   card on every world that authors it — never frosted, unlike the
//!   document. Measured directly: opening the theme picker over ordinary
//!   prose still renders "THEMES" as a giant placard, and opening the
//!   command palette over the SAME document renders "COMMANDS" instead — the
//!   wordmark is the OVERLAY'S OWN TITLE, not anything from the document, so
//!   the busiest ground a candidate row could ever sit against is this
//!   picker's own chrome, present on every real capture, not a hypothetical
//!   document heading.
//!
//!   The naive claim — no row's glyph box ever geometrically meets the
//!   placard's — is FALSE, measured: the real theme picker's own item count
//!   (every shipped world, not a hand-picked few) is tall enough that
//!   Firetail's last few rows sit squarely inside the placard's box on the
//!   canonical 1200x800 canvas. That is not a legibility bug — `Bars` backs
//!   every row with an OPAQUE plate, drawn after the placard, so the row's
//!   own ink is unchanged by whatever sits behind it — but it does mean the
//!   right claim is masking, not clearance. This law reads it directly:
//!   render the picker once with the world's own `TitleStyle` and once with
//!   the placard suppressed (`TitleStyle::InlinePrefix`, which
//!   `overlay_shape_placard` refuses outright) — title style touches nothing
//!   else in `render/plan`, so the two frames differ ONLY in whether the
//!   placard drew — and diff every candidate row's own glyph box between
//!   them. A `Bars` row passes because its plate masks; a `Diagonal`/`Ruled`
//!   row (no plate at all) passes only where geometry keeps it clear, which
//!   the enrolled worlds' own placement (`derived_placard_corner`) does.

use super::super::*;
use super::dither::{offscreen, read_pixels};
use super::{headless_dqp, view};

/// Every world backing its card with nothing (`Diagonal`/`Bars`/`Ruled`),
/// derived from the roster rather than named.
fn bare_plate_worlds() -> Vec<&'static str> {
    crate::theme::THEMES
        .iter()
        .filter(|t| t.render_caps.list_style.list_backing(false) == theme::ListBacking::BarePlates)
        .map(|t| t.name)
        .collect()
}

/// Every world whose title composes as a canvas-anchored wordmark placard,
/// derived from the roster rather than named.
fn placard_worlds() -> Vec<&'static str> {
    crate::theme::THEMES
        .iter()
        .filter(|t| matches!(t.render_caps.title_style, theme::TitleStyle::Placard { .. }))
        .map(|t| t.name)
        .collect()
}

/// A theme-picker-shaped fixture: `n` candidates, a real hint, selection on
/// the last row (the one nearest the footer/placard, the adversarial end of
/// the list).
fn theme_view(n: usize) -> ViewState {
    let mut v = view("hello world\nsecond line\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = "themes".to_string();
    v.overlay_hint = "type to filter   ↵ keep   esc revert".to_string();
    v.overlay_items = (0..n).map(|i| format!("World candidate {i}")).collect();
    v.overlay_selected = n.saturating_sub(1);
    v
}

fn rect_overlap(
    (ax, ay, aw, ah): (f32, f32, f32, f32),
    (bx, by, bw, bh): (f32, f32, f32, f32),
) -> bool {
    ax < bx + bw && bx < ax + aw && ay < by + bh && by < ay + ah
}

/// One rendered frame's pixels, mirroring `frost_footprint.rs`'s own helper.
fn render_frame(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &mut TextPipeline,
    w: u32,
    h: u32,
) -> Vec<[u8; 4]> {
    p.prepare(device, queue, w, h).unwrap();
    let (texture, tview) = offscreen(device, w, h);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("awl footer gap magnitude encoder"),
    });
    p.render(&mut encoder, &tview).unwrap();
    queue.submit(Some(encoder.finish()));
    read_pixels(device, queue, &texture, w, h)
}

/// How many pixels inside `rect` differ between two same-sized frames.
fn box_residue(
    a: &[[u8; 4]],
    b: &[[u8; 4]],
    width: u32,
    height: u32,
    rect: (f32, f32, f32, f32),
) -> usize {
    let x0 = rect.0.floor().max(0.0) as u32;
    let x1 = ((rect.0 + rect.2).ceil() as u32).min(width);
    let y0 = rect.1.floor().max(0.0) as u32;
    let y1 = ((rect.1 + rect.3).ceil() as u32).min(height);
    let mut n = 0;
    for y in y0..y1 {
        for x in x0..x1 {
            let i = (y * width + x) as usize;
            if a[i] != b[i] {
                n += 1;
            }
        }
    }
    n
}

/// LAW A. Swept over the bare-plate roster (derived), both DPIs, and a few
/// vs. many candidate shape — the axis the item names explicitly, since a
/// window-clamped tall list absorbs overhead by dropping a row rather than
/// by shrinking the gap row's own compact height (`overlay_height_clamp_law`'s
/// own documented absorption shape), which is exactly the case where a
/// magnitude regression could hide.
#[test]
fn the_footer_gap_clears_the_retired_dials_own_ceiling_on_every_bare_plate_world() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!(
            "skipping \
             the_footer_gap_clears_the_retired_dials_own_ceiling_on_every_bare_plate_world: \
             no wgpu adapter"
        );
        return;
    };
    let worlds = bare_plate_worlds();
    assert!(
        !worlds.is_empty(),
        "no world enrols in the bare-plate roster — this law has no subject"
    );
    let (lw, lh_win) = (1200u32, 800u32);
    let mut graded = 0usize;
    let mut few_graded = 0usize;
    let mut many_graded = 0usize;
    for world in &worlds {
        crate::theme::set_active_by_name(world).unwrap();
        p.sync_theme();
        for dpi in [1.0f32, 2.0] {
            p.set_dpi(dpi);
            let (cw, ch) = ((lw as f32 * dpi) as u32, (lh_win as f32 * dpi) as u32);
            p.set_size(cw as f32, ch as f32);
            for (shape, n) in [("few", 3usize), ("many", 40usize)] {
                let v = theme_view(n);
                p.set_view(&v);
                p.prepare(&device, &queue, cw, ch).unwrap();
                let ctx = format!("{world} dpi={dpi} shape={shape}");
                let lh = p.overlay_lh();
                let Some((content_bottom, hint_top, hint_bottom)) = p.overlay_hint_gap_probe(cw)
                else {
                    panic!("{ctx}: this fixture always sets a hint, but none was drawn");
                };
                // PRESENCE FLOORS, so the magnitude claim below cannot be
                // satisfied by either side of the gap collapsing to nothing.
                assert!(
                    hint_bottom > hint_top + 1.0,
                    "{ctx}: the hint's own drawn line has near-zero height"
                );
                assert!(
                    content_bottom > 0.0,
                    "{ctx}: the content band's own bottom reads as collapsed"
                );
                let gap = hint_top - content_bottom;
                // THE FLOOR — strictly above what the RETIRED 0.45-row dial
                // could ever have produced at this world/DPI/shape's own line
                // height, so reverting the magnitude bump alone (leaving
                // every other mechanism untouched) turns this red even
                // though `hint_gap.rs`'s mere-presence law stays green.
                let retired_ceiling = (lh * 0.45).round();
                assert!(
                    gap > retired_ceiling,
                    "{ctx}: the footer gap ({gap}px) does not clear the retired dial's own \
                     ceiling ({retired_ceiling}px at {lh}px line height) — the widened \
                     separator did not actually widen anything here"
                );
                graded += 1;
                match shape {
                    "few" => few_graded += 1,
                    "many" => many_graded += 1,
                    _ => unreachable!(),
                }
            }
        }
    }
    p.set_dpi(1.0);
    theme::set_active(theme::DEFAULT_THEME);
    assert!(graded > 20, "the sweep must actually run, got {graded}");
    assert!(
        few_graded > 5 && many_graded > 5,
        "both the few- and many-candidate shapes must be reached: few={few_graded} \
         many={many_graded}"
    );
}

/// LAW B. Over the placard-title roster (derived), at the canonical canvas
/// and a short, cramped one, with the picker's REAL item count (every
/// shipped world — the shape that actually puts rows under the placard on
/// Firetail): no candidate row's own rendered pixels change between the
/// world's authored `TitleStyle` and the placard suppressed outright. A
/// presence floor requires the swept cells to actually put at least one row
/// under the placard's box — otherwise the masking claim was never really
/// exercised — and a second requires the placard to have actually drawn
/// somewhere (title style truly toggled, not a no-op override).
#[test]
fn no_candidate_row_pixel_changes_when_the_pickers_own_placard_draws() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!(
            "skipping \
             no_candidate_row_pixel_changes_when_the_pickers_own_placard_draws: no wgpu adapter"
        );
        return;
    };
    let worlds = placard_worlds();
    assert!(
        !worlds.is_empty(),
        "no world enrols in the placard-title roster — this law has no subject"
    );
    let n = crate::theme::THEMES.len();
    let mut rows_checked = 0usize;
    let mut placard_collisions = 0usize;
    let mut placard_drew_somewhere = 0usize;
    for world in &worlds {
        crate::theme::set_active_by_name(world).unwrap();
        p.sync_theme();
        for (cw, ch) in [(1200u32, 800u32), (900u32, 460u32)] {
            p.set_dpi(1.0);
            p.set_size(cw as f32, ch as f32);
            let mut v = theme_view(n);
            v.overlay_selected = n - 1;
            p.set_view(&v);
            let a = render_frame(&device, &queue, &mut p, cw, ch);
            let geom = p.overlay_geometry(cw);
            let plan = p.overlay_row_plan(&geom);
            let placard = p.overlay_shape_placard(&geom);

            crate::render::set_title_style_test_override(Some(theme::TitleStyle::InlinePrefix));
            let b = render_frame(&device, &queue, &mut p, cw, ch);
            crate::render::set_title_style_test_override(None);

            let Some(placard) = placard else {
                continue;
            };
            placard_drew_somewhere += 1;
            // PRESENCE — the two frames must genuinely differ where the
            // placard itself drew, or the override suppressed nothing.
            let placard_residue = box_residue(&a, &b, cw, ch, placard);
            assert!(
                placard_residue > 0,
                "{world} {cw}x{ch}: suppressing the placard produced no pixel change inside \
                 its own box — the override did not actually turn it off"
            );

            // Row ink GROWN to whatever actually opaquely protects it (a Bars
            // plate's own scrim, styles-safe on every list style, empty a
            // real answer) — the production owner of "what covers this row,"
            // never re-derived. A Bars plate hugs its row's own label and
            // stops SHORT of the full line-height glyph box on either side
            // (`BarConfig::SHIPPED.gap`, the visual breathing room between
            // plates): that inter-plate sliver shows the card's own ground
            // through by design, and only a genuinely UNBLURRED chrome layer
            // (the picker's own title placard, never a document) can leave a
            // step there — so the claim below is scoped to what the row's
            // own ink is actually backed by, not the taller glyph-box the
            // shaper reserves around it.
            let row_ink = p.overlay_row_ink_probe();
            let first_row_line = geom.shaped_first_row_line();
            for row in plan.rows().iter().filter(|r| r.item.is_some()) {
                let Some(row_box) = p.overlay_line_glyph_box(first_row_line + row.display) else {
                    continue;
                };
                let row_box = (row_box[0], row_box[1], row_box[2], row_box[3]);
                if rect_overlap(row_box, placard) {
                    placard_collisions += 1;
                }
                // Intersect the glyph box with whichever row-ink rect it
                // sits inside, if any — the OPAQUE region this row is
                // actually promised. No match (a style with no per-row
                // backing at all) keeps the full glyph box: nothing narrows
                // the claim for a composition that never made it.
                // 2px inward on every side of the row-ink rect itself (never
                // the glyph box, which may already be the fallback) — the
                // same anti-aliased-skirt tolerance `CardInk`'s own dilation
                // uses elsewhere in this suite: a scrim's rounded edge blends
                // over a physical pixel or two, and grading that seam as
                // "not masked" would fail on the plate's own antialiasing
                // rather than on anything behind it.
                const EDGE_AA_TOLERANCE: f32 = 4.0;
                // The BEST-overlapping row-ink rect, not the first one that
                // touches at all — two rows can share a boundary within
                // float rounding, and `rect_overlap`'s strict `<` is not
                // itself immune to that on a shared edge; picking by area
                // can't be fooled by a sliver from the row above or below.
                let overlap_area = |&[ix, iy, iw, ih]: &[f32; 4]| {
                    let x0 = row_box.0.max(ix);
                    let y0 = row_box.1.max(iy);
                    let x1 = (row_box.0 + row_box.2).min(ix + iw);
                    let y1 = (row_box.1 + row_box.3).min(iy + ih);
                    (x1 - x0).max(0.0) * (y1 - y0).max(0.0)
                };
                // Only a genuine PLATE narrows the claim — a `Ruled` world's
                // hairline rule sits in `row_ink` too but backs nothing (its
                // own row draws bare, trusting the frost behind it), so
                // requiring near-full row height keeps a rule from being
                // mistaken for opaque backing.
                let backed = row_ink
                    .iter()
                    .filter(|&&[.., ih]| ih >= row_box.3 * 0.5)
                    .filter(|r| overlap_area(r) > 1.0)
                    .max_by(|a, b| overlap_area(a).total_cmp(&overlap_area(b)))
                    .map(|&[ix, iy, iw, ih]| {
                        let x0 = row_box.0.max(ix + EDGE_AA_TOLERANCE);
                        let y0 = row_box.1.max(iy + EDGE_AA_TOLERANCE);
                        let x1 = (row_box.0 + row_box.2).min(ix + iw - EDGE_AA_TOLERANCE);
                        let y1 = (row_box.1 + row_box.3).min(iy + ih - EDGE_AA_TOLERANCE);
                        (x0, y0, (x1 - x0).max(0.0), (y1 - y0).max(0.0))
                    })
                    .unwrap_or(row_box);
                assert!(
                    backed.2 > 1.0 && backed.3 > 1.0,
                    "{world} {cw}x{ch}: candidate row {} has no real backed region to grade \
                     (glyph box {row_box:?}, row ink {row_ink:?})",
                    row.display
                );
                let residue = box_residue(&a, &b, cw, ch, backed);
                assert!(
                    residue == 0,
                    "{world} {cw}x{ch}: candidate row {} changed by {residue}px inside its \
                     own backed region when the picker's own placard drew — its label is \
                     not fully masked from the busiest ground in the room",
                    row.display
                );
                rows_checked += 1;
            }
        }
    }
    p.set_dpi(1.0);
    theme::set_active(theme::DEFAULT_THEME);
    assert!(
        rows_checked > 30,
        "the row sweep must actually run, got {rows_checked}"
    );
    assert!(
        placard_drew_somewhere > 3,
        "the placard must actually draw somewhere in this sweep, got {placard_drew_somewhere}"
    );
    assert!(
        placard_collisions > 0,
        "no cell in this sweep ever put a row under the placard — the adversarial case this \
         law exists for never occurred, so masking was never really exercised"
    );
}

/// NON-VACUITY for the law above, on a world with NOTHING backing its rows
/// (`Diagonal`, so there is no plate to mask a real collision): forcing the
/// placard onto the CARD'S OWN corner — the exact misplacement
/// `derived_placard_corner`'s complementary rule exists to prevent — must
/// produce real residue in a row's own glyph box, so the check above is
/// provably capable of going red rather than passing by construction.
#[test]
fn the_placard_masking_check_fires_on_an_unbacked_row_forced_under_it() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!(
            "skipping \
             the_placard_masking_check_fires_on_an_unbacked_row_forced_under_it: no wgpu adapter"
        );
        return;
    };
    // Mangrove: `Diagonal` (draws no row plate at all — `draws_row_plates`)
    // and `CardAnchor::TopRight`, so the production Auto corner resolves to
    // BL (`derived_placard_corner`). Forcing TR — the CARD'S OWN corner — is
    // the misplacement that rule exists to prevent, on the one style with
    // nothing to mask it if it happened for real.
    crate::theme::set_active_by_name("Mangrove").unwrap();
    p.sync_theme();
    p.set_dpi(1.0);
    p.set_size(1200.0, 800.0);
    let n = crate::theme::THEMES.len();
    let mut v = theme_view(n);
    v.overlay_selected = n - 1;
    p.set_view(&v);
    let a = render_frame(&device, &queue, &mut p, 1200, 800);
    let geom = p.overlay_geometry(1200);
    let plan = p.overlay_row_plan(&geom);

    crate::render::set_title_style_test_override(Some(theme::TitleStyle::Placard {
        corner: theme::PlacardCorner::TR,
        scale: 4.5,
        ink: theme::PlacardInk::Bold,
    }));
    let placard = p
        .overlay_shape_placard(&geom)
        .expect("a forced Placard style must shape a wordmark");
    let b = render_frame(&device, &queue, &mut p, 1200, 800);
    crate::render::set_title_style_test_override(None);
    theme::set_active(theme::DEFAULT_THEME);

    let first_row_line = geom.shaped_first_row_line();
    let mut collided_with_residue = false;
    let mut any_collision = false;
    for row in plan.rows().iter().filter(|r| r.item.is_some()) {
        let Some(row_box) = p.overlay_line_glyph_box(first_row_line + row.display) else {
            continue;
        };
        let row_box = (row_box[0], row_box[1], row_box[2], row_box[3]);
        if !rect_overlap(row_box, placard) {
            continue;
        }
        any_collision = true;
        if box_residue(&a, &b, 1200, 800, row_box) > 0 {
            collided_with_residue = true;
        }
    }
    assert!(
        any_collision,
        "forcing the placard onto the card's own corner must produce a real row/placard \
         geometric intersection on this unbacked style — otherwise the mutation set up no \
         adversarial case at all"
    );
    assert!(
        collided_with_residue,
        "an unbacked row forced under the placard must show real pixel residue — otherwise \
         the masking law above could never fire"
    );
}
