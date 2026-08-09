//! TWO ROWS IN THE SAME (DISABLED) STATE DREW THEIR ACCESSORY
//! COLUMN IN DIFFERENT INKS.
//!
//! **THE PREMISE HELD, RE-MEASURED AGAINST HEAD.** Copy and Paste are not both
//! disabled in the current `context_menu::rows` roster (only Cut/Copy are, on
//! the `Body` target the plain right-click reaches) — so the item's literal
//! pair was stale — but its own hypothesis, checked FIRST per its own
//! instruction, was correct: **the accessory ink IS resolved from the wrong
//! row's state**, not a `faint()`/`muted()` split.
//!
//! **THE MECHANISM, diagnosed before any colour changed.** `shape_overlay_right`
//! already resolves each row's secondary ink from ITS OWN row index
//! (`vis.reads_selected(li)`, `li` the enumeration index into `bind_strs`) —
//! that part was already correct, and is a DIFFERENT mechanism from a
//! shared-per-frame ink constant applied to every instance alike (the class
//! a range-rail thumb regression exposed on a neighbouring surface, fixed by
//! resolving each instance's ink separately). The defect was one row
//! EARLIER, in how `right_bind_lines` (`overlay_shape.rs`) turns those per-row
//! strings into buffer LINES: `let leads = if k == 0 { header_rows.max(1) }
//! else { 1 };` forced the FIRST label to lead with at least one blank line
//! EVEN WHEN `header_rows` is `0` — true only for a CONTEXTUAL card
//! (`overlay.rs`'s `!contextual` gate), i.e. only the right-click Context menu,
//! the one kind that combines `header_rows == 0` with a non-empty secondary
//! column. `OverlayRowPlan::secondary_top`'s own doc states the buffer's
//! contract exactly: `secondary_top() + (header_rows + r) * lh == row_top(r)`
//! — the leading count must be `header_rows`, not `header_rows.max(1)`.
//!
//! **THE OBSERVED EFFECT:** every row's secondary CONTENT — text and the ink
//! resolved for it — landed on the display row ONE BELOW its own. Row 0 (the
//! menu's default-selected row) drew no accessory at all; row 1 carried row
//! 0's text in row 0's ink (flipped, if row 0 read selected); and so on down
//! the list. Proven with the authoritative oracle — `overlay_ink_flip_probe`,
//! which reads the colour actually committed to the shaped glyphs, never a
//! parallel recomputation — mutated back to the pre-fix line and watched fail
//! by name (panic text pasted in the landing note).
//!
//! **GROUND-TRUTH ILLEGIBILITY, three worlds, before the fix:** the
//! selected row's flip ink, landing one row down on a plain (un-banded) row,
//! measured **ΔE 0.0 (byte-identical)** against that row's own ground on
//! Wagtail, **ΔE 1.9** (under the 2.3 JND) on Cassowary, and **ΔE 7.5** on
//! Firetail (`selected_row_secondary_ink` there is `#17090c` — the same
//! constant already recorded elsewhere as byte-identical to Firetail's OWN
//! `base_100`, on that neighbouring rail surface).
//!
//! **THE FIX** is a one-line change in `right_bind_lines`: lead with
//! `header_rows`, not `header_rows.max(1)`. Byte-identical for every other
//! kind (`header_rows` is always `1` there, so `.max(1)` was already a no-op).

use super::super::*;
use super::{headless_dqp, pixeldiff, view};
use crate::context_menu::{ContextState, ContextTarget};

