//! CASSOWARY'S LOCATION CUE IS THE WORDMARK'S VERTICAL COMPANION.
//!
//! **Defect:** `LocationStyle::RotatedRail` drew the active facet's name small,
//! muted and flush with the CARD's own left border — a whisper in the card's
//! gutter, which is neither the card's voice nor the room's.
//!
//! **Build:** rotated 90° along the ROOM's own outer margin (the margin the
//! Archivo-Black wordmark placard already keeps), seated just ABOVE that
//! placard, at exactly [`ROTATED_RAIL_PLACARD_FRACTION`] of its type size and
//! in its ink.
//!
//! The laws here grade the four claims that composition makes, and every
//! appearance claim is arithmetic over real GPU pixels (the sidecar reports
//! state, never whether anything is visible):
//!
//! - **THE ⅔ RELATION**, from both sides, at both DPI tiers — asserted on the
//!   size the product actually decided AND on the ink that reached the screen.
//! - **PRESENCE**, so the non-overlap laws below cannot be satisfied by a cue
//!   that faded into the ground or shrank toward nothing: the run's ink is as
//!   STRONG as the wordmark's (they share an ink) and as BIG as ⅔ predicts.
//! - **NON-OVERLAP AND NO CLIPPING** against the placard's own line box, the
//!   card's TRUE drawn span (wider than its box — the selected plate grows
//!   outward past it) and the canvas edge.
//! - **THE PARK ARM**: where the margin cannot hold the ⅔ run, the cue is
//!   absent rather than shrunk, and that is proven to be the park rather than a
//!   missing plan line.
//!
//! **THE SWEEP IS OVER EVERY LENS LABEL IN THE PRODUCT × both DPI tiers**,
//! derived from the roster rather than from a hand-picked name — the feature was
//! calibrated against "Navigate" and "Settings", and neither is the longest
//! label a lens roster carries ("Keybindings", 11 characters; "This folder",
//! 11; "Appearance", 10). Every label is driven through the faceted CARD shape,
//! including the ones whose own kind draws as a summoned WORKSPACE and so shows
//! no location line at all (`workspace_faceting_kinds_carry_no_location_line`
//! records which those are): the cue's composition is a function of the LABEL
//! and the card's geometry, so sweeping the longest strings in the roster
//! bounds the shortest by the same arithmetic — which is the whole of this
//! item's responsive risk.

use super::super::*;
use super::{headless_dqp, view};
use crate::overlay::{OverlayKind, OverlayState};
use crate::render::rotated_location::{
    LOCATION_SCALE, ROTATED_RAIL_PLACARD_FRACTION, ROTATED_RAIL_PLACARD_GAP_EM, placard_font_size,
};

/// The two DPI tiers, as the SAME logical room at two device scales — a Retina
/// panel has twice the device pixels for the same window, so the honest 2×
/// tier doubles the canvas AND the scale. (Doubling the scale alone shrinks the
/// room to half its logical size, where the card enters its fill regime and
/// draws no wordmark at all — a real state, graded by the park law below, but
/// not a second measurement of this one.)
const TIERS: [(u32, u32, f32); 2] = [(1200, 800, 1.0), (2400, 1600, 2.0)];

/// Every faceting picker × every lens that names a place, from the roster.
fn roster_cells() -> Vec<(OverlayKind, usize, &'static str)> {
    let mut out = Vec::new();
    for kind in OverlayKind::ALL {
        let Some(scheme) = crate::facets::scheme(kind) else {
            continue;
        };
        for i in 1..scheme.strip.len() {
            let Some(label) = scheme.location(i) else {
                continue;
            };
            out.push((kind, i, label));
        }
    }
    out
}

