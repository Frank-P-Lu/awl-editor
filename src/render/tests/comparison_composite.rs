//! THE COMPARISON IS COMPOSITED, AND IT IS COMPOSITED *ON* THE
//! WORKSPACE'S SURFACE.
//!
//! The document layer's GEOMETRY was relocated into a workspace's content
//! region and stopped there, pinning what it had NOT done as a law
//! (`the_relocated_document_is_geometrically_placed_but_not_yet_composited`)
//! whose own message asked to be deleted and replaced by a containment and
//! visibility law the day a workspace drew comparison content. This is that
//! replacement.
//!
//! The user's compositing call (2026-08-02) picked the arm these laws describe:
//! the comparison sits **ON** the workspace surface. The card stays ONE OPAQUE
//! SURFACE — "A WORKSPACE IS ONE SURFACE" survives intact — and the document's
//! CONTENT is submitted AFTER it, into the carved region, without re-drawing its
//! own ground. The rejected arm (a window THROUGH the card) would have shown the
//! BACKDROP's ground, because the ground punch is at the page column and not at
//! the region, and on a blur-eligible world would have frosted the frame AROUND
//! the workspace — exactly where the region is not.
//!
//! So there are three claims, and each is asserted with an oracle that cannot
//! read the code under test:
//!
//! 1. **CONTAINMENT (surface).** A workspace surface still covers the whole
//!    comparison region, in every world. The card did not grow a hole.
//! 2. **VISIBILITY + CONTAINMENT (pixels).** Two frames whose ONLY difference is
//!    the PROSE of the document inside the region: its pixels must differ
//!    perceptibly (the transcript reaches the screen at all) and every pixel
//!    OUTSIDE the region must be BYTE-IDENTICAL (no glyph escapes onto the
//!    workspace's face, and no frosted ghost of the transcript lands in the
//!    frame around it). One differential, both halves, real pixels.
//! 3. **THE SWEPT AXIS.** Claim 2 over the whole world roster, and separately
//!    over the canvas / zoom / DPI range — because the region's box is derived
//!    from all three, and the axis a law sweeps must be the one the author did
//!    not think of.

use super::super::TextPipeline;
use super::pixeldiff::{DistinguishFloor, Region, assert_identical, assert_perceptibly_different};
use super::{comparison_view, headless_dqp, view};

/// The "is this prose on the screen" floor. Body text inks a small percentage of
/// its own column, so the DEFAULT floor (a fill band's) is the wrong instrument:
/// the magnitude bar is RAISED instead (a glyph against its surface is a strong
/// per-channel delta, not a rounding wobble) and the area bar lowered, with the
/// slack taken up by grading the region in thirds. Measured headroom on the
/// shipping roster at 1200x800: ~0.6% of a whole region differs, ~6x this bar,
/// while a caret alone is under 0.1% and lands in exactly one third.
const TEXT_INK_FLOOR: DistinguishFloor = DistinguishFloor {
    min_fraction: 0.0015,
    min_max_delta: 40,
};

/// A markdown transcript with the shape a real writer's diff has — a title, a
/// blockquote, a highlighted run, a struck run — long enough that it fills the
/// comparison region and overruns it at the bottom, so the clip genuinely bites
/// rather than the fixture simply fitting.
///
/// `body` is the only thing the two arms of the differential vary. The TITLE
/// LINE, the line COUNT and the HEADING count are held identical between them on
/// purpose, because the BACKDROP legitimately reads two of those: the ground
/// punch sits at the page column, which the adaptive-column policy shifts when
/// the margin outline wants a rail — and `outline_wants_rail` is
/// `!outline_headings.is_empty()`. A control with no headings would move the
/// backdrop's own punch and read as "document ink escaped the region" when
/// nothing had escaped at all. (Deliberately NOT fixed by gating that rail on the
/// comparison: releasing it would make the backdrop's page column JUMP the moment
/// a workspace opens, which is worse than the stability it has today.)
///
/// EVERY WORD is the arm's own token, which is stronger than it looks and was
/// arrived at the hard way. A fixture where the token appears once per paragraph
/// leaves every WRAPPED CONTINUATION row byte-identical between the two arms, and
/// at 2x DPI or a deep zoom a whole band of the region can consist of nothing but
/// continuation rows — which reads as "the document is not drawn here" when it
/// plainly is. This law's own DPI/zoom sweep failed on exactly that band, twice,
/// on two successively less naive fixtures.
pub(super) fn sample_transcript() -> String {
    transcript(ARM_A)
}

