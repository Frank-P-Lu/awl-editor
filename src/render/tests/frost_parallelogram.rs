//! THE FROSTED FOOTPRINT'S SILHOUETTE, MEASURED IN PIXELS — IT IS A PARALLELOGRAM.
//!
//! User-reported and user-decided against a `Diagonal` world's theme picker: *"the blur
//! was not achieved… you can see how it's kinda like a square right? that's wrong… it
//! should be like a parallelogram."*
//!
//! The cause was structural rather than mistuned, and it needed no reproduction to be
//! believed. `blur::extent::footprint_dist_outside` was `upright.min(leaning)` — a UNION
//! whose upright term was the card's WHOLE BOX. The box is the bounding box of the leaning
//! rows, so the union always contained the full rectangle and the shear could only add two
//! overhang ears to it. **A parallelogram silhouette was impossible by construction at any
//! shear on any world.** The lean was real and was doing exactly what it was built to do;
//! it was invisible because the floor drawn beside it was larger.
//!
//! # What is measured here, and what is measured at the purer seam
//!
//! The SHAPE's arithmetic — both faces translating together, the frosted area falling
//! short of its own bounding box by the two triangular corners, and
//! `footprint_box`'s coverage floor — is graded in `blur::tests`, over the pure mirror,
//! with no GPU and no fixture. That is the purest reachable seam and it sweeps shears this
//! file cannot reach.
//!
//! What can only be measured HERE, off real pixels of a real frame:
//!
//! * **THE POSITIVE CLAIM the user asked for.** The card box's two OFF-RAKE corners are
//!   not frosted, so the document showing through them is SHARP — carrying real glyph
//!   edges at the same threshold. A rectangle has no such corners, which is why this
//!   is the one figure that could not be satisfied before the union was retired.
//! * **THE COVERAGE FLOOR, over the card's own drawn INK rather than over an enumeration
//!   of it.** Every pixel the card draws must sit inside the frost. The ink is DERIVED (the
//!   same picker over an empty document has a smooth backdrop, so any strong local step in
//!   it is the card's drawing), which is what makes this a safety net rather than a second
//!   list: an upright surface nobody remembered fails here by existing. This is the
//!   narrowed descendant of the retired union, and narrowing it is not deleting it.
//!
//! Swept over the enrolled roster — derived from `blur::footprint_frost_applies`, so
//! Paperbark (shear 0, whose parallelogram IS its rectangle) is in it and a world that
//! changes list style changes what this sweeps — at 1× and 2×, and over BOTH menu-bar
//! states, because the bar's reserve comes off the card's height budget and therefore off
//! the rake the rail resolves, and its default is platform-forked so an unforced run only
//! ever sees the host's own branch.
//!
//! ⚠️ The bar's arm is taken from the AMBIENT value and its negation, never from `cfg!`:
//! inside a test `cfg!(target_os = …)` reflects the host that COMPILED it rather than the
//! branch the value actually took, so a restore written that way restores the wrong value
//! under any forcing. `testlock::serial()` does not carry this global, so each law restores
//! it itself on the way out.

use super::super::*;
use super::frost_card_ink::{CardInk, INK_GRADIENT, luma, step};
use super::frost_feather::{DENSE, enrolled_worlds, render_frame, theme_picker};
use super::headless_dqp;

/// A local luma step that only a document EDGE produces — the threshold at the
/// same place in the same measured valley (that tree's frosted residue peaks near 5 and
/// its sharp residue near 190, so it is not load-bearing to within a factor of four).
pub(super) const STRONG_GRADIENT: f32 = 24.0;

/// THE COVERAGE FLOOR on the mask under the card's own ink: how much of the frost must be
/// present beneath a pixel the card draws.
///
/// Not `1.0`, and the reason is arithmetic rather than slack. `footprint_box` widens the
/// rect until the chrome sits EXACTLY on the shape's face, where the mask is exactly 1.0 —
/// but a glyph's anti-aliased skirt, and the two logical px `CardInk` dilates by to
/// swallow it, reach a little past the ink's own origin, and the mask ramps outward from
/// the face. A `smoothstep` over a 28 logical px feather is still 0.996 one px out and
/// 0.985 two px out, so this floor is set well under the roster's tightest MEASURED value
/// (reported by the law) and far above the 0.5 a half-covered edge would give.
const INK_FROST_FLOOR: f32 = 0.9;

