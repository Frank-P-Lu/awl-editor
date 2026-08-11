//! **THE ONE SURFACE WHOSE WHOLE PURPOSE IS CHOOSING BY APPEARANCE, AND THE ONE
//! PICKER NOTHING COULD GRADE.**
//!
//! Every other picker's selection is graded by a true A/B — one row, two frames,
//! selected and not — because stepping the selection changes nothing else. The
//! theme picker cannot be driven that way: its selection AUDITIONS the world
//! (`actions::overlay_nav::preview_overlay`, reached from every input kind through
//! `preview_move`), so two frames of a moving selection are two DIFFERENT WORLDS
//! and their difference says nothing about selection. The world even relocates the
//! card (`reanchor_crossing_law`), so the rows are not in the same place twice.
//!
//! `scripts/pretag-journeys.py` therefore grades this picker with a within-frame
//! substitute — the selected row's textless tail against its unselected
//! neighbours' — and that substitute can only speak where two UNSELECTED rows are
//! pixel-interchangeable. On a textured ground (Galah, Paperbark) or a staggered
//! composition (Magpie, Mangrove) they are not: two unselected rows already differ
//! by as much as the selection does, so the sweep abstains by name on most of the
//! roster rather than reporting a number it cannot stand behind.
//!
//! # The seam, and why it is not a new door
//!
//! The audition lives in the ACTION path, not the render path.
//! `preview_overlay` is a side effect of moving `OverlayState::selected`; the
//! renderer receives the world (`theme::active()`, pinned here by
//! [`theme::WorldPin`]) and the selection (`ViewState::overlay_selected`) as two
//! INDEPENDENT inputs and has never had a channel between them — `overlay_items`
//! are plain strings, and no row treatment can know that a row names a world.
//!
//! So holding the world still while the selection moves needs NO new seam: it is
//! the ordinary `ViewState` boundary every render law in this directory already
//! drives, and there is nothing here the product could reach that it could not
//! reach before. The product's own coupling is proved separately and stays
//! proved: `overlay::tests::only_the_declared_auditions_move_the_live_editor`
//! asserts that a theme card's selection move really does move the live world,
//! and that exactly the kinds declaring an audition do so.
//!
//! What that costs in honesty, stated rather than hidden: frame A here is the
//! SHIPPED state (world W, its own row selected — exactly what a user sees), and
//! frame B is world W with a different row selected, which the product never
//! shows. The law therefore grades the TREATMENT — "does this world draw a
//! selected row differently from the same row unselected" — which is the question
//! the abstention left unanswered, and not a claim that frame B is reachable.
//!
//! # The oracle
//!
//! Every number is one rendered pixel against another rendered pixel from the same
//! run on the same device; no authored colour appears in any comparison, because
//! the same frame is `[230,230,230]` on Metal and `[227,227,228]` on lavapipe.
//!
//! * **PRESENCE** — asserted first and counted separately. A "the selected row is
//!   distinct" floor is satisfied perfectly by a card that draws no rows, and
//!   separately by a treatment fading toward the page. So each graded row must
//!   carry real INK over its own ground in BOTH frames before any difference
//!   between the frames is believed.
//! * **THE FLOOR** — the row that gained the selection, and the row that lost it,
//!   must each CHANGE: over area (pixels differing at all) and in magnitude (the
//!   largest ΔE any pixel of that row moved). Area alone passes a treatment washed
//!   to four bytes off the page; magnitude alone passes a single stray pixel.
//! * **THE CONTROL** — a row far from both must be BYTE-IDENTICAL across the two
//!   frames. This is the whole advantage of a true A/B over the within-frame
//!   substitute, and it is what makes the change attributable to the selection
//!   rather than to a textured ground or a staggered row: the ground, the
//!   stagger, the card, the document behind it and the world itself are all the
//!   same in both frames, so they cancel exactly instead of drowning the signal.
//!
//! # Proved non-vacuous on the cell that matters
//!
//! Broken deliberately on GALAH — a textured-ground `Bars` world, one of the
//! cells the pre-tag probe abstains on — by emitting no selected plate and
//! letting the unselected pass cover that row instead. The selected row then
//! renders BYTE-IDENTICALLY to an unselected one (0 px changed, ΔE 0.00) and
//! this law goes red naming Galah.
//!
//! Two partial breakages measured on the way there are worth recording, because
//! each shows what this law does NOT claim. Deleting only the selected plate
//! leaves the row the only UNPLATED one on a `BarCoverage::All` world — still
//! distinct, correctly still green. Deleting the plate but keeping the ink flip
//! drops Galah from ΔE 32.68 to 5.67, which clears this floor: a selected row
//! distinguished ONLY by an ink resolved for a band that is not under it is
//! `selected_secondary_ink_law`'s subject, not this one's, and that law grades it
//! on exactly that axis.

