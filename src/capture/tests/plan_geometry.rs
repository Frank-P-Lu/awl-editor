//! **`overlay.window.band` / `overlay.window.rows` REACH THE ARTEFACT** — the
//! serializer's own half of the three-way agreement law.
//!
//! `render/tests/overlay_plan_law.rs` grades the published geometry against the
//! ink and against the pointer, but it grades the REPORT STRUCT. A serializer
//! sitting between that struct and the JSON can still drop a key, reorder a
//! rect, or — the failure this file exists for — quietly rescale a number.
//!
//! **EVERY CAPTURE RUNS AT `--capture-dpi 1`, the one scale at which a
//! device-pixel bug looks correct.** So the law here is the DPI RELATION: the
//! same picker, same logical window, captured at dpi 1 and dpi 2, must report
//! row geometry that doubles. A serializer that divided by the scale factor, or a
//! report that had been quietly converted to logical units, passes every
//! single-scale check and fails this one. The 1x figures are additionally
//! anchored against the canvas so "doubles" cannot be satisfied by two equally
//! wrong numbers.
//!
//! ⚠️ **AND THE CHECK ITSELF RAN IN ONE CONFIGURATION, WHICH WAS ITS OWN
//! UNTESTED HYPOTHESIS.** The law swept DPI at the capture door's DEFAULT ZOOM
//! of 1.0 — and 1.0 is precisely the value at which the summoned card's width
//! cap cannot be caught misbehaving. The cap resolves GROW-ONLY, and while its
//! floor was a bare `1.0` rather than the display's own ratio, `zoom * dpi`
//! cleared that floor at both densities whenever `zoom >= 1` and the cap doubled
//! correctly; below it the floor bound at 1x and not at 2x, so the SAME logical
//! window drew a 545-logical-px card for a 1x reader and a 436-logical-px one
//! for a 2x reader — at an explicit below-default zoom. Document wrap is invariant
//! under this trade and was swept; the CARD was the axis nobody swept. The zoom
//! sweep is therefore part of the law, not a second law beside it.
//!
//! **The sidecar arm is paired with a DRAWN arm** for the reason the tripwire
//! names: a DPI-invariance claim is satisfiable by a card that is never painted.
//! The card's own extent is read out of the PNG differentially — the columns that
//! change when the picker is summoned over the same canvas — which is a rendered
//! pixel against another rendered pixel, carries a presence floor set under the
//! roster's narrowest real card, and is tied back to the reported band so the two
//! arms grade one object.

use super::super::*;
use super::adapter_available;
use crate::buffer::Buffer;
use crate::overlay::{OverlayKind, OverlayState};
use crate::testscratch::ScratchDir;

/// A real flat picker, driven through the production `OverlayState` so the fold
/// carries exactly what a live summon would. Shared with the sibling accessory
/// law, which grades the LANE keys of the same block rather than a second copy of
/// this fold.
pub(super) fn flat_picker_opts(ov: &OverlayState, canvas: (u32, u32), dpi: f32) -> CaptureOpts {
    let mut opts = CaptureOpts {
        canvas: Some(canvas),
        dpi: Some(dpi),
        ..CaptureOpts::default()
    };
    opts.overlay = Some(OverlayInfo {
        align: crate::render::effective_card_anchor(),
        active: true,
        mode: ov.kind.as_str(),
        title: ov.kind.title(),
        query: ov.query.text().to_string(),
        items: ov.item_strings(),
        bindings: ov.item_bindings(),
        ranges: ov.item_range_fracs(),
        git: ov.item_git_tags(),
        selected_index: ov.selected,
        hint: ov.foot_hint(),
        browse_dir: ov.browse_dir.clone(),
        return_to: None,
        spell_target: None,
        context_anchor: None,
        capture: None,
        notice: String::new(),
        lens: ov.active_facet_id(),
        lens_strip: ov.lens_strip(),
        sections: ov.item_sections(),
        preview_id: None,
        preview_view: None,
        // ASKED OF THE KIND, never hardcoded: a kind whose `workspace_shape`
        // answers unconditionally is ALWAYS a workspace in the real product, so
        // pinning this to `false` would fold a state no summon can reach. Every
        // card-shaped kind still answers `None` and folds exactly as before.
        workspace: ov.kind.workspace_shape().is_some(),
        detail_focus: false,
        diff_scroll: 0,
        empty: ov.empty_notice(),
        show_hidden: false,
    });
    opts
}

