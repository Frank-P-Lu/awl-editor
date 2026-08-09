//! THE WHOLE CLUSTER MIRRORS, NOT JUST THE RAIL.
//!
//! A diagonal row is `name + gap + accessory`, and the composition's anchor is
//! the SPINE: the name hangs on the spine end of its cluster and the accessory
//! on the outer end, each growing back toward the other. Mangrove's descending
//! `\` reads that outward to the right; Magpie's ascending `/` is its mirror
//! image and must read it outward to the left.
//!
//! It did not. The cluster BOX mirrored — it moved to the left of the spine —
//! while the two columns inside it kept the descending world's anchoring, which
//! is a TRANSLATION rather than a reflection: Magpie's names were left-aligned
//! at the far edge with their ragged ends facing the spine, and the chords sat
//! against the spine where the names belonged. A short name therefore floated at
//! the card's outer edge, the whole card's width away from the row's own spine
//! attachment, and the ragged edge landed on the one line the composition is
//! built around.
//!
//! The mirror is one signed answer — `ColumnFlow`, taken once in
//! `diagonal::label_flow_of` and `mirrored()` for the accessory — so label, gap
//! and accessory move together and cannot half-mirror. These laws grade the
//! outcome rather than that arithmetic:
//!
//!   * the NAME's spine-side ink edge sits on the cluster's spine end on BOTH
//!     worlds, at REAL PIXELS, whatever the name's length;
//!   * the accessory hangs on the outer end and never crosses the name;
//!   * and every drawn name is clickable across its own drawn ink — the claim
//!     the mirror most owes, since it moved where a row's ink is.
//!
//! ⚠️ THE AXIS THAT MATTERS IS THE NAME'S LENGTH. A roster of similar-width rows
//! satisfies every claim below under the pre-mirror composition too, because the
//! slack that was left beside a short name is exactly what varies with it. The
//! fixture is built to hold a three-character name and a ninety-character one in
//! the same picker, and the laws REFUSE to pass unless that spread survived
//! elision into the drawn pixels.

use super::super::*;
use super::{headless_dqp, pixeldiff, view};
use crate::render::rowlayout::ColumnFlow;

/// Canvases, because a cluster is measured against its card and one width is one
/// hypothesis: a roomy card, a taller one, and a cramped one whose budget bites.
const CANVASES: [(u32, u32); 3] = [(1200, 800), (1400, 900), (1040, 760)];

const WORLDS: [&str; 2] = ["Mangrove", "Magpie"];