use super::super::*;
use super::{headless_dqp, pixeldiff};
use crate::overlay::OverlayState;

const LOGICAL: (f32, f32) = (1200.0, 800.0);

/// Horizontal slack around a row's own planned span, in LOGICAL px, scaled by the
/// frame's dpi at the call site. A selection treatment is not obliged to stay
/// inside the row box: a `Bars` plate's scrim bleeds outward by `BAR_SCRIM_PAD`,
/// and the selected-row marker stands on the row's OUTER edge. Padding sideways
/// is free (there is only page out there); padding VERTICALLY is not — it would
/// let a selected row's own bleed be counted as its neighbour's change — so the
/// row slot is taken exactly.
const SIDE_PAD: f32 = 16.0;

/// The least area, in device pixels, one exact colour must hold to count as a
/// row's INK rather than as an antialiased edge — the same reasoning and the same
/// number `selected_secondary_ink_law` spends on the identical question.
const INK_AREA_FLOOR: usize = 4;

/// ΔE a row's ink must clear against its own ground to count as drawn at all.
/// The just-noticeable difference; presence, not legibility (contrast floors over
/// theme roles are their own laws).
const INK_PRESENT_DE: f64 = 2.3;

/// **THE MAGNITUDE FLOOR.** The largest ΔE any pixel of a row moves when that row
/// gains or loses the selection.
///
/// **The roster's tightest real value is ΔE 6.50 — Magpie**, whose `Diagonal`
/// composition selects with a thin ascending spine mark (weight 1.25, aperture
/// 0.45) and no row fill at all, and whose figure is at that reading barely
/// separated from the ground it crosses. The next tightest is Tawny at 14.66, and
/// the roster's loudest is Wagtail at 100.00 (`theme_picker_selection_report`
/// prints the whole table, tightest first).
///
/// So the floor sits at **4.0**: under Magpie by 38%, and still 1.7x the
/// just-noticeable 2.3 — high enough that a treatment washed toward the page
/// (the recorded "contrast floor got HAPPIER as its subject vanished at `0x04`
/// alpha" failure) is red long before a reader has to squint, and low enough
/// that it grades Magpie's authored thinness rather than legislating against it.
/// **It is not a legibility standard**: whether Magpie's 6.50 is loud ENOUGH is a
/// contrast question over theme roles, and belongs to those laws, not this one.
const MOVE_DE_FLOOR: f64 = 4.0;

/// **THE AREA FLOOR**, as a multiple of the row's own drawn HEIGHT rather than an
/// absolute count, so it scales with the device scale by itself and cannot be
/// satisfied on a Retina frame by the area that failed on a 1x one. The unit is
/// the thinnest mark this tree ever asks a reader to see — a caret: a one-pixel
/// column the full height of its row.
///
/// **The roster's tightest real value is 26.88 row-heights — Magpie again**, at
/// 1x with the menu bar on (731 device px over a 27.2px row); the loudest is
/// Bombora's full-width `Pane` band at 1084. The floor sits at **8.0**: 3.4x
/// under the tightest shipped world, and eight times the caret it is measured in,
/// so a treatment that shrank to a token mark fails here even while its colour
/// still clears the magnitude floor above.
const MOVE_AREA_ROWS: f32 = 8.0;

