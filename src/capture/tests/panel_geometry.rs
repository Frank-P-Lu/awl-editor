//! **`search.panel` REACHES THE ARTEFACT, AND DESCRIBES THE CARD IN THE PNG** —
//! the serializer's half of the search panel's agreement law.
//!
//! `render/plan/tests/panel_law.rs` grades the published geometry against the
//! shaped ink and the pointer, but it grades the REPORT STRUCT. A serializer
//! between that struct and the JSON can still drop a key, reorder a band, or —
//! the failure this file exists for — quietly rescale a number. So the oracle
//! here is the **rendered card's own rim**: the float primitive draws a
//! one-pixel border, which is the strongest luminance step anywhere along a row
//! that crosses the card, and its two positions are the card's real left and
//! right edges. Nothing in that measurement reads the sidecar.
//!
//! ⚠️ **BOTH EDGES, NEVER ONE.** A rect pinned only by its origin accepts any
//! uniform scaling of its extent, at every scale — so the law locates the two
//! strongest steps on the row and requires them to be the published `x` and
//! `x + w`. Halving `w` puts the second one mid-card, where the fill is flat.
//!
//! ⚠️ **AND EVERY CAPTURE OTHERWISE RUNS AT `--capture-dpi 1`**, the one scale at
//! which a device-pixel figure and a logical one look identical. The same panel is
//! captured at dpi 1 and dpi 2 and the rim is re-measured there, so a report that
//! had been divided by the scale factor fails against the ink rather than against
//! a hand-written expectation.
//!
//! ⚠️ **WHAT DOUBLES IS MEASURED, NOT ASSUMED.** The row pitch and the `Aa` span
//! come off `metrics.line_height` and the shaped advances, and they double
//! exactly. The card's `y` does NOT: its outer margin and inner pad are unscaled
//! constants, so they are counted once at either scale and the residual is theirs.
//! The law therefore asserts the exact doubling only where the quantity is
//! metric-derived, and elsewhere asserts the direction — a figure that does not
//! grow at all is the logical-units bug this block exists to expose. It is written
//! to stay green if the pad is ever scaled, because that is a layout question, not
//! a reporting one.

use super::super::*;
use super::adapter_available;
use crate::buffer::Buffer;
use crate::testscratch::ScratchDir;

/// One `search.panel`, read back out of the JSON rather than out of the report
/// struct — so a serializer that dropped or renamed a key fails at the `expect`
/// instead of passing on a defaulted zero.
struct Panel {
    card: [f64; 4],
    text_left: f64,
    text_top: f64,
    rows: Vec<(u64, f64, f64)>,
    toggle: (f64, f64),
}

fn read_panel(png: &std::path::Path) -> Panel {
    let text = std::fs::read_to_string(png.with_extension("json")).expect("sidecar exists");
    let v: serde_json::Value = serde_json::from_str(&text).expect("sidecar parses");
    let p = &v["search"]["panel"];
    assert!(
        p.is_object(),
        "schema /203: an active search must publish a `search.panel` object, got {p}"
    );
    let c = &p["card"];
    let rows = p["rows"]
        .as_array()
        .expect("schema /203: `rows` is an array")
        .iter()
        .map(|r| {
            (
                r["row"].as_u64().expect("row"),
                r["top"].as_f64().expect("top"),
                r["h"].as_f64().expect("h"),
            )
        })
        .collect();
    let toggle = &p["case_toggle"];
    assert!(
        toggle.is_object(),
        "schema /203: a shaped find row must publish its `case_toggle`, got {toggle}"
    );
    Panel {
        card: [
            c["x"].as_f64().expect("card x"),
            c["y"].as_f64().expect("card y"),
            c["w"].as_f64().expect("card w"),
            c["h"].as_f64().expect("card h"),
        ],
        text_left: p["text"]["left"].as_f64().expect("text left"),
        text_top: p["text"]["top"].as_f64().expect("text top"),
        rows,
        toggle: (
            toggle["x0"].as_f64().expect("x0"),
            toggle["x1"].as_f64().expect("x1"),
        ),
    }
}