fn transcript(body: &str) -> String {
    let words = |n: usize| -> String {
        (0..n)
            .map(|_| format!("{body} "))
            .collect::<String>()
            .trim_end()
            .to_string()
    };
    let mut s = format!("# {} \n\n", words(4));
    for _ in 0..48 {
        s.push_str(&format!(
            "{}\n\n> ==({})== and ~~{}~~\n\n",
            words(16),
            words(4),
            words(4)
        ));
    }
    s
}

/// The two arms of the differential: structurally identical documents whose
/// PROSE differs. Any pixel that differs between their two frames is document
/// ink, and nothing else.
const ARM_A: &str = "Paragraph";
const ARM_B: &str = "Zzzzzzzzz";

/// TWO rendered frames of the comparison workspace whose only difference is the
/// PROSE of the document relocated into it ([`ARM_A`] versus [`ARM_B`]).
///
/// Everything else — the summoned workspace, its rows, its lens, the canvas, the
/// zoom, the DPI, the world, the document's line and heading structure — is held
/// identical, so any pixel that differs between the two is document ink and
/// nothing else. That is what makes this a differential oracle rather than a
/// screenshot comparison: the ground, the card, the timeline rows and the chrome
/// all cancel exactly.
struct Pair {
    with_doc: Vec<[u8; 4]>,
    other: Vec<[u8; 4]>,
    region: [f32; 4],
    /// The DOCUMENT's own line height on this frame — the scale the spread claim
    /// below is expressed in, so the same law reads correctly at zoom 0.7 on a 1x
    /// panel and at zoom 1.9 on a 2x one.
    lh: f32,
    /// The DOCUMENT's own character width on this frame — the scale the region's
    /// WIDTH is judged in, for the same reason.
    cw: f32,
    w: u32,
    h: u32,
}

fn render_pair(
    p: &mut TextPipeline,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    w: u32,
    h: u32,
    zoom: f32,
    focus: bool,
) -> Option<Pair> {
    // THE CARET IS PARKED OUT OF THE DIFFERENTIAL. Its cursor sits far below the
    // visible window, so the clip's park gate drops it entirely and neither frame
    // draws one. Otherwise the caret's own ink-box borrow reads the glyph under
    // it — which differs between the arms — and a band could clear the floor on
    // a caret alone while the prose behind it was not composited at all.
    let arm = |body: &str| {
        let mut v = comparison_view(&transcript(body), 400, 0);
        v.zoom = zoom;
        v.is_markdown = true;
        // On the NARROW stage only the FOCUSED region is drawn, so a comparison
        // exists there exactly when the content stage holds focus.
        v.overlay_detail_focus = focus;
        v
    };
    p.set_view(&arm(ARM_A));
    p.prepare(device, queue, w, h).ok()?;
    let region = p.comparison_viewport()?;
    let with_doc = super::pixeldiff::render_frame(p, device, queue, w, h);

    p.set_view(&arm(ARM_B));
    p.prepare(device, queue, w, h).ok()?;
    // The other arm must resolve the SAME region, or the two frames are not
    // comparable — the region is derived from the workspace, never the document,
    // so this is an invariant worth failing on rather than tolerating.
    let control_region = p
        .comparison_viewport()
        .expect("the control frame must place the same comparison region");
    assert_eq!(
        control_region, region,
        "the comparison region must not depend on the document inside it"
    );
    let other = super::pixeldiff::render_frame(p, device, queue, w, h);
    Some(Pair {
        with_doc,
        other,
        region,
        lh: p.metrics.line_height,
        cw: p.metrics.char_width,
        w,
        h,
    })
}