/// The frost's own mask at a canvas pixel, evaluated through the SHIPPING policy's pure
/// mirror rather than a second copy of the arithmetic — so a retuned shape moves this
/// reading with it instead of leaving a law grading a shape the frame stopped drawing.
fn mask_at(frost: crate::render::blur::Frost, dpi: f32, px: f32, py: f32) -> f32 {
    crate::render::blur::footprint_mask_for(frost, dpi, px, py)
}

/// THE TWO FRAMES ONE CELL OF THESE SWEEPS NEEDS — the SAME pair used for the
/// same reason.
///
/// `open` is the picker over dense prose; `empty` is the identical picker over an EMPTY
/// document. The card's drawing is bit-identical between them, so their residue is the
/// DOCUMENT alone, and `ink` is the derived veto over the card's own pixels.
struct Frame {
    open: Vec<[u8; 4]>,
    empty: Vec<[u8; 4]>,
    ink: CardInk,
    /// The card's own box, as `overlay_card_rect` reports it and the pointer hit-test reads
    /// it — NOT the frost's.
    card: [f32; 4],
    frost: crate::render::blur::Frost,
    shear: f32,
    /// EVERY SURFACE THE CHROME PATH SAYS IT DREW — the production owner the shipped frost's
    /// own box is derived from, so a law asking "is this region card-free" asks the same
    /// enumeration the narrowing did rather than a threshold over pixels.
    surfaces: Vec<[f32; 4]>,
    w: i64,
    h: i64,
}

/// THE THEME PICKER WITH A TYPED QUERY — the shipping state the user photographed, and the
/// one that gives this file's subject a real width.
///
/// The shared fixture leaves the query EMPTY, which shapes a head band of the bare
/// `›` sigil: 10.4 logical px of ink on Magpie, and 142 non-ink pixels behind it. A coverage
/// floor over a band that small is nearly a floor over nothing, and its own presence guard
/// said so on the first run. A typed query is also the state whose CARET the head band's
/// coverage has to hold, so this is the more honest cell in both directions.
fn typed_picker(text: &str) -> ViewState {
    let mut v = theme_picker(text);
    v.overlay_query = "mangrove".to_string();
    v.overlay_query_caret = v.overlay_query.chars().count();
    v
}

fn capture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &mut TextPipeline,
    w: u32,
    h: u32,
    dpi: f32,
) -> Frame {
    p.set_view(&typed_picker(DENSE));
    let open = render_frame(device, queue, p, w, h);
    let card = p
        .overlay_card_rect()
        .expect("the crisp picker has a card box");
    let frost = p.frost_mode().expect("an enrolled world reaches the frost");
    let shear = match frost {
        crate::render::blur::Frost::Footprint(f) => f.shear,
        other => panic!("expected the footprint arm, got {other:?}"),
    };
    let surfaces = super::frost_card_ink::declared_card_surfaces(p, w);
    p.set_view(&typed_picker(""));
    let empty = render_frame(device, queue, p, w, h);
    let (wi, hi) = (w as i64, h as i64);
    Frame {
        ink: CardInk::derive(&empty, wi, hi, dpi),
        open,
        empty,
        card,
        frost,
        shear,
        surfaces,
        w: wi,
        h: hi,
    }
}

/// THE DOCUMENT'S SHARPNESS over a region — `(pixels measured, pixels carrying an edge,
/// peak local step)`. One owner, because both laws below ask it of different regions and a
/// second copy would be a second definition of "sharp".
///
/// ⚠️ **THE CARD-INK EXCLUSION IS THE CALLER'S, STATED AT THE CALL SITE, AND IT IS NOT THE
/// SAME EXCLUSION IN BOTH LAWS.** `CardInk`'s premise — "what the card draws over is a blur
/// of a blank page, and a blur has no step in it" — holds only WHERE THE FROST REACHES.
/// Under the head band it does. In the rake's own unfrosted corners it does not, and there
/// the veto cannot tell the card's ink from the world's live ground: a busier ground leaves
/// a law less to measure, with nobody having touched either the law or the product. Whether
/// this function excludes anything is therefore a decision each law makes about its own
/// region rather than one made here for both.
fn sharpness(f: &Frame, field: &[f32], keep: impl Fn(i64, i64) -> bool) -> (u64, u64, f32) {
    let (mut measured, mut edges, mut peak) = (0u64, 0u64, 0.0f32);
    for y in 0..f.h {
        for x in 0..f.w {
            if !keep(x, y) {
                continue;
            }
            let s = step(field, f.w, f.h, x, y);
            measured += 1;
            peak = peak.max(s);
            if s >= STRONG_GRADIENT {
                edges += 1;
            }
        }
    }
    (measured, edges, peak)
}