/// A faceted card of `kind` at lens `i`, folded the way `App::sync_view` folds
/// one. The sections are `vec![label; n]` because that IS what every lens
/// produces — `palette_location`'s premise law pins that every lens
/// groups into exactly one section whose label is the lens's own.
fn faceted_view(kind: OverlayKind, lens: usize) -> ViewState {
    let scheme = crate::facets::scheme(kind).expect("a faceting kind");
    let label = scheme.location(lens).expect("a lens that names a place");
    let items: Vec<String> = match kind {
        // The real catalog: the widest content any faceted card carries, which
        // is what pushes a right-anchored card's own left edge leftward.
        OverlayKind::Command => crate::commands::names(),
        _ => (0..14)
            .map(|k| format!("a-document-with-a-long-name-{k}.md"))
            .collect(),
    };
    let n = items.len();
    let mut v = view("hello world\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = kind.title();
    v.overlay_items = items;
    v.overlay_bindings = vec![String::new(); n];
    v.overlay_lens = scheme.strip_labels(lens);
    v.overlay_sections = vec![label.to_string(); n];
    v.overlay_location = Some(label.to_string());
    v.overlay_selected = 0;
    v
}

/// **THE REFERENCE FRAME, and it is not "no location".** Withholding the datum
/// makes the row planner emit the retired uppercase `Header` in the same slot —
/// a real inline glyph run inside the card — so the diff would carry the card's
/// own content as well as the cue's. A BLANK location (the datum present, its
/// text whitespace) keeps the plan, the row heights, the plates and every glyph
/// byte-identical while the cue's own whitespace case parks the pipeline. What
/// differs between the two shots is then the cue's ink and nothing else, over
/// the WHOLE canvas — which is what lets the non-overlap arms scan everywhere
/// rather than inside a window that could hide a collision.
fn blank_location(v: &mut ViewState) {
    let n = v.overlay_items.len();
    v.overlay_location = Some(" ".to_string());
    v.overlay_sections = vec![" ".to_string(); n];
}

/// A COMMAND palette, the widest card, at the lens named `label`.
fn command_view(label: &str) -> ViewState {
    let lens = crate::facets::scheme(OverlayKind::Command)
        .expect("the command palette facets")
        .strip
        .iter()
        .position(|f| f.label == label)
        .unwrap_or_else(|| panic!("no {label} lens on the command palette"));
    let names = crate::commands::names();
    let hidden = vec![false; names.len()];
    let mut ov =
        OverlayState::new_command(names, crate::commands::effective_bindings(&[], &[]), hidden);
    ov.set_facet_lens(lens);
    let mut v = view("hello world\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = OverlayKind::Command.title();
    v.overlay_items = ov.item_strings();
    v.overlay_bindings = ov.item_bindings();
    v.overlay_lens = ov.lens_strip();
    v.overlay_sections = ov.item_sections();
    v.overlay_location = ov.location().map(std::string::ToString::to_string);
    v.overlay_selected = ov.selected;
    v
}

fn shoot(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &mut TextPipeline,
    w: u32,
    h: u32,
) -> Vec<[u8; 4]> {
    let (texture, tview) = super::dither::offscreen(device, w, h);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("awl item297 rotated-rail encoder"),
    });
    p.render(&mut encoder, &tview).unwrap();
    queue.submit(Some(encoder.finish()));
    super::dither::read_pixels(device, queue, &texture, w, h)
}

fn luma(c: [u8; 4]) -> f32 {
    0.2126 * c[0] as f32 + 0.7152 * c[1] as f32 + 0.0722 * c[2] as f32
}

/// An ink bounding box `(x0, x1, y0, y1, count, peak_luma)` over the pixels of
/// `a` that DIFFER from `b` inside the window — the differential oracle, so
/// ground, texture, wordmark, strip and rows all cancel and what is left is
/// attributable to the location treatment alone.
type InkStats = (i64, i64, i64, i64, usize, f32);

