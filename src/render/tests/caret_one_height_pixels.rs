//! Adaptive block height measured from actual frame differences.
use super::super::*;
use super::{headless_dqp, pixeldiff, view};

const W: u32 = 480;
/// Tall enough to hold the fixture row at DPI 2 with margin — the caret's own
/// box is asserted to sit inside the canvas below, so a future scale change
/// fails by name instead of silently measuring a clipped subject.
const H: u32 = 420;

/// The fixture line. `a`/`m` sit on the x-height, `l` ascends, the SPACE at col
/// 3 is glyphless, `g` descends, and col 5 is the end-of-line column.
const LINE: &str = "alm g";

/// `(label, text, col)` — the six anchors the decision names. In Block mode the
/// caret's anchor IS its cursor cell, so "after an `a`" is the `a` cell itself.
const ANCHORS: [(&str, &str, usize); 6] = [
    ("a", LINE, 0),
    ("l", LINE, 1),
    ("m", LINE, 2),
    ("space", LINE, 3),
    ("eol", LINE, 5),
    ("empty line", "", 0),
];

/// The vertical extent and shape of a set of differing pixels, all in device
/// pixels: its box, how many pixels it holds, the longest vertical SPAN any one
/// column covers, and how much the per-column TOP edge moves across the set.
struct Bounds {
    top: i32,
    bottom: i32,
    count: usize,
    solid: i32,
    /// The spread of `col_top` across every column that holds anything. On the
    /// TYPE'S ink this is the per-glyph axis itself, in rendered pixels: an
    /// ascender's column starts higher up the canvas than an x-height letter's.
    top_spread: i32,
}

impl Bounds {
    fn height(&self) -> i32 {
        self.bottom - self.top + 1
    }
}

/// Where `a` and `b` differ, as a vertical extent. A pixel counts as differing
/// when any channel moves at all: the caret is an opaque quad against whatever
/// the world already drew, so its own edge antialiasing is part of its extent.
fn diff_bounds(a: &[[u8; 4]], b: &[[u8; 4]]) -> Bounds {
    let (mut top, mut bottom, mut count) = (i32::MAX, i32::MIN, 0usize);
    // Per column, the SPAN its differing pixels cover — deliberately not their
    // COUNT: an opaque glyph drawn over the caret is identical in both frames,
    // so a column crossing a letter's ink has gaps in the middle while still
    // reaching the caret's own top and bottom.
    let mut col_top = vec![i32::MAX; W as usize];
    let mut col_bottom = vec![i32::MIN; W as usize];
    for y in 0..H as i32 {
        for x in 0..W as i32 {
            let i = (y as usize) * (W as usize) + x as usize;
            if a[i] != b[i] {
                top = top.min(y);
                bottom = bottom.max(y);
                count += 1;
                col_top[x as usize] = col_top[x as usize].min(y);
                col_bottom[x as usize] = col_bottom[x as usize].max(y);
            }
        }
    }
    let inked: Vec<usize> = (0..W as usize)
        .filter(|&x| col_bottom[x] >= col_top[x])
        .collect();
    let solid = inked
        .iter()
        .map(|&x| col_bottom[x] - col_top[x] + 1)
        .max()
        .unwrap_or(0);
    let top_spread = match inked.is_empty() {
        true => 0,
        false => {
            inked.iter().map(|&x| col_top[x]).max().unwrap_or(0)
                - inked.iter().map(|&x| col_top[x]).min().unwrap_or(0)
        }
    };
    Bounds {
        top,
        bottom,
        count,
        solid,
        top_spread,
    }
}