/// A faithful theme card, projected into a `ViewState` through the SAME accessors
/// `App::sync_view` reads it through (`item_strings`, `lens_strip`, `window_rows`,
/// `foot_hint`, `location`, `item_sections`) — so this fixture cannot drift into
/// a card shape the product never builds. The `overlay_lens` strip is what routes
/// the theme picker's own faceted geometry; a flat card would be a different
/// surface wearing this one's name.
fn theme_view(ov: &OverlayState, selected_item: usize) -> ViewState {
    let mut v = super::view("hello world\nsecond line\nthird line\n", 0, 0);
    v.overlay_active = true;
    v.overlay_crisp = ov.kind.keeps_backdrop_crisp();
    v.overlay_title = ov.kind.title();
    v.overlay_items = ov.item_strings();
    v.overlay_sections = ov.item_sections();
    v.overlay_lens = ov.lens_strip();
    v.overlay_location = ov.location().map(str::to_string);
    v.overlay_hint = ov.foot_hint();
    v.overlay_window_rows = ov.window_rows();
    v.overlay_scroll = ov.scroll;
    v.overlay_selected = selected_item;
    v
}

/// The pixel region of display row `d`: its own planned x-span (`row_x_span` — the
/// ONE owner, shared with the pointer inverse and the published geometry, so a law
/// and a frame cannot disagree about where a row is) widened by [`SIDE_PAD`], over
/// its own vertical slot exactly.
fn row_region(
    plan: &crate::render::plan::OverlayRowPlan,
    d: usize,
    scale: f32,
) -> pixeldiff::Region {
    let (x0, x1) = plan.row_x_span(d).expect("a planned row has an x span");
    let row = &plan.rows()[d];
    let pad = SIDE_PAD * scale;
    pixeldiff::Region::new(x0 - pad, row.top, (x1 - x0) + 2.0 * pad, row.height)
}

/// The modal exact colour over a region — the surface actually drawn there, since
/// glyph ink is a minority of a row's area.
fn region_mode(pixels: &[[u8; 4]], width: i64, r: pixeldiff::Region) -> [u8; 4] {
    let mut counts: std::collections::HashMap<[u8; 4], usize> = Default::default();
    for y in r.y.max(0)..(r.y + r.h) {
        for x in r.x.max(0)..(r.x + r.w) {
            let idx = (y * width + x) as usize;
            if idx < pixels.len() {
                *counts.entry(pixels[idx]).or_default() += 1;
            }
        }
    }
    counts
        .into_iter()
        .max_by_key(|(_, n)| *n)
        .map(|(c, _)| c)
        .unwrap_or([0, 0, 0, 0])
}

/// Does this region carry INK — an exact colour holding real area at a visible
/// distance from the region's OWN modal surface? The presence question, asked of
/// the pixels rather than of the plan, because a plan that lists twenty rows and
/// a frame that draws none agree on everything except this.
fn has_ink(pixels: &[[u8; 4]], width: i64, r: pixeldiff::Region) -> bool {
    let ground = region_mode(pixels, width, r);
    let mut counts: std::collections::HashMap<[u8; 4], usize> = Default::default();
    for y in r.y.max(0)..(r.y + r.h) {
        for x in r.x.max(0)..(r.x + r.w) {
            let idx = (y * width + x) as usize;
            if idx < pixels.len() {
                *counts.entry(pixels[idx]).or_default() += 1;
            }
        }
    }
    counts
        .into_iter()
        .any(|(c, n)| n >= INK_AREA_FLOOR && pixeldiff::delta_e(c, ground) >= INK_PRESENT_DE)
}

/// The largest ΔE any single pixel of `r` moved between the two frames — the
/// magnitude half of "this row changed". Rendered against rendered; the region's
/// own two readings, never an authored colour.
fn max_delta_e(a: &[[u8; 4]], b: &[[u8; 4]], width: i64, r: pixeldiff::Region) -> f64 {
    let mut worst = 0.0f64;
    for y in r.y.max(0)..(r.y + r.h) {
        for x in r.x.max(0)..(r.x + r.w) {
            let idx = (y * width + x) as usize;
            if idx < a.len() && idx < b.len() && a[idx] != b[idx] {
                worst = worst.max(pixeldiff::delta_e(a[idx], b[idx]));
            }
        }
    }
    worst
}

/// One row's A/B reading.
struct Moved {
    /// Pixels of the row's region that differ between the two frames at all.
    area: usize,
    /// The largest ΔE any of them moved.
    de: f64,
    /// The row's own drawn height, in device px — the area floor's own scale.
    height: f32,
    ink_a: bool,
    ink_b: bool,
}

