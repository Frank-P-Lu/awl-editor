//! THE FROSTED FOOTPRINT'S WIDTH — BOUNDED FROM BOTH ENDS, IN PIXELS.
//!
//! User-reported against Mangrove's theme picker, with a screenshot: *"there's a bit too
//! much blur on the left and right sides."* The silhouette was already right — a feathered
//! parallelogram — and its WIDTH was not: the frost's box was `overlay_card_rect`, and that
//! box is a PLACEMENT POLICY. `overlay_desired_w(CARD_MAX_W…)` is a fixed desired width
//! clamped to the window, with no relation to how wide the shaped rows turned out, and on a
//! composition that draws no panel and no plate nothing occupies the difference. Measured
//! before the fix: a cross-section of `card_w + 2 × feather` = **576 logical px** at any
//! single row, over a candidate row carrying **61–110 logical px of ink**.
//!
//! # WHY THE FEATHER WAS THE WRONG LEVER
//!
//! `the_footprint_feather_is_at_least_the_blur_it_edges` floors the feather at the
//! Gaussian's own 16 logical px reach, and the shipped 28 is only 12 above it. Spending the
//! whole margin buys back 24 of ~100 logical px and reintroduces the hard edge item 312
//! removed. The width was never the feather's.
//!
//! # THE TWO LAWS, AND WHY NEITHER IS SUFFICIENT ALONE
//!
//! * **TIGHTNESS** — the frost's box contains the drawn surfaces' own union in the shape's
//!   un-sheared frame, and exceeds it by no more than [`TIGHTNESS_ALLOWANCE_PX`]. A coverage
//!   floor alone is satisfied PERFECTLY by the 520px box that prompted this item; that is
//!   the "law satisfiable by its own subject" family, read from the loose end.
//! * **COVERAGE** — every pixel the card draws has the shipping mask at or above
//!   [`INK_FROST_FLOOR`] beneath it. A tightness bound alone gets strictly HAPPIER as the
//!   frost shrinks, and an over-narrow frost strands real chrome over sharp document, which
//!   is worse than the reported defect. `frost_parallelogram_item318`'s own coverage floor
//!   over the card's declared upright chrome stays exactly as it was; this is the broader net
//!   beside it.
//!
//! # THE CARD-INK ORACLE HERE IS POSITIVE, AND IT IS NOT AN INVERTED VETO
//!
//! `frost_card_ink`'s `CardInk` is a VETO and does not invert: outside the frost's reach the
//! same frame shows the world's live ground at full sharpness, so its flagged set is a
//! superset of the card's drawing whose surplus is the WORLD's. Two oracles built on the
//! inverted reading were falsified on their first run, and intersecting it with an
//! open-versus-closed difference does not rescue it, because wherever the frost lands that
//! difference carries `blur(ground) − ground`.
//!
//! **So the confound is removed rather than inverted.** `blur::set_frost_suppressed` turns
//! the frost off, and two frames of the same document at the same size on the same world —
//! one with the picker up, one without — then share their ground and their document
//! exactly. The residue between them IS the card's drawing. Measured on this roster it is
//! 9.6k–73k pixels per cell, and the one thing it also contains is the window-anchored
//! PLACARD, excluded here by the box its own production owner declares
//! (`overlay_shape_placard`) rather than by a name.
//!
//! That oracle is also what earns every EXCLUSION from the drawn-surface enumeration. A
//! surface nobody remembered is card ink with no frost under it and fails the coverage law
//! by existing, so "this composition draws nothing else" is a measurement rather than a list
//! someone maintains.
//!
//! Swept over the enrolled roster — derived from `blur::footprint_frost_applies`, so
//! Paperbark (shear 0, `Rules`, whose selected rule runs the card's full band and whose
//! frost is therefore UNCHANGED) keeps the sweep honest — at 1× and 2×, and over BOTH
//! menu-bar states, whose reserve comes off the card's height budget and therefore off the
//! rake the rail resolves. The bar's arm is taken from the AMBIENT value and its negation,
//! never from `cfg!`, which inside a test reflects the host that COMPILED it.

use super::super::*;
use super::frost_card_ink::luma;
use super::frost_feather_item312::{DENSE, enrolled_worlds, render_frame, theme_picker};
use super::headless_dqp;

