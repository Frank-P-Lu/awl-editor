//! THE FROSTED FOOTPRINT'S EDGE, MEASURED AS A RAMP — AND ITS LEAN, MEASURED AGAINST
//! THE DRAWN SPINE.
//!
//! The crisp picker's frost was scoped to the card's own box with a SCISSOR. A
//! scissor answers yes or no per pixel, so the boundary was a knife edge: words at the
//! card's edge were sliced mid-glyph, sharp on one side and defocused on the other. It
//! was hard by CONSTRUCTION — `blend: None` plus an alpha of `1.0` out of `fs_comp` —
//! and no value anywhere in that path could soften it. The extent is now a feathered
//! MASK carried in the composite's alpha, and on a `Diagonal` composition it LEANS with
//! the spine drawn inside it.
//!
//! # WHAT IS MEASURED, AND WHY IT IS MEASURABLE
//!
//! The frost's contribution at a pixel is `mask × (blurred − sharp)`. `(blurred −
//! sharp)` is a property of the document there and varies wildly per pixel, which is
//! why a per-pixel reading of the ramp says nothing. Averaged along a face over
//! hundreds of columns it is very nearly constant in the direction PERPENDICULAR to the
//! face — so the profile of `mean|open − closed|` against distance from the face is
//! proportional to the MASK, and the mask's ramp is directly readable off it.
//!
//! Three things keep that from measuring something adjacent to its subject:
//!
//! * **The profile is taken OUTSIDE the card**, in the skirt, where there is no card
//!   ink at all to confound it — the frost's new territory is exactly the region where
//!   the only thing that can differ between a picker-open and a picker-closed frame is
//!   the frost itself.
//! * **The card's own ink is vetoed anyway**, by the derived `CardInk` oracle (see
//!   `frost_card_ink` for what that veto can and cannot answer — it is a superset of the
//!   card's drawing and does not invert). The card's ring is card ink, and it sits within
//!   a pixel or two of the very face being profiled.
//! * **A PRESENCE floor runs beside the ramp.** A mask that faded the whole footprint to
//!   nothing has no hard edge anywhere and would satisfy a ramp test perfectly. So the
//!   profile's own amplitude at the face must clear a floor, and the interior
//!   presence floor inside the footprint still runs.
//!
//! Swept over the enrolled roster — derived from `blur::footprint_frost_applies`, never
//! a name list, so Paperbark is in it and a world that changes list style changes what
//! this sweeps — and at 1× and 2×, because the feather is an authored LOGICAL length
//! and every capture in the tree runs at `--capture-dpi 1`.

use super::super::*;
use super::dither::{offscreen, read_pixels};
use super::frost_card_ink::{CardInk, luma};
use super::{headless_dqp, view_md};

/// Prose dense enough to put real glyph structure in the skirt band on every face of
/// the card at both device scales — the profile is a mean over that structure, so a
/// blank region beside a face would read as a frost that reaches nothing.
pub(super) const DENSE: &str = concat!(
    "# The feathered footprint\n\n",
    "Prose is the product, and the prose is what a summoned picker draws over. The\n",
    "frost's boundary used to be a knife edge, and the words that crossed it were\n",
    "sliced: sharp on one side of the card's own box and defocused on the other.\n",
    "This paragraph exists so dense glyph structure sits in the skirt band beside\n",
    "every face of the card, at both device scales, since the ramp is measured as a\n",
    "mean over that structure and a blank region reads as no frost at all.\n\n",
    "A Gaussian whose reach exceeds a stem cannot leave a step anywhere, and a mask\n",
    "that ramps over a feather cannot leave one either. What a scissor leaves is a\n",
    "step exactly one pixel wide along the whole of the card's boundary.\n\n",
    "- a list row with several short words in it\n",
    "- another list row, similar in shape\n",
    "- a third row, so the block has height\n",
    "- a fourth, and the block reaches the card's foot\n\n",
    "Every line of this document sits beside or beneath the picker's own card, which\n",
    "is the whole point of the measurement this file performs. More prose follows so\n",
    "the page is full at every geometry the sweep visits, including the tall one.\n\n",
    "The edge is the subject. The interior is item 294's subject, and its laws still\n",
    "run beside these: an interior that lost its frost fails there, not here.\n",
);

