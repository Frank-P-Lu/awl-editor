//! ITEM 247 — THE MARKER IS ONE ROTATABLE SYMBOL, and its turn is the thing
//! that carries meaning.
//!
//! Two surfaces draw this mark: the overlay's selected-row marker on a
//! `ListStyle::Diagonal` world (`chrome::diagonal::selected_chevron`) and the
//! fold chevron in the writing column's leading pad
//! (`layers::fold_chevron::prepare_fold_chevron_marks`). They have TWO entry
//! points and must have ONE shape, because the design that makes the mark worth
//! turning — a chevron is the simplest mark with no rotational symmetry, so its
//! angle is legible AT REST — is a property of the shape, not of either caller.
//!
//! ⚠️ ONLY THE FOLD CHEVRON TURNS. The selected-row marker is upright: it says
//! which row by standing beside it, and its DIRECTION is the mirror the whole
//! diagonal cluster carries, not a rotation. Its parameterization is still swept
//! by the revolution law below, because the shape it draws is still this owner's
//! and a symmetry that folded the mark onto itself would make the FOLD
//! chevron's turn unreadable at the marker's own proportions.
//!
//! Both laws below therefore grade the ANGLE and the POINTS, never the instance
//! count: the mark is exactly two segments at every turn, which is precisely
//! what a counting law cannot see.

use crate::render::layers::fold_chevron::fold_chevron_mark_metrics;
use crate::selection::chevron_arms;

const EPS: f32 = 1e-3;

/// A `spine_segment` triple's two endpoints, recovered from `(center, half,
/// axis)` — `half[0]` is the segment's own half-LENGTH, so `center ∓
/// axis*half[0]` are exactly the points it was built from.
fn ends(seg: ([f32; 2], [f32; 2], [f32; 2])) -> ([f32; 2], [f32; 2]) {
    let (center, half, axis) = seg;
    let (dx, dy) = (axis[0] * half[0], axis[1] * half[0]);
    (
        [center[0] - dx, center[1] - dy],
        [center[0] + dx, center[1] + dy],
    )
}

/// THE DRAWN INK, and nothing else: a chevron reduced to its two segments'
/// endpoints, each segment's own pair sorted and then the two segments sorted
/// against each other.
///
/// ⚠️ This canonicalization is the reason the symmetry law below can see what
/// it is named for. A representation that kept the VERTEX as a distinguished
/// point would report a plain BAR as asymmetric — a bar's two ends are its
/// vertex and its back, and swapping them is a different labelling of the
/// identical drawn quad. The shader draws a rotated rounded rect from a centre,
/// a half-extent and an axis; `axis` and `-axis` are the same rectangle. So the
/// shape is graded on the unordered ink, which is what a reader sees.
fn shape(arms: [([f32; 2], [f32; 2], [f32; 2]); 2]) -> [[f32; 2]; 4] {
    let seg = |s: ([f32; 2], [f32; 2], [f32; 2])| -> [[f32; 2]; 2] {
        let (a, b) = ends(s);
        if (a[0], a[1]) <= (b[0], b[1]) {
            [a, b]
        } else {
            [b, a]
        }
    };
    let (p, q) = (seg(arms[0]), seg(arms[1]));
    let (lo, hi) = if (p[0][0], p[0][1], p[1][0], p[1][1]) <= (q[0][0], q[0][1], q[1][0], q[1][1]) {
        (p, q)
    } else {
        (q, p)
    };
    [lo[0], lo[1], hi[0], hi[1]]
}

/// How far apart two chevrons read, in pixels — the largest displacement of any
/// canonical endpoint. `0.0` means the two draw the same ink.
fn shape_distance(x: [[f32; 2]; 4], y: [[f32; 2]; 4]) -> f32 {
    x.iter()
        .zip(y.iter())
        .map(|(p, q)| ((p[0] - q[0]).powi(2) + (p[1] - q[1]).powi(2)).sqrt())
        .fold(0.0_f32, f32::max)
}

/// The diagonal marker's own parameters expressed in the shared owner's terms.
/// `selected_chevron` takes a spine abscissa, an arm abscissa and the row's two
/// inset ends; `chevron_arms` takes a centre, a signed reach along the pointing
/// axis, a signed spread across it and a turn. The map between them is pure
/// arithmetic with no free constant: the centre is the midpoint of spine and
/// arm, the reach is half the SIGNED distance from arm to spine (so a
/// Descending world's rightward cluster and an Ascending world's leftward one
/// are the same expression with opposite sign, never a second branch), and the
/// spread is half the row's inset height.
fn marker_in_owner_terms(spine_x: f32, arm_x: f32, top: f32, bottom: f32) -> ([f32; 2], f32, f32) {
    (
        [(spine_x + arm_x) * 0.5, (top + bottom) * 0.5],
        (spine_x - arm_x) * 0.5,
        (top - bottom) * 0.5,
    )
}