/// One `overlay.window.rows[]` entry, read back out of the JSON rather than out
/// of the report struct — so a serializer that dropped or renamed a key fails
/// here at the `expect` rather than passing on a defaulted zero.
struct Row {
    display: u64,
    item: Option<u64>,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

struct Band {
    first_top: f64,
    pitch: f64,
    footer_top: f64,
    x: f64,
    w: f64,
    rows: Vec<Row>,
    sel_row: u64,
    canvas_h: f64,
}

fn read_band(png: &std::path::Path) -> Band {
    let text = std::fs::read_to_string(png.with_extension("json")).expect("sidecar exists");
    let v: serde_json::Value = serde_json::from_str(&text).expect("sidecar parses");
    let w = &v["overlay"]["window"];
    assert!(!w.is_null(), "an open picker must report a window");
    let b = &w["band"];
    assert!(
        b.is_object(),
        "schema /201: an open picker's window must carry a `band` object, got {b}"
    );
    let rows = w["rows"]
        .as_array()
        .expect("schema /201: `rows` is an array")
        .iter()
        .map(|r| Row {
            display: r["display"].as_u64().expect("display"),
            item: r["item"].as_u64(),
            x: r["x"].as_f64().expect("x"),
            y: r["y"].as_f64().expect("y"),
            w: r["w"].as_f64().expect("w"),
            h: r["h"].as_f64().expect("h"),
        })
        .collect();
    Band {
        first_top: b["first_top"].as_f64().expect("first_top"),
        pitch: b["pitch"].as_f64().expect("pitch"),
        footer_top: b["footer_top"].as_f64().expect("footer_top"),
        x: b["x"].as_f64().expect("band x"),
        w: b["w"].as_f64().expect("band w"),
        rows,
        sel_row: w["sel_row"].as_u64().expect("sel_row"),
        canvas_h: w["canvas_h"].as_f64().expect("canvas_h"),
    }
}

/// ONE SCALE'S BLOCK, on its own terms: contiguous slots at the reported pitch,
/// seated at the reported origin, meeting the content band, ending where the
/// footer begins, and `sel_row` names one of them. Checked at BOTH
/// scales, because a relation that holds between two scales says nothing about
/// whether either one describes a real card.
fn assert_internally_consistent(name: &str, g: &Band) {
    for (i, row) in g.rows.iter().enumerate() {
        let (x, y, w, h) = (row.x, row.y, row.w, row.h);
        assert_eq!(
            row.display as usize, i,
            "{name}: rows must be in draw order"
        );
        assert!(
            (y - (g.first_top + i as f64 * g.pitch)).abs() < 0.01,
            "{name}: row {i} is at y {y}, not first_top {} + {i} * pitch {}",
            g.first_top,
            g.pitch
        );
        assert!(
            (h - g.pitch).abs() < 0.01,
            "{name}: row {i} is {h} tall against a pitch of {}",
            g.pitch
        );
        // Overlap, not containment: a staggering composition's selected row steps
        // OUTWARD past the band edge on purpose (the shipped Saltpan Settings card
        // does exactly this), so containment would be a false law.
        assert!(
            x + w > g.x && x < g.x + g.w && w > 1.0 && h > 1.0,
            "{name}: row {i} spans [{x}, {}] ({w}x{h}), which does not meet \
             the band [{}, {}]",
            x + w,
            g.x,
            g.x + g.w
        );
    }
    // `sel_row` is asserted to be IN RANGE rather than matched against a per-row
    // flag: the rows carry no selection, on purpose. The one published selection
    // comes from the owner that also colours the band, and a second answer here
    // could only be the plan's logical row -- a different fact that disagrees with
    // the drawn one throughout every selection move.
    assert!(
        (g.sel_row as usize) < g.rows.len(),
        "{name}: sel_row {} is outside the {} published rows",
        g.sel_row,
        g.rows.len()
    );
    let last = g.rows.last().expect("rows");
    assert!(
        g.footer_top >= last.y + last.h - 0.01,
        "{name}: footer_top {} must be at or below the last row's bottom {}",
        g.footer_top,
        last.y + last.h
    );
}

/// The zooms the DPI trade is swept at. The capture door's own default is 1.0,
/// and that is the ONE value at which a cap floored at a bare `1.0` cannot be
/// caught: there `zoom * dpi` is exactly `dpi`, so the floor never binds at
/// either density and the cap doubles for the wrong reason. So the sweep also
/// carries the app's real shipped type size — where a 1x reader gets the whole
/// authored cap and a 2x reader, on the same logical window, would not — and one
/// value above 1, where the floor is out of the way entirely.
const SWEPT_ZOOMS: &[f32] = &[0.8, crate::range::ZOOM.default, 1.6];

/// The picker fold at a stated canvas, density and zoom.
fn picker_at(ov: &OverlayState, canvas: (u32, u32), dpi: f32, zoom: f32) -> CaptureOpts {
    let mut opts = flat_picker_opts(ov, canvas, dpi);
    opts.zoom = Some(zoom);
    opts
}

/// The SAME frame with no card summoned — the reference the drawn arm differences
/// against. Every field but `overlay` matches [`picker_at`], so the pixels that
/// change between the two are the card and only the card.
fn bare_at(canvas: (u32, u32), dpi: f32, zoom: f32) -> CaptureOpts {
    CaptureOpts {
        canvas: Some(canvas),
        dpi: Some(dpi),
        zoom: Some(zoom),
        ..CaptureOpts::default()
    }
}

/// The card's DRAWN horizontal extent (device px, inclusive) across the reported
/// row band: the LONGEST CONTIGUOUS RUN of columns at which the summoned frame
/// differs from the bare one. `None` when nothing changed anywhere in the band,
/// which is the answer that makes the presence floor fail rather than pass
/// quietly.
///
/// **A run, not the outer min/max, and the difference is measured.** Summoning a
/// card also settles the MARGIN chrome either side of the page — a few columns at
/// each canvas edge change too — so `min`/`max` over every changed column reports
/// nearly the whole canvas (`[4, 1192]` of 1200) and would be grading the margins.
/// The card is the one solid block in that profile, so the longest run is the
/// card by construction, and the measurement never consults the reported band to
/// find it (which would make the tie-back below circular).
///
/// The threshold is a channel-sum against the OTHER RENDER of the same pixel, so
/// no authored colour appears here and a host whose ramp resolves a shade off
/// (this tree's Metal vs CI's lavapipe) moves both sides together.
fn drawn_card_span(
    bare: &std::path::Path,
    card: &std::path::Path,
    band: &Band,
) -> Option<(f64, f64)> {
    let a = image::open(bare).expect("decode the bare frame").to_rgba8();
    let b = image::open(card)
        .expect("decode the carded frame")
        .to_rgba8();
    assert_eq!(
        a.dimensions(),
        b.dimensions(),
        "the two frames must be one canvas for a differential read"
    );
    let (w, h) = a.dimensions();
    let y0 = band.first_top.max(0.0) as u32;
    let y1 = (band.footer_top as u32).min(h);
    let mut changed = vec![false; w as usize];
    for y in y0..y1 {
        for x in 0..w {
            let (p, q) = (a.get_pixel(x, y), b.get_pixel(x, y));
            let d: i32 = (0..3).map(|c| (p[c] as i32 - q[c] as i32).abs()).sum();
            if d > 40 {
                changed[x as usize] = true;
            }
        }
    }
    let (mut best, mut run_start) = (None::<(u32, u32)>, None::<u32>);
    for x in 0..=w {
        let on = (x < w) && changed[x as usize];
        match (on, run_start) {
            (true, None) => run_start = Some(x),
            (false, Some(s)) => {
                let run = (s, x - 1);
                if best.is_none_or(|(bs, be)| run.1 - run.0 > be - bs) {
                    best = Some(run);
                }
                run_start = None;
            }
            _ => {}
        }
    }
    best.map(|(s, e)| (s as f64, e as f64))
}

#[test]
fn published_row_geometry_is_physical_pixels_and_scales_with_capture_dpi() {
    if !adapter_available() {
        eprintln!(
            "skipping published_row_geometry_is_physical_pixels_and_scales_with_capture_dpi: \
             no wgpu adapter"
        );
        return;
    }
    let _tg = crate::testlock::serial();
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl_plan_geometry_{}", std::process::id())),
    );
    let buf = Buffer::from_str("a document behind the card\n");
    let items: Vec<String> = (0..30).map(|i| format!("candidate {i:02}")).collect();
    let mut ov = OverlayState::new(OverlayKind::Goto, items, vec![], vec![]);
    ov.move_sel(3);