/// A picker whose names span the whole plausible range in ONE visible window —
/// the axis the pre-mirror composition hid behind. Half the rows carry a chord,
/// so the accessory column is real and the gap between the two columns is too.
fn ragged_view() -> ViewState {
    let mut v = view("hello\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = "go to";
    v.overlay_items = (0..24)
        .map(|i| match i % 4 {
            0 => format!("n{i}"),
            1 => format!("a-mid-length-note-{i}.md"),
            2 => format!("{}-{i}.md", "a-considerably-longer-note-name".repeat(2)),
            _ => format!("note-{i}.md"),
        })
        .collect();
    v.overlay_bindings = (0..24)
        .map(|i| {
            if i % 2 == 0 {
                "C-c C-o".into()
            } else {
                String::new()
            }
        })
        .collect();
    v.overlay_selected = 3;
    v.overlay_window_rows = 12;
    v
}

/// How far a shaped name's last ink may sit from the edge it is aligned to: a
/// glyph's own right side bearing, plus the two-pixel guard the scan keeps
/// between the name column and the selected row's chevron. Everything the mirror
/// exists to remove is an order of magnitude larger — the pre-mirror slack
/// beside a short name measures in the hundreds of pixels.
const EDGE_TOL: f32 = 10.0;

/// The name's SPINE-SIDE ink edge for display row `d`, read off the frame's own
/// pixels — never off the measurement the draw used, which would make this a
/// restatement of the arithmetic rather than a claim about what was drawn.
///
/// The scan covers the name's own territory only: from the accessory column's
/// inner edge to two pixels short of the cluster's spine end, so neither a chord
/// nor the selected row's chevron (which is drawn on the SPINE side of that end)
/// can stand in for a name.
fn name_edge_px(
    p: &TextPipeline,
    frame: &[[u8; 4]],
    w: u32,
    h: u32,
    d: usize,
    flow: ColumnFlow,
) -> Option<(f32, f32)> {
    let geom = p.overlay_geometry(w);
    let plan = p.overlay_row_plan(&geom);
    let row = plan.rows()[d];
    let probe = p.diagonal_cluster_probe().expect("a diagonal cluster");
    let anchor = probe.label_anchor(d);
    let (al, ar) = probe.accessory_span(d);
    let guard = 2.0;
    let (x0, x1) = match flow {
        // The name runs right from the spine end; the accessory ends the row.
        ColumnFlow::Rightward => (anchor + guard, al),
        // Mirrored: the accessory begins the row and the name ends at the spine.
        ColumnFlow::Leftward => (ar, anchor - guard),
    };
    if x1 - x0 < 4.0 {
        return None;
    }
    // The card's own ground, sampled inside this row's band but outside every
    // column — a diagonal world draws no row fill, so one sample is the ground.
    let bg = {
        let sx = ((x0 + x1) * 0.5) as i64;
        let sy = (row.top + row.height * 0.5) as i64;
        // A pixel in the row's blank gap: the midpoint between the two columns
        // is inside the cluster and outside both, by construction of the gap.
        frame[(sy * w as i64 + sx).clamp(0, frame.len() as i64 - 1) as usize]
    };
    let bands = pixeldiff::ink_column_bands(
        frame,
        w as i64,
        x0.max(0.0) as i64,
        (x1.min(w as f32)) as i64,
        (row.top.max(0.0)) as i64,
        (row.bottom().min(h as f32)) as i64,
        bg,
        24,
    );
    let inked: Vec<_> = bands.iter().filter(|b| b.ink).collect();
    let first = inked.first()?;
    let last = inked.last()?;
    // (spine-side edge, far edge) — mirrored with the flow.
    Some(match flow {
        ColumnFlow::Rightward => (first.x0 as f32, last.x1 as f32),
        ColumnFlow::Leftward => (last.x1 as f32, first.x0 as f32),
    })
}

/// THE HEADLINE: every name's spine-side edge lands on its cluster's spine end,
/// on both mirrors, at real pixels, and the name's LENGTH does not move it.
///
/// The pre-mirror composition put an ascending world's names at the far edge, so
/// this distance was the row's whole leftover slack — hundreds of pixels, and
/// different for every name. It is now a side bearing.
#[test]
fn every_diagonal_name_hangs_on_its_own_spine_end_however_long_the_name_is() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping the cluster-mirror pixel law: no wgpu adapter");
        return;
    };
    let mut graded = 0usize;
    let mut spread_seen = 0usize;
    for world in WORLDS {
        let _pin = theme::WorldPin::world(world).expect("both diagonal worlds ship");
        for (w, h) in CANVASES {
            p.set_size(w as f32, h as f32);
            p.sync_theme();
            p.set_view(&ragged_view());
            p.prepare(&device, &queue, w, h).unwrap();
            let frame = pixeldiff::render_frame(&mut p, &device, &queue, w, h);
            let probe = p.diagonal_cluster_probe().unwrap_or_else(|| {
                panic!("{world} @ {w}x{h}: a diagonal world measures a cluster")
            });
            let flow = probe.label_flow();
            let rows = p.overlay_row_plan(&p.overlay_geometry(w)).rows().len();
            let at = format!("{world} @ {w}x{h}");

            let mut widths: Vec<f32> = Vec::new();
            for d in 0..rows {
                let Some((near, far)) = name_edge_px(&p, &frame, w, h, d, flow) else {
                    continue;
                };
                let anchor = probe.label_anchor(d);
                // How far INTO the cluster the name's own ink begins.
                let off = (near - anchor) * flow.sign();
                assert!(
                    (0.0..=EDGE_TOL).contains(&off),
                    "{at}: display {d}'s name ends {off} px from its cluster's spine \
                     end (anchor {anchor}, drawn edge {near}) — the cluster mirrored \
                     its box but not the columns inside it",
                );
                // AND THE ONE OWNER AGREES WITH THE PIXELS: the origin the draw
                // asked the rail for is the left edge the frame actually inked.
                let ink_w = (far - near).abs();
                assert!(
                    (probe.label_origin(d, ink_w) - near.min(far)).abs() <= EDGE_TOL,
                    "{at}: display {d} inked {}..{} but the rail's own origin for a \
                     {ink_w} px name is {}",
                    near.min(far),
                    near.max(far),
                    probe.label_origin(d, ink_w),
                );
                widths.push(ink_w);
                graded += 1;
            }
            // NON-VACUITY: the drawn names really do differ in length in this
            // very window. Without it the law would pass on a roster of equal
            // rows no matter which end the names hung on.
            let lo = widths.iter().copied().fold(f32::INFINITY, f32::min);
            let hi = widths.iter().copied().fold(f32::NEG_INFINITY, f32::max);
            spread_seen += usize::from(hi - lo > 100.0);
        }
    }
    assert!(
        graded > 50,
        "the law must grade real drawn names, got {graded}"
    );
    assert!(
        spread_seen >= WORLDS.len(),
        "no swept cell drew names more than 100 px apart in width — the axis this \
         law exists for never varied, so it proves nothing ({spread_seen} cells)"
    );
}