fn opts(canvas: (u32, u32), dpi: f32) -> CaptureOpts {
    CaptureOpts {
        canvas: Some(canvas),
        dpi: Some(dpi),
        search: Some("hello".to_string()),
        search_replace_active: true,
        search_replacement: "goodbye".to_string(),
        ..CaptureOpts::default()
    }
}

fn rel_lum(px: image::Rgba<u8>) -> f64 {
    0.2126 * px[0] as f64 + 0.7152 * px[1] as f64 + 0.0722 * px[2] as f64
}

/// The `n` strongest luminance steps along a scan line, as positions, with
/// non-maximum suppression so one rim (which spans two or three pixels as it
/// blends) contributes one answer rather than three.
///
/// Deliberately RELATIVE: it compares neighbouring pixels to each other and
/// never to a theme constant, so it says the same thing on a rasterizer that
/// rounds a channel differently.
fn strongest_steps(samples: &[f64], n: usize) -> Vec<usize> {
    let mut steps: Vec<(f64, usize)> = (1..samples.len())
        .map(|i| ((samples[i] - samples[i - 1]).abs(), i - 1))
        .collect();
    steps.sort_by(|a, b| b.0.partial_cmp(&a.0).expect("finite luminance"));
    let mut out: Vec<usize> = Vec::new();
    for (_, at) in steps {
        if out.len() == n {
            break;
        }
        if out.iter().all(|had| had.abs_diff(at) > 3) {
            out.push(at);
        }
    }
    out.sort_unstable();
    out
}

/// The card's real edges, measured off the PNG: the two strongest horizontal
/// steps on a row inside the card's own pad, and the two strongest vertical steps
/// in a column inside it. Sampled in the pad rather than over the text, because a
/// glyph edge is a stronger step than a rim and would win.
fn measured_edges(png: &std::path::Path, p: &Panel) -> ([usize; 2], [usize; 2]) {
    let img = image::open(png).expect("decode PNG").to_rgba8();
    let (w, h) = (img.width() as usize, img.height() as usize);
    // 4px below the published card top: inside the card's unscaled 12px pad at
    // every capture scale, and above the first row of glyphs.
    let y = (p.card[1] as usize + 4).min(h - 1);
    let row: Vec<f64> = (0..w)
        .map(|x| rel_lum(*img.get_pixel(x as u32, y as u32)))
        .collect();
    let xs = strongest_steps(&row, 2);
    // 4px right of the published card left edge: the left pad, no glyphs.
    let x = (p.card[0].max(0.0) as usize + 4).min(w - 1);
    let col: Vec<f64> = (0..h)
        .map(|y| rel_lum(*img.get_pixel(x as u32, y as u32)))
        .collect();
    let ys = strongest_steps(&col, 2);
    assert_eq!(
        (xs.len(), ys.len()),
        (2, 2),
        "the card must present two horizontal and two vertical rims to measure"
    );
    ([xs[0], xs[1]], [ys[0], ys[1]])
}

/// The published rect against the rim the frame actually drew, on all four sides
/// AND as a pair of extents.
///
/// The rim is drawn just OUTSIDE the fill and is about two pixels wide, so the
/// step this scan reports sits up to 2px outside the rect's own edge (measured:
/// 2.09px at the leading edges, 0.0 at the trailing ones, identically at both
/// capture scales — the rim width is unscaled too). `EDGE_TOL` is 4px: nearly
/// twice the observed worst case, so a rasterizer that anti-aliases the rim
/// differently cannot redden this, while the smallest error it must catch — a
/// uniformly rescaled rect — moves an edge by hundreds of pixels.
const EDGE_TOL: f64 = 4.0;