/// One world × one configuration, fully read: the row that GAINS the selection,
/// the row that LOSES it, the untouched control, and the shipped row's identity.
struct Cell {
    gained: Moved,
    lost: Moved,
    control_area: usize,
    control_display: usize,
    style: String,
    rows: usize,
}

/// Render world `w`'s theme card twice at the current dpi — the shipped frame
/// (its own row selected) and the same world with a distant row selected — and
/// read the three rows. `None` when the card plans too few item rows to seat a
/// control three clear of both graded rows, which is counted by the caller so a
/// silent skip cannot pass for a sweep.
#[allow(clippy::too_many_arguments)]
/// The three rows an A/B needs, off the card's own plan: every item row it drew,
/// the far end of the drawn band (six clear of the shipped selection, so neither
/// graded row is the other's neighbour), and a CONTROL three clear of both.
///
/// Three, because a plated world's scrim really does bleed into the adjacent row,
/// and that bleed is a rendering fact rather than this law's subject. `None` when
/// the card is too short to seat all three, which every caller counts so a silent
/// skip cannot pass for a sweep.
fn graded_rows(
    plan: &crate::render::plan::OverlayRowPlan,
    d_a: usize,
) -> Option<(Vec<usize>, usize, usize)> {
    let item_rows: Vec<usize> = plan
        .rows()
        .iter()
        .filter(|r| r.item.is_some())
        .map(|r| r.display)
        .collect();
    let d_b = *item_rows
        .iter()
        .max_by_key(|d| d.abs_diff(d_a))
        .filter(|d| d.abs_diff(d_a) >= 6)?;
    let (lo, hi) = (d_a.min(d_b), d_a.max(d_b));
    let d_c = *item_rows
        .iter()
        .find(|d| d.abs_diff(lo) >= 3 && d.abs_diff(hi) >= 3)?;
    Some((item_rows, d_b, d_c))
}