    for &zoom in SWEPT_ZOOMS {
        assert_dpi_trade_holds_at(&dir, &buf, &ov, zoom);
    }
}

/// ONE zoom's cell of the sweep: the same logical window at both densities,
/// graded through the sidecar and then again through the frame.
fn assert_dpi_trade_holds_at(dir: &ScratchDir, buf: &Buffer, ov: &OverlayState, zoom: f32) {
    {
        // ONE logical window at two scales: 1200x800 at dpi 1 and 2400x1600 at dpi 2
        // are the same (W/N)x(H/N) logical window, so every difference below is the
        // scale factor and nothing else.
        let one = dir.join(format!("dpi1_z{zoom}.png"));
        capture_with(&one, buf, &picker_at(ov, (1200, 800), 1.0, zoom)).expect("dpi 1 capture");
        let two = dir.join(format!("dpi2_z{zoom}.png"));
        capture_with(&two, buf, &picker_at(ov, (2400, 1600), 2.0, zoom)).expect("dpi 2 capture");
        let (a, b) = (read_band(&one), read_band(&two));

        // ANCHORS at 1x, so "doubles" cannot be satisfied by two equally wrong
        // numbers: the band is a real fraction of a real canvas, and it carries rows.
        assert!(
            !a.rows.is_empty() && a.rows.len() == b.rows.len(),
            "zoom {zoom}: both scales must publish the same non-empty row band, got {} and {}",
            a.rows.len(),
            b.rows.len()
        );
        assert!(
            a.pitch > 8.0 && a.first_top > 0.0 && a.first_top < a.canvas_h,
            "zoom {zoom}: the 1x band must sit inside its own canvas: \
             first_top {} pitch {} canvas_h {}",
            a.first_top,
            a.pitch,
            a.canvas_h
        );
        assert!(
            a.w > 100.0 && a.x > 0.0,
            "zoom {zoom}: the 1x content band must be a real width at a real x: x {} w {}",
            a.x,
            a.w
        );

        assert_internally_consistent(&format!("1x z{zoom}"), &a);
        assert_internally_consistent(&format!("2x z{zoom}"), &b);

        // THE DPI RELATION. Physical pixels, so every figure doubles.
        //
        // Two tolerances, because two families arrive here. The HORIZONTAL
        // quantities are pure multiplications and double to within float noise —
        // this is the family the card's cap lives in, and it is held to one
        // device pixel exactly as the law always held it. The VERTICAL ones ride
        // the query BEAT, which is deliberately `.round()`ed onto the device grid
        // (`overlay_header_gap`) so the slab of air under the query line lands on
        // a whole pixel, and `2*round(v)` differs from `round(2v)` by up to one
        // device px by construction. Measured, that residual is exactly 1.0 on
        // the whole y family at zoom 0.8 and 0.0 at zoom 1.0 and 1.6, so `GRID`
        // is that one rounding unit and nothing more: a second, unrounded term
        // drifting into the vertical chain still fails here.
        const PROPORTIONAL: f64 = 1.0;
        const GRID: f64 = 1.5;
        let doubles_within = |lo: f64, hi: f64, what: &str, tol: f64| {
            assert!(
                (hi - lo * 2.0).abs() <= tol,
                "at zoom {zoom}, {what} is {lo} at --capture-dpi 1 and {hi} at 2 \
                 (tolerance {tol}): a PHYSICAL-pixel field must double with the scale \
                 factor (a logical one would not move, and a double-scaled one would \
                 quadruple). The CARD's own x and w are swept at more than one zoom \
                 because a width cap resolved GROW-ONLY against a bare 1.0 floor \
                 doubles at zoom >= 1 and does NOT below it — which makes the summoned \
                 card's composition a property of the reader's DISPLAY, and which hides \
                 completely at the capture door's own default zoom of 1.0"
            );
        };
        doubles_within(a.first_top, b.first_top, "band.first_top", GRID);
        doubles_within(a.pitch, b.pitch, "band.pitch", PROPORTIONAL);
        doubles_within(a.footer_top, b.footer_top, "band.footer_top", GRID);
        doubles_within(a.x, b.x, "band.x", PROPORTIONAL);
        doubles_within(a.w, b.w, "band.w", PROPORTIONAL);
        for (i, (lo, hi)) in a.rows.iter().zip(b.rows.iter()).enumerate() {
            assert_eq!(
                (lo.display, lo.item),
                (hi.display, hi.item),
                "zoom {zoom} row {i}: display / item must not depend on the capture scale"
            );
            doubles_within(lo.x, hi.x, &format!("rows[{i}].x"), PROPORTIONAL);
            doubles_within(lo.y, hi.y, &format!("rows[{i}].y"), GRID);
            doubles_within(lo.w, hi.w, &format!("rows[{i}].w"), PROPORTIONAL);
            doubles_within(lo.h, hi.h, &format!("rows[{i}].h"), PROPORTIONAL);
        }

        // THE DRAWN ARM. Everything above is the sidecar, which is a state oracle:
        // it reported `selected_index: 2` on a row that rendered fully invisible
        // once, and an invariance law over a card that is never drawn is satisfied
        // by the emptiest possible frame. So the same claim is made again over
        // PIXELS, and the card's extent is measured DIFFERENTIALLY — the columns
        // that CHANGE when the same canvas is captured with the picker summoned —
        // which compares a rendered pixel to another rendered pixel on the same
        // frame, needs no authored colour, and reports nothing at all if no card
        // was painted.
        let bare_one = dir.join(format!("bare1_z{zoom}.png"));
        capture_with(&bare_one, buf, &bare_at((1200, 800), 1.0, zoom)).expect("dpi 1 bare");
        let bare_two = dir.join(format!("bare2_z{zoom}.png"));
        capture_with(&bare_two, buf, &bare_at((2400, 1600), 2.0, zoom)).expect("dpi 2 bare");
        let span_one = drawn_card_span(&bare_one, &one, &a);
        let span_two = drawn_card_span(&bare_two, &two, &b);

        // PRESENCE, set under the roster's own tightest real card. The narrowest
        // card any shipping world draws at zoom 0.8 is Kite's
        // content-hugged one at ~216 LOGICAL px (Cassowary's is ~275; every
        // cap-bound world's is 545), so a floor of 150 logical px is under the
        // roster and still far above the noise a bare frame could produce.
        for (name, span, dpi) in [("1x", span_one, 1.0_f64), ("2x", span_two, 2.0)] {
            let (lo, hi) = span.unwrap_or_else(|| {
                panic!(
                    "zoom {zoom} {name}: summoning the picker changed NO pixels across \
                     the reported row band — the card was not drawn at all, and every \
                     invariance claim above is vacuous on this frame"
                )
            });
            assert!(
                (hi - lo + 1.0) / dpi >= 150.0,
                "zoom {zoom} {name}: the card's DRAWN span is {} device px ({} logical) \
                 across the reported row band — under the 150 logical px floor set \
                 beneath the roster's own narrowest real card",
                hi - lo + 1.0,
                (hi - lo + 1.0) / dpi
            );
        }

        // …and the drawn span is the one the sidecar reported, so the two arms are
        // grading one card rather than two unrelated numbers that happen to agree.
        let (lo1, hi1) = span_one.expect("1x span");
        let (lo2, hi2) = span_two.expect("2x span");
        assert!(
            (lo1 - a.x).abs() <= 4.0 && (hi1 - (a.x + a.w)).abs() <= 4.0,
            "zoom {zoom}: the 1x card is DRAWN across [{lo1}, {hi1}] while the sidecar \
             reports the band at [{}, {}] — the geometry oracle and the frame disagree",
            a.x,
            a.x + a.w
        );

        // THE APPEARANCE-SIDE TRADE, in pixels: the same logical window at twice
        // the density must paint a card of the same LOGICAL width. `<= 3.0` is one
        // logical pixel of rim either side, not a slack budget.
        let (w1, w2) = (hi1 - lo1 + 1.0, (hi2 - lo2 + 1.0) / 2.0);
        assert!(
            (w2 - w1).abs() <= 3.0,
            "zoom {zoom}: the card is DRAWN {w1} logical px wide at dpi 1 and {w2} at \
             dpi 2 over the SAME logical window. A reader on a denser panel is being \
             shown a different composition — the elision budget, and so how much of a \
             row's name is legible, would be a property of the display"
        );
    }
}