/// THE CROSS-OWNER LAW — the overlay's selected-row marker is the SHARED
/// chevron owner at a derived parameterization, and nothing else.
///
/// This is the law that makes the mark's two entry points one shape in fact
/// rather than by intention. It binds `chrome::diagonal::selected_chevron`'s
/// output to `selection::chevron_arms`' output point for point, so a change to
/// either that is not a change to both goes red — which is the failure mode a
/// duplicated shape has always had, and the reason the fold chevron carried its
/// own copy of this arithmetic until it was merged into the one owner.
///
/// Swept across both reach SIGNS (the two worlds' mirrored clusters), row
/// heights from the degenerate floor upward, three row origins, three spine
/// abscissae and three stroke weights — because the identity must hold for
/// every row the planner can produce, not for one hand-picked geometry, and
/// because a mapping that quietly assumed a positive reach would pass a
/// one-sided sweep and fail on the mirrored world.
#[test]
fn the_diagonal_marker_is_the_shared_chevron_owner_at_a_derived_parameterization() {
    let mut cases = 0;
    for top in [0.0_f32, 137.5, 1024.0] {
        for height in [12.0_f32, 27.5, 44.0, 88.0] {
            for spine_x in [0.0_f32, 64.0, 933.25] {
                // BOTH signs: a Descending world reaches right, an Ascending
                // one left.
                for reach in [-40.0_f32, -10.0, -3.0, 3.0, 10.0, 40.0] {
                    for thickness in [1.0_f32, 3.0, 7.5] {
                        let (t, b) = (top + 2.0, top + height - 2.0);
                        let arm_x = spine_x + reach;
                        let drawn = crate::render::chrome::diagonal::selected_chevron(
                            spine_x, arm_x, t, b, thickness,
                        );
                        let (center, owner_reach, owner_spread) =
                            marker_in_owner_terms(spine_x, arm_x, t, b);
                        let owned = chevron_arms(center, owner_reach, owner_spread, 0.0, thickness);
                        let ctx = format!(
                            "top {top} height {height} spine_x {spine_x} reach {reach} \
                             thickness {thickness}"
                        );
                        let apart = shape_distance(shape(drawn), shape(owned));
                        assert!(
                            apart < EPS,
                            "{ctx}: the diagonal marker must BE the shared chevron owner's \
                             shape, not merely resemble it — {apart:.6}px apart; drawn {:?}, \
                             owner {:?}",
                            shape(drawn),
                            shape(owned)
                        );
                        // And on the drawn quad's other dimension too: one
                        // stroke weight in, one half-thickness out.
                        for (i, (d, o)) in drawn.iter().zip(owned.iter()).enumerate() {
                            assert!(
                                (d.1[1] - o.1[1]).abs() < EPS,
                                "{ctx}: arm {i} half-thickness differs: {} vs {}",
                                d.1[1],
                                o.1[1]
                            );
                        }
                        cases += 1;
                    }
                }
            }
        }
    }
    assert_eq!(cases, 648, "the sweep must not silently shrink");
}