/// A REAL contextual-menu `ViewState` for `target`, built through the actual
/// production policy (`context_menu::rows`) rather than a synthetic
/// candidate list — the same rows/labels/`enabled` flags `App::on_right_press`
/// would summon. `selected` places the keyboard/mouse highlight, so the law
/// can sweep every row position rather than only the default (row 0), which
/// is the one position the off-by-one bug's OWN geometry could not expose
/// (its receiving row falls off the end).
fn context_menu_view(target: ContextTarget, state: ContextState, selected: usize) -> ViewState {
    let rows = crate::context_menu::rows(target, state, crate::commands::Platform::Native);
    let labels: Vec<String> = rows.iter().map(|r| r.label.to_string()).collect();
    let secondaries: Vec<String> = rows
        .iter()
        .map(|r| {
            if r.enabled {
                String::new()
            } else {
                "unavailable".to_string()
            }
        })
        .collect();
    let mut v = view("hello world\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = crate::overlay::OverlayKind::Context.title();
    v.overlay_items = labels;
    v.overlay_bindings = secondaries;
    v.overlay_selected = selected.min(rows.len().saturating_sub(1));
    v.overlay_context_anchor = Some((300.0, 300.0));
    v
}

/// The real-app states that reach a Context menu with at least one DISABLED
/// row — derived from the roster (every `ContextTarget` × one state that
/// starves every gate `context_menu::rows` reads: no selection, no named
/// file), never a hardcoded "Body" name. `document_target` picks `Body` for
/// this state; `Filename` is reached directly (the gutter's own context
/// summon), so both are swept by construction — a target this state cannot
/// starve (Link/Heading/Folder/Edge, all unconditionally enabled) simply
/// contributes nothing, which is itself asserted below (non-vacuity).
fn starved_state() -> ContextState {
    ContextState {
        has_selection: false,
        link: false,
        heading: false,
        heading_folded: false,
        misspelled: false,
        named_file: false,
    }
}

/// This law's own enrolled targets: whichever `ContextTarget`s `rows()`
/// actually hands back a disabled row for, under [`starved_state`] — read off
/// the roster, not named.
fn enrolled_targets() -> Vec<ContextTarget> {
    let state = starved_state();
    ContextTarget::ALL
        .iter()
        .copied()
        .filter(|&t| {
            crate::context_menu::rows(t, state, crate::commands::Platform::Native)
                .iter()
                .any(|r| !r.enabled)
        })
        .collect()
}

/// MECHANISM LAW — cheap, exact, no GPU pixels: the disabled row's accessory
/// ink is read from THAT row's own selection state, never the row below (or
/// above) it. `overlay_ink_flip_probe` reads the colour actually committed to
/// the shaped glyphs, so this cannot be satisfied by a parallel
/// reimplementation of the bug.
#[test]
fn disabled_row_accessory_ink_reads_its_own_rows_state_not_a_neighbours() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping accessory-ink flip-row mechanism law: no wgpu adapter");
        return;
    };
    let targets = enrolled_targets();
    assert!(
        !targets.is_empty(),
        "the sweep's own enrolment starved to nothing — this law would prove nothing"
    );
    let mut graded = 0usize;
    for &target in &targets {
        let state = starved_state();
        let n = crate::context_menu::rows(target, state, crate::commands::Platform::Native).len();
        for selected in 0..n {
            let v = context_menu_view(target, state, selected);
            p.set_view(&v);
            p.prepare(&device, &queue, 1200, 800).unwrap();
            let geom = p.overlay_geometry(1200);
            let (_, secondary_flip_rows) = p.overlay_ink_flip_probe(&geom);
            // The row that reads selected is `selected` itself (a settled,
            // unarmed headless pipeline carries exactly the logical row — see
            // `VisualSelection::rows`'s own doc). If that row's secondary text
            // is non-empty (it is disabled here), its flip — if the world's
            // list style flips at all — must land ON row `selected`, never on
            // any other row.
            for &flipped in &secondary_flip_rows {
                assert_eq!(
                    flipped, selected,
                    "{target:?} selected={selected}: a secondary-column glyph read as \
                     FLIPPED (on-band) ink on display row {flipped}, not the selected row \
                     itself — the accessory ink is resolved from the WRONG row's state \
                     (the off-by-one `right_bind_lines` regression item 299 was filed \
                     against, one row downstream of the row that actually earned it)"
                );
            }
            graded += 1;
        }
    }
    assert!(graded > 3, "the sweep must actually run, got {graded}");
}

/// The tight canvas rect a disabled row's "unavailable" accessory glyphs
/// occupy — right-aligned against the card's own text edge, using the SAME
/// geometry the draw pass places it at (`overlay_row_secondary_px` for the
/// measured width, `overlay_text_hpad`/`overlay_card_rect` for the right
/// edge), never a fraction-of-the-row guess.
fn accessory_region(
    p: &TextPipeline,
    geom: &crate::render::chrome::OverlayGeom,
    card: [f32; 4],
    row: &crate::render::plan::PlannedRow,
) -> Option<pixeldiff::Region> {
    let w = *p.overlay_row_secondary_px(geom).get(&row.display)?;
    let hpad = p.overlay_text_hpad();
    let right_edge = card[0] + card[2] - hpad;
    Some(pixeldiff::Region::new(
        right_edge - w - 2.0,
        row.top,
        w + 4.0,
        row.bottom() - row.top,
    ))
}

/// The row band's own clean background — the modal colour over the region
/// (glyph ink is a small minority of a text row's area), so this reads
/// whatever surface is actually drawn there: plain card ground, an unselected
/// row, OR a selected band/plate — never a recomputation of what SHOULD be
/// there.
fn region_mode_color(pixels: &[[u8; 4]], width: i64, r: pixeldiff::Region) -> [u8; 4] {
    use std::collections::HashMap;
    let mut counts: HashMap<[u8; 4], usize> = HashMap::new();
    for y in r.y.max(0)..(r.y + r.h) {
        for x in r.x.max(0)..(r.x + r.w) {
            let idx = (y * width + x) as usize;
            if idx < pixels.len() {
                *counts.entry(pixels[idx]).or_insert(0) += 1;
            }
        }
    }
    counts
        .into_iter()
        .max_by_key(|(_, n)| *n)
        .map(|(c, _)| c)
        .unwrap_or([0, 0, 0, 0])
}