/// HOW FAR THE FROST'S BOX MAY EXCEED THE DRAWN SURFACES' OWN UNION, in LOGICAL px — the
/// stated allowance the tightness bound is not asked to be exact within.
///
/// **The roster's measured excess is 0.00 at every one of the twelve cells** (world × DPI ×
/// menu bar), because the box is DERIVED from the union rather than compared against it, and
/// `footprint_box`'s coverage widening for the upright head band already lands inside it. So
/// this is not slack the product spends; it is the margin under which a shaper or rasterizer
/// difference cannot flip the law.
///
/// TWO logical px, bounded from both sides. Above: float noise, and the difference between a
/// shaped line's ADVANCE (what `overlay_head_band_ink` reports) and its glyph CELLS (what the
/// drawn-ink union measures) — sub-pixel on this shaper, not guaranteed sub-pixel on another.
/// Below: the smallest authored horizontal length in the composition is the 7 logical px
/// marker gap, with the 10 px cluster connector and the 12 px card text inset above it, so a
/// regression that reintroduced ANY structural slack moves the excess by at least 7 and
/// fails, while 2 cannot hide one.
const TIGHTNESS_ALLOWANCE_PX: f32 = 2.0;

/// THE COVERAGE FLOOR under a pixel the card draws — the same quantity, for the same
/// arithmetic reason, that `frost_parallelogram_item318` floors its head band at.
///
/// Not `1.0`: the box lands exactly ON the outermost surface's own edge, where the mask is
/// exactly 1.0, and a quad's anti-aliased skirt reaches a fraction of a pixel past the edge
/// it declares. A `smoothstep` over a 28 logical px feather is still 0.996 one px out. The
/// roster's measured worst is reported by the law and sits far above this.
const INK_FROST_FLOOR: f32 = 0.9;

/// A luma difference (of 255) that is the card DRAWING rather than the two frames being the
/// same frame. Both are frost-free and share their ground and document exactly, so the
/// residue is either zero or the card, and three units sits well inside that gap.
const CARD_INK_DELTA: f32 = 3.0;

/// THE THEME PICKER WITH A TYPED QUERY — the shipping state the user photographed, and the
/// one that gives the head band a real width (an empty query shapes the bare `›` sigil, and
/// a claim over that band is nearly a claim over nothing).
fn typed_picker(text: &str) -> ViewState {
    let mut v = theme_picker(text);
    v.overlay_query = "mangrove".to_string();
    v.overlay_query_caret = v.overlay_query.chars().count();
    v
}

/// One cell of the sweep.
struct Cell {
    /// The card's own box, as `overlay_card_rect` reports it and the POINTER hit-test reads
    /// it — deliberately NOT the frost's, because the two are separate quantities and this
    /// item moved only one of them.
    card: [f32; 4],
    frost: crate::render::blur::Frost,
    rect: [f32; 4],
    shear: f32,
    surfaces: Vec<[f32; 4]>,
}

/// A BOX'S HORIZONTAL SPAN IN THE SHAPE'S UN-SHEARED FRAME.
///
/// The only frame in which "left" and "right" are properties of the box: at row `py` both
/// faces translate by `shear × (py − cy)`, so a canvas-x reading of a leaning shape's width
/// measures the rake instead of the width. The rows lean at exactly the shear and therefore
/// collapse onto ONE span here, which is why a narrowed parallelogram never pays for its own
/// lean.
///
/// Asked of the production narrowing itself rather than re-derived: `footprint_narrow` over a
/// card that IS the box returns the box's own un-sheared span, so this reads the shipping
/// arithmetic instead of a second copy of it.
fn unsheared(s: [f32; 4], shear: f32, cy: f32) -> (f32, f32) {
    // A card wide enough that the narrowing's clamp cannot bind, centred on the same `cy` so
    // the pivot matches the real shape's.
    let card = [-1.0e6, cy - 1.0e6, 2.0e6, 2.0e6];
    let [x, _, w, _] = crate::render::blur::footprint_narrow(card, shear, &[s]);
    (x, x + w)
}

fn open_cell(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &mut TextPipeline,
    w: u32,
    h: u32,
    label: &str,
) -> Cell {
    p.set_view(&typed_picker(DENSE));
    let _ = render_frame(device, queue, p, w, h);
    let card = p
        .overlay_card_rect()
        .expect("the crisp picker has a card box");
    let frost = p.frost_mode().expect("an enrolled world reaches the frost");
    let (rect, shear) = match frost {
        crate::render::blur::Frost::Footprint(f) => (f.rect, f.shear),
        other => panic!("{label}: expected the footprint arm, got {other:?}"),
    };
    let geom = p.overlay_geometry(w);
    let plan = p.overlay_row_plan(&geom);
    let surfaces = p.overlay_drawn_surfaces(&geom, &plan);
    Cell {
        card,
        frost,
        rect,
        shear,
        surfaces,
    }
}