/// Assert claim 2 over one rendered pair: perceptible difference INSIDE the
/// region, byte-identity everywhere OUTSIDE it.
///
/// The outside is graded as FOUR bands (above / below / left / right of the
/// region) rather than as "the canvas minus the region", because a whole-canvas
/// diff cannot be expressed as one `Region` and a hand-rolled mask would be a
/// second implementation of the thing under test.
/// Returns whether the VISIBILITY half was graded. Containment is graded on
/// every cell; visibility needs a region that can hold prose at all, and at the
/// extreme end of the geometry range (a small canvas at 2x DPI and a deep zoom)
/// the region is barely one line tall — the app's own minimum window keeps a real
/// session out of that corner, but the sweep visits it, and grading "the document
/// fills the region" against a region that cannot hold a line would be grading
/// the fixture.
#[must_use]
fn assert_contained_and_visible(pair: &Pair, label: &str) -> bool {
    let [rx, ry, rw, rh] = pair.region;
    let (cw, ch) = (pair.w as i64, pair.h as i64);
    // THE SPREAD CLAIM NEEDS A REGION THAT CAN HOLD PROSE — in BOTH axes.
    //
    // Height was already guarded. Width was not, and the sweep visits a cell
    // where the pane is under thirteen characters wide (a 450x700 LOGICAL window
    // at 190% zoom on a 2x panel): there the fixture's own nine-letter word
    // breaks across three rows of a giant heading and the top of the region is
    // occupied by the blockquote's open-quote ORNAMENT, which is arm-identical
    // by construction — so "the first DIFFERING pixel" lands wherever that
    // decoration happens to end, and the claim is grading the fixture's wrap,
    // not whether the document was composited. Twenty characters is a short
    // prose line; below it the spread claim says nothing this law means.
    let gradeable = pair.lh > 0.0 && rh >= 4.0 * pair.lh && rw >= 20.0 * pair.cw;
    // THE VISIBILITY HALF, in TWO independent claims.
    //
    // (a) MAGNITUDE — the region's pixels differ at all. Prose is sparse (a page
    //     of body text inks well under 1% of its own column), so
    //     `DistinguishFloor::DEFAULT`, calibrated for a fill band, is the wrong
    //     instrument; `TEXT_INK_FLOOR` raises the per-channel bar instead.
    //
    // (b) SPREAD — the differing ink REACHES both ends of the region, to within
    //     two line-heights. This is what catches the failure that magnitude alone
    //     cannot: a document that draws its first row and is then clipped away, or
    //     one whose tail is cut by a stale canvas-derived row budget, still clears
    //     a fraction floor on its opening lines. Expressed in the DOCUMENT's own
    //     line height rather than in a fraction of the region, because at zoom 1.9
    //     on a 2x panel a third of the region is less than one line's leading —
    //     an earlier cut of this law graded fixed thirds and failed there on a
    //     fixture that was drawing perfectly.
    if gradeable {
        assert_perceptibly_different(
            &pair.with_doc,
            &pair.other,
            cw,
            ch,
            Region::new(rx, ry, rw, rh),
            TEXT_INK_FLOOR,
            &format!(
                "{label}: the relocated document must be VISIBLE inside the comparison region — \
             the workspace card is drawn over the document layer, so a comparison that is \
             not submitted AFTER the card renders an EMPTY WORKSPACE, which is exactly what \
             item 114 forbids. Two documents that differ in every word must not render the \
             same pixels here"
            ),
        );
        let (mut first, mut last) = (f32::MAX, f32::MIN);
        let (x0, x1) = (rx.max(0.0) as i64, ((rx + rw) as i64).min(cw));
        let (y0, y1) = (ry.max(0.0) as i64, ((ry + rh) as i64).min(ch));
        for y in y0..y1 {
            for x in x0..x1 {
                let i = (y * cw + x) as usize;
                if pair.with_doc[i] != pair.other[i] {
                    first = first.min(y as f32);
                    last = last.max(y as f32);
                    break;
                }
            }
        }
        let slack = 2.0 * pair.lh;
        assert!(
            first <= ry + slack && last >= ry + rh - slack,
            "{label}: the relocated document must fill the comparison region, not merely open \
         in it — differing ink spans y {first}..{last} inside a region running \
         {ry}..{} (line height {}, tolerance {slack}). Ink that stops short means the \
         document's own tail is not reaching the region's bottom",
            ry + rh,
            pair.lh
        );
    }

    assert_nothing_escaped(pair, label);
    gradeable
}