fn diff_ink(
    a: &[[u8; 4]],
    b: &[[u8; 4]],
    w: i64,
    h: i64,
    x: (i64, i64),
    y: (i64, i64),
) -> Option<InkStats> {
    let (mut x0, mut x1, mut y0, mut y1) = (i64::MAX, i64::MIN, i64::MAX, i64::MIN);
    let (mut count, mut peak) = (0usize, 0.0f32);
    for row in y.0.max(0)..y.1.min(h) {
        for col in x.0.max(0)..x.1.min(w) {
            let i = (row * w + col) as usize;
            if a[i] == b[i] {
                continue;
            }
            x0 = x0.min(col);
            x1 = x1.max(col);
            y0 = y0.min(row);
            y1 = y1.max(row);
            count += 1;
            peak = peak.max(luma(a[i]));
        }
    }
    (count > 0).then_some((x0, x1, y0, y1, count, peak))
}

/// The wordmark's own drawn ink box + peak, measured in its own band left of
/// the card — `(height, peak_luma)`.
fn placard_ink(
    px: &[[u8; 4]],
    w: i64,
    h: i64,
    x: (i64, i64),
    y: (i64, i64),
    ground: f32,
) -> (f32, f32) {
    let (mut y0, mut y1, mut peak) = (i64::MAX, i64::MIN, 0.0f32);
    for row in y.0.max(0)..y.1.min(h) {
        for col in x.0.max(0)..x.1.min(w) {
            let l = luma(px[(row * w + col) as usize]);
            if (l - ground).abs() > 24.0 {
                y0 = y0.min(row);
                y1 = y1.max(row);
                peak = peak.max(l);
            }
        }
    }
    match y1 >= y0 {
        true => ((y1 - y0 + 1) as f32, peak),
        false => (0.0, 0.0),
    }
}

// ---------------------------------------------------------------------------
// THE ROSTER — who carries this composition, and what it needs of them.
// ---------------------------------------------------------------------------

/// **CASSOWARY IS THE SOLE CARRIER, and the count is pinned so adding a second
/// one has to read this file first.** Derived from the roster rather than
/// asserted of a name: a world reassigned to `RotatedRail` shows up here, not in
/// a surprise on someone's screen.
#[test]
fn cassowary_is_the_rosters_only_rotated_rail_world() {
    let carriers: Vec<&str> = theme::THEMES
        .iter()
        .filter(|t| t.render_caps.location_style == theme::LocationStyle::RotatedRail)
        .map(|t| t.name)
        .collect();
    assert_eq!(
        carriers,
        ["Cassowary"],
        "the `RotatedRail` roster moved — item 297's composition is specified against \
         Cassowary's wordmark, so a new carrier needs its own look confirmed"
    );
    assert_eq!(
        theme::THEMES.len() - carriers.len(),
        19,
        "the world roster's size moved; the byte-identity claim is stated over 19 others"
    );
}

/// **THE COMPOSITION NEEDS A FLOOR-ANCHORED WORDMARK.** The cue rises from just
/// above the placard, so a placard pinned to the room's CEILING would need the
/// mirrored vertical anchor — which `rotated_rail_placement` deliberately does
/// not have, parking instead. Every `RotatedRail` world must therefore resolve
/// to a bottom corner. Non-vacuity: the same derivation asked for a hand-pinned
/// `TL` must answer `TL`, so this law can tell a bottom corner from a top one.
#[test]
fn every_rotated_rail_world_anchors_its_wordmark_to_the_rooms_floor() {
    use theme::PlacardCorner;
    let mut graded = 0usize;
    for t in theme::THEMES
        .iter()
        .filter(|t| t.render_caps.location_style == theme::LocationStyle::RotatedRail)
    {
        let theme::TitleStyle::Placard { corner, .. } = t.render_caps.title_style else {
            panic!(
                "{}: a `RotatedRail` world with no wordmark placard has nothing for its cue \
                 to be the companion of",
                t.name
            );
        };
        let derived = crate::render::derived_placard_corner(corner, t.render_caps.card_anchor);
        assert!(
            matches!(derived, PlacardCorner::BL | PlacardCorner::BR),
            "{}: wordmark corner resolves to {derived:?} — item 297's cue is composed \
             against a FLOOR-anchored wordmark and parks against any other",
            t.name
        );
        assert!(
            matches!(
                crate::render::derived_placard_corner(PlacardCorner::TL, t.render_caps.card_anchor),
                PlacardCorner::TL
            ),
            "{}: the corner derivation ignored a hand-pinned TL — this law cannot tell a \
             floor-anchored wordmark from a ceiling-anchored one",
            t.name
        );
        graded += 1;
    }
    assert_eq!(graded, 1, "the RotatedRail roster moved");
}