/// THE SWEEP — enrolled roster × 1×/2× × both menu-bar arms. Returns the number of cells
/// that ran, so each law can distinguish "green" from "no adapter".
fn sweep(
    mut cell: impl FnMut(&wgpu::Device, &wgpu::Queue, &mut TextPipeline, u32, u32, f32, String),
) -> usize {
    let entry = crate::theme::active_index();
    let ambient_bar = crate::menubar::menu_bar_on();
    let mut ran = 0usize;
    for world in enrolled_worlds() {
        for bar in [ambient_bar, !ambient_bar] {
            for (dpi, w, h) in [(1.0f32, 1200u32, 900u32), (2.0, 2400, 1800)] {
                let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
                    eprintln!("skipping the frost-width sweep: no wgpu adapter");
                    crate::menubar::set_menu_bar_on(ambient_bar);
                    crate::theme::set_active(entry);
                    return ran;
                };
                crate::theme::set_active_by_name(world).unwrap();
                crate::menubar::set_menu_bar_on(bar);
                p.set_dpi(dpi);
                let label = format!("{world} @ {dpi}x ({w}x{h}) bar {bar}");
                cell(&device, &queue, &mut p, w, h, dpi, label);
                ran += 1;
            }
        }
    }
    crate::menubar::set_menu_bar_on(ambient_bar);
    crate::theme::set_active(entry);
    ran
}

/// THE TIGHTNESS LAW: THE FROST'S BOX IS THE DRAWN SURFACES' OWN UNION, TO WITHIN A STATED
/// ALLOWANCE — AND IT CONTAINS THEM.
///
/// Both halves are load-bearing and point in opposite directions. CONTAINMENT is what stops
/// this being a "make it narrower" ratchet; the ALLOWANCE is what the coverage floor beside
/// it cannot say, because a floor over the card's ink is satisfied perfectly by the whole
/// 520px box the user photographed.
///
/// ⚠️ **THE ENROLMENT IS DERIVED AND BOTH ITS ARMS ARE REQUIRED NON-EMPTY.** A composition
/// that draws something at the card's FULL band — every `Rules` world, whose selected rule
/// runs `(band_x, band_w)` — legitimately narrows by nothing and must come out bit-identical.
/// A leaning composition narrows. If the sweep ever held only the first kind, "the box equals
/// the union" would be true of the defect.
#[test]
fn the_frost_is_no_wider_than_the_surfaces_it_backs() {
    let _g = crate::testlock::serial();
    let mut narrowed: Vec<String> = Vec::new();
    let mut unchanged: Vec<String> = Vec::new();
    let mut worst_excess = f32::NEG_INFINITY;
    let mut worst_at = String::new();
    let ran = sweep(|device, queue, p, w, h, dpi, label| {
        let c = open_cell(device, queue, p, w, h, &label);
        assert!(
            !c.surfaces.is_empty(),
            "{label}: the card reports NO drawn surface at all, so every bound below is a \
             bound over an empty set — and the narrowing's own inert answer is to keep the \
             whole box, so this law would pass by finding nothing"
        );
        let cy = c.card[1] + c.card[3] * 0.5;
        let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
        for s in &c.surfaces {
            let (a, b) = unsheared(*s, c.shear, cy);
            lo = lo.min(a);
            hi = hi.max(b);
        }
        assert!(
            hi - lo > 20.0 * dpi,
            "{label}: the drawn surfaces' union spans only {:.1} physical px — a presence \
             guard, because a tightness bound over a degenerate union is satisfied by a \
             frost of nothing",
            hi - lo
        );
        let (fx0, fx1) = (c.rect[0], c.rect[0] + c.rect[2]);
        let excess = (c.rect[2] - (hi - lo)) / dpi;
        if excess > worst_excess {
            worst_excess = excess;
            worst_at = label.clone();
        }
        eprintln!(
            "MEASURED {label}: card {:.1} logical wide, frost {:.1}, drawn-surface union \
             {:.1} (excess {excess:.2}) — the frost's left face sits {:.1} logical px inside \
             the card's and its right face {:.1}, over {} surfaces",
            c.card[2] / dpi,
            c.rect[2] / dpi,
            (hi - lo) / dpi,
            (c.rect[0] - c.card[0]) / dpi,
            (c.card[0] + c.card[2] - fx1) / dpi,
            c.surfaces.len(),
        );
        let slack = TIGHTNESS_ALLOWANCE_PX * dpi;
        assert!(
            lo >= fx0 - slack && hi <= fx1 + slack,
            "{label}: the drawn surfaces span [{lo:.1}, {hi:.1}] in the shape's un-sheared \
             frame while the frost's box is [{fx0:.1}, {fx1:.1}] — the frost does not contain \
             what the card drew, so this narrowing has cut real chrome loose over sharp \
             document"
        );
        assert!(
            excess <= TIGHTNESS_ALLOWANCE_PX,
            "{label}: the frost's box is {:.1} logical px wide against a drawn-surface union \
             of {:.1} — {excess:.2} px of slack against an allowance of \
             {TIGHTNESS_ALLOWANCE_PX}. The card's own box is {:.1} wide, and treating THAT as \
             the frost's extent is the reported defect: it is a placement policy, not a \
             surface",
            c.rect[2] / dpi,
            (hi - lo) / dpi,
            c.card[2] / dpi,
        );
        if c.rect[2] < c.card[2] - 1.0 {
            narrowed.push(label);
        } else {
            unchanged.push(label);
        }
    });
    if ran == 0 {
        return;
    }
    eprintln!("ROSTER WORST excess {worst_excess:.2} logical px at {worst_at}");
    assert!(
        !narrowed.is_empty() && !unchanged.is_empty(),
        "this sweep must contain a composition whose frost NARROWS and one whose frost is \
         unchanged (a full-band `Rules` world) — otherwise 'the box equals the union' is true \
         of the box the user photographed: narrowed {narrowed:?}, unchanged {unchanged:?}"
    );
}