/// THE PROPERTY THE WHOLE TURN RESTS ON — the mark has NO rotational symmetry,
/// so its angle is information at rest rather than only in flight.
///
/// This is the arithmetic that chose the chevron over a bar, a plus, a cross, a
/// diamond and an asterisk: every one of those maps onto itself under some
/// rotation, so half a turn of it is indistinguishable from no turn, and a
/// direction cue built on it would exist only while the animation played —
/// which Reduce Motion removes. A mark that reads the same at two different
/// turns cannot say which way the selection travelled.
///
/// Graded over a full revolution at one-degree resolution, at BOTH consumers'
/// REAL parameterizations rather than a pretty hand-picked pair: the fold
/// chevron's own `(reach, spread, thickness)` at char widths from a tiny face
/// to a large one, and the diagonal marker's derived `(reach, spread)` at real
/// row heights and both cluster signs. The nearest-approach distance is
/// asserted, not merely "some pair differs", so a shape that is ALMOST
/// symmetric — the direction of the degradation — is caught before it reads as
/// symmetric on screen.
///
/// ⚠️ SCOPE, stated plainly: this law grades ONE function, the shared owner.
/// It is the law above that makes that one function the whole story for the
/// diagonal marker as well.
#[test]
fn the_marker_reads_differently_at_every_turn_over_a_full_revolution() {
    let mut params: Vec<(f32, f32, f32, String)> = Vec::new();
    for char_width in [4.0_f32, 7.2, 11.5, 24.0] {
        let (reach, spread, thickness) = fold_chevron_mark_metrics(char_width);
        params.push((
            reach,
            spread,
            thickness,
            format!("fold chevron @ char_width {char_width}"),
        ));
    }
    for height in [12.0_f32, 27.5, 88.0] {
        for reach in [-40.0_f32, -3.0, 3.0, 40.0] {
            let (_, r, s) = marker_in_owner_terms(64.0, 64.0 + reach, 0.0, height);
            params.push((
                r,
                s,
                3.0,
                format!("diagonal marker @ height {height} reach {reach}"),
            ));
        }
    }
    assert_eq!(params.len(), 16, "the parameter roster must not shrink");

    let center = [200.0_f32, 120.0];
    for (reach, spread, thickness, what) in params {
        let shapes: Vec<[[f32; 2]; 4]> = (0..360)
            .map(|deg| shape(chevron_arms(center, reach, spread, deg as f32, thickness)))
            .collect();
        // The mark's own scale — the closest two DISTINCT turns may come and
        // still be honestly distinct is a fraction of the mark's own size, not
        // an absolute pixel count, so this holds for a 4px mark and a 24px one.
        let scale = reach.abs().max(spread.abs());
        let mut worst = f32::MAX;
        let mut worst_pair = (0usize, 0usize);
        for (i, a) in shapes.iter().enumerate() {
            for (j, b) in shapes.iter().enumerate().skip(i + 1) {
                let d = shape_distance(*a, *b);
                if d < worst {
                    worst = d;
                    worst_pair = (i, j);
                }
            }
        }
        // One degree apart on a mark of half-size `scale` moves a defining
        // point by ~`scale * 0.017`; anything at or below a tenth of that is a
        // shape that has folded onto itself somewhere in the revolution.
        let floor = scale * 0.0017;
        assert!(
            worst > floor,
            "{what}: the mark reads the SAME at {}° and {}° (closest approach \
             {worst:.6}px, floor {floor:.6}px) — a mark with rotational symmetry \
             cannot say which way the selection travelled, and its direction cue \
             would exist only while the animation played",
            worst_pair.0,
            worst_pair.1
        );
    }
}

/// THE NEW BATCH-CORNER CONSUMER IS INERT AT EVERY SHIPPED METRIC.
///
/// `prepare_fold_chevron_marks` narrows its one shared `set_corner` value
/// through `selection::narrowed_spine_corner_px` across every arm it built,
/// because the radius is a per-BATCH uniform while the arms are not all the
/// same size. At the shipped fractions an arm is always several times longer
/// than the stroke is thick, so the fold changes nothing — which is what makes
/// the change byte-identical rather than merely small. Asserted here across the
/// char-width range rather than argued, and paired with the case that DOES
/// bind, so the guard is not silently vacuous.
#[test]
fn the_fold_chevron_batch_corner_is_unchanged_at_every_shipped_char_width() {
    for char_width in [2.0_f32, 4.0, 7.2, 11.5, 24.0, 48.0] {
        let (reach, spread, thickness) = fold_chevron_mark_metrics(char_width);
        let arms = chevron_arms([0.0, 0.0], reach, spread, 37.0, thickness);
        let corner = arms.iter().fold(thickness * 0.5, |corner, (_, half, _)| {
            crate::selection::narrowed_spine_corner_px(corner, half[0], half[1])
        });
        assert!(
            (corner - thickness * 0.5).abs() < 1e-6,
            "char_width {char_width}: the narrowed batch corner must equal the \
             un-narrowed {} the mark shipped with, got {corner}",
            thickness * 0.5
        );
    }
    // The case the guard exists for: an arm SHORTER than its own stroke is
    // thick. Without the narrowing this over-rounds as it shortens.
    let stubby = chevron_arms([0.0, 0.0], 0.4, 0.3, 0.0, 6.0);
    let corner = stubby.iter().fold(3.0_f32, |corner, (_, half, _)| {
        crate::selection::narrowed_spine_corner_px(corner, half[0], half[1])
    });
    assert!(
        corner < 3.0,
        "a mark whose arms are shorter than its stroke is thick MUST narrow its \
         corner — got {corner}, the un-narrowed 3.0"
    );
}