// ---------------------------------------------------------------------------
// THE ⅔ RELATION, THE PRESENCE FLOORS, AND NON-OVERLAP — real pixels.
// ---------------------------------------------------------------------------

/// **THE CUE IS EXACTLY ⅔ OF THE WORDMARK, VISIBLY THERE, AND TOUCHES
/// NOTHING** — swept over every faceting kind × every lens × both DPI tiers.
///
/// Five claims per cell, and the middle two exist because the outer ones get
/// HAPPIER as the cue disappears:
///
/// 1. **⅔ from both sides**, on the size the product decided: the cue's own
///    natural size against the placard's, to a tenth of a percent. Plus
///    non-vacuity — that size must be nowhere near the retired card-sized cue.
/// 2. **PRESENCE by extent**: the drawn ink's across-axis extent sits in the
///    band ⅔ predicts for this face (a run of capitals and ascenders, some with
///    descenders, against the wordmark's own all-capital height). The retired
///    treatment measures ≈0.18 of the wordmark and fails the floor; a
///    full-size run measures ≥1.0 and fails the ceiling.
/// 3. **PRESENCE by strength**: the cue and the wordmark share an ink
///    (`theme::placard_ink`), so the cue's strongest pixel must be within a
///    tenth of the wordmark's. A wash toward the ground fails here — the shape
///    this board has already shipped once.
/// 4. **NON-OVERLAP**: not one differing pixel at or below the placard's own
///    line-box top, nor at or right of the card's TRUE drawn left edge (which is
///    LEFT of `card_x` — the selected plate grows outward past the box).
/// 5. **NO CLIPPING**: the ink box sits strictly inside the canvas.
///
/// Claim 4's placard arm doubles as the safety proof for
/// `rotated_rail_placement` re-asking the placard's own owner for its rect
/// AFTER that buffer's upload was taken: if the second shaping moved one
/// wordmark pixel, it would differ between these two shots and land in the
/// forbidden band.
#[test]
fn rotated_rail_is_two_thirds_of_the_wordmark_present_and_clear_of_everything() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping item297 rail composition law: no wgpu adapter");
        return;
    };
    let _pin = theme::WorldPin::world("Cassowary").expect("Cassowary ships");
    let cells = roster_cells();
    assert!(cells.len() >= 12, "the faceting roster shrank: {cells:?}");

    let mut graded = 0usize;
    let mut drawn = 0usize;
    for (cw, chh, dpi) in TIERS {
        for &(kind, lens, label) in &cells {
            graded += 1;
            drawn += usize::from(grade_one_cell(
                &device,
                &queue,
                &mut p,
                (cw, chh, dpi),
                kind,
                lens,
                label,
            ));
        }
    }
    theme::set_active(theme::DEFAULT_THEME);
    assert_eq!(
        graded,
        cells.len() * TIERS.len(),
        "the kind x lens x tier sweep moved"
    );
    assert_eq!(
        drawn, graded,
        "the cue must draw in EVERY cell of this sweep — a parked cell here means the \
         composition's own regime has narrowed and these laws are grading absence"
    );
}