/// THE COVERAGE LAW: EVERY PIXEL THE CARD DRAWS HAS THE FROST FULLY BENEATH IT.
///
/// This is what refuses the tightness bound above, which gets strictly happier as the frost
/// shrinks — a frost of nothing satisfies it perfectly. It is also the COMPLETENESS proof for
/// the drawn-surface enumeration the frost's box is derived from: a surface nobody remembered
/// is card ink with no frost under it, and it fails here by existing rather than by someone
/// maintaining a list.
///
/// The card's ink is measured, not enumerated, and the oracle is POSITIVE — see this module's
/// header for why an inverted `CardInk` is not available and why suppressing the frost is what
/// makes a sound one.
#[test]
fn every_pixel_the_card_draws_is_fully_frosted_beneath() {
    let _g = crate::testlock::serial();
    let mut tightest = f32::INFINITY;
    let mut tightest_at = String::new();
    let ran = sweep(|device, queue, p, w, h, dpi, label| {
        let c = open_cell(device, queue, p, w, h, &label);
        // TWO FROST-FREE FRAMES whose only difference is the card. The flag is restored
        // before any assertion below, so an unwinding path cannot leak it.
        crate::render::blur::set_frost_suppressed(true);
        p.set_view(&typed_picker(""));
        let open = render_frame(device, queue, p, w, h);
        let suppressed = p.frost_mode().is_none();
        let geom = p.overlay_geometry(w);
        let placard = p.overlay_shape_placard(&geom);
        let mut shut_view = typed_picker("");
        shut_view.overlay_active = false;
        p.set_view(&shut_view);
        let shut = render_frame(device, queue, p, w, h);
        crate::render::blur::set_frost_suppressed(false);
        assert!(
            suppressed,
            "{label}: the suppression door did not hold, so these two frames differ by the \
             FROST as well as by the card and the residue below is not the card's drawing"
        );
        let (wi, hei) = (w as i64, h as i64);
        // `NAN` bounds match nothing, which is the right inert answer for a card that shapes
        // no placard at all.
        let pl = placard.unwrap_or((f32::NAN, f32::NAN, 0.0, 0.0));
        let (mut ink, mut below) = (0u64, 0u64);
        let mut worst = 1.0f32;
        let mut worst_px = (0.0f32, 0.0f32);
        for y in 0..hei {
            for x in 0..wi {
                let i = (y * wi + x) as usize;
                if (luma(open[i]) - luma(shut[i])).abs() < CARD_INK_DELTA {
                    continue;
                }
                let (fx, fy) = (x as f32, y as f32);
                // THE PLACARD IS NOT THE CARD. Its own owner anchors it to the CANVAS, it
                // sits hundreds of px from the card's box on this roster, and no frost —
                // before this item or after it — has ever covered it.
                if fx >= pl.0 && fx <= pl.0 + pl.2 && fy >= pl.1 && fy <= pl.1 + pl.3 {
                    continue;
                }
                ink += 1;
                let m = crate::render::blur::footprint_mask_for(c.frost, dpi, fx, fy);
                if m < worst {
                    worst = m;
                    worst_px = (fx, fy);
                }
                if m < INK_FROST_FLOOR {
                    below += 1;
                }
            }
        }
        eprintln!(
            "MEASURED {label}: {ink} pixels of the card's own drawing, tightest frost \
             coverage {worst:.4} at {worst_px:?}, {below} under the floor {INK_FROST_FLOOR}"
        );
        assert!(
            ink > 2000,
            "{label}: only {ink} pixels of card drawing found (delta {CARD_INK_DELTA} of 255) \
             — a coverage claim over an empty set is satisfied by anything, and this count is \
             what says the two frames really do differ by a whole drawn card"
        );
        assert_eq!(
            below, 0,
            "{label}: {below} of {ink} pixels the card DRAWS sit where the frost's coverage is \
             under {INK_FROST_FLOOR} (tightest {worst:.4} at {worst_px:?}) — that chrome is \
             over SHARP document. The frost's box is derived from the drawn surfaces, so \
             either the narrowing is wrong or a surface is missing from \
             `overlay_drawn_surfaces`"
        );
        if worst < tightest {
            tightest = worst;
            tightest_at = label;
        }
    });
    if ran == 0 {
        return;
    }
    eprintln!(
        "ROSTER TIGHTEST card-ink coverage {tightest:.4} at {tightest_at} — the floor \
         {INK_FROST_FLOOR} sits under it"
    );
}