fn assert_card_matches_the_ink(name: &str, png: &std::path::Path, p: &Panel) {
    let (xs, ys) = measured_edges(png, p);
    let want_x = [p.card[0], p.card[0] + p.card[2]];
    let want_y = [p.card[1], p.card[1] + p.card[3]];
    for (i, (measured, published)) in xs.iter().zip(want_x.iter()).enumerate() {
        assert!(
            (*measured as f64 - published).abs() <= EDGE_TOL,
            "{name}: the card's {} rim is drawn at x {measured} while the sidecar \
             publishes {published} (card {:?}, measured rims {xs:?})",
            if i == 0 { "left" } else { "right" },
            p.card
        );
    }
    // The EXTENT, not only the two origins: a rect pinned edge-by-edge already
    // catches a uniform scaling, and asking for the measured span outright says so
    // in the failure message rather than leaving a reader to subtract.
    assert!(
        ((xs[1] - xs[0]) as f64 - p.card[2]).abs() <= EDGE_TOL,
        "{name}: the drawn card spans {}px between its rims while the sidecar \
         publishes a width of {} (rims {xs:?})",
        xs[1] - xs[0],
        p.card[2]
    );
    for (i, (measured, published)) in ys.iter().zip(want_y.iter()).enumerate() {
        assert!(
            (*measured as f64 - published).abs() <= EDGE_TOL,
            "{name}: the card's {} rim is drawn at y {measured} while the sidecar \
             publishes {published} (card {:?}, measured rims {ys:?})",
            if i == 0 { "top" } else { "bottom" },
            p.card
        );
    }
    assert!(
        ((ys[1] - ys[0]) as f64 - p.card[3]).abs() <= EDGE_TOL,
        "{name}: the drawn card spans {}px between its top and bottom rims while \
         the sidecar publishes a height of {} (rims {ys:?})",
        ys[1] - ys[0],
        p.card[3]
    );
}

/// One scale's block on its own terms — bands stepping contiguously at one
/// pitch from the text origin, the toggle inside the card, the ink origin inside
/// the card. Checked at BOTH scales, because a relation between two scales says
/// nothing about whether either describes a real card.
fn assert_internally_consistent(name: &str, p: &Panel) {
    let [cx, cy, cw, ch] = p.card;
    assert!(
        p.rows.len() >= 3,
        "{name}: the replace state shapes a field, a replacement and a hint row, \
         got {}",
        p.rows.len()
    );
    let pitch = p.rows[0].2;
    assert!(
        pitch > 4.0,
        "{name}: a {pitch}px row pitch is not a drawn row"
    );
    for (i, (row, top, h)) in p.rows.iter().enumerate() {
        assert_eq!(*row as usize, i, "{name}: bands must be in draw order");
        assert!(
            (top - (p.text_top + i as f64 * pitch)).abs() < 0.01,
            "{name}: row {i} is at {top}, not text_top {} + {i} * pitch {pitch}",
            p.text_top
        );
        assert!(
            (h - pitch).abs() < 0.01,
            "{name}: row {i} is {h} tall against a pitch of {pitch}"
        );
        // Overlap, not containment: the card is a rect around ink, and the two
        // have separate owners.
        assert!(
            top + h > cy && *top < cy + ch,
            "{name}: row {i}'s band [{top}, {}] does not meet the card's \
             [{cy}, {}]",
            top + h,
            cy + ch
        );
    }
    assert!(
        p.text_left > cx && p.text_top > cy,
        "{name}: the ink origin ({}, {}) must sit inside the card at ({cx}, {cy})",
        p.text_left,
        p.text_top
    );
    let (x0, x1) = p.toggle;
    assert!(
        x1 - x0 > 4.0 && x0 > p.text_left && x1 < cx + cw,
        "{name}: the Aa span [{x0}, {x1}] must be a real width inside the card's \
         [{cx}, {}]",
        cx + cw
    );
}