/// ONE `(tier, kind, lens)` cell of
/// `rotated_rail_is_two_thirds_of_the_wordmark_present_and_clear_of_everything`
/// — split out so the sweep above stays a visible loop over the cases graded
/// rather than the grading itself. Returns whether the cue DREW: a cell whose
/// margin cannot hold the ⅔ run belongs to the park law, not to this one.
fn grade_one_cell(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &mut TextPipeline,
    (cw, chh, dpi): (u32, u32, f32),
    kind: OverlayKind,
    lens: usize,
    label: &str,
) -> bool {
    p.set_size(cw as f32, chh as f32);
    p.set_dpi(dpi);
    p.sync_theme();
    let v = faceted_view(kind, lens);
    p.set_view(&v);
    p.prepare(device, queue, cw, chh).unwrap();
    let geom = p.overlay_geometry(cw);
    let cell = format!("lens {label:?} (roster: {kind:?}) on a faceted card, {cw}x{chh} dpi {dpi}");
    assert_eq!(
        geom.plan_labels_probe()
            .iter()
            .filter(|s| s.as_str() == format!("loc:{label}"))
            .count(),
        1,
        "{cell}: no location line planned ({:?})",
        geom.plan_labels_probe()
    );

    let Some((placard_x, placard_y, _pw, placard_h)) = p.overlay_shape_placard(&geom) else {
        // No wordmark: the park law owns this state, not this one.
        assert!(
            p.rotated_rail_probe(&geom).is_none(),
            "{cell}: no wordmark, yet the cue claimed a placement"
        );
        return false;
    };
    let Some((natural, _fit, _bottom, flush_x)) = p.rotated_rail_probe(&geom) else {
        return false; // the margin could not hold it — the park law's cell
    };

    // (1) THE ⅔ RELATION, from above and below.
    let want = placard_font_size(placard_h) * ROTATED_RAIL_PLACARD_FRACTION;
    assert!(
        (natural - want).abs() <= want * 1e-3,
        "{cell}: cue type is {natural:.2}px against a wordmark of \
         {:.2}px — the ⅔ relation wants {want:.2}px",
        placard_font_size(placard_h)
    );
    let card_sized =
        p.metrics.font_size * crate::render::effective_overlay_scale() * LOCATION_SCALE;
    assert!(
        natural > card_sized * 3.0,
        "{cell}: cue type {natural:.2}px is within reach of the RETIRED card-sized \
         cue ({card_sized:.2}px) — the scale class did not change"
    );

    // The two shots: identical but for the cue's own ink (`blanked`).
    let with = shoot(device, queue, p, cw, chh);
    let mut b = faceted_view(kind, lens);
    blank_location(&mut b);
    p.set_view(&b);
    p.prepare(device, queue, cw, chh).unwrap();
    let without = shoot(device, queue, p, cw, chh);

    let (w, h) = (cw as i64, chh as i64);
    let (card_left, _card_right) = p.overlay_card_drawn_span_probe(&geom);
    grade_cell_pixels(
        &with,
        &without,
        (w, h),
        &cell,
        card_left.floor() as i64,
        geom.band_x_probe(),
        (placard_x, placard_y),
        flush_x,
    );
    true
}