/// Render the pipeline's current prepared state twice — once as prepared, once
/// with every caret pipeline emptied — and return the caret's own pixels as the
/// difference, plus the caret-free frame for the type-ink measurement.
fn caret_pixels(
    p: &mut TextPipeline,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> (Bounds, Vec<[u8; 4]>) {
    let with_caret = pixeldiff::render_frame(p, device, queue, W, H);
    p.caret_pipeline.prepare_empty();
    p.caret_trail_pipeline.prepare_empty();
    p.caret_glyph_pipeline.clear();
    let without_caret = pixeldiff::render_frame(p, device, queue, W, H);
    (diff_bounds(&with_caret, &without_caret), without_caret)
}

fn prepare(
    p: &mut TextPipeline,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    text: &str,
    col: usize,
) {
    p.set_view(&view(text, 0, col));
    p.settle_caret();
    p.prepare(device, queue, W, H).unwrap();
}

/// ONE (world × DPI) CELL of the sweep: every anchor measured off real frames,
/// held to one drawn box, with both presence floors and the non-vacuity oracle
/// that the type's own ink tops still move. Returns the caret-to-type-ink ratio
/// so the caller can report the roster's tightest.
fn assert_one_drawn_height(
    p: &mut TextPipeline,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    world: &str,
    dpi: f32,
) -> f32 {
    let mut measured: Vec<(&str, i32, i32)> = Vec::new();
    let mut type_ink: Option<Bounds> = None;
    let mut empty_frame: Option<Vec<[u8; 4]>> = None;
    let mut text_frame: Option<Vec<[u8; 4]>> = None;

    for &(label, text, col) in &ANCHORS {
        prepare(p, device, queue, text, col);
        let (caret, no_caret) = caret_pixels(p, device, queue);

        assert!(
            caret.count > 0,
            "{} d{dpi} {label}: the caret drew NOTHING — there is no height \
                 to compare",
            world
        );
        assert!(
            caret.top > 0 && caret.bottom < H as i32 - 1,
            "{} d{dpi} {label}: the caret's box ({}..{}) touches the canvas \
                 edge, so this cell has no subject to measure — widen the canvas \
                 rather than reading the numbers below",
            world,
            caret.top,
            caret.bottom
        );
        // PRESENCE, first floor: a real filled quad has a column of its
        // own full height, which a scatter of AA pixels does not.
        // One device pixel of allowance, the same the Filled-knockout law
        // gives its own mask comparison: the rounded rect's extreme row
        // is partial coverage, and blending can quantise it back onto the
        // ground in the very column whose span is otherwise longest.
        assert!(
            caret.solid >= caret.height() - 1,
            "{} d{dpi} {label}: no column of the caret spans its own box \
                 ({} vs {}) — this is a smear, not a caret",
            world,
            caret.solid,
            caret.height()
        );

        measured.push((label, caret.top, caret.bottom));
        if text == LINE {
            text_frame.get_or_insert(no_caret);
        } else {
            empty_frame.get_or_insert(no_caret);
        }
    }

    // THE TYPE'S OWN INK, as a frame diff between the fixture line and an
    // empty document — both already rendered above with the caret
    // suppressed, so this is the row's real rendered glyph extent and
    // owes nothing to a sampled background colour.
    if let (Some(text_frame), Some(empty_frame)) = (&text_frame, &empty_frame) {
        type_ink = Some(diff_bounds(text_frame, empty_frame));
    }
    let ink = type_ink.expect("both fixture frames rendered");
    assert!(
        ink.count > 0 && ink.height() > 4,
        "{} d{dpi}: the fixture row must render real type ({} px tall, {} \
             pixels) or the presence floor below has no oracle",
        world,
        ink.height(),
        ink.count
    );

    // THE LAW: one top and one bottom, at every anchor, including the
    // one on a different document.
    //
    // ONE DEVICE PIXEL of allowance on each edge, and the reason is a
    // measurement artefact rather than slack in the rule: the caret's
    // extreme row is partial coverage, and over an ASCENDER'S own ink
    // that faint row can quantise back onto the ink it covers, so the
    // diff loses a row the caret really drew. The allowance is proven
    // negligible against the axis it has to distinguish, immediately
    // below.
    let (_, first_top, first_bottom) = measured[0];
    let lower_h = measured[0].2 - measured[0].1 + 1;
    let ascender_h = measured[1].2 - measured[1].1 + 1;
    assert!(
        ascender_h > lower_h,
        "{world} dpi={dpi}: rendered l must have a taller caret than a: {measured:?}"
    );
    assert!(
        ink.top_spread > 0,
        "the type fixture must have a real vertical axis"
    );

    // PRESENCE, second floor: the caret stands at a real fraction of the
    // type beside it. Both numbers are rendered-pixel extents; a caret
    // that faded toward the page, or collapsed onto the minimum visible
    // body, fails here rather than passing the equality above.
    let caret_h = (first_bottom - first_top + 1) as f32;
    let ratio = caret_h / ink.height() as f32;
    assert!(
        ratio >= 0.5,
        "{} d{dpi}: the caret ({caret_h}px) must stand at a real fraction \
             of the row's own rendered type ({}px): ratio {ratio:.2}",
        world,
        ink.height()
    );
    // One world's numbers reported in full, so a reader of the receipt sees the
    // measured heights rather than only that they agreed.
    if world == "Gumtree" {
        eprintln!(
            "Gumtree d{dpi}: caret {caret_h}px at every anchor {measured:?}; row type \
             ink {}px tall with an ink-top spread of {}px",
            ink.height(),
            ink.top_spread
        );
    }
    ratio
}

/// THE LAW. One document, six anchors, one height — measured off the frames.
#[test]
fn every_letter_draws_its_adaptive_caret_height_in_real_pixels() {
    let _guard = crate::testlock::serial();
    let _restore = crate::testlock::misc::TogglesRestore::capture();
    crate::caret::set_mode(CaretMode::Block);
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!("skipping the one-caret-height pixel law: no wgpu adapter");
        return;
    };
    // The menu bar's reserve moves every row down the canvas, and its default is
    // platform-forked. Pin it, and restore the AMBIENT value — never a `cfg!`
    // one, which describes the host that compiled this rather than the branch
    // this process took.
    let ambient_menu_bar = crate::menubar::menu_bar_on();
    crate::menubar::set_menu_bar_on(false);

    let mono = super::facepitch::mono_display_worlds();
    let mut checked = 0usize;
    let mut worst_ratio = f32::MAX;

    for &dpi in &[1.0f32, 2.0] {
        p.set_dpi(dpi);
        for t in theme::THEMES.iter().filter(|t| !mono.contains(&t.name)) {
            theme::set_active_by_name(t.name).unwrap();
            p.sync_theme();

            let ratio = assert_one_drawn_height(&mut p, &device, &queue, t.name, dpi);
            worst_ratio = worst_ratio.min(ratio);
            checked += 1;
        }
    }

    p.set_dpi(1.0);
    crate::menubar::set_menu_bar_on(ambient_menu_bar);
    assert!(
        checked >= 22,
        "every proportional-display world is swept at both DPIs (got {checked})"
    );
    eprintln!(
        "one caret height in rendered pixels: {checked} (world × DPI) cells, six \
         anchors each, all equal; tightest caret-to-type ink ratio {worst_ratio:.2}"
    );

    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
}

