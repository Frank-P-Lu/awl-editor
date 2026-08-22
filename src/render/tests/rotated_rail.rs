//! CASSOWARY'S LOCATION CUE IS A SUBORDINATE TECHNICAL LOCATOR.
//!
//! **Defect:** `LocationStyle::RotatedRail` drew the active facet's name small,
//! muted and flush with the CARD's own left border — a whisper in the card's
//! gutter, which is neither the card's voice nor the room's.
//!
//! **Build:** rotated 90° along the ROOM's own outer margin (the margin the
//! Archivo-Black wordmark placard already keeps), seated just ABOVE that
//! placard, but authored independently as Iosevka Regular, 0.28 placard scale,
//! muted, 0.06em tracked, and truthfully indexed (`03 / NAVIGATE`).
//!
//! The laws here grade the four claims that composition makes, and every
//! appearance claim is arithmetic over real GPU pixels (the sidecar reports
//! state, never whether anything is visible):
//!
//! - **HIERARCHY**, from both sides, at both DPI tiers — asserted on the size
//!   the product actually decided AND on the ink that reached the screen.
//! - **PRESENCE**, so the non-overlap laws below cannot be satisfied by a cue
//!   that faded into the ground or shrank toward nothing: the run's ink is as
//!   visibly weaker than the wordmark, with a real ground-separation floor.
//! - **NON-OVERLAP AND NO CLIPPING** against the placard's own line box, the
//!   card's TRUE drawn span (wider than its box — the selected plate grows
//!   outward past it) and the canvas edge.
//! - **THE PARK ARM**: where the margin cannot hold the authored run, the cue is
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
    LOCATION_SCALE, ROTATED_RAIL_PLACARD_GAP_EM, active_location_index, format_location_text,
    placard_font_size,
};