/// The PIXEL half of `grade_one_cell` — presence by extent, presence by
/// strength, non-overlap against the wordmark and the card, and no clipping.
/// Split from its caller so neither half exceeds a readable length; the two
/// shots and every bound come from the caller's own frame.
#[allow(clippy::too_many_arguments)]
fn grade_cell_pixels(
    with: &[[u8; 4]],
    without: &[[u8; 4]],
    (w, h): (i64, i64),
    cell: &str,
    card_left_px: i64,
    card_band_x: f32,
    (placard_x, placard_y): (f32, f32),
    flush_x: f32,
) {
    let ink = diff_ink(with, without, w, h, (0, w), (0, h))
        .unwrap_or_else(|| panic!("{cell}: the cue drew NOTHING anywhere on the canvas"));
    let (x0, x1, y0, y1, count, peak) = ink;

    // (2) PRESENCE BY EXTENT — the band ⅔ predicts for this face.
    let ground = luma(without[((h / 2) * w + card_left_px / 2) as usize]);
    let (mark_h, mark_peak) = placard_ink(
        without,
        w,
        h,
        (0, card_left_px),
        (placard_y.floor() as i64, h),
        ground,
    );
    assert!(
        mark_h > 0.0,
        "{cell}: no wordmark ink found — this law's own reference is missing"
    );
    let across = (x1 - x0 + 1) as f32;
    let ratio = across / mark_h;
    assert!(
        (0.58..=0.98).contains(&ratio),
        "{cell}: the cue's ink is {across:.0}px across against a {mark_h:.0}px \
         wordmark (ratio {ratio:.3}) — ⅔ of Archivo Black predicts 0.58..0.98 \
         (measured 0.641..0.897 over this roster, the spread being which labels \
         carry descenders), while the RETIRED whisper measures ≈0.18 and a \
         full-size run ≥1.0"
    );
    let area = across * (y1 - y0 + 1) as f32;
    assert!(
        count as f32 >= 0.10 * area,
        "{cell}: only {count} ink pixels in a {area:.0}px box — the cue drew an \
         outline or a hairline, not a run of type"
    );

    // (3) PRESENCE BY STRENGTH — the cue and the wordmark share an ink.
    assert!(
        peak >= 0.9 * mark_peak,
        "{cell}: the cue's strongest pixel is {peak:.1} against the wordmark's \
         {mark_peak:.1} — the cue has washed toward the ground"
    );

    // (4) NON-OVERLAP, both bounds, scanned over the whole canvas.
    let placard_top = placard_y.floor() as i64;
    let below = diff_ink(with, without, w, h, (0, w), (placard_top, h));
    assert!(
        below.is_none(),
        "{cell}: {:?} differing pixels at or below the wordmark's own line box \
         (y >= {placard_top}) — the cue is on top of the wordmark, or the second \
         placard shaping moved it",
        below.map(|b| b.4)
    );
    let onto_card = diff_ink(with, without, w, h, (card_left_px, w), (0, h));
    assert!(
        onto_card.is_none(),
        "{cell}: {:?} differing pixels at or right of the card's TRUE drawn left \
         edge ({card_left_px}; its own band starts at {:.1}) — the cue reaches the card",
        onto_card.map(|b| b.4),
        card_band_x
    );

    // (5) NO CLIPPING.
    assert!(
        x0 > 0 && y0 > 0 && x1 < w - 1 && y1 < h - 1,
        "{cell}: ink box ({x0}..{x1}, {y0}..{y1}) touches the canvas edge — clipped"
    );
    assert!(
        flush_x >= placard_x - 0.01,
        "{cell}: the cue seats at x={flush_x:.1}, inside the wordmark's own margin \
         ({placard_x:.1})"
    );
}

// ---------------------------------------------------------------------------
// THE PARK ARM.
// ---------------------------------------------------------------------------

/// **WHERE THE MARGIN CANNOT HOLD THE ⅔ RUN THE CUE IS ABSENT, NOT SMALLER.**
/// At 1.8× zoom the widest card's own drawn left edge closes the room's margin;
/// the wordmark bleeds behind the card there and a cue seated on it would too.
/// Graded on real pixels — zero cue ink in the whole margin — and proven to be
/// the PARK rather than a missing location line (the plan still carries one).
///
/// NON-VACUITY IS THE SECOND ARM, and it is the product one notch out: at 1.7×
/// the same cell draws, so this law can tell "parked" from "cannot draw here at
/// all", and the sweep above is not silently grading an empty regime.
#[test]
fn rotated_rail_parks_rather_than_shrinking_when_the_rooms_margin_closes() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping item297 park law: no wgpu adapter");
        return;
    };
    let _pin = theme::WorldPin::world("Cassowary").expect("Cassowary ships");
    p.sync_theme();

    let mut seen = Vec::new();
    for zoom in [1.7f32, 1.8] {
        let mut v = command_view("Navigate");
        v.zoom = zoom;
        p.set_view(&v);
        p.prepare(&device, &queue, 1200, 800).unwrap();
        let geom = p.overlay_geometry(1200);
        assert_eq!(
            geom.plan_labels_probe(),
            ["loc:Navigate"],
            "zoom {zoom}: the location line itself is missing, so an absent cue would \
             prove nothing about the park"
        );
        let with = shoot(&device, &queue, &mut p, 1200, 800);
        let mut b = command_view("Navigate");
        b.zoom = zoom;
        blank_location(&mut b);
        p.set_view(&b);
        p.prepare(&device, &queue, 1200, 800).unwrap();
        let without = shoot(&device, &queue, &mut p, 1200, 800);
        // THE PIXELS ARE THE ORACLE. The park is taken in either of two places
        // — the placement declines (no wordmark, no margin at all) or the
        // shared preparation measures the ⅔ run over its fit box and clears —
        // and which one fired is not the product claim. That the cue is ABSENT
        // rather than SHRUNK is.
        let ink = diff_ink(&with, &without, 1200, 800, (0, 1200), (0, 800));
        seen.push((zoom, ink.is_some()));
    }
    theme::set_active(theme::DEFAULT_THEME);
    assert_eq!(
        seen,
        [(1.7, true), (1.8, false)],
        "the park threshold moved: the cue must draw while the margin holds its ⅔ run and \
         be ABSENT once it does not — never shrunk into it"
    );
}