/// ΔE floor an ink must clear against the ground it is ACTUALLY drawn on to
/// count as legible — the same number `range_rail.rs` already holds a picker
/// affordance's own ink to twice (`RAIL_INK_PRESENCE_MIN`/
/// `THUMB_PRESENCE_MIN`), reused rather than inventing a second number for
/// the same question ("does this ink clear the ground it sits on").
const ACCESSORY_INK_CONTRAST_FLOOR: f64 = 3.0;

/// APPEARANCE LAW — real GPU pixels. Every disabled row's "unavailable"
/// accessory clears a legibility floor against its OWN row's drawn ground,
/// swept over the world roster × selected-row position (the axis the offset
/// hypothesis lives on — a selected-row-0 fixture alone could not see a
/// receiving row that only exists for `selected >= 1`) × 1x/2x DPI. Paired
/// with a PRESENCE floor (the ink must be found at all) so the contrast floor
/// cannot be satisfied by the accessory failing to draw.
#[test]
fn every_disabled_rows_accessory_clears_a_contrast_floor_against_its_own_row() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping accessory-ink contrast-floor law: no wgpu adapter");
        return;
    };
    let targets = enrolled_targets();
    assert!(!targets.is_empty(), "enrolment starved to nothing");
    let names = crate::theme::world_names();
    let mut graded = 0usize;
    let mut presence_graded = 0usize;
    for dpi in [1.0f32, 2.0] {
        p.set_dpi(dpi);
        let (cw, ch) = ((1200.0 * dpi) as u32, (800.0 * dpi) as u32);
        p.set_size(cw as f32, ch as f32);
        for &world in &names {
            crate::theme::set_active_by_name(world).expect("a named world exists");
            p.sync_theme();
            for &target in &targets {
                let state = starved_state();
                let rows =
                    crate::context_menu::rows(target, state, crate::commands::Platform::Native);
                for selected in 0..rows.len() {
                    let v = context_menu_view(target, state, selected);
                    p.set_view(&v);
                    p.prepare(&device, &queue, cw, ch).unwrap();
                    let geom = p.overlay_geometry(cw);
                    let plan = p.overlay_row_plan(&geom);
                    let card = p.overlay_card_rect().expect("the context card is open");
                    let pixels = pixeldiff::render_frame(&mut p, &device, &queue, cw, ch);
                    for row in plan.rows() {
                        if rows.get(row.display).is_some_and(|r| r.enabled) {
                            continue; // an enabled row draws no accessory at all
                        }
                        let ctx = format!(
                            "{world}/{target:?} dpi={dpi} selected={selected} row={}",
                            row.display
                        );
                        let Some(region) = accessory_region(&p, &geom, card, row) else {
                            panic!(
                                "{ctx}: a DISABLED row drew no secondary text at all — \
                                 its own \"unavailable\" accessory is missing (this is \
                                 exactly the off-by-one's row-0 symptom: the receiving row \
                                 shows nothing while its content silently landed on the \
                                 row below)"
                            );
                        };
                        let bg = region_mode_color(&pixels, cw as i64, region);
                        let ink = pixeldiff::dominant_ink_color(
                            &pixels, cw as i64, ch as i64, region, bg, 16,
                        );
                        let Some(ink) = ink else {
                            panic!(
                                "{ctx}: found no ink pixels in the accessory's own region \
                                 (bg={bg:?}) — the presence floor: a contrast floor here \
                                 would be satisfied by the text having vanished"
                            );
                        };
                        presence_graded += 1;
                        let de = pixeldiff::delta_e(ink, bg);
                        assert!(
                            de >= ACCESSORY_INK_CONTRAST_FLOOR,
                            "{ctx}: accessory ink {ink:?} clears only ΔE {de:.2} against its \
                             own row's ground {bg:?} (floor {ACCESSORY_INK_CONTRAST_FLOOR}) — \
                             the near-invisible pairing item 299 was filed against"
                        );
                        graded += 1;
                    }
                }
            }
        }
    }
    p.set_dpi(1.0);
    theme::set_active(theme::DEFAULT_THEME);
    assert!(graded > 100, "the sweep must actually run, got {graded}");
    assert_eq!(
        graded, presence_graded,
        "every graded cell must have passed the presence floor too"
    );
}