/// EVERY SURFACE THE CHROME PATH SAYS IT DREW SITS INSIDE THE FROST — the DECLARED
/// counterpart of the derived ink floor, asked of the owner the shipped narrowing is itself
/// derived from (`overlay_drawn_surfaces`): the seats glyphon was handed, the rules, the
/// rails, the spine's caps, every row's mark. Returns how many surfaces were graded.
///
/// It is what makes the corner law's unfrosted region a DOCUMENT measurement. That region is
/// derived from the mask alone and holds no card ink exactly while every declared surface is
/// inside the frost. The alternative is a pixel-derived veto, and in those corners a veto
/// cannot tell the card's rows and ring from a lava lamp — so what it excludes is a property
/// of the WORLD, and the law's subject shrinks as a theme's ground gets busier without anyone
/// having touched either the law or the shape.
///
/// ⚠️ It catches a narrowing that CUTS a declared surface loose, and it cannot catch one that
/// was never declared: the shipped box is derived from the same list, so dropping a term
/// keeps every remaining term inside. Completeness belongs to the coverage law that measures
/// the card's ink off a frost-suppressed frame.
fn every_declared_surface_is_frosted(f: &Frame, dpi: f32, label: &str) -> usize {
    assert!(
        !f.surfaces.is_empty(),
        "{label}: the chrome path declares NO drawn surface, so the card-free claim the corner \
         law rests on is a claim over an empty enumeration — and the narrowing's own inert \
         answer to that is to keep the whole box, so nothing downstream would notice"
    );
    for s in &f.surfaces {
        let (worst, wx, wy) = tightest_coverage(f.frost, dpi, *s);
        assert!(
            worst >= INK_FROST_FLOOR,
            "{label}: a surface the card DECLARES it draws, {s:?}, reaches ({wx:.1},{wy:.1}) \
             where the frost's coverage is only {worst:.4} (floor {INK_FROST_FLOOR}). That is \
             card ink over sharp document, and it also breaks the premise the corner sharpness \
             stands on — that a region the mask does not reach carries no card ink"
        );
    }
    f.surfaces.len()
}

/// EVERY ONE OF THE CARD'S OWN FOUR CORNERS, NAMED, WITH ITS FROST COVERAGE — the figure
/// that separates a parallelogram from a rectangle when it is asked of the CARD rather than
/// of the shape's own bounding box.
fn card_corner_coverage(f: &Frame, dpi: f32) -> Vec<(&'static str, f32)> {
    let [rx, ry, rw, rh] = f.card;
    [
        ("top-left", rx, ry),
        ("top-right", rx + rw, ry),
        ("bottom-left", rx, ry + rh),
        ("bottom-right", rx + rw, ry + rh),
    ]
    .iter()
    .map(|(n, px, py)| (*n, mask_at(f.frost, dpi, *px, *py)))
    .collect()
}