/// The PRESENCE floor on the profile, in luma: how much the frost must actually change
/// the page one pixel outside the card's face. Without it, a mask that faded the whole
/// footprint to nothing passes every ramp assertion below — the ramp would be measured
/// on noise, and noise has no step in it.
const SKIRT_PRESENCE_FLOOR: f32 = 1.5;

pub(super) fn theme_picker(text: &str) -> ViewState {
    let mut v = view_md(text, 0, 0);
    v.overlay_active = true;
    v.overlay_crisp = true;
    v.overlay_items = crate::theme::THEMES.iter().map(|t| t.name.into()).collect();
    v.overlay_sections = vec![String::new(); v.overlay_items.len()];
    v.overlay_selected = 11;
    v.overlay_title = "themes";
    v.overlay_hint = "type to filter   ↵ keep   esc revert".to_string();
    v
}

/// A crisp picker whose card HUGS one-glyph rows, with no title and no foot hint — as
/// narrow as the composition can make it.
///
/// Its only job is to make the spine give up rake. `TRAVEL_MAX_BAND_FRACTION` bounds the
/// spine's total travel by a share of the card's SIDE TERRITORY, and on a card this
/// narrow the attachment inset itself yields first, so the share left is small: measured
/// on this tree the spine rakes at 3.99 physical px per row against an authored 7.0. The
/// theme picker's roomier card never reaches that bound, so a law swept only over it
/// compares the measured step where it EQUALS the authored constant, and proves nothing
/// about which of the two the frost read.
fn hug_picker() -> ViewState {
    let mut v = theme_picker("");
    v.overlay_items = (0..13).map(|_| "a".to_string()).collect();
    v.overlay_sections = vec![String::new(); 13];
    v.overlay_selected = 1;
    v.overlay_title = "";
    v.overlay_hint = String::new();
    v
}

/// The per-row step the AUTHORED constant resolves to at this frame's scale — the number
/// a frost that re-derived its own lean would have used. Never an input to the frost: it
/// exists only so the sweep can prove it contains a geometry where the two DIFFER.
fn authored_step(p: &TextPipeline) -> f32 {
    match crate::render::effective_list_style() {
        crate::theme::ListStyle::Diagonal(spine) => {
            crate::render::chrome::diagonal::DiagonalComposition::resolve(spine, p.metrics.scale)
                .row_step
        }
        crate::theme::ListStyle::Pane
        | crate::theme::ListStyle::Bars
        | crate::theme::ListStyle::Rules(_) => 0.0,
    }
}

pub(super) fn render_frame(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &mut TextPipeline,
    w: u32,
    h: u32,
) -> Vec<[u8; 4]> {
    p.prepare(device, queue, w, h).unwrap();
    let (texture, tview) = offscreen(device, w, h);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("awl frost feather encoder"),
    });
    p.render(&mut encoder, &tview).unwrap();
    queue.submit(Some(encoder.finish()));
    read_pixels(device, queue, &texture, w, h)
}

/// Every world whose composition enrols in the footprint frost, taken from the ROSTER's
/// own predicate rather than named.
pub(super) fn enrolled_worlds() -> Vec<&'static str> {
    crate::theme::THEMES
        .iter()
        .filter(|t| crate::render::blur::footprint_frost_applies(t.render_caps.list_style))
        .map(|t| t.name)
        .collect()
}

