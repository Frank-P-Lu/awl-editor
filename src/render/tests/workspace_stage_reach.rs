//! THE SUMMONED WORKSPACE'S NARROW REGIME: IT STAGES, AND NO STAGE IS BLANK.
//!
//! A workspace too narrow for both its regions at once shows one at a time —
//! `workspace_is_wide` is the single width decision the whole feature makes, and
//! below it the lifecycle's focus stage becomes WHICH region you are looking at.
//! So a planned row count of ZERO is an ordinary state of a perfectly healthy
//! card: the stage that is showing the other region. The published row geometry
//! reports it faithfully by being empty.
//!
//! **An empty row window and a card that has stopped drawing publish the same
//! thing, and the difference has already been misread as a defect.** A single
//! width, on a single door, at a zoom the product does not launch at, produced an
//! empty `overlay.window.rows` and was reported as "the card draws nothing below
//! ~860px". Two separate facts would have refuted that on sight, and neither was
//! a law:
//!
//! 1. **A ZERO-ROW STAGE IS ALWAYS THE NARROW REGIME, AND THE OTHER REGION IS
//!    ALWAYS DRAWN THERE.** Never a wide card with no rows; never both regions
//!    gone at once.
//! 2. **A STAGE THAT YIELDS EVERY CANDIDATE STILL DRAWS ITS TEACHING FOOTER OR
//!    THE OTHER REGION** — at every window a real session can be in, down to the
//!    app's own enforced minimum, across the whole authored zoom band. The footer
//!    reservation may honestly spend the last candidate at the maximum zoom; it
//!    may never turn that degradation into a blank card.
//!
//! Fact 2 is the PRESENCE FLOOR fact 1 needs. "There are no rows on this stage"
//! is satisfied perfectly by a workspace whose rows exist on no stage at any
//! width, so fact 1 alone would grade a blank product as correct.
//!
//! **The zoom axis is here because it is the axis that produced the misreading.**
//! The threshold is a width in LOGICAL px and every term feeding it is scaled
//! text, so it moves with zoom. Launch and capture share the authored 1.0
//! default, but a single quoted width is still only the threshold at the zoom
//! that measured it; the below-default stress arm proves that distinction.
//!
//! Enrolment is derived from `OverlayKind`'s roster through `workspace_shape()`,
//! and the swept arms are the distinct `rows_are_primary()` answers that roster
//! actually produces — so a new workspace kind enrols itself, and a new
//! `WorkspaceShape` that answers differently forces a new swept arm rather than
//! quietly riding an existing one.

use super::super::*;
use super::pixeldiff::Region;
use super::{comparison_view, headless_dqp, view};
use crate::overlay::workspace::WorkspaceShape;
use crate::overlay::{OverlayKind, OverlayState};

/// Logical window sizes every sweep here crosses: comfortably wide, around the
/// staging threshold at both zooms that matter, genuinely narrow, and the app's
/// OWN enforced minimum window — `MIN_COLS(30) * CHAR_WIDTH + 2 * TEXT_LEFT` by
/// `MIN_LINES(8) * LINE_HEIGHT + 2 * TEXT_TOP` (`app::lifecycle`), the smallest
/// window a user can drag to. Derived from those metrics rather than written out,
/// so a change to either moves this cell with it.
fn logical_canvases() -> Vec<(u32, u32)> {
    let min_w = (30.0 * CHAR_WIDTH + 2.0 * TEXT_LEFT.0).ceil() as u32;
    let min_h = (8.0 * LINE_HEIGHT + 2.0 * TEXT_TOP.0).ceil() as u32;
    vec![
        (min_w, min_h),
        (520, 400),
        (640, 800),
        (700, 620),
        (860, 800),
        (1000, 760),
        (1400, 900),
    ]
}

/// THE WORKSPACE ROSTER, asked of the owner. Every kind `workspace_shape()`
/// claims, paired with the one geometry fact its shape decides.
fn workspace_roster() -> Vec<(OverlayKind, bool)> {
    OverlayKind::ALL
        .iter()
        .copied()
        .filter_map(|kind| {
            kind.workspace_shape()
                .map(|shape| (kind, shape.rows_are_primary()))
        })
        .collect()
}