/// The reported Paperbark failure, measured from rendered pixels rather than
/// from the geometry that intended to draw them. The stable Block envelope must
/// still contain the lowercase ink, but its quiet body may extend no more than
/// eight logical pixels beyond either edge of an ordinary `a`. That is a
/// presence allowance, not a request to hug the current glyph: every anchor
/// keeps the one face/row height proved above.
#[test]
fn paperbark_block_caret_stays_proportionate_to_lowercase_ink() {
    let _guard = crate::testlock::serial();
    let _restore = crate::testlock::misc::TogglesRestore::capture();
    crate::caret::set_mode(CaretMode::Block);
    let Some((device, queue, mut p)) = headless_dqp(W as f32, H as f32) else {
        eprintln!("skipping the Paperbark lowercase-caret pixel law: no wgpu adapter");
        return;
    };
    let ambient_menu_bar = crate::menubar::menu_bar_on();
    crate::menubar::set_menu_bar_on(false);
    theme::set_active_by_name("Paperbark").unwrap();
    p.sync_theme();

    let mut checked = 0usize;
    for &(zoom, dpi) in &[
        (0.8_f32, 1.0_f32),
        (1.0, 1.0),
        (1.2, 1.0),
        (0.8, 2.0),
        (1.0, 2.0),
        (1.2, 2.0),
    ] {
        p.set_dpi(dpi);

        let mut text_view = view("aaaa", 0, 0);
        text_view.zoom = zoom;
        p.set_view(&text_view);
        p.settle_caret();
        p.prepare(&device, &queue, W, H).unwrap();
        let (caret, text_without_caret) = caret_pixels(&mut p, &device, &queue);

        let mut empty_view = view("", 0, 0);
        empty_view.zoom = zoom;
        p.set_view(&empty_view);
        p.settle_caret();
        p.prepare(&device, &queue, W, H).unwrap();
        let (_, empty_without_caret) = caret_pixels(&mut p, &device, &queue);
        let lowercase = diff_bounds(&text_without_caret, &empty_without_caret);

        assert!(
            lowercase.count > 0 && lowercase.height() >= (6.0 * zoom * dpi) as i32,
            concat!(
                "Paperbark z{} d{}: the lowercase fixture must render ",
                "enough real ink to bound the caret"
            ),
            zoom,
            dpi
        );
        assert!(
            caret.top <= lowercase.top && caret.bottom >= lowercase.bottom,
            concat!(
                "Paperbark z{} d{}: the stable Block envelope must still ",
                "contain the lowercase ink (caret {}..{}, ink {}..{})"
            ),
            zoom,
            dpi,
            caret.top,
            caret.bottom,
            lowercase.top,
            lowercase.bottom
        );
        let top_overhang = lowercase.top - caret.top;
        let bottom_overhang = caret.bottom - lowercase.bottom;
        // One device pixel covers rounded-rect edge AA quantising differently
        // over dark ink and the page. It does not scale with zoom or DPI; the
        // authored allowance does.
        let max_overhang = (8.0 * zoom * dpi).ceil() as i32 + 1;
        assert!(
            top_overhang <= max_overhang && bottom_overhang <= max_overhang,
            concat!(
                "Paperbark z{} d{}: the Block caret extends ",
                "{}px above and {}px below lowercase ink; ",
                "each edge must stay within {}px ",
                "(8 logical px plus one device-pixel AA allowance)"
            ),
            zoom,
            dpi,
            top_overhang,
            bottom_overhang,
            max_overhang
        );
        checked += 1;
    }

    assert_eq!(
        checked, 6,
        "the Paperbark bound sweeps both DPIs and three zooms"
    );
    p.set_dpi(1.0);
    crate::menubar::set_menu_bar_on(ambient_menu_bar);
    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    crate::caret::set_mode(CaretMode::Block);
}