/// THE HEADLINE LAW: THE CARD BOX'S TWO OFF-RAKE CORNERS ARE NOT FROSTED, AND THE
/// DOCUMENT SHOWING THROUGH THEM IS SHARP.
///
/// This is the user's own figure, in pixels. A rectangle — and the retired box-union,
/// which contained one — has no unfrosted corner inside the card's box at all, so the
/// region this law measures is EMPTY under the defect and the law reports its own vacuity
/// rather than passing green.
///
/// WHICH corners is derived, never named: each one's coverage is read off the shipping
/// policy's own mirror. On an upright composition (`shear == 0`, whose parallelogram IS its
/// rectangle, and whose box is its card's) every corner is fully frosted, which is why that
/// arm is required to leave NOTHING short and is named on failure.
///
/// ⚠️ **THE COUNT IS AN UPPER BOUND ON THE *FULLY FROSTED* CORNERS, NOT AN EXACT COUNT OF
/// THE SHORT ONES**, and the difference is the frost's WIDTH. This law once required exactly
/// two of the four to fall short, which held only while the shape's box was bounded BELOW by
/// the card's: once the box is narrowed to the surfaces the card actually draws, a corner can
/// be short because the frost is narrower there as well as because the rake left it, and a
/// mirrored card measures three. What survives the narrowing is the property that separates
/// the two shapes: the retired box-UNION *contains* the card's box, so all FOUR of its
/// corners are fully frosted, always, and a parallelogram of any width holds at most two.
///
/// # ⚠️ THE SUBJECT MUST NOT BE A FUNCTION OF THE WORLD'S GROUND
///
/// The sharpness half of this law grades the part of the card's box the frost does not reach
/// at all, and that region is the one place `CardInk` cannot do its job: the card genuinely
/// draws rows and a ring in the off-rake corners, the world's live ground is at full
/// sharpness there too, and no threshold separates them. A veto there is sound (it biases
/// DOWN) but it DEGRADES SILENTLY — a busier ground leaves the law less to measure, so the
/// subject can shrink toward nothing through a THEME change, with nobody having touched
/// either this law or the shape it grades. That is the "law satisfiable by deleting its own
/// subject" family one step removed, and a presence guard alone only tells you the day it
/// finally bites.
///
/// So the subject is split into two pieces neither of which is a threshold over pixels:
///
/// * **THE REGION** is the card's box intersected with `mask == 0`, read through the
///   shipping policy's own mirror. It touches no pixel, so `measured` is arithmetic over the
///   shape and cannot erode.
/// * **THE FIELD** is the residue between the picker over prose and the picker over an empty
///   document. The card's drawing and the world's ground are bit-identical between those two
///   frames, so both cancel and what is left is the DOCUMENT — which is what the law claims
///   to be measuring.
/// * **THE CARD-FREE CLAIM** comes from the chrome path's own declaration
///   (`TextPipeline::overlay_drawn_surfaces`, the owner the shipped narrowing is derived
///   from): every surface it reports is required to sit inside the frost, which is what puts
///   all of it outside the region. A narrowing that cut one loose fails by name.
#[test]
fn the_card_boxs_two_off_rake_corners_are_unfrosted_and_the_document_there_is_sharp() {
    let _g = crate::testlock::serial();
    let entry = crate::theme::active_index();
    let mut leaning: Vec<String> = Vec::new();
    let mut upright: Vec<String> = Vec::new();
    let mut fewest_surfaces = usize::MAX;
    let ambient_bar = crate::menubar::menu_bar_on();
    for world in enrolled_worlds() {
        for bar in [ambient_bar, !ambient_bar] {
            for (dpi, w, h) in [(1.0f32, 1200u32, 900u32), (2.0, 2400, 1800)] {
                let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
                    eprintln!("skipping the parallelogram sweep: no wgpu adapter");
                    crate::menubar::set_menu_bar_on(ambient_bar);
                    return;
                };
                crate::theme::set_active_by_name(world).unwrap();
                crate::menubar::set_menu_bar_on(bar);
                p.set_dpi(dpi);
                let f = capture(&device, &queue, &mut p, w, h, dpi);
                let label = format!("{world} @ {dpi}x ({w}x{h}) bar {bar}");
                // THE CARD'S OWN BOX is the frame of reference, because it is the thing
                // the user sees an outline of and asks "is that a parallelogram?".
                //
                // ⚠️ NOT the shape's own bounding box, and that distinction is the whole
                // reason this law reads as it does: the FIRST version of it asked whether two
                // corners of the shape's bbox were unfrosted, and it PASSED under its own
                // mutation. The retired union's two ears reach exactly the same bbox, so its
                // bbox corners are unfrosted too. The figure that separates the two shapes is
                // asked of the CARD: a union CONTAINS the card's box, so all four of its
                // corners are fully frosted, always. A parallelogram leaves two behind.
                let corners = card_corner_coverage(&f, dpi);
                let short: Vec<_> = corners.iter().filter(|(_, m)| *m < 1.0).collect();
                let full = corners.len() - short.len();

                // Stated BEFORE the shear branch on purpose, so the upright `Rules` member is
                // graded too — it is the one composition that draws at the card's full band,
                // and therefore the only one whose surfaces a narrowing can strand.
                fewest_surfaces =
                    fewest_surfaces.min(every_declared_surface_is_frosted(&f, dpi, &label));
                if f.shear == 0.0 {
                    upright.push(label.clone());
                    assert!(
                        short.is_empty(),
                        "{label}: shear is 0, so the parallelogram IS the card's rectangle, \
                         yet {short:?} came back short of fully frosted"
                    );
                    continue;
                }
                leaning.push(label.clone());
                assert!(
                    full <= 2,
                    "{label}: {full} of the CARD's own four corners are FULLY frosted (shear \
                     {}, coverages {:?}). A parallelogram can hold at most two; FOUR is the \
                     retired box-union, which contained the card's box as one of its terms, \
                     and four is the rectangle the user photographed",
                    f.shear,
                    corners
                );

                // THE DOCUMENT IN THOSE CORNERS IS SHARP, over the part of the card's own box
                // the frost's mask does not reach AT ALL. Under the union that region is EMPTY
                // by construction, so the count is both a presence guard and the mutation's
                // own tripwire.
                //
                // ⚠️ TWO THINGS MAKE THIS SUBJECT A PROPERTY OF THE SHAPE AND NOT OF THE
                // WORLD'S GROUND. The REGION reads no pixel at all — the card's box and the
                // shipping mask, and nothing else — so `measured` is arithmetic and cannot
                // erode as a theme's ground gets busier. And the FIELD is the two frames'
                // residue rather than the open frame's luma, so the card's ink and the
                // world's ground both cancel by construction and what remains is the page.
                // A veto here would be neither: the corners are exactly where `CardInk`
                // cannot tell the card's rows and ring from a lava lamp, and what it took
                // was sample size.
                let residue: Vec<f32> = f
                    .open
                    .iter()
                    .zip(f.empty.iter())
                    .map(|(a, b)| luma(*a) - luma(*b))
                    .collect();
                let [rx, ry, rw, rh] = f.card;
                let in_corner = |fx: f32, fy: f32| {
                    fx >= rx
                        && fx < rx + rw
                        && fy >= ry
                        && fy < ry + rh
                        && mask_at(f.frost, dpi, fx, fy) == 0.0
                };
                let (measured, edges, peak) =
                    sharpness(&f, &residue, |x, y| in_corner(x as f32, y as f32));

                eprintln!(
                    "MEASURED {label}: shear {:.5}, card corner coverages {:?} ({full} full), \
                     {measured} wholly unfrosted px INSIDE the card's box (region derived from \
                     the mask alone, {} declared surfaces all inside the frost), {edges} \
                     carrying a document edge (peak step {peak:.1})",
                    f.shear,
                    corners,
                    f.surfaces.len()
                );
                assert!(
                    measured > 500,
                    "{label}: only {measured} pixels of the CARD's own box are outside the \
                     frost. Under the retired box-union that count is exactly ZERO, so a \
                     small number here is this law reporting that its subject does not exist \
                     rather than the product passing"
                );
                assert!(
                    edges > 40 && peak >= STRONG_GRADIENT,
                    "{label}: the {measured} unfrosted pixels inside the CARD's box carry \
                     only {edges} document edges (peak step {peak:.1}, threshold \
                     {STRONG_GRADIENT}) — the corners the rake leaves behind are supposed to \
                     show the page's own SHARP document, which is what makes the silhouette \
                     read as a parallelogram rather than as a box"
                );
            }
        }
    }
    crate::menubar::set_menu_bar_on(ambient_bar);
    assert!(
        !leaning.is_empty() && !upright.is_empty(),
        "the roster must contain a LEANING enrolled world and an UPRIGHT one, or one of \
         this law's two arms never ran: leaning {leaning:?}, upright {upright:?}"
    );
    eprintln!("ROSTER FEWEST declared drawn surfaces in one cell: {fewest_surfaces}");
    // THE ROSTER-SCOPE PRESENCE FLOOR, not a per-cell one: three is what a composition
    // organised by ABSENCE declares — its one shaped column, its query caret, and the rule
    // band that runs the card's full width — and the leaning compositions declare 29. A
    // per-cell floor at three would go red on a legitimate quieter composition; here a drop
    // is visible without being pinned to the tightest member the roster happens to hold.
    assert!(
        fewest_surfaces >= 3,
        "the quietest cell declares only {fewest_surfaces} drawn surfaces — the enumeration \
         the card-free claim rests on has thinned out, and every consumer of it (this law and \
         the shipped narrowing alike) is reading a shorter list than the card draws"
    );
    crate::theme::set_active(entry);
}