/// `RuleSpans::x_reach` CONTAINS EVERY RECT `rules_ink` CAN EMIT — swept over row counts,
/// selections, both marks, and three gutter regimes.
///
/// The frost reads that reach because a rule is the one thing a ruled list draws at the
/// card's FULL width, and a frost narrowed to the glyphs would leave every selected rule
/// hanging over sharp document. The reach is therefore a claim about `rules_ink` itself, and
/// this is what keeps it one: a fourth x-extent added to that function fails here rather than
/// escaping into a frost that no longer covers it.
///
/// Pure — no device, no fixture — so it sweeps parameter space the render tier cannot reach.
#[test]
fn the_rule_reach_contains_every_rule_a_ruled_list_draws() {
    let _g = crate::testlock::serial();
    use crate::render::chrome::overlay_rules::{RuleRow, RuleSpans, rules_ink};
    let mut checked = 0usize;
    // A gutter WIDER than the authored segment, one NARROWER than it (the contextual popup's
    // own inset, where the segment shortens into the room it has), and one of zero — so the
    // segment's `max(band_x)` floor is exercised in both directions.
    for (measure_x, band_x) in [(30.0f32, 0.0f32), (5.0, 0.0), (30.0, 30.0)] {
        let spans = RuleSpans {
            hair: 1.0,
            heavy: 3.0,
            measure: (measure_x, 460.0),
            band: (band_x, 520.0),
            mark: (13.0, 9.0),
        };
        let (rl, rr) = spans.x_reach();
        for n in 0..6usize {
            for sel in 0..n.max(1) {
                for mark in [theme::RuleSelection::Weight, theme::RuleSelection::Gutter] {
                    let rows: Vec<RuleRow> = (0..n)
                        .map(|i| RuleRow {
                            top: 100.0 + i as f32 * 28.0,
                            bottom: 128.0 + i as f32 * 28.0,
                            selected: i == sel,
                        })
                        .collect();
                    let (hair, heavy) = rules_ink(&rows, mark, &spans);
                    for r in hair.iter().chain(heavy.iter()) {
                        checked += 1;
                        assert!(
                            r[0] >= rl - 1e-3 && r[0] + r[2] <= rr + 1e-3,
                            "a rule {r:?} ({mark:?}, {n} rows, selected {sel}, measure_x \
                             {measure_x}, band_x {band_x}) runs outside the declared reach \
                             [{rl}, {rr}] — the footprint frost is scoped to that reach, so \
                             this rule would draw over sharp document"
                        );
                    }
                }
            }
        }
    }
    assert!(
        checked > 100,
        "only {checked} rules graded — the sweep must actually emit rules, or this law is a \
         claim about an empty set"
    );
    eprintln!("MEASURED: {checked} emitted rules, every one inside its declared reach");
}