fn probe_world(
    p: &mut TextPipeline,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    cw: u32,
    ch: u32,
    scale: f32,
    world: &str,
) -> Option<Cell> {
    let names: Vec<String> = theme::world_names()
        .iter()
        .map(|n| (*n).to_string())
        .collect();
    let ov = OverlayState::new_theme(names, theme::active_index());
    // The SHIPPED frame: the world is `world` and the selected row is `world`'s
    // own — the audition's invariant, so frame A is a state the product really
    // renders rather than a fixture's invention.
    let mut v = theme_view(&ov, ov.selected);
    p.set_view(&v);
    p.prepare(device, queue, cw, ch).ok()?;

    // PIN THE ITEM WINDOW to the one this world's card just drew, and render frame
    // A again against it. The pipeline slides its own window to keep the selection
    // visible (`chrome::scroll_window`), so an A/B that only set `overlay_selected`
    // would compare two DIFFERENT slices of the roster in the same slots — every
    // row changed, nothing attributable. Pinning `overlay_scroll` to the drawn
    // window's own top is not a fixture's liberty: it is exactly the state
    // `OverlayState::scroll_to_selected` leaves behind, and moving the selection
    // WITHIN the visible window is the one motion that leaves the product's own
    // scroll untouched. Both graded rows are chosen from inside it, so the window
    // is stable by construction and asserted so below.
    let first = p.overlay_row_plan(&p.overlay_geometry(cw));
    let window_top = first
        .rows()
        .iter()
        .find_map(|r| r.item)
        .expect("a theme card draws item rows");
    v.overlay_scroll = window_top;
    p.set_view(&v);
    p.prepare(device, queue, cw, ch).ok()?;
    let geom = p.overlay_geometry(cw);
    let plan = p.overlay_row_plan(&geom);
    let d_a = plan.selected_display()?;
    let (item_rows, d_b, d_c) = graded_rows(&plan, d_a)?;
    // The DRAWN selection must have settled on the logical row, or the A/B is
    // reading a glide rather than a selection.
    let vis = p.resolve_visual_selection(&geom, &plan);
    assert_eq!(
        vis.rows(),
        [d_a],
        "{world}: the drawn selection has not settled on the logical row — this \
         frame is mid-glide and no A/B over it means anything"
    );
    let frame_a = pixeldiff::render_frame(p, device, queue, cw, ch);

    // FRAME B: the same world, a distant row selected. Only `overlay_selected`
    // moves; the world, the card, the ground, the stagger and the document behind
    // it are untouched, which is precisely what the capture door cannot arrange.
    v.overlay_selected = plan.item_at(d_b)?;
    p.set_view(&v);
    p.prepare(device, queue, cw, ch).ok()?;
    let geom_b = p.overlay_geometry(cw);
    let plan_b = p.overlay_row_plan(&geom_b);
    // THE PRECONDITION THE WHOLE A/B RESTS ON: every slot still holds the same
    // roster entry. A row-COUNT check would pass a window that slid by three and
    // re-lettered every row — the exact failure this assertion caught while the
    // law was being calibrated.
    let map = |pl: &crate::render::plan::OverlayRowPlan| {
        pl.rows()
            .iter()
            .map(|r| (r.display, r.item))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        map(&plan_b),
        map(&plan),
        "{world}: moving the selection re-windowed the card, so no row can be compared \
         to itself — the slots now hold different worlds"
    );
    assert_eq!(
        plan_b.selected_display(),
        Some(d_b),
        "{world}: the card did not put the selection on the row it was asked for"
    );
    let vis_b = p.resolve_visual_selection(&geom_b, &plan_b);
    assert_eq!(
        vis_b.rows(),
        [d_b],
        "{world}: the drawn selection has not settled on the logical row in frame B"
    );
    let frame_b = pixeldiff::render_frame(p, device, queue, cw, ch);

    let width = cw as i64;
    let read = |d: usize| -> Moved {
        let r = row_region(&plan, d, scale);
        let diff = pixeldiff::diff_region(&frame_a, &frame_b, width, ch as i64, r);
        Moved {
            area: diff.differing,
            de: max_delta_e(&frame_a, &frame_b, width, r),
            height: plan.rows()[d].height,
            ink_a: has_ink(&frame_a, width, r),
            ink_b: has_ink(&frame_b, width, r),
        }
    };
    // `d_b` GAINS the selection between A and B; `d_a` LOSES it.
    let gained = read(d_b);
    let lost = read(d_a);
    let control_area = pixeldiff::diff_region(
        &frame_a,
        &frame_b,
        width,
        ch as i64,
        row_region(&plan, d_c, scale),
    )
    .differing;
    Some(Cell {
        gained,
        lost,
        control_area,
        control_display: d_c,
        style: format!("{:?}", crate::render::effective_list_style()),
        rows: item_rows.len(),
    })
}

/// The tightest cell this run saw, on each of the two floors — carried so a green
/// result can REPORT the margin it passed by instead of only that it passed.
struct Tightest {
    de: f64,
    de_at: String,
    area: f32,
    area_at: String,
}

impl Tightest {
    fn new() -> Self {
        Tightest {
            de: f64::INFINITY,
            de_at: String::new(),
            area: f32::INFINITY,
            area_at: String::new(),
        }
    }
}

