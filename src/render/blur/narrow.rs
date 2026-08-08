//! THE FROST'S BOX, NARROWED TO WHAT THE CARD ACTUALLY DREW — pure, device-free, and
//! the one place the shape's own un-sheared frame is arithmetic rather than a guess.
//!
//! The footprint's box began as the card's LAYOUT box, and on a composition that draws
//! no panel and no plate that box is a placement policy rather than a surface: a fixed
//! desired width clamped to the window, with no relation to how wide the shaped rows
//! turned out. A frost scoped to it treats air — measured on this tree, a cross-section
//! of 576 logical px over a row carrying at most 110 of ink, and the slack is ASYMMETRIC
//! because the rake redistributes it, so a symmetric trim would leave the larger half.
//!
//! THE NARROWING IS A CHANGE TO THE FROST ALONE. `overlay_card_rect` and the pointer
//! hit-region are untouched — the region a click means something in is still the box the
//! rows occupy, and it was already the established split that the frost's extent and the
//! hit region are separate quantities.
//!
//! # THE UN-SHEARED FRAME
//!
//! The shape is a parallelogram: a box sheared about its own vertical centre, so a pixel
//! `(px, py)` is inside iff `x ≤ px − shear·(py − cy) ≤ x + w`. That expression is the
//! whole content of this module. Ask it of a drawn surface's corners and you get the
//! box's own faces at their tightest for that surface; ask it of every surface and take
//! the extremes, and you get the narrowest parallelogram of that shear that contains
//! them all. The rows lean at exactly the shear, so they collapse onto one span here —
//! which is why the narrowed shape does not have to widen for the rake, only for the
//! chrome that does NOT rake.
//!
//! Only the HORIZONTAL faces move. The reported defect is a width, the card's height is
//! already derived from its row budget rather than from a cap, and the two axes decouple
//! exactly ([`super::extent::footprint_box`]'s own note: the pivot `cy` is a function of
//! `y` and `h` alone, so narrowing in x cannot move it).

/// THE HORIZONTAL SPAN A BOX OCCUPIES IN THE SHAPE'S UN-SHEARED FRAME — `[left, top,
/// right, bottom]` in, `(min, max)` out, about a shape whose vertical centre is `cy`.
///
/// The displacement `shear·(py − cy)` is linear in `py`, so its extremes over the box's
/// own row range are at the range's ends and no interior sampling is needed. Subtracting
/// the LARGER displacement gives the leftmost the box reaches; the smaller, the
/// rightmost.
///
/// A non-finite shear or box collapses to the box's own x span, which is the inert answer:
/// the caller then narrows to a rectangle rather than to nothing.
pub(super) fn unsheared_x_span(ltrb: [f32; 4], shear: f32, cy: f32) -> (f32, f32) {
    let [l, t, r, b] = ltrb;
    if !(shear.is_finite() && t.is_finite() && b.is_finite() && cy.is_finite()) {
        return (l, r);
    }
    let (d0, d1) = (shear * (t - cy), shear * (b - cy));
    (l - d0.max(d1), r - d0.min(d1))
}