/// THE ACCESSORY HANGS ON THE OTHER END, and the gap between the two columns is
/// on the inside. This is the half of "mirror the WHOLE cluster" that the name's
/// own alignment cannot witness: right-aligning the names alone would have run
/// them straight through the chord column that used to sit at the spine.
#[test]
fn the_accessory_column_hangs_on_the_cluster_end_the_name_does_not() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping the accessory-end law: no wgpu adapter");
        return;
    };
    let mut graded = 0usize;
    for world in WORLDS {
        let _pin = theme::WorldPin::world(world).expect("both diagonal worlds ship");
        for (w, h) in CANVASES {
            p.set_size(w as f32, h as f32);
            p.sync_theme();
            p.set_view(&ragged_view());
            p.prepare(&device, &queue, w, h).unwrap();
            let probe = p.diagonal_cluster_probe().expect("a diagonal cluster");
            let flow = probe.label_flow();
            let rows = p.overlay_row_plan(&p.overlay_geometry(w)).rows().len();
            let at = format!("{world} @ {w}x{h}");
            for d in 0..rows {
                let (al, ar) = probe.accessory_span(d);
                let outer = probe.accessory_anchor(d);
                let anchor = probe.label_anchor(d);
                // The accessory's own outer edge IS the cluster's outer end.
                assert!(
                    (outer - if flow.sign() > 0.0 { ar } else { al }).abs() < 0.01,
                    "{at}: display {d}'s accessory column ({al}..{ar}) does not hang on \
                     the cluster's outer end ({outer})"
                );
                // It grows back TOWARD the name, never past the spine end.
                assert!(
                    (anchor - al).min(ar - anchor).abs() >= 0.0
                        && (al.min(ar) >= anchor.min(outer) - 0.01)
                        && (ar.max(al) <= anchor.max(outer) + 0.01),
                    "{at}: display {d}'s accessory ({al}..{ar}) escapes its cluster \
                     ({anchor}..{outer})"
                );
                graded += 1;
            }
        }
    }
    assert!(graded > 50, "graded too little: {graded}");
}

/// DRAWN ↔ HIT-TEST, over the ink the mirror MOVED. A name that is drawn
/// right-aligned against an ascending spine must be clickable across exactly
/// that ink — both of its ends and its middle — and the row a pointer lands on
/// must be the row whose name it is standing on.
///
/// The pointer x's come from the PIXELS, not from the cluster arithmetic, so a
/// composition that drew its names somewhere its own planner did not expect
/// fails here rather than agreeing with itself.
#[test]
fn a_mirrored_name_is_clickable_across_exactly_the_ink_it_is_drawn_at() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping the mirrored hit-test law: no wgpu adapter");
        return;
    };
    let mut graded = 0usize;
    for world in WORLDS {
        let _pin = theme::WorldPin::world(world).expect("both diagonal worlds ship");
        for (w, h) in CANVASES {
            p.set_size(w as f32, h as f32);
            p.sync_theme();
            p.set_view(&ragged_view());
            p.prepare(&device, &queue, w, h).unwrap();
            let frame = pixeldiff::render_frame(&mut p, &device, &queue, w, h);
            let probe = p.diagonal_cluster_probe().expect("a diagonal cluster");
            let flow = probe.label_flow();
            let geom = p.overlay_geometry(w);
            let plan = p.overlay_row_plan(&geom);
            let at = format!("{world} @ {w}x{h}");
            for (d, row) in plan.rows().iter().enumerate() {
                let Some(item) = row.item else { continue };
                let Some((near, far)) = name_edge_px(&p, &frame, w, h, d, flow) else {
                    continue;
                };
                let y = row.top + row.height * 0.5;
                for (what, x) in [
                    ("its spine-side edge", near - 0.5 * flow.sign()),
                    ("its far edge", far + 0.5 * flow.sign()),
                    ("its middle", (near + far) * 0.5),
                ] {
                    assert_eq!(
                        p.overlay_row_at(x, y),
                        Some(item),
                        "{at}: display {d} draws its name across {near}..{far} but a \
                         pointer on {what} (x={x}, y={y}) does not land on it"
                    );
                    graded += 1;
                }
            }
        }
    }
    assert!(
        graded > 100,
        "the hit-test law must grade real drawn names, got {graded}"
    );
}