/// The CONTAINMENT half: every pixel OUTSIDE the region is byte-identical
/// between the two arms.
///
/// The outside is graded as FOUR bands (above / below / left / right of the
/// region) rather than as "the canvas minus the region", because a whole-canvas
/// diff cannot be expressed as one `Region` and a hand-rolled mask would be a
/// second implementation of the thing under test.
fn assert_nothing_escaped(pair: &Pair, label: &str) {
    let [rx, ry, rw, rh] = pair.region;
    let (cw, ch) = (pair.w as i64, pair.h as i64);
    // THE SEAM. The region's own boundary ROW/COLUMN is excluded from the
    // outside bands: the clip is a float edge and the rasterizer resolves a quad
    // ending exactly on it with partial coverage, which can tint the boundary
    // pixel by a channel step or two (measured: 2 pixels at delta 3, on Galah).
    // That is the clip working, not the document escaping — a real escape is a
    // whole GLYPH, tens of pixels tall and hundreds wide, so excluding one pixel
    // costs the law nothing it could otherwise catch.
    const SEAM: i64 = 1;
    let (top, bottom) = (ry as i64, (ry + rh).ceil() as i64);
    let (left, right) = (rx as i64, (rx + rw).ceil() as i64);
    let bands: [(&str, Region); 4] = [
        (
            "above",
            Region {
                x: 0,
                y: 0,
                w: cw,
                h: top - SEAM,
            },
        ),
        (
            "below",
            Region {
                x: 0,
                y: bottom + SEAM,
                w: cw,
                h: ch,
            },
        ),
        (
            "left",
            Region {
                x: 0,
                y: top,
                w: left - SEAM,
                h: bottom - top,
            },
        ),
        (
            "right",
            Region {
                x: right + SEAM,
                y: top,
                w: cw,
                h: bottom - top,
            },
        ),
    ];
    for (side, band) in bands {
        assert_identical(
            &pair.with_doc,
            &pair.other,
            cw,
            ch,
            band,
            &format!(
                "{label}: nothing of the relocated document may reach the canvas {side} the \
                 comparison region {:?}. Two ways it can: a GLYPH that outruns the region \
                 (the content is drawn after the card now, so it would land on the \
                 workspace's own face), or — on a blur-eligible world — the offscreen \
                 backdrop capture including the document, which frosts the transcript's \
                 ghost into the frame AROUND the workspace, exactly where the region is not",
                pair.region
            ),
        );
    }
}

/// CLAIM 1 — THE CARD DID NOT GROW A HOLE.
///
/// The compositing call was "ON the surface", not "a window THROUGH it", and the
/// difference is visible right here: a workspace surface must still cover the
/// whole comparison region. This is the containment half, kept and
/// re-aimed — it used to read "no comparison content is composited yet"; it now
/// reads "the comparison is composited onto a surface that is still one piece".
#[test]
fn the_workspace_surface_still_covers_the_whole_comparison_region_in_every_world() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping the_workspace_surface_still_covers_the_comparison_region: no adapter");
        return;
    };
    let mut graded = 0usize;
    for world in crate::theme::THEMES {
        crate::theme::set_active_by_name(world.name).expect("a roster world");
        p.sync_theme();
        let w = world.name;
        p.set_view(&comparison_view(&transcript(ARM_A), 0, 0));
        p.prepare(&device, &queue, 1200, 800).unwrap();
        let [vx, vy, vw, vh] = p
            .comparison_viewport()
            .unwrap_or_else(|| panic!("{w}: the document must be relocated"));
        let geom = p.workspace_geometry(1200);
        let plan = p.overlay_row_plan(&geom);
        let fills = p.overlay_pane_fills(&geom, &plan);
        assert!(
            fills.iter().any(|&[fx, fy, fw, fh]| fx <= vx
                && fy <= vy
                && fx + fw >= vx + vw
                && fy + fh >= vy + vh),
            "{w}: a workspace is ONE SURFACE and the comparison sits ON it, so one fill must \
             still contain the whole region — a hole punched here would show the BACKDROP's \
             ground, because the ground punch is at the page column and not at the region \
             (fills {fills:?}, region {:?})",
            [vx, vy, vw, vh]
        );
        graded += 1;
    }
    assert_eq!(
        graded,
        crate::theme::THEMES.len(),
        "every world must be graded"
    );
    crate::theme::set_active(crate::theme::DEFAULT_THEME);
}