/// THE CARD'S BOX, NARROWED IN X to the parallelogram of the same shear that contains
/// every one of `surfaces` (`[left, top, right, bottom]` canvas boxes) — `y` and `h`
/// unchanged.
///
/// IT ONLY EVER SHRINKS, and it is clamped to the card: a surface reaching outside the
/// card's own box does not widen the frost here (the coverage floor that widens the shape
/// for upright chrome is [`super::extent::footprint_box`], applied after this and asked of
/// the box a production owner declares). So the result is always inside the box this
/// frame would have frosted before, which is what makes every claim about the page
/// OUTSIDE the old footprint hold unchanged.
///
/// An EMPTY surface list returns the card untouched. That is the honest answer rather
/// than a collapse to nothing: a frame that reports no drawn surface is a frame this
/// module knows nothing about, and treating "I found none" as "there are none" is how a
/// frost comes to leave real chrome over sharp document.
pub(crate) fn footprint_narrow(card: [f32; 4], shear: f32, surfaces: &[[f32; 4]]) -> [f32; 4] {
    let [x, y, w, h] = card;
    if !(x.is_finite() && w.is_finite() && h.is_finite()) {
        return card;
    }
    let cy = y + h * 0.5;
    let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
    for s in surfaces {
        if !s.iter().all(|v| v.is_finite()) {
            continue;
        }
        let (a, b) = unsheared_x_span(*s, shear, cy);
        lo = lo.min(a);
        hi = hi.max(b);
    }
    if !(lo.is_finite() && hi > lo) {
        return card;
    }
    let x0 = lo.max(x);
    let x1 = hi.min(x + w);
    if x1 <= x0 {
        return card;
    }
    [x0, y, x1 - x0, h]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// AN UPRIGHT SHAPE'S UN-SHEARED FRAME IS THE CANVAS. The one cell where the
    /// arithmetic must be the identity, because a `Rules` world is enrolled at shear 0
    /// and any displacement there would move a shipped frost.
    #[test]
    fn an_upright_shape_leaves_every_span_where_it_found_it() {
        let _g = crate::testlock::serial();
        for cy in [0.0f32, 12.0, 900.0] {
            for b in [[10.0f32, 0.0, 40.0, 5.0], [-3.0, 800.0, 7.0, 812.0]] {
                assert_eq!(
                    unsheared_x_span(b, 0.0, cy),
                    (b[0], b[2]),
                    "{b:?} at cy {cy}"
                );
            }
        }
    }

    /// THE NARROWING NEVER GROWS THE BOX, at any shear and for any surface — including a
    /// surface entirely outside the card, which is the case that would silently turn a
    /// narrowing into a widening and break every byte-identity claim beyond the old
    /// footprint.
    #[test]
    fn the_narrowed_box_is_always_inside_the_card_it_narrows() {
        let _g = crate::testlock::serial();
        let card = [200.0f32, 100.0, 520.0, 450.0];
        for shear in [-0.4f32, -0.05, 0.0, 0.05, 0.4] {
            for s in [
                [-500.0f32, 0.0, 5000.0, 1000.0],
                [0.0, 0.0, 1.0, 1.0],
                [700.0, 500.0, 900.0, 560.0],
                [210.0, 110.0, 300.0, 130.0],
            ] {
                let [nx, ny, nw, nh] = footprint_narrow(card, shear, &[s]);
                assert!(
                    nx >= card[0] - 1e-3
                        && nx + nw <= card[0] + card[2] + 1e-3
                        && ny == card[1]
                        && nh == card[3],
                    "shear {shear} surface {s:?} narrowed to [{nx},{ny},{nw},{nh}] outside \
                     the card {card:?}"
                );
            }
        }
    }

    /// EVERY SURFACE STAYS INSIDE THE NARROWED PARALLELOGRAM — the containment the whole
    /// module exists to provide, asserted through the shipping mask's own mirror so it
    /// cannot drift onto a shape the shader stopped drawing.
    #[test]
    fn every_surface_is_inside_the_parallelogram_the_narrowing_returns() {
        let _g = crate::testlock::serial();
        let card = [200.0f32, 100.0, 520.0, 450.0];
        let surfaces = [
            [212.0f32, 110.0, 330.0, 132.0],
            [260.0, 300.0, 371.0, 322.0],
            [300.0, 500.0, 480.0, 528.0],
        ];
        // The claim is that narrowing never CUTS anything, so it is asked only of the
        // corners the un-narrowed shape already covered. A corner the card's own sheared
        // shape never reached was not frosted before either, and no shrink-only narrowing
        // can be blamed for it: at a steep enough rake the card's span at one row slides
        // clear of a surface sitting near the opposite face, which is a fact about the
        // card's placement rather than about this arithmetic. (`footprint_box`'s coverage
        // floor is the mechanism that widens the shape for chrome the rake leaves behind,
        // and it runs after this.) Demanding containment unconditionally asked for the
        // impossible in exactly one swept cell and passed everywhere else — so the
        // COVERED count below is the law's own non-vacuity clause: without it, a shape
        // that covered nothing would satisfy the assertion perfectly.
        let mut covered = 0usize;
        for shear in [-0.3f32, -0.02, 0.0, 0.02, 0.3] {
            let rect = footprint_narrow(card, shear, &surfaces);
            let foot = super::super::Footprint { rect, shear };
            let before = super::super::Footprint { rect: card, shear };
            for s in &surfaces {
                for (px, py) in [(s[0], s[1]), (s[2], s[1]), (s[0], s[3]), (s[2], s[3])] {
                    if super::super::extent::footprint_dist_outside(before, px, py) > 1e-3 {
                        continue;
                    }
                    covered += 1;
                    let d = super::super::extent::footprint_dist_outside(foot, px, py);
                    assert!(
                        d <= 1e-3,
                        "shear {shear}: surface corner ({px},{py}) was inside the card's own \
                         sheared shape {card:?} and sits {d} OUTSIDE the narrowed shape \
                         {rect:?} — the narrowing has cut a surface that WAS frosted"
                    );
                }
            }
        }
        assert!(
            covered >= 50,
            "only {covered} surface corners were inside the un-narrowed shape across the \
             swept shears, so this law graded almost nothing — check the fixture before \
             trusting a green run"
        );
    }

    /// AND THE NARROWING IS TIGHT, not merely safe: with the card's own corners as the
    /// only surfaces it gives the card back, and with one small surface it gives back
    /// something far narrower. A narrowing that returned the card always would satisfy
    /// the containment law above perfectly.
    #[test]
    fn the_narrowing_actually_narrows() {
        let _g = crate::testlock::serial();
        let card = [200.0f32, 100.0, 520.0, 450.0];
        let whole = [[200.0f32, 100.0, 720.0, 550.0]];
        assert_eq!(footprint_narrow(card, 0.0, &whole), card);
        let small = [[212.0f32, 110.0, 322.0, 132.0]];
        let [_, _, nw, _] = footprint_narrow(card, 0.0, &small);
        assert!(
            nw < card[2] * 0.3,
            "one 110-wide surface in a 520-wide card narrowed to {nw}"
        );
    }
}