/// THE HEADLINE LAW: THE EDGE IS A RAMP, NOT A STEP — AND THE RAMP IS THE SAME WIDTH IN
/// LOGICAL PX AT 1× AND 2×.
///
/// The profile is taken across the card's LEFT face, FOLLOWING THE FACE the shape
/// actually has: the union's left boundary at row `py` is `rx + min(0, shear × (py −
/// cy))`, and the mask a distance `d` outside it is exactly `1 − smoothstep(0, f, d)`.
/// Sampling a fixed column instead would smear the ramp across the lean and report a
/// wide soft edge on a world whose edge was a knife — the leaning face is the one that
/// makes a column-wise reading wrong. It reports the half-strength crossing, which for a
/// `smoothstep` over a feather `f` sits at `f / 2`; a step crosses at 1.
///
/// The DPI half covers the blur's own reach: the feather is an
/// authored logical length, so a version held in device px would halve the reader's edge
/// softness on retina, and every capture in this tree runs at `--capture-dpi 1`.
#[test]
fn the_footprints_edge_is_a_ramp_of_the_authored_width_in_logical_px_at_every_dpi() {
    let _g = crate::testlock::serial();
    let entry = crate::theme::active_index();
    let worlds = enrolled_worlds();
    assert!(
        !worlds.is_empty(),
        "no world enrols in the footprint frost — this law has no subject"
    );
    let feather = crate::render::blur::FOOTPRINT_FEATHER_PX;
    for world in &worlds {
        let mut per_dpi: Vec<(f32, f32)> = Vec::new();
        for (dpi, w, h) in [(1.0f32, 1200u32, 900u32), (2.0, 2400, 1800)] {
            let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
                eprintln!("skipping the frost-feather ramp sweep: no wgpu adapter");
                return;
            };
            crate::theme::set_active_by_name(world).unwrap();
            p.set_dpi(dpi);

            p.set_view(&theme_picker(DENSE));
            let open = render_frame(&device, &queue, &mut p, w, h);
            // THE BOX AND THE SHEAR THE FRAME ACTUALLY SENT THE SHADER — never a
            // re-derivation, so the face this follows is the face that was drawn.
            //
            // ⚠️ The BOX is the frost's own, not `overlay_card_rect`'s: the frost's box is
            // widened where it must be to seat the card's upright chrome, so a profile
            // taken from the card's rect would follow a face a whole widening away from
            // the drawn one.
            let (foot, [rx, ry, rw, rh], shear) = match p.frost_mode() {
                Some(crate::render::blur::Frost::Footprint(f)) => (f, f.rect, f.shear),
                other => panic!("{world}: expected the footprint arm, got {other:?}"),
            };
            // How far the frost's box was WIDENED past the card's, in this law's own label.
            p.set_view(&theme_picker(""));
            let empty = render_frame(&device, &queue, &mut p, w, h);
            let mut plain = view_md(DENSE, 0, 0);
            plain.overlay_active = false;
            p.set_view(&plain);
            let closed = render_frame(&device, &queue, &mut p, w, h);

            let (wi, hi) = (w as i64, h as i64);
            let ink = CardInk::derive(&empty, wi, hi, dpi);
            let f_px = feather * dpi;
            let depth = (f_px * 2.0) as i64;
            let label = format!("{world} @ {dpi}x ({w}x{h}), frost box [{rx},{ry},{rw},{rh}]");
            // The rows profiled: the card's own height less one feather at each end, so
            // the top and bottom faces' own skirts never enter a reading of a VERTICAL
            // face's. `profile_face` below computes each face's own x at every row, so
            // the reading follows the lean rather than smearing across it.
            let rows: Vec<i64> = ((ry + f_px) as i64..(ry + rh - f_px) as i64)
                .filter(|y| (0..hi).contains(y))
                .collect();
            assert!(
                rows.len() > 40,
                "{label}: only {} rows to profile — the fixture, not the product, is \
                 what failed",
                rows.len()
            );
            // THE PROFILE, outward from a face: for each distance `d`, the mean absolute
            // luma difference between the picker-open frame and the picker-closed one.
            // Card ink is vetoed, which is what keeps the card's own ring out of the
            // first few samples.
            //
            // BOTH VERTICAL FACES are profiled, and which of them carries the claim is
            // DERIVED rather than chosen: a face whose skirt lands on a flat margin has
            // no document to defocus there, so `open == closed` beside it and the
            // profile is honestly zero — that is Magpie's left face, whose card sits on
            // the other side of the page column from Mangrove's. Naming one face would
            // have made this law a property of one world's card anchoring.
            //
            // ⚠️ THE FACE IS THE PARALLELOGRAM'S, and BOTH faces translate by the same
            // `shear × (py − cy)`. This used to read `.min(0.0)` on the left face and
            // `.max(0.0)` on the right — the retired box-UNION's boundary, where each face
            // moved on only the half of the card the rake reached toward and stood still on
            // the other. Left as it was, this law would profile a face up to `|shear| · h/2`
            // away from the drawn one on half of every leaning card, smearing the ramp across
            // the lean and reporting a soft edge where there might be a knife.
            let profile_face = |outward: f32| -> Option<Vec<f32>> {
                // THE FACE COMES FROM THE SHAPE'S OWN OWNER, never a copy here — see
                // `blur::extent::footprint_face_x`, which exists because the copy that used
                // to live on this line outlived the shape it described.
                let face = |py: f32| crate::render::blur::footprint_face_x(foot, py, outward);
                if !rows.iter().all(|y| {
                    let x = face(*y as f32) + outward * depth as f32;
                    x > 0.0 && x < wi as f32 - 1.0
                }) {
                    return None;
                }
                Some(
                    (1..=depth)
                        .map(|d| {
                            let mut acc = 0.0f64;
                            let mut n = 0.0f64;
                            for y in &rows {
                                let x = (face(*y as f32) as i64) + (outward as i64) * d;
                                let i = (y * wi + x) as usize;
                                if ink.vetoes(x, *y) {
                                    continue;
                                }
                                acc += (luma(open[i]) - luma(closed[i])).abs() as f64;
                                n += 1.0;
                            }
                            if n < 20.0 { 0.0 } else { (acc / n) as f32 }
                        })
                        .collect(),
                )
            };

            let mut graded = 0usize;
            let mut best: Option<f32> = None;
            for (name, outward) in [("left", -1.0f32), ("right", 1.0)] {
                let Some(profile) = profile_face(outward) else {
                    eprintln!("MEASURED {label}: {name} face has no room on canvas, skipped");
                    continue;
                };
                // The face-adjacent amplitude, and the far tail. The reference is the
                // MEDIAN of the first tenth of the ramp rather than a single sample, so
                // one stray unmasked ring pixel cannot set the scale.
                let near = median(&profile[0..(depth as usize / 10).max(2)]);
                let far = median(&profile[(depth as usize * 3 / 4)..]);
                eprintln!(
                    "MEASURED {label} shear {shear:.4} {name} face: near={near:.2} \
                     far={far:.2} profile={:?}",
                    profile
                        .iter()
                        .step_by((dpi * 2.0) as usize)
                        .map(|v| (v * 10.0).round() / 10.0)
                        .collect::<Vec<_>>()
                );
                // THE PRESENCE FLOOR. The ramp below is measured as a fraction of
                // `near`, so a frost that changed nothing would divide noise by noise
                // and find a beautifully graded edge. Faces with no document beside
                // them are excluded here, and the sweep requires at least one to remain.
                if near - far < SKIRT_PRESENCE_FLOOR {
                    continue;
                }
                graded += 1;

                // THE RAMP: the first distance at which the profile falls half way from
                // the face to the tail. A hard edge crosses at d = 1.
                let half = near - (near - far) * 0.5;
                let crossing = profile
                    .iter()
                    .position(|v| *v <= half)
                    .unwrap_or(profile.len()) as f32
                    + 1.0;
                let logical = crossing / dpi;
                best = Some(best.map_or(logical, |b: f32| b.max(logical)));
                assert!(
                    logical >= 0.2 * feather,
                    "{label} {name} face: the frost falls to half strength {logical:.1} \
                     LOGICAL px out of the card's face, against an authored {feather} px \
                     feather — that is a knife edge, and it is what slices a word \
                     mid-glyph. (near {near:.2}, far {far:.2}, half {half:.2})"
                );
                assert!(
                    logical <= 1.2 * feather,
                    "{label} {name} face: the frost is still at half strength \
                     {logical:.1} LOGICAL px out, past a {feather} px feather — the \
                     skirt reaches further onto the live page than the authored width, \
                     and the page around the card is what the crisp picker exists to \
                     preview"
                );
            }
            assert!(
                graded > 0,
                "{label}: neither of the card's vertical faces has a measurable skirt \
                 (floor {SKIRT_PRESENCE_FLOOR} luma). There is no ramp OUTSIDE the \
                 card's face at all, which is what BOTH failure modes look like from \
                 here: a knife edge (a mask that answers yes or no, as a scissor does — \
                 the reported defect) and a frost that faded its whole footprint to \
                 nothing. The ramp assertions above are measured as a fraction of this \
                 amplitude, so they would be satisfied by having nothing to ramp"
            );
            per_dpi.push((dpi, best.expect("a graded face reports a crossing")));
        }
        // THE DPI AXIS: the same edge, in the units a reader sees, at both scales.
        let (a, b) = (per_dpi[0].1, per_dpi[1].1);
        assert!(
            (a - b).abs() <= 0.35 * feather,
            "{world}: the edge falls to half strength {a:.1} logical px out at 1× and \
             {b:.1} at 2× — a feather held in DEVICE px halves the softness a reader \
             sees on retina, and no `--capture-dpi 1` capture can see it"
        );
    }
    crate::theme::set_active(entry);
}