/// CLAIM 2 + 3(a) — VISIBLE INSIDE, INVISIBLE OUTSIDE, IN EVERY WORLD.
///
/// The roster is the axis that matters most here: the three failure modes item
/// 116b named are all world-dependent. An OPAQUE card hides the region outright
/// (Tawny), a TRANSLUCENT one shows a muddled ghost (Mangrove), and a
/// BLUR-ELIGIBLE one frosts the whole document layer into the frame around the
/// workspace (every world whose `Backdrop` is not `Flat`). One differential
/// catches all three, and it catches them per world rather than on whichever
/// world a fixture happened to pick.
#[test]
fn the_relocated_document_is_visible_inside_the_region_and_nowhere_else_in_every_world() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping the_relocated_document_is_visible_inside_the_region: no adapter");
        return;
    };
    let mut graded = 0usize;
    let mut blur_worlds = 0usize;
    for world in crate::theme::THEMES {
        crate::theme::set_active_by_name(world.name).expect("a roster world");
        p.sync_theme();
        let pair = render_pair(&mut p, &device, &queue, 1200, 800, 1.0, false)
            .unwrap_or_else(|| panic!("{}: the document must be relocated", world.name));
        // Read AFTER the pair, so the gate sees the frame it actually graded.
        if p.backdrop_blur() {
            blur_worlds += 1;
        }
        assert!(
            assert_contained_and_visible(&pair, world.name),
            "{}: the roster cell must grade the VISIBILITY half — 1200x800 at zoom 1 is not \
             a degenerate geometry",
            world.name
        );
        graded += 1;
    }
    assert_eq!(
        graded,
        crate::theme::THEMES.len(),
        "every world must be graded"
    );
    // NON-VACUITY on the axis that hides: the blur arm only means something if
    // the roster actually contains blur-eligible worlds, and a `Backdrop::Flat`
    // roster would make the frosted-ghost half of this law untestable in silence.
    assert!(
        blur_worlds >= 2,
        "the frosted-backdrop arm must actually see blur-eligible worlds, got {blur_worlds}"
    );
    crate::theme::set_active(crate::theme::DEFAULT_THEME);
}