fn rail_style() -> theme::LocationLabelStyle {
    let cassowary = theme::THEMES
        .iter()
        .find(|world| world.name == "Cassowary")
        .expect("Cassowary ships");
    let theme::LocationStyle::RotatedRail(style) = cassowary.render_caps.location_style else {
        panic!("Cassowary must carry the rotated rail");
    };
    style
}

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
        label: Some("awl rotated-rail encoder"),
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
        .filter(|t| {
            matches!(
                t.render_caps.location_style,
                theme::LocationStyle::RotatedRail(_)
            )
        })
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
    for t in theme::THEMES.iter().filter(|t| {
        matches!(
            t.render_caps.location_style,
            theme::LocationStyle::RotatedRail(_)
        )
    }) {
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
// THE SUBORDINATE RELATION, PRESENCE FLOORS, AND NON-OVERLAP — real pixels.
// ---------------------------------------------------------------------------

/// **THE CUE IS SUBORDINATE TO THE WORDMARK, VISIBLY THERE, AND TOUCHES
/// NOTHING** — swept over every faceting kind × every lens × both DPI tiers.
///
/// Five claims per cell, and the middle two exist because the outer ones get
/// HAPPIER as the cue disappears:
///
/// 1. **Scale from both sides**, on the size the product decided: the cue's own
///    natural size against the placard's, to a tenth of a percent.
/// 2. **PRESENCE by extent**: the drawn ink's across-axis extent sits in the
///    band a 0.28-scale mono locator predicts against Archivo Black.
/// 3. **PRESENCE by strength**: the cue clears the ground while staying below
///    the bold wordmark, so `COMMANDS` remains the lone poster headline.
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
fn rotated_rail_is_subordinate_to_the_wordmark_present_and_clear_of_everything() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping rail composition law: no wgpu adapter");
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
/// `rotated_rail_is_subordinate_to_the_wordmark_present_and_clear_of_everything`
/// — split out so the sweep above stays a visible loop over the cases graded
/// rather than the grading itself. Returns whether the cue DREW: a cell whose
/// margin cannot hold the authored run belongs to the park law, not to this one.
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

    // (1) THE AUTHORED HIERARCHY, from above and below.
    let style = rail_style();
    let want = placard_font_size(placard_h) * style.scale;
    assert!(
        (natural - want).abs() <= want * 1e-3,
        "{cell}: cue type is {natural:.2}px against a wordmark of \
         {:.2}px — the authored {:.2} relation wants {want:.2}px",
        placard_font_size(placard_h),
        style.scale
    );
    let card_sized =
        p.metrics.font_size * crate::render::effective_overlay_scale() * LOCATION_SCALE;
    let headline_ceiling = placard_font_size(placard_h) * 0.4;
    assert!(
        natural > card_sized && natural < headline_ceiling,
        "{cell}: cue type {natural:.2}px must sit between the card readout ({card_sized:.2}px) \
         and 40% of the placard ({headline_ceiling:.2}px) — a readable locator, never a \
         second headline"
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

    // (2) PRESENCE BY EXTENT — the authored scale/face band.
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
        (0.18..=0.48).contains(&ratio),
        "{cell}: the cue's ink is {across:.0}px across against a {mark_h:.0}px \
         wordmark (ratio {ratio:.3}) — the authored 0.28-scale Iosevka locator \
         must remain visibly subordinate to Archivo Black without collapsing"
    );
    let area = across * (y1 - y0 + 1) as f32;
    assert!(
        count as f32 >= 0.10 * area,
        "{cell}: only {count} ink pixels in a {area:.0}px box — the cue drew an \
         outline or a hairline, not a run of type"
    );

    // (3) PRESENCE BY STRENGTH — muted locator below the bold wordmark.
    assert!(
        peak >= ground + 28.0 && peak <= 0.82 * mark_peak,
        "{cell}: the cue's strongest pixel is {peak:.1} (ground {ground:.1}) against \
         the wordmark's {mark_peak:.1} — it must be present in muted ink and weaker \
         than the lone poster headline"
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

/// **WHERE THE MARGIN CANNOT HOLD THE AUTHORED RUN THE CUE IS ABSENT, NOT SMALLER.**
/// The zoom sweep grows the widest card until its drawn left edge closes the
/// room's margin.
/// Graded on real pixels — zero cue ink in the whole margin — and proven to be
/// the PARK rather than a missing location line (the plan still carries one).
///
/// NON-VACUITY is the lower-zoom arm: the same cell draws before it parks.
#[test]
fn rotated_rail_parks_rather_than_shrinking_when_the_rooms_margin_closes() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping rail park law: no wgpu adapter");
        return;
    };
    let _pin = theme::WorldPin::world("Cassowary").expect("Cassowary ships");
    p.sync_theme();

    let mut seen = Vec::new();
    for zoom in [1.0f32, 1.8, 2.4] {
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
        // shared preparation measures the authored run over its fit box and clears —
        // and which one fired is not the product claim. That the cue is ABSENT
        // rather than SHRUNK is.
        let ink = diff_ink(&with, &without, 1200, 800, (0, 1200), (0, 800));
        seen.push((zoom, ink.is_some()));
    }
    theme::set_active(theme::DEFAULT_THEME);
    assert!(
        seen.first().is_some_and(|(_, drawn)| *drawn) && seen.iter().any(|(_, drawn)| !drawn),
        "the zoom sweep must cross from a present locator into a parked one without \
         shrinking it: {seen:?}"
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

/// The hierarchy is literal theme data, not a renderer constant that silently
/// makes every carrier read the same. The companion pixel law above checks the
/// outcome; this law checks the authored mechanism and tracking seam.
#[test]
#[allow(clippy::assertions_on_constants)] // the constants ARE the subject
fn cassowarys_locator_typography_is_authored_as_theme_data() {
    let style = rail_style();
    assert!(
        (style.scale - 0.28).abs() < 1e-6,
        "Cassowary's locator scale is {} — it must remain a compact telemetry line, \
         not the retired two-thirds headline echo",
        style.scale
    );
    assert_eq!(style.face, theme::LocationFace::Mono);
    assert_eq!(
        style.ink,
        theme::LocationInk::Flat(theme::PaletteRole::Muted)
    );
    assert!((style.tracking_em - 0.06).abs() < 1e-6);
    assert_eq!(
        style.locator,
        theme::LocationLocator::IndexOnly { digits: 2 }
    );
    assert!(
        ROTATED_RAIL_PLACARD_GAP_EM > 0.0 && ROTATED_RAIL_PLACARD_GAP_EM < 0.5,
        "ROTATED_RAIL_PLACARD_GAP_EM ({ROTATED_RAIL_PLACARD_GAP_EM}) must be a real fraction \
         of the cue's own em: zero lets the pair read as one broken line, half pushes the \
         cue off the composition it belongs to"
    );
}

/// The locator number is the active lens's REAL one-based position in its
/// scheme, never a label-derived or Cassowary-specific fiction.
#[test]
fn index_only_locator_is_truthful_for_every_faceted_lens() {
    let style = rail_style();
    let mut graded = 0usize;
    for kind in OverlayKind::ALL {
        let Some(scheme) = crate::facets::scheme(kind) else {
            continue;
        };
        for (zero_based, facet) in scheme.strip.iter().enumerate().skip(1) {
            let strip = scheme.strip_labels(zero_based);
            let got = format_location_text(style, facet.label, active_location_index(&strip))
                .expect("a real indexed lens must format");
            assert_eq!(
                got,
                format!("{:02}", zero_based + 1),
                "{kind:?} lens {zero_based}: locator disagrees with the strip"
            );
            graded += 1;
        }
    }
    assert!(graded >= 12, "the faceted-lens roster unexpectedly shrank");
    assert!(
        format_location_text(style, "Navigate", None).is_none(),
        "an indexed treatment with no real strip position must park, not fabricate 03"
    );
}

/// Cassowary's unified Pane no longer grows selected Bars beyond the card box.
/// The drawn-span owner must therefore resolve to the pane's real left edge,
/// and the rotated index must finish before that edge at every enrolled zoom.
#[test]
fn the_cue_is_bounded_by_the_unified_panes_drawn_left_edge() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping rail card-reach law: no wgpu adapter");
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
        let (card_left, _) = p.overlay_card_drawn_span_probe(&geom);
        let box_left = geom.band_x_probe();
        assert!(
            (box_left - card_left).abs() <= 0.01,
            "zoom {zoom}: unified Pane drawn edge {card_left:.1} drifted from box {box_left:.1}"
        );
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