/// **WHY THE SWEEP IS LABEL-DRIVEN: two faceting kinds never show this cue at
/// all.** A summoned WORKSPACE stands its facet strip on end as a navigation
/// rail and plans no `PlanLine::Location`, so Settings' and History's own lenses
/// reach no rotated cue on any world. Their labels are still swept above — they
/// are the roster's longest — but pinned here so a reader knows the sweep's
/// shape is a fact rather than an oversight, and so promoting either kind to a
/// faceted card has to read this file.
#[test]
fn workspace_faceting_kinds_carry_no_location_line() {
    let mut workspaces = Vec::new();
    let mut cards = Vec::new();
    for kind in OverlayKind::ALL {
        if crate::facets::scheme(kind).is_none() {
            continue;
        }
        match kind.workspace_shape().is_some() {
            true => workspaces.push(format!("{kind:?}")),
            false => cards.push(format!("{kind:?}")),
        }
    }
    assert_eq!(
        (workspaces.as_slice(), cards.as_slice()),
        (
            ["History", "Settings"].map(String::from).as_slice(),
            ["Goto", "Project", "Browse", "Command"]
                .map(String::from)
                .as_slice()
        ),
        "the faceting roster's split between workspace and card shapes moved — item 297's \
         cue only ever draws on the card shapes"
    );
}

/// **THE ⅔ IS THE NUMBER, not merely whatever the constant says.** Every other
/// law here reads `ROTATED_RAIL_PLACARD_FRACTION` and so cannot tell two thirds
/// from any other fraction — a size relation asserted against its own constant
/// is a tautology, and the pixel band alone is a wide net. This is the value
/// itself, pinned: a scale class down, which is what makes the cue read as the
/// wordmark's companion rather than as a second title (1.0) or a caption
/// (≈0.3). The gap is pinned as a real fraction of an em for the same reason.
#[test]
#[allow(clippy::assertions_on_constants)] // the constants ARE the subject
fn the_wordmark_fraction_is_two_thirds_and_the_gap_is_a_real_em_fraction() {
    assert!(
        (ROTATED_RAIL_PLACARD_FRACTION - 2.0 / 3.0).abs() < 1e-6,
        "ROTATED_RAIL_PLACARD_FRACTION is {ROTATED_RAIL_PLACARD_FRACTION} — the composition \
         is two thirds of the wordmark, and the pixel band this file grades against \
         (0.58..0.98 of the wordmark's own ink) is calibrated for exactly that"
    );
    assert!(
        ROTATED_RAIL_PLACARD_GAP_EM > 0.0 && ROTATED_RAIL_PLACARD_GAP_EM < 0.5,
        "ROTATED_RAIL_PLACARD_GAP_EM ({ROTATED_RAIL_PLACARD_GAP_EM}) must be a real fraction \
         of the cue's own em: zero lets the pair read as one broken line, half pushes the \
         cue off the composition it belongs to"
    );
}