/// CLAIM 3(b) — THE GEOMETRY RANGE.
///
/// The region's box is derived from the canvas (`workspace_margin` is a fraction
/// of the smaller dimension), from the zoom (through the row pitch and the header
/// beat) and from the DPI. So a composition that happens to contain itself at
/// 1200x800 zoom 1.0 at 1x says nothing about the narrow stage, a deep zoom, or a
/// retina panel — and the narrow stage is a genuinely different composition, since
/// `workspace_is_wide` is false there and the content pane takes the whole
/// interior.
#[test]
fn the_composited_comparison_holds_across_canvas_zoom_and_dpi() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!(
            "skipping the_composited_comparison_holds_across_canvas_zoom_and_dpi: no adapter"
        );
        return;
    };
    let mut graded = 0usize;
    let mut narrow_cells = 0usize;
    let mut visible_cells = 0usize;
    for (cw, ch) in [(1200u32, 800u32), (1600, 1000), (760, 620), (900, 1400)] {
        for zoom in [0.7f32, 1.0, 1.9] {
            for dpi in [1.0f32, 2.0] {
                p.set_dpi(dpi);
                p.set_size(cw as f32, ch as f32);
                // The WIDE stage draws both regions, so the comparison is there
                // whichever region has focus; the NARROW stage draws only the
                // focused one. Try the resting composition first and fall back to
                // the staged one, which is the narrow cell by construction.
                let (pair, narrow) = match render_pair(&mut p, &device, &queue, cw, ch, zoom, false)
                {
                    Some(pair) => (pair, false),
                    None => match render_pair(&mut p, &device, &queue, cw, ch, zoom, true) {
                        Some(pair) => (pair, true),
                        None => continue,
                    },
                };
                if narrow {
                    narrow_cells += 1;
                }
                if assert_contained_and_visible(&pair, &format!("{cw}x{ch} zoom={zoom} dpi={dpi}"))
                {
                    visible_cells += 1;
                }
                graded += 1;
            }
        }
    }
    p.set_dpi(1.0);
    p.set_size(1200.0, 800.0);
    assert!(
        graded >= 20,
        "the sweep must grade real cells, got {graded}"
    );
    assert!(
        visible_cells >= 14,
        "most cells must be able to grade the VISIBILITY half too, got {visible_cells} of \
         {graded} — if this collapses, the sweep has quietly become a containment-only law"
    );
    // NON-VACUITY: the NARROW stage is a different composition (no rail, the pane
    // takes the whole interior), and a sweep that never reached it would be
    // grading one arm twenty times.
    assert!(
        narrow_cells > 0,
        "the sweep must reach the NARROW stage, where the content pane takes the whole \
         workspace interior — got {narrow_cells} narrow cells"
    );
}

/// The GLYPH CLIP's own unit-level law: off a comparison it is the identity, and
/// on one it is an intersection. Cheap, device-free, and it fails on the exact
/// reversion (dropping the X arm) that the pixel laws would only catch on a world
/// whose transcript happened to run wide.
#[test]
fn the_glyph_clip_is_the_identity_off_a_comparison_and_an_intersection_on_one() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping the_glyph_clip_is_the_identity_off_a_comparison: no adapter");
        return;
    };
    let full = glyphon::TextBounds {
        left: 0,
        top: 0,
        right: 1200,
        bottom: 800,
    };
    p.set_view(&view("ordinary prose\n", 0, 0));
    p.prepare(&device, &queue, 1200, 800).unwrap();
    assert!(p.comparison_viewport().is_none(), "precondition");
    assert_eq!(
        p.clip_text_bounds(full),
        full,
        "off a comparison the glyph clip must be the IDENTITY — every ordinary frame in \
         the tree uploads its own bounds unchanged"
    );

    p.set_view(&comparison_view(&transcript(ARM_A), 0, 0));
    p.prepare(&device, &queue, 1200, 800).unwrap();
    let [x, y, w, h] = p.comparison_viewport().expect("relocated");
    let clipped = p.clip_text_bounds(full);
    assert!(
        clipped.left >= x as i32 && clipped.right <= (x + w).ceil() as i32,
        "the glyph clip's X arm must be the region ({x}..{}), got {clipped:?} — without it a \
         glyph that outran the region would land on the workspace card it is drawn AFTER",
        x + w
    );
    assert!(
        clipped.top >= y as i32 && clipped.bottom <= (y + h).ceil() as i32,
        "the glyph clip's Y arm must be the region ({y}..{}), got {clipped:?}",
        y + h
    );
    // A caller's own TIGHTER bounds must survive: this intersects, it does not
    // replace — the table grid's per-column clip is narrower than the region and
    // must stay narrower.
    let tight = glyphon::TextBounds {
        left: x as i32 + 10,
        top: y as i32 + 10,
        right: (x + w) as i32 - 10,
        bottom: (y + h) as i32 - 10,
    };
    assert_eq!(
        p.clip_text_bounds(tight),
        tight,
        "the glyph clip must INTERSECT, never replace — a caller whose own bounds are \
         already inside the region keeps them"
    );
}