fn median(v: &[f32]) -> f32 {
    let mut s: Vec<f32> = v.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap());
    s[s.len() / 2]
}

/// THE LEAN IS THE DRAWN SPINE'S OWN SLOPE — ASSERTED AGAINST THE SPINE, NOT A CONSTANT.
///
/// `chrome/diagonal` resolves the spine's per-row step under a responsive bound
/// (`TRAVEL_MAX_BAND_FRACTION`), so a cramped card gives up rake. A second copy of
/// `ROW_STEP` would part company with the drawn spine at exactly the geometry a law
/// forgets to sweep — so this reads the two abscissae the spine QUAD was built from
/// (`DiagonalClusterProbe::spine_x` at the plan's first and last drawn rows) and
/// requires the frost's shear to be the slope between them.
///
/// ENROLMENT DERIVES FROM WHAT THE FRAME DREW, not from a world's name: the frost leans
/// exactly when the frame measured a diagonal rail with a rake, so the `Rules` half of
/// the roster takes the feather and keeps its upright rectangle without anything here
/// naming it. Both branches are required to be non-empty, and both are named on failure.
///
/// ⚠️ **THE YIELD ARM IS ITSELF PROVED NON-VACUOUS**, because without that this law is
/// satisfied by a shear read off the authored `ROW_STEP`: the measured step and the
/// constant AGREE on every ordinary card, so equality to the drawn spine is a test of
/// nothing unless the sweep contains a geometry where the rake actually yields. Measured
/// on this tree, a card hugging one-glyph rows gives up most of its rake, drawing a
/// spine that steps
/// 3.99 px per row against an authored 7.0 — while the theme picker's roomier card never
/// reaches the bound at all. The sweep is required to contain such a case, or this law
/// reports its own vacuity rather than passing green.
#[test]
fn the_footprints_lean_is_read_from_the_spine_the_frame_actually_drew() {
    let _g = crate::testlock::serial();
    let entry = crate::theme::active_index();
    let mut leaning: Vec<&str> = Vec::new();
    let mut upright: Vec<&str> = Vec::new();
    let mut yielded: Vec<String> = Vec::new();
    // Swept over geometries as well as worlds, because the rake YIELDS on a cramped
    // card and a law pinned to one window size would never see the yield it exists for.
    for world in enrolled_worlds() {
        for (dpi, w, h, hug) in [
            (1.0f32, 1200u32, 900u32, false),
            (2.0, 2400, 1800, false),
            // DELIBERATELY CRAMPED, and measured to be so: a card hugging one-glyph rows
            // has so little side territory that `TRAVEL_MAX_BAND_FRACTION` takes most of
            // the rake, and the drawn spine rakes at 3.99 px per row against an authored
            // 7.0. A frost reading the constant agrees with the spine on the two
            // geometries above and over-leans by three quarters here.
            (1.0, 1200, 900, true),
        ] {
            let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
                eprintln!("skipping the frost-lean sweep: no wgpu adapter");
                return;
            };
            crate::theme::set_active_by_name(world).unwrap();
            p.set_dpi(dpi);
            p.set_view(&if hug {
                hug_picker()
            } else {
                theme_picker(DENSE)
            });
            p.prepare(&device, &queue, w, h).unwrap();
            let mut encoder = device.create_command_encoder(&Default::default());
            let (_t, tview) = offscreen(&device, w, h);
            p.render(&mut encoder, &tview).unwrap();
            queue.submit(Some(encoder.finish()));

            let label = format!("{world} @ {dpi}x ({w}x{h}) hug {hug}");
            let shear = match p.frost_mode() {
                Some(crate::render::blur::Frost::Footprint(f)) => f.shear,
                other => panic!(
                    "{label}: an enrolled world's crisp picker must reach the \
                                 footprint arm, got {other:?}"
                ),
            };
            match p.diagonal_cluster_probe() {
                None => {
                    if !upright.contains(&world) {
                        upright.push(world);
                    }
                    assert_eq!(
                        shear, 0.0,
                        "{label}: the frame drew no diagonal rail, so the frost has no \
                         spine to lean with, yet its shear is {shear}"
                    );
                }
                Some(probe) => {
                    let geom = p.overlay_geometry(w);
                    let plan = p.overlay_row_plan(&geom);
                    let rows = plan.rows();
                    let (first, last) = (
                        *rows.first().expect("a drawn row"),
                        *rows.last().expect("a drawn row"),
                    );
                    assert!(
                        rows.len() > 1,
                        "{label}: one drawn row has no slope — the fixture failed"
                    );
                    // THE DRAWN SPINE's own two endpoints, the pair `spine()` hands the
                    // quad: the rail's abscissa at each row, at each row's centre.
                    let (x0, y0) = (probe.spine_x(first.display), first.top + first.height * 0.5);
                    let (x1, y1) = (probe.spine_x(last.display), last.top + last.height * 0.5);
                    let drawn = (x1 - x0) / (y1 - y0);
                    eprintln!(
                        "MEASURED {label}: frost shear {shear:.5} vs drawn spine slope \
                         {drawn:.5} (({x0:.1},{y0:.1}) → ({x1:.1},{y1:.1})), step \
                         {:.3}",
                        probe.spine_step()
                    );
                    assert!(
                        (shear - drawn).abs() <= 1e-3,
                        "{label}: the frost leans at {shear:.5} while the spine the frame \
                         DREW rakes at {drawn:.5}. The rake yields on a cramped card \
                         (`TRAVEL_MAX_BAND_FRACTION`), so a shear read off the authored \
                         `ROW_STEP` agrees on an ordinary card and diverges here"
                    );
                    assert!(
                        shear.abs() > 1e-3,
                        "{label}: the frost's shear is {shear} on a world that drew a \
                         spine — a law that only compares two numbers is satisfied by \
                         both of them being nothing, and a lean of nothing is the \
                         rectangle this item exists to remove"
                    );
                    if !leaning.contains(&world) {
                        leaning.push(world);
                    }
                    // WHICH ARM this geometry exercised, recorded rather than assumed: a
                    // card that gave up rake is the only case where the measured step and
                    // the authored constant disagree, and therefore the only case where
                    // this law's equality has any force.
                    let authored = authored_step(&p);
                    if (probe.spine_step() - authored).abs() > 1e-3 {
                        yielded.push(format!(
                            "{label} (drawn {:.3} vs authored {authored:.3})",
                            probe.spine_step()
                        ));
                    }
                }
            }
        }
    }
    eprintln!("ENROLLED leaning={leaning:?} upright={upright:?} yielded={yielded:?}");
    assert!(
        !leaning.is_empty(),
        "no enrolled world leans — the shear has no subject (upright={upright:?})"
    );
    assert!(
        !yielded.is_empty(),
        "no geometry in this sweep took the rake YIELD, so every case compared the \
         measured step where it equals the authored `ROW_STEP` — and this law is then \
         satisfied by a frost that read the constant, which is the one thing it exists \
         to refuse. Widen the geometry sweep until a card gives up its rake."
    );
    assert!(
        !upright.is_empty(),
        "every enrolled world leans — the feather-only arm has no subject, and a \
         `Rules` world silently taking a shear is exactly what a name list would hide \
         (leaning={leaning:?})"
    );
    crate::theme::set_active(entry);
}