/// **THE CARD REACHES OUTSIDE ITS OWN BOX, AND THE CUE IS BOUNDED BY THE REACH.**
/// Under `Bars` on a right-anchored card the SELECTED row's plate grows OUTWARD
/// past `card_x` (`grow_span`, mirrored) and its scrim pads that again, so
/// `card_x` is not where the card's ink stops. This law is the witness for that
/// discovery, and it has three arms because the first two alone are satisfiable
/// by the defect:
///
/// 1. The span probe reports a left edge genuinely OUTSIDE the box — a bound
///    reverted to `card_x` reads zero here and fails by name.
/// 2. The plate's own leftmost PIXEL is outside the box too, so arm 1 is a fact
///    about the drawn card rather than about arithmetic, and the probe's bound is
///    at or left of it (conservative, never optimistic).
/// 3. The cue's own fit box ends at or before that bound, so the ⅔ run is
///    measured against where the card's ink stops rather than where its box does.
#[test]
fn the_cue_is_bounded_by_the_cards_drawn_reach_not_by_its_box() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping item297 card-reach law: no wgpu adapter");
        return;
    };
    let _pin = theme::WorldPin::world("Cassowary").expect("Cassowary ships");
    p.sync_theme();

    let mut graded = 0usize;
    for zoom in [1.0f32, 1.5] {
        let mut v = command_view("Navigate");
        v.zoom = zoom;
        p.set_view(&v);
        p.prepare(&device, &queue, 1200, 800).unwrap();
        let geom = p.overlay_geometry(1200);
        let plan = p.overlay_row_plan(&geom);
        let (card_left, _) = p.overlay_card_drawn_span_probe(&geom);
        let box_left = geom.band_x_probe();
        let reach = box_left - card_left;
        assert!(
            reach >= 10.0,
            "zoom {zoom}: the card's drawn span starts at {card_left:.1} against a box at \
             {box_left:.1} — only {reach:.1}px of outward reach, so the cue is being bounded \
             by the BOX and the selected plate's own growth is unaccounted for"
        );

        // Arm 2: the plate's leftmost pixel, off the frame itself.
        let selected = plan
            .rows()
            .iter()
            .find(|r| r.item == Some(0))
            .copied()
            .expect("the selected row is planned");
        let px = shoot(&device, &queue, &mut p, 1200, 800);
        let mid = (selected.top + selected.height * 0.5).round() as i64;
        let ground = luma(px[(mid * 1200 + (card_left * 0.5) as i64) as usize]);
        let plate_left = (0..1200)
            .find(|x| {
                let l = luma(px[(mid * 1200 + x) as usize]);
                *x as f32 >= card_left - 2.0 && (l - ground).abs() > 24.0
            })
            .map(|x| x as f32)
            .unwrap_or_else(|| panic!("zoom {zoom}: no selected plate ink on row {mid}"));
        assert!(
            plate_left < box_left - 1.0,
            "zoom {zoom}: the selected plate's leftmost ink is {plate_left:.0}, inside the \
             card box ({box_left:.1}) — this law's own premise (the plate grows outward) is \
             no longer true and the cue's bound should be revisited"
        );
        assert!(
            card_left <= plate_left + 1.0,
            "zoom {zoom}: the span probe's {card_left:.1} is RIGHT of the plate's own ink at \
             {plate_left:.0} — the bound is optimistic, not conservative"
        );

        // Arm 3: the cue's fit box respects it.
        let (_natural, fit, _bottom, flush_x) = p
            .rotated_rail_probe(&geom)
            .expect("the cue draws at these zooms");
        assert!(
            flush_x + fit[0] <= card_left + 0.01,
            "zoom {zoom}: the cue may occupy up to x={:.1} against a card reaching \
             {card_left:.1}",
            flush_x + fit[0]
        );
        graded += 1;
    }
    theme::set_active(theme::DEFAULT_THEME);
    assert_eq!(graded, 2, "the zoom sweep moved");
}