/// The distinct `rows_are_primary()` answers the roster produces, each with the
/// kinds that produced it — the sweep's arms, and the failure message's names.
fn swept_arms() -> Vec<(bool, Vec<&'static str>)> {
    let roster = workspace_roster();
    let mut arms: Vec<(bool, Vec<&'static str>)> = Vec::new();
    for (kind, rows_primary) in roster {
        match arms.iter_mut().find(|(value, _)| *value == rows_primary) {
            Some((_, names)) => names.push(kind.as_str()),
            None => arms.push((rows_primary, vec![kind.as_str()])),
        }
    }
    arms
}

/// A REAL Settings workspace card, standing on the home lens, with focus placed
/// by the LIFECYCLE rather than assigned. The row-window decision reads the item
/// count and the lens strip, not the accessory values, so the corpus is the
/// authored settings roster and no value cells are seeded.
fn settings_card(detail: bool) -> OverlayState {
    let mut ov = OverlayState::new(
        OverlayKind::Settings,
        crate::settings::visible_names(),
        Vec::new(),
        Vec::new(),
    );
    ov.set_facet_lens(0);
    let mut journey = crate::overlay::Journey::seeded(Some(ov));
    if detail {
        journey.toggle_detail();
    }
    journey.card().expect("the card is up").clone()
}

/// One arm's `ViewState` at one focus stage, projected the way `App::sync_view`
/// projects it — with `overlay_workspace` and `overlay_rows_primary` read off the
/// kind's own shape owner rather than written as literals.
fn arm_view(rows_primary: bool, detail: bool) -> ViewState {
    if rows_primary {
        // The timeline shape: rows in the primary column, read-only comparison
        // prose in the pane beside it.
        let mut v = comparison_view("# Transcript\n\nSome compared prose here.\n", 0, 0);
        v.overlay_items = (0..24)
            .map(|i| format!("{i} hr ago · edited \"A heading in the draft\""))
            .collect();
        v.overlay_bindings = (0..24).map(|i| format!("+{i} −{i}")).collect();
        v.overlay_window_rows = v.overlay_items.len();
        v.overlay_detail_focus = detail;
        return v;
    }
    let ov = settings_card(detail);
    let mut v = view("hello\nthere\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = OverlayKind::Settings.title();
    v.overlay_items = ov.item_strings();
    v.overlay_lens = ov.lens_strip();
    v.overlay_workspace = ov.workspace_shape().is_some();
    v.overlay_rows_primary = ov
        .workspace_shape()
        .is_some_and(WorkspaceShape::rows_are_primary);
    v.overlay_detail_focus = ov.detail_focus;
    v.overlay_sections = ov.item_sections();
    v.overlay_hint = ov.foot_hint();
    v.overlay_selected = ov.selected;
    v.overlay_scroll = ov.scroll;
    v.overlay_window_rows = ov.window_rows();
    v
}

/// ONE GRADED CELL: which arm, logical window, zoom, scale, and stage.
#[derive(Clone, Copy, PartialEq)]
struct Cell {
    rows_primary: bool,
    w: u32,
    h: u32,
    zoom: f32,
    dpi: f32,
    detail: bool,
}

impl Cell {
    fn describe(&self, kinds: &[&'static str]) -> String {
        format!(
            "rows_primary={} ({}) {}x{} logical zoom={} dpi={} detail={}",
            self.rows_primary,
            kinds.join("/"),
            self.w,
            self.h,
            self.zoom,
            self.dpi,
            self.detail
        )
    }
}

/// What one rendered cell committed: how many rows the row-owning region planned,
/// whether this width fits both regions, and whether the OTHER region is drawn.
struct StageOutcome {
    rows: usize,
    wide: bool,
    other_region_drawn: bool,
    footer_drawn: bool,
}

/// The existing maximum-zoom height limit for the staged OTHER region. Its
/// header leaves neither a rail grid nor comparison viewport at the two shortest
/// windows, and the protected footer belongs to the paired row-owning stage.
/// Kept exact so this footer-first change neither claims that separate corner
/// nor lets it expand.
const MAX_ZOOM_OTHER_REGION_LIMIT: &[Cell] = &[
    Cell {
        rows_primary: false,
        w: 464,
        h: 288,
        zoom: 3.0,
        dpi: 1.0,
        detail: false,
    },
    Cell {
        rows_primary: false,
        w: 464,
        h: 288,
        zoom: 3.0,
        dpi: 2.0,
        detail: false,
    },
    Cell {
        rows_primary: false,
        w: 520,
        h: 400,
        zoom: 3.0,
        dpi: 1.0,
        detail: false,
    },
    Cell {
        rows_primary: false,
        w: 520,
        h: 400,
        zoom: 3.0,
        dpi: 2.0,
        detail: false,
    },
    Cell {
        rows_primary: true,
        w: 464,
        h: 288,
        zoom: 3.0,
        dpi: 1.0,
        detail: true,
    },
    Cell {
        rows_primary: true,
        w: 464,
        h: 288,
        zoom: 3.0,
        dpi: 2.0,
        detail: true,
    },
    Cell {
        rows_primary: true,
        w: 520,
        h: 400,
        zoom: 3.0,
        dpi: 1.0,
        detail: true,
    },
    Cell {
        rows_primary: true,
        w: 520,
        h: 400,
        zoom: 3.0,
        dpi: 2.0,
        detail: true,
    },
];

fn stage(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &mut TextPipeline,
    cell: Cell,
) -> StageOutcome {
    let Cell {
        rows_primary,
        zoom,
        dpi,
        detail,
        ..
    } = cell;
    let (w, h) = ((cell.w as f32 * dpi) as u32, (cell.h as f32 * dpi) as u32);
    p.set_dpi(dpi);
    p.set_size(w as f32, h as f32);
    let mut v = arm_view(rows_primary, detail);
    v.zoom = zoom;
    p.set_view(&v);
    p.prepare(device, queue, w, h).unwrap();
    let probe = p.workspace_rail_probe(w);
    // THE OTHER REGION'S OWN OWNER ANSWERS, per arm. With rows in the pane the
    // other region is the label RAIL, whose box and per-entry rects come from the
    // rail owner the draw and the hit-test share. With rows in the primary column
    // it is the relocated document's viewport, the one owner of where the
    // document layer draws.
    let other_region_drawn = match rows_primary {
        true => p
            .comparison_viewport()
            .is_some_and(|[_, _, vw, vh]| vw > 0.0 && vh > 0.0),
        false => {
            probe
                .rail
                .is_some_and(|[_, _, rw, rh]| rw > 0.0 && rh > 0.0)
                && probe
                    .rows
                    .iter()
                    .any(|slot| slot.is_some_and(|[_, _, sw, sh]| sw > 0.0 && sh > 0.0))
        }
    };
    StageOutcome {
        rows: probe.visible,
        wide: p.workspace_is_wide(w),
        other_region_drawn,
        footer_drawn: p.overlay_hint_gap_probe(w).is_some(),
    }
}

/// THE SWEEP'S RUNNING RECORD. Collected rather than asserted in place so one
/// run reports every new blank or stale maximum-zoom control.
#[derive(Default)]
struct Tally {
    graded: usize,
    staged: usize,
    wide: usize,
    zero_row_with_footer: usize,
    comparison_limit_hits: usize,
    new_blank: Vec<String>,
    healed_limit: Vec<String>,
}

impl Tally {
    /// Grade ONE logical window at one zoom and scale, across both focus stages —
    /// the pairing the presence floor needs, since "some stage has rows" is a fact
    /// about the pair rather than about either cell.
    fn grade_window(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        p: &mut TextPipeline,
        window: Cell,
        kinds: &[&'static str],
    ) {
        let mut with_presence = 0usize;
        for detail in [false, true] {
            let cell = Cell { detail, ..window };
            let what = cell.describe(kinds);
            let out = stage(device, queue, p, cell);
            self.graded += 1;
            if out.rows > 0 {
                with_presence += 1;
                self.wide += usize::from(out.wide);
                continue;
            }
            self.staged += 1;
            assert!(
                !out.wide,
                "{what}: a WIDE workspace planned no rows — wide shows both regions, so this \
                 is a card that stopped drawing its list, not a stage"
            );
            let present = out.other_region_drawn || out.footer_drawn;
            let limited = MAX_ZOOM_OTHER_REGION_LIMIT.contains(&cell);
            self.comparison_limit_hits += usize::from(limited);
            match (present, limited) {
                (false, false) => self.new_blank.push(what),
                (true, true) => self.healed_limit.push(what),
                _ => {}
            }
            with_presence += usize::from(present);
            self.zero_row_with_footer += usize::from(out.footer_drawn);
        }
        assert!(
            with_presence > 0,
            "{}: neither stage draws candidates, a teaching footer, or its other region",
            window.describe(kinds)
        );
    }

    fn finish(&self, expected: usize) {
        assert_eq!(
            self.graded, expected,
            "every arm x canvas x zoom x dpi x stage cell must be graded"
        );
        assert!(
            self.zero_row_with_footer > 0,
            "the sweep never reached the protected-footer degradation; deleting footer \
             reservation could pass vacuously"
        );
        assert!(
            self.new_blank.is_empty(),
            "a zero-row stage also lost its footer/other region outside the known \
             maximum-zoom other-region limit:\n  {}",
            self.new_blank.join("\n  ")
        );
        assert!(
            self.healed_limit.is_empty(),
            "the maximum-zoom other-region limit healed; remove these stale control cells:\n  {}",
            self.healed_limit.join("\n  ")
        );
        assert_eq!(
            self.comparison_limit_hits,
            MAX_ZOOM_OTHER_REGION_LIMIT.len(),
            "every maximum-zoom comparison control must still be reached exactly"
        );
        assert!(
            self.staged > 0 && self.wide > 0,
            "the sweep must cross the staging threshold in both directions (staged {}, wide \
             {}) — otherwise one regime went ungraded and the law grades nothing it was \
             written for",
            self.staged,
            self.wide
        );
    }
}

/// **A ZERO-ROW STAGE IS ALWAYS NARROW, AND STILL DRAWS TEACHING OR CONTENT.**
///
/// Three assertions per cell, and the third is what keeps the first two from
/// being satisfied by a workspace that draws nothing anywhere:
///
///   * a stage with zero planned rows is NOT a wide card (a wide card shows both
///     regions, so zero rows there is a card that stopped drawing, not a stage);
///   * a stage with zero planned rows has its teaching footer or other region
///     drawn;
///   * and the paired stages have some visible presence at the same geometry.
///
/// The zoom axis spans the authored band (`crate::range::ZOOM`) plus a smaller
/// non-default zoom, because the staging threshold is a scaled-text width and a
/// geometry claim made at one zoom cannot be carried onto another.
#[test]
fn a_zero_row_workspace_stage_is_narrow_and_still_draws_teaching_or_content() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1400.0, 900.0) else {
        eprintln!("skipping zero-row workspace stage law: no wgpu adapter");
        return;
    };
    // The authored zoom band's own ends and default, plus a smaller non-default
    // value. One quoted width is never the threshold for the whole zoom axis.
    let zooms = [
        crate::range::ZOOM.min,
        0.8,
        crate::range::ZOOM.default,
        crate::range::ZOOM.max,
    ];
    let arms = swept_arms();
    assert!(
        !arms.is_empty(),
        "no OverlayKind claims a workspace shape — the sweep would grade nothing"
    );
    let mut tally = Tally::default();
    for (rows_primary, kinds) in &arms {
        for (w, h) in logical_canvases() {
            for zoom in zooms {
                for dpi in [1.0f32, 2.0] {
                    let window = Cell {
                        rows_primary: *rows_primary,
                        w,
                        h,
                        zoom,
                        dpi,
                        detail: false,
                    };
                    tally.grade_window(&device, &queue, &mut p, window, kinds);
                }
            }
        }
    }
    p.set_dpi(1.0);
    p.set_size(1400.0, 900.0);
    tally.finish(arms.len() * logical_canvases().len() * zooms.len() * 4);
}

/// **AND THE BLANK-LOOKING STAGE IS NOT BLANK IN THE PIXELS.**
///
/// The sweep above is geometry: it proves the other region was PLANNED. This is
/// the appearance half — the card whose rows are staged away must still put ink
/// on the canvas, measured over the frame's own pixels rather than over the state
/// that intended to draw them.
///
/// The floor is RELATIVE and within-frame, so no rasterizer's rounding enters it:
/// the region the staged-away rows would have occupied is compared against the
/// region the other stage's own list occupies, on the same frame. A card that
/// drew nothing scores zero on the second and fails; a card drawing its label
/// rail scores far above it.
#[test]
fn the_staged_workspace_still_puts_ink_in_its_card() {
    let _g = crate::testlock::serial();
    let (w, h) = (640u32, 800u32);
    let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
        eprintln!("skipping the_staged_workspace_still_puts_ink_in_its_card: no wgpu adapter");
        return;
    };
    for (rows_primary, kinds) in swept_arms() {
        let mut v = arm_view(rows_primary, rows_primary);
        // A below-default stress zoom at which this canvas is genuinely staged.
        v.zoom = 0.8;
        p.set_dpi(1.0);
        p.set_size(w as f32, h as f32);
        p.set_view(&v);
        p.prepare(&device, &queue, w, h).unwrap();
        let probe = p.workspace_rail_probe(w);
        assert_eq!(
            probe.visible,
            0,
            "rows_primary={rows_primary} ({}): precondition — this cell must BE the staged, \
             zero-row stage, or the law grades the wrong frame",
            kinds.join("/")
        );
        let card = probe.card;
        let pixels = super::pixeldiff::render_frame(&mut p, &device, &queue, w, h);
        // The card's own ground, sampled inside it and clear of every list.
        let ground = {
            let x = (card[0] + card[2] * 0.5) as usize;
            let y = (card[1] + card[3] - 4.0) as usize;
            pixels[y * w as usize + x]
        };
        let ink = |r: Region| -> u64 {
            let mut n = 0u64;
            for y in r.y.max(0)..(r.y + r.h).min(h as i64) {
                for x in r.x.max(0)..(r.x + r.w).min(w as i64) {
                    let c = pixels[y as usize * w as usize + x as usize];
                    if super::pixeldiff::delta_e(c, ground) > 4.0 {
                        n += 1;
                    }
                }
            }
            n
        };
        let interior = ink(Region::new(card[0], card[1], card[2], card[3]));
        assert!(
            interior > 0,
            "rows_primary={rows_primary} ({}): the staged card's whole interior carries no ink \
             that differs from its own ground — a card drawing nothing at all",
            kinds.join("/")
        );
        // The band the staged-away rows would have filled is empty; the band the
        // shown region fills is not. Both come off the same frame, so the world's
        // palette and the dither cancel.
        let blank_band = ink(Region::new(
            card[0],
            card[1] + card[3] * 0.6,
            card[2],
            card[3] * 0.35,
        ));
        let shown_band = ink(Region::new(
            card[0],
            card[1] + card[3] * 0.05,
            card[2],
            card[3] * 0.35,
        ));
        assert!(
            shown_band > blank_band,
            "rows_primary={rows_primary} ({}): the stage that IS showing carries no more ink \
             ({shown_band}) than the emptied band below it ({blank_band}) — the visible region \
             is not actually drawing",
            kinds.join("/")
        );
    }
    p.set_size(1400.0, 900.0);
}