/// `Frost::Full` UNDER ALPHA BLENDING IS STILL A REPLACE — MEASURED, NOT ARGUED.
///
/// The composite target now blends, because the footprint's coverage rides in the
/// alpha. The claim that the full-takeover arm is unaffected rests on an alpha of
/// exactly 1.0 making `src × 1 + dst × 0` a replace — arithmetic that is easy to state
/// and easy to be wrong about (a mask that returned 0.999, a flag read off the wrong
/// component, a backend that treats `dst × 0` loosely).
///
/// So the property is measured as DESTINATION-INDEPENDENCE, which is what "a replace"
/// means operationally: the same composite drawn over two deliberately hostile and
/// DIFFERENT destinations must produce bit-for-bit identical pixels. If any pixel's
/// alpha were under 1.0, the two would differ there. It is asserted on the composite
/// pipeline directly, through `draw_backdrop` into a pass that LOADS its target, so no
/// clear can hide the destination the way a full frame's own first pass would.
#[test]
fn the_full_frosts_composite_is_destination_independent() {
    let _g = crate::testlock::serial();
    let Some((device, queue, _p)) = headless_dqp(400.0, 300.0) else {
        eprintln!("skipping the destination-independence law: no wgpu adapter");
        return;
    };
    let (w, h) = (400u32, 300u32);
    let mut bd = crate::render::blur::BlurBackdrop::new(&device, super::dither::FMT);
    bd.ensure(
        &device,
        &queue,
        crate::render::blur::BlurSurface {
            width: w,
            height: h,
            dpi: 1.0,
        },
        [0.5, 0.25, 0.75],
        crate::render::blur::Frost::Full,
    );

    // One run: clear the target to `clear`, then composite over it in a SECOND pass
    // that loads what the first left. The blurred source is whatever the (undrawn) doc
    // chain produced — identical between runs, which is all this needs.
    let run = |clear: wgpu::Color| {
        let (texture, tview) = offscreen(&device, w, h);
        let mut encoder = device.create_command_encoder(&Default::default());
        bd.encode_blur(&mut encoder);
        {
            let _ = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("hostile clear"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &tview,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("composite over a loaded target"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &tview,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            bd.draw_backdrop(&mut pass);
        }
        queue.submit(Some(encoder.finish()));
        read_pixels(&device, &queue, &texture, w, h)
    };

    let red = run(wgpu::Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    });
    let blue = run(wgpu::Color {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    });
    // NON-VACUITY FIRST: the two clears really are different, so a composite that drew
    // NOTHING would fail here rather than pass the identity below.
    let hostile;
    {
        let (t, v) = offscreen(&device, w, h);
        let mut enc = device.create_command_encoder(&Default::default());
        {
            let _ = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("bare red"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &v,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 1.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }
        queue.submit(Some(enc.finish()));
        let bare = read_pixels(&device, &queue, &t, w, h);
        hostile = bare.iter().zip(red.iter()).filter(|(a, b)| a != b).count();
    }
    assert!(
        hostile > (w * h / 2) as usize,
        "only {hostile} pixels differ between the bare red clear and the composite over \
         it — the composite drew nothing, so the identity below would be vacuous"
    );

    let differing = red.iter().zip(blue.iter()).filter(|(a, b)| a != b).count();
    assert_eq!(
        differing,
        0,
        "{differing} of {} pixels differ between a full-canvas composite over a RED \
         destination and the same composite over a BLUE one. `Frost::Full`'s mask must \
         be exactly 1.0 at every pixel — at `srcA == 1` the blend equation is \
         `src × 1 + dst × 0`, a replace, which is the whole reason the full-takeover arm \
         needs no second unblended pipeline. Any alpha under 1.0 lets the destination \
         through, and every full-takeover frost in the tree (palette, held HUD, lifetime \
         card, hold-⌘ peek) changes.",
        w * h
    );
}