/// GRADE ONE CELL — one world at one device scale under one menu-bar arm — with
/// the three arms in the order that makes each one mean something: presence
/// first (a floor over a treatment is satisfied by deleting the treatment's
/// subject), then the two floors, then the control.
fn grade_cell(c: &Cell, ctx: &str, t: &mut Tightest) {
    // PRESENCE, before any difference is believed. An empty card satisfies "the
    // selected row differs from the unselected ones" perfectly.
    for (what, m) in [("gaining", &c.gained), ("losing", &c.lost)] {
        assert!(
            m.ink_a && m.ink_b,
            "{ctx}: the row {what} the selection carries NO ink over its own ground \
             (selected frame: {}, unselected frame: {}) — the card planned {} item rows \
             and drew an empty one, and every difference measured below would be a \
             difference between two blanks",
            m.ink_a,
            m.ink_b,
            c.rows
        );
    }

    // THE FLOOR — area AND magnitude, on BOTH the row that gains the selection and
    // the row that loses it. Area alone passes a treatment washed to a few bytes
    // off the page; magnitude alone passes one stray pixel.
    for (what, m) in [("gaining", &c.gained), ("losing", &c.lost)] {
        let area_floor = (m.height * MOVE_AREA_ROWS).ceil() as usize;
        if (m.area as f32 / m.height) < t.area {
            t.area = m.area as f32 / m.height;
            t.area_at = format!("{ctx} ({what})");
        }
        if m.de < t.de {
            t.de = m.de;
            t.de_at = format!("{ctx} ({what})");
        }
        assert!(
            m.area >= area_floor,
            "{ctx}: the row {what} the selection changed on only {} pixels, under \
             {area_floor} — {MOVE_AREA_ROWS} one-pixel columns the full height of its own \
             row ({:.1}px). The world is IDENTICAL in both frames, so whatever this world \
             does to say 'this row is selected' covers less area than a few carets.",
            m.area,
            m.height
        );
        assert!(
            m.de >= MOVE_DE_FLOOR,
            "{ctx}: the row {what} the selection moved by at most ΔE {:.2} on its loudest \
             pixel, under the {MOVE_DE_FLOOR} floor — it changed over {} pixels, so a \
             treatment IS being drawn, but it is washed so close to what was already there \
             that selecting the row is not visible.",
            m.de,
            m.area
        );
    }

    // THE CONTROL — the arm the within-frame substitute cannot have, and the one
    // that makes the difference above attributable to the SELECTION rather than to
    // this card's ground or its stagger.
    assert_eq!(
        c.control_area, 0,
        "{ctx}: display row {} neither gained nor lost the selection and yet changed on \
         {} pixels between the two frames. Something other than the selection moves when \
         the selection moves, so neither graded row's change can be attributed to it.",
        c.control_display, c.control_area
    );
}

/// **THE LAW.** Every shipped world, both device scales, both menu-bar arms: the
/// theme picker's selected row is VISIBLY selected — it is drawn, it changes when
/// it gains or loses the selection, and a row that neither gained nor lost it does
/// not move at all.
///
/// The worlds the pre-tag sweep abstains on — the textured grounds and the
/// staggered cards — are not an afterthought here; they are the reason the law
/// exists, and they enrol on exactly the same terms as the easy ones because the
/// A/B cancels a ground instead of competing with it.
#[test]
fn the_theme_pickers_selected_row_is_drawn_as_selected_on_every_world() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(LOGICAL.0, LOGICAL.1) else {
        eprintln!(
            "skipping the theme picker's selected-row A/B: no wgpu adapter (\
             the_theme_pickers_selected_row_is_drawn_as_selected_on_every_world)"
        );
        return;
    };
    // AMBIENT, never `cfg!` — `cfg!(target_os = ..)` inside a test reports the
    // host that COMPILED it, not the branch this process's flag actually took, so
    // a forced full-suite arm would be restored to the wrong value.
    let ambient_bar = crate::menubar::menu_bar_on();
    let saved_reduced = crate::motion::reduced();
    crate::motion::set_reduced(true);

    let mut graded = 0usize;
    let mut skipped: Vec<String> = Vec::new();
    let mut styles: std::collections::BTreeSet<String> = Default::default();
    let mut tightest = Tightest::new();

    for bar in [false, true] {
        crate::menubar::set_menu_bar_on(bar);
        for dpi in [1.0f32, 2.0] {
            p.set_dpi(dpi);
            let (cw, ch) = ((LOGICAL.0 * dpi) as u32, (LOGICAL.1 * dpi) as u32);
            p.set_size(cw as f32, ch as f32);
            for world in theme::world_names() {
                let _pin = theme::WorldPin::world(world).expect("a rostered world sets active");
                p.sync_theme();
                let Some(c) = probe_world(&mut p, &device, &queue, cw, ch, dpi, world) else {
                    skipped.push(format!("{world}@{dpi}x bar={bar}"));
                    continue;
                };
                let ctx = format!("{world} [{}] bar={bar} dpi={dpi}", c.style);
                styles.insert(c.style.split('(').next().unwrap_or("?").to_string());

                grade_cell(&c, &ctx, &mut tightest);
                graded += 1;
            }
        }
    }

    p.set_dpi(1.0);
    crate::motion::set_reduced(saved_reduced);
    crate::menubar::set_menu_bar_on(ambient_bar);
    theme::set_active(theme::DEFAULT_THEME);

    // WHAT THIS RUN ACTUALLY COVERED, printed rather than assumed — a green law
    // says nothing about the configuration it ran in unless it says so.
    eprintln!(
        "theme-picker selection A/B: graded {graded} cells ({} worlds x 2 dpi x 2 menu-bar \
         arms), skipped {skipped:?}. Tightest magnitude ΔE {:.2} at {} (floor \
         {MOVE_DE_FLOOR}); tightest area {:.2} row-heights at {} (floor {MOVE_AREA_ROWS}).",
        theme::THEMES.len(),
        tightest.de,
        tightest.de_at,
        tightest.area,
        tightest.area_at,
    );
    assert!(
        skipped.is_empty(),
        "the sweep could not seat an A/B on {skipped:?} — a card that plans too few rows \
         to hold two distant selections plus a clear control is not graded here, and an \
         ungraded world is exactly the gap this law was written to close"
    );
    assert_eq!(
        graded,
        theme::THEMES.len() * 4,
        "the sweep graded {graded} cells where the roster offers {} — it is not reading \
         the roster it thinks it is",
        theme::THEMES.len() * 4
    );
    // ENROLMENT, derived from the roster rather than pinned to named worlds: all
    // four list families must have been reached, or this law is about a subset of
    // the product wearing the whole product's name.
    assert_eq!(
        styles.len(),
        4,
        "the sweep reached list styles {styles:?} — the selection treatment has one owner \
         shared by all four families, so a sweep that misses one cannot see a \
         family-specific defect"
    );
}