/// THE TIGHTEST FROST COVERAGE anywhere in a box, and where — sampled on a grid through the
/// SHIPPING mask. The head band's own guarantee, asked of the box its production owner
/// declares rather than of a list of the surfaces inside it.
fn tightest_coverage(frost: crate::render::blur::Frost, dpi: f32, b: [f32; 4]) -> (f32, f32, f32) {
    let [l, t, r, bo] = b;
    let (mut worst, mut at) = (1.0f32, (l, t));
    for iy in 0..=24 {
        for ix in 0..=24 {
            let px = l + (r - l) * ix as f32 / 24.0;
            let py = t + (bo - t) * iy as f32 / 24.0;
            let m = mask_at(frost, dpi, px, py);
            if m < worst {
                worst = m;
                at = (px, py);
            }
        }
    }
    (worst, at.0, at.1)
}

/// THE COVERAGE FLOOR, NARROWED AND NOT DELETED: THE CARD'S UPRIGHT CHROME IS FROSTED, AND
/// NO DOCUMENT EDGE SURVIVES BEHIND IT.
///
/// The retired union frosted the card's whole box because the card's HEAD band is upright
/// and flush to its text edge while the rows rake away from it, so a shape that only
/// followed the rake left that band over sharp document — the reported defect, moved onto
/// the card's own chrome. The duty survives; its owner moved into the shape's own WIDTH
/// (`blur::extent::footprint_box`), which widens the rect until the parallelogram contains
/// the band. This law is that guarantee, stated twice over the SAME box:
///
/// 1. **ARITHMETIC.** Every point of the band's box, sampled on a grid, has the shipping
///    mask at or above [`INK_FROST_FLOOR`]. Read through `footprint_mask_for`, so it grades
///    the coverage the composite pass was actually handed.
/// 2. **PIXELS.** Behind that band, no glyph edge of the DOCUMENT survives — the same
///    statistic and the same threshold the headline law uses, over the residue
///    between the picker-over-prose frame and the picker-over-empty one. Arithmetic alone
///    would pass a mask that was right about a shape the shader never drew.
///
/// ⚠️ It is the companion the shape law needs, not an ornament beside it. "The silhouette is
/// not a rectangle" gets strictly HAPPIER as the frost shrinks — a shape that frosted
/// nothing would leave every corner unfrosted and every document edge intact and satisfy
/// that law perfectly. This is what refuses it.
///
/// # ⚠️ Why the subject is a DECLARED box and not a derived ink mask
///
/// Two ink oracles were built here and both were falsified on their first run, which is
/// worth more than the law they were meant to serve. Both read `CardInk` as an inclusion
/// set — "these pixels are where the card is" — and it is a VETO that does not invert:
/// its flagged set is a superset of the card's drawing, and the surplus is the world's own
/// live ground outside the frost's reach. Intersecting it with a picker-open-versus-closed
/// difference does not rescue it either, because the card's soft SHADOW changes ground
/// structure it does not own. `frost_card_ink` states that contract at the definition and
/// keeps both figures measured.
///
/// So the subject is the box the PRODUCTION owner declares
/// (`TextPipeline::overlay_head_band_ink`), which is also the box `footprint_box` is handed
/// — the two cannot drift, and a third upright surface is a change to that owner rather than
/// to this law. The broad net over everything else is the headline law, whose
/// interior region narrows to the parallelogram.
#[test]
fn the_cards_upright_chrome_is_frosted_and_no_document_edge_survives_behind_it() {
    let _g = crate::testlock::serial();
    let entry = crate::theme::active_index();
    let mut tightest = f32::INFINITY;
    let mut tightest_at = String::new();
    let ambient_bar = crate::menubar::menu_bar_on();
    for world in enrolled_worlds() {
        for bar in [ambient_bar, !ambient_bar] {
            for (dpi, w, h) in [(1.0f32, 1200u32, 900u32), (2.0, 2400, 1800)] {
                let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
                    eprintln!("skipping the chrome-coverage sweep: no wgpu adapter");
                    crate::menubar::set_menu_bar_on(ambient_bar);
                    return;
                };
                crate::theme::set_active_by_name(world).unwrap();
                crate::menubar::set_menu_bar_on(bar);
                p.set_dpi(dpi);
                let f = capture(&device, &queue, &mut p, w, h, dpi);
                let label = format!("{world} @ {dpi}x ({w}x{h}) bar {bar}");
                let geom = p.overlay_geometry(w);
                let plan = p.overlay_row_plan(&geom);
                let [hl, ht, hr, hb] = p
                    .overlay_head_band_ink(&geom, &plan)
                    .unwrap_or_else(|| panic!("{label}: the card plans a header band"));
                assert!(
                    hr > hl && hb > ht,
                    "{label}: the head band's declared box [{hl},{ht},{hr},{hb}] is empty, so \
                     everything below it is a floor over nothing"
                );

                // (1) ARITHMETIC: the whole declared box, on a grid, through the shipping
                // mask. (2) PIXELS: the document's own residue behind that same box.
                let (worst, wx, wy) = tightest_coverage(f.frost, dpi, [hl, ht, hr, hb]);
                if worst < tightest {
                    tightest = worst;
                    tightest_at = label.clone();
                }
                let residue: Vec<f32> = f
                    .open
                    .iter()
                    .zip(f.empty.iter())
                    .map(|(a, b)| luma(*a) - luma(*b))
                    .collect();
                // THE VETO IS SOUND HERE and nowhere else in this file: the head band sits
                // where the frost's coverage is at its floor or above, so the empty frame
                // under it is a blur of a blank page and every step in it is the card's own.
                let (measured, edges, peak) = sharpness(&f, &residue, |x, y| {
                    let (fx, fy) = (x as f32, y as f32);
                    fx >= hl && fx <= hr && fy >= ht && fy <= hb && !f.ink.vetoes(x, y)
                });
                eprintln!(
                    "MEASURED {label}: head band [{hl:.1},{ht:.1},{hr:.1},{hb:.1}] — tightest \
                     frost coverage {worst:.4} at ({wx:.1},{wy:.1}), and {edges}/{measured} \
                     non-ink px behind it carry a document edge (peak step {peak:.1})"
                );
                assert!(
                    worst >= INK_FROST_FLOOR,
                    "{label}: the card's upright head band reaches ({wx:.1},{wy:.1}) where \
                     the frost's coverage is only {worst:.4} (floor {INK_FROST_FLOOR}) — that \
                     chrome is sitting over SHARP document, which is the reported defect \
                     moved onto the card's own chrome. `blur::extent::footprint_box` widens \
                     the shape's box exactly to prevent it, so either the widening is wrong \
                     or the band it was handed is not the band that was drawn"
                );
                assert!(
                    measured > 200,
                    "{label}: only {measured} non-ink px behind the head band (gradient \
                     {INK_GRADIENT}) — a zero-edge claim over an empty set is satisfied by \
                     anything"
                );
                assert_eq!(
                    edges, 0,
                    "{label}: {edges} of {measured} pixels behind the card's head band carry \
                     a document EDGE (step >= {STRONG_GRADIENT}, peak {peak:.1}) — the page \
                     is still drawing as sharp TEXT under the query field"
                );
            }
        }
    }
    crate::menubar::set_menu_bar_on(ambient_bar);
    eprintln!(
        "ROSTER TIGHTEST head-band coverage {tightest:.4} at {tightest_at} — the floor \
         {INK_FROST_FLOOR} sits under it"
    );
    crate::theme::set_active(entry);
}