#[test]
fn published_panel_geometry_matches_the_drawn_card_at_both_capture_scales() {
    if !adapter_available() {
        eprintln!(
            "skipping published_panel_geometry_matches_the_drawn_card_at_both_capture_scales: \
             no wgpu adapter"
        );
        return;
    }
    let _tg = crate::testlock::serial();
    crate::theme::set_active(crate::theme::DEFAULT_THEME);
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl_panel_geometry_{}", std::process::id())),
    );
    let buf = Buffer::from_str("hello world\nhello again\n");

    // ONE logical window at two scales: 1200x800 at dpi 1 and 2400x1600 at dpi 2
    // are the same (W/N)x(H/N) logical window, so every difference is the scale
    // factor and the unscaled chrome constants, and nothing else.
    let one = dir.join("dpi1.png");
    capture_with(&one, &buf, &opts((1200, 800), 1.0)).expect("dpi 1 capture");
    let two = dir.join("dpi2.png");
    capture_with(&two, &buf, &opts((2400, 1600), 2.0)).expect("dpi 2 capture");
    let (a, b) = (read_panel(&one), read_panel(&two));

    assert_internally_consistent("1x", &a);
    assert_internally_consistent("2x", &b);
    // THE INK, at each scale. This is the assertion a rescaled report dies on,
    // and it needs no expectation written down anywhere.
    assert_card_matches_the_ink("1x", &one, &a);
    assert_card_matches_the_ink("2x", &two, &b);

    // ---- what a PHYSICAL-pixel figure does across the two scales -------------
    // Metric- and ink-derived, so these double exactly.
    let doubles = |lo: f64, hi: f64, what: &str| {
        assert!(
            (hi - lo * 2.0).abs() <= 1.0,
            "{what} is {lo} at --capture-dpi 1 and {hi} at 2: this figure comes \
             straight off the scaled metrics, so a PHYSICAL-pixel report doubles \
             it (a logical one would not move; a double-scaled one quadruples)"
        );
    };
    assert_eq!(
        a.rows.len(),
        b.rows.len(),
        "the same panel must shape the same rows at either scale"
    );
    for (i, (lo, hi)) in a.rows.iter().zip(b.rows.iter()).enumerate() {
        doubles(lo.2, hi.2, &format!("panel.rows[{i}].h (the row pitch)"));
    }
    doubles(
        a.toggle.1 - a.toggle.0,
        b.toggle.1 - b.toggle.0,
        "the Aa span's width (shaped advances)",
    );
    // The card's own rect carries the unscaled 12px margin and pad, so it grows
    // by twice-plus-a-residual rather than exactly twice. What must hold — and
    // what a logical-unit report breaks — is that it grows at all.
    for (lo, hi, what) in [
        (a.card[0], b.card[0], "panel.card.x"),
        (a.card[2], b.card[2], "panel.card.w"),
        (a.card[3], b.card[3], "panel.card.h"),
        (a.text_left, b.text_left, "panel.text.left"),
        (a.toggle.0, b.toggle.0, "the Aa span's x0"),
    ] {
        assert!(
            hi > lo * 1.5,
            "{what} is {lo} at --capture-dpi 1 and {hi} at 2: a physical-pixel \
             figure grows with the scale factor (this one carries the card's own \
             unscaled 12px margin/pad, so it is not exactly 2x)"
        );
    }
}

/// **THE PANEL DOWN REPORTS `null`, NOT A STALE CARD.** The gate is the same
/// `search_active` the draw asks before it paints anything, and a report that
/// answered from a shaped-but-unused buffer would publish a card the PNG does not
/// carry — the class of disagreement the whole block exists to make visible.
#[test]
fn a_panel_that_is_down_publishes_null_rather_than_where_it_would_have_gone() {
    if !adapter_available() {
        eprintln!(
            "skipping a_panel_that_is_down_publishes_null_rather_than_where_it_would_have_gone: \
             no wgpu adapter"
        );
        return;
    }
    let _tg = crate::testlock::serial();
    crate::theme::set_active(crate::theme::DEFAULT_THEME);
    let dir = ScratchDir::new(
        std::env::temp_dir().join(format!("awl_panel_down_{}", std::process::id())),
    );
    let buf = Buffer::from_str("hello world\nhello again\n");
    let png = dir.join("down.png");
    capture_with(&png, &buf, &CaptureOpts::default()).expect("capture");
    let text = std::fs::read_to_string(png.with_extension("json")).expect("sidecar");
    let v: serde_json::Value = serde_json::from_str(&text).expect("sidecar parses");
    assert_eq!(
        v["search"]["active"],
        serde_json::json!(false),
        "the fixture must leave the panel down"
    );
    assert!(
        v["search"]["panel"].is_null(),
        "schema /203: a panel that is down publishes null, got {}",
        v["search"]["panel"]
    );
}