/// MEASUREMENT REPORT, not a law — the roster table the two floors above were
/// calibrated from, tightest cell first. `#[ignore]`d by default:
/// `cargo test --bin awl theme_picker_selection_report -- --ignored --nocapture`
#[test]
#[ignore]
fn theme_picker_selection_report() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(LOGICAL.0, LOGICAL.1) else {
        eprintln!("skipping theme_picker_selection_report: no wgpu adapter");
        return;
    };
    let ambient_bar = crate::menubar::menu_bar_on();
    let saved_reduced = crate::motion::reduced();
    crate::motion::set_reduced(true);
    let mut rows: Vec<(f64, String)> = Vec::new();
    for bar in [false, true] {
        crate::menubar::set_menu_bar_on(bar);
        for dpi in [1.0f32, 2.0] {
            p.set_dpi(dpi);
            let (cw, ch) = ((LOGICAL.0 * dpi) as u32, (LOGICAL.1 * dpi) as u32);
            p.set_size(cw as f32, ch as f32);
            for world in theme::world_names() {
                let _pin = theme::WorldPin::world(world).expect("a rostered world sets active");
                p.sync_theme();
                match probe_world(&mut p, &device, &queue, cw, ch, dpi, world) {
                    Some(c) => rows.push((
                        c.gained.de.min(c.lost.de),
                        format!(
                            "{world:<10} {:<16} bar={bar:<5} dpi={dpi}  gained ΔE {:>6.2} / \
                             {:>5} px ({:>5.2} rows)  lost ΔE {:>6.2} / {:>5} px ({:>5.2} rows)  \
                             control {} px  h={:.1}  item-rows={}",
                            c.style,
                            c.gained.de,
                            c.gained.area,
                            c.gained.area as f32 / c.gained.height,
                            c.lost.de,
                            c.lost.area,
                            c.lost.area as f32 / c.lost.height,
                            c.control_area,
                            c.gained.height,
                            c.rows,
                        ),
                    )),
                    None => rows.push((
                        f64::NAN,
                        format!("{world:<10} bar={bar} dpi={dpi}  NOT SEATED"),
                    )),
                }
            }
        }
    }
    p.set_dpi(1.0);
    crate::motion::set_reduced(saved_reduced);
    crate::menubar::set_menu_bar_on(ambient_bar);
    theme::set_active(theme::DEFAULT_THEME);
    rows.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    eprintln!("--- theme picker selection A/B, tightest first ---");
    for (_, line) in &rows {
        eprintln!("{line}");
    }
}
