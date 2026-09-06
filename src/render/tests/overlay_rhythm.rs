//! The shared overlay card's vertical-rhythm outcome laws.
//!
//! The first law sweeps the closed `OverlayKind` roster without a wildcard and
//! points at real shaped title/query, facet, candidate, and footer glyph runs,
//! then asks the production hit-test owners what each point means. The second
//! law measures the INSTRUCTION BAND at real GPU pixels: the hint's own drawn
//! ink sits CENTRED in the band that runs from the content band's bottom to
//! the card's own bottom edge, and that band stays close to a row tall.

use super::super::*;
use super::{headless_dqp, pixeldiff, view};

#[test]
fn explanatory_hint_yields_before_action_cells_at_narrow_width_on_both_dpis() {
    let hint = "type to filter   ↵ open   ←/→ lens   esc close";
    for dpi in [1.0, 2.0] {
        for (geometry, physical_w, expected) in [
            ("narrow", 720.0 * dpi, "↵ open   ←/→ lens   esc close"),
            ("ordinary", 1200.0 * dpi, hint),
            ("wide", 1600.0 * dpi, hint),
        ] {
            assert_eq!(
                crate::render::chrome::hint_yielding_explanation(hint, physical_w / dpi),
                expected,
                "{geometry} at {dpi}x makes the authored yield decision"
            );
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SurfaceContract {
    Flat,
    Faceted,
    Contextual,
}

/// Compile-time enrollment for the whole overlay roster. A new kind cannot
/// inherit an accidental title/facet/candidate/footer ordering.
fn surface_contract(kind: crate::overlay::OverlayKind) -> SurfaceContract {
    use crate::overlay::OverlayKind as K;
    match kind {
        K::Goto | K::Project | K::Browse | K::Command | K::History | K::Settings => {
            SurfaceContract::Faceted
        }
        K::Spell => SurfaceContract::Contextual,
        K::Theme
        | K::Caret
        | K::MoveDest
        | K::ExportDest
        // The switch-project door's navigator: one directory level, no lens
        // strip over the folders you are stepping through.
        | K::ProjectBrowse
        | K::Dictionary
        | K::CjkLang
        | K::Date
        | K::Keymap
        | K::Keybindings
        | K::Assets
        | K::Rename
        | K::InsertLink
        | K::KeepName
        // A workspace, but not a FACETED one: its three views are the whole
        // list, so there is no lens strip above them to order. Credits is the
        // same shape with one row instead of three.
        | K::Conflict
        | K::Credits
        | K::Context
        | K::TableDims
        | K::SearchFolder => SurfaceContract::Flat,
    }
}

fn mid(rect: [f32; 4]) -> (f32, f32) {
    (rect[0] + rect[2] * 0.5, rect[1] + rect[3] * 0.5)
}

fn mid_in_row(rect: [f32; 4], row_h: f32) -> (f32, f32) {
    (rect[0] + rect[2] * 0.5, rect[1] + rect[3].min(row_h) * 0.5)
}

fn point_inside(card: [f32; 4], q: (f32, f32)) -> bool {
    q.0 >= card[0] && q.0 <= card[0] + card[2] && q.1 >= card[1] && q.1 <= card[1] + card[3]
}

/// The TITLE/QUERY region's own law: present exactly when `header_rows > 0`,
/// drawn inside the card, and hit-testing as the query (never a candidate or
/// facet). Returns the new `previous_bottom` the next region orders against
/// (the card top, unchanged, when there is no header row at all).
fn assert_title_query_region(
    p: &TextPipeline,
    kind: crate::overlay::OverlayKind,
    card: [f32; 4],
    header_rows: usize,
) -> f32 {
    if header_rows == 0 {
        return card[1];
    }
    let title_query = p
        .overlay_line_glyph_box(0)
        .unwrap_or_else(|| panic!("{kind:?}: title/query glyphs must draw"));
    // The deliberate header gap lives in the shaped line's trailing
    // cell metrics. Probe its glyph-bearing row, not that blank tail.
    let q = mid_in_row(title_query, p.overlay_lh());
    assert!(
        point_inside(card, q),
        "{kind:?}: title/query glyphs stay inside the card"
    );
    assert!(
        p.over_overlay_query(q.0, q.1),
        "{kind:?}: a point on the drawn title/query line belongs to the query hit region"
    );
    assert_eq!(
        p.overlay_row_at(q.0, q.1),
        None,
        "{kind:?}: title/query glyphs never masquerade as a candidate"
    );
    assert_eq!(
        p.overlay_lens_at(q.0, q.1),
        None,
        "{kind:?}: title/query glyphs never masquerade as a facet"
    );
    title_query[1] + title_query[3]
}

/// The FACET STRIP region's own law: present only under
/// [`SurfaceContract::Faceted`], follows the title/query region, and its
/// active mark hits its own strip index (never a candidate or the query).
/// Returns the unchanged `previous_bottom` on every non-faceted kind.
fn assert_facet_region(
    p: &TextPipeline,
    kind: crate::overlay::OverlayKind,
    contract: SurfaceContract,
    previous_bottom: f32,
) -> f32 {
    if contract != SurfaceContract::Faceted {
        return previous_bottom;
    }
    let facet = p
        .overlay_line_glyph_box(1)
        .unwrap_or_else(|| panic!("{kind:?}: facet glyphs must draw"));
    assert!(
        facet[1] >= previous_bottom - 0.5,
        "{kind:?}: facet strip follows the title/query line"
    );
    let [ux, _uy, uw, _uh] = p
        .overlay_theme_underline
        .expect("the active facet is visibly marked");
    let facet_point = (ux + uw * 0.5, facet[1] + facet[3] * 0.5);
    assert_eq!(
        p.overlay_lens_at(facet_point.0, facet_point.1),
        Some(1),
        "{kind:?}: the active drawn facet hits its own strip index"
    );
    assert_eq!(
        p.overlay_row_at(facet_point.0, facet_point.1),
        None,
        "{kind:?}: facet glyphs never masquerade as candidates"
    );
    assert!(
        !p.over_overlay_query(facet_point.0, facet_point.1),
        "{kind:?}: facet glyphs are below the query hit region"
    );
    facet[1] + facet[3]
}

/// The CANDIDATE ROWS region's own law: both seeded candidates follow the
/// preceding region and hit-test as their own row index (never the facet
/// strip or the query). Returns the bottom of the last candidate drawn.
fn assert_candidate_rows(
    p: &TextPipeline,
    kind: crate::overlay::OverlayKind,
    first_candidate_line: usize,
    previous_bottom: f32,
) -> f32 {
    let mut previous_bottom = previous_bottom;
    for row in 0..2usize {
        let line = first_candidate_line + row;
        let candidate = p
            .overlay_line_glyph_box(line)
            .unwrap_or_else(|| panic!("{kind:?}: candidate {row} glyphs must draw"));
        assert!(
            candidate[1] >= previous_bottom - 0.5,
            "{kind:?}: candidate {row} follows the preceding semantic region"
        );
        let q = mid(candidate);
        assert_eq!(
            p.overlay_row_at(q.0, q.1),
            Some(row),
            "{kind:?}: drawn candidate {row} hits the same candidate"
        );
        assert_eq!(
            p.overlay_lens_at(q.0, q.1),
            None,
            "{kind:?}: candidate {row} never hits the facet strip"
        );
        assert!(
            !p.over_overlay_query(q.0, q.1),
            "{kind:?}: candidate {row} never hits the query"
        );
        previous_bottom = candidate[1] + candidate[3];
    }
    previous_bottom
}

/// The FOOTER region's own law: absent under
/// [`SurfaceContract::Contextual`] (the word-anchored popup's geometry never
/// draws one) and absent on any kind whose product hint is empty (today only
/// the pointer-anchored context menu, `SurfaceContract::Flat` in every other
/// respect but with no teaching line to draw) — otherwise follows every
/// candidate, draws inside the card, hit-tests as inert (neither a
/// candidate, a facet, nor the query), and never spills past the card's own
/// bottom.
fn assert_footer_region(
    p: &TextPipeline,
    kind: crate::overlay::OverlayKind,
    contract: SurfaceContract,
    card: [f32; 4],
    first_candidate_line: usize,
    last_candidate_bottom: f32,
) {
    if contract == SurfaceContract::Contextual || kind.hint().is_empty() {
        return;
    }
    // `+ 1` for the blank separator row `overlay_hint_gap_rows`
    // reserves ahead of the hint's own line (`+ 2` is the two candidate
    // rows `assert_candidate_rows` just drew).
    let footer_line = first_candidate_line + 2 + 1;
    let footer = p
        .overlay_line_glyph_box(footer_line)
        .unwrap_or_else(|| panic!("{kind:?}: footer glyphs must draw"));
    assert!(
        footer[1] >= last_candidate_bottom - 0.5,
        "{kind:?}: footer follows every candidate"
    );
    let q = mid(footer);
    assert!(
        point_inside(card, q),
        "{kind:?}: footer glyphs stay inside the card"
    );
    assert_eq!(
        p.overlay_row_at(q.0, q.1),
        None,
        "{kind:?}: inert footer is not candidate-clickable"
    );
    assert_eq!(
        p.overlay_lens_at(q.0, q.1),
        None,
        "{kind:?}: inert footer is not a facet"
    );
    assert!(
        !p.over_overlay_query(q.0, q.1),
        "{kind:?}: inert footer is not the query"
    );
    assert!(
        footer[1] + footer[3] <= card[1] + card[3] + 0.5,
        "{kind:?}: footer stays above the card bottom"
    );
}

/// TITLE/QUERY → optional FACET → CANDIDATES → FOOTER, over every kind.
/// Points come from the shaped glyph buffer the draw uploads; classification
/// comes from the production pointer doors. This is intentionally stronger
/// than the older row-y agreement: every semantic surface must both draw and
/// occupy exactly its own interactive (or deliberately inert) region.
#[test]
fn every_overlay_kind_orders_drawn_title_facet_candidates_footer_and_hits_the_same_regions() {
    let _g = crate::testlock::serial();
    let (w, h) = (1200u32, 800u32);
    let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
        eprintln!(
            "skipping every_overlay_kind_orders_drawn_title_facet_candidates_footer_and_hits_the_same_regions: no wgpu adapter"
        );
        return;
    };
    let _g = crate::testlock::serial();
    theme::set_active_by_name("Tawny").unwrap();
    p.sync_theme();

    for kind in crate::overlay::OverlayKind::ALL {
        let contract = surface_contract(kind);
        let mut v = view("teh\n", 0, 0);
        v.overlay_active = true;
        v.overlay_title = if kind.draws_title_prefix() {
            kind.title().to_string()
        } else {
            "".to_string()
        };
        v.overlay_items = vec!["Alpha candidate".into(), "Omega candidate".into()];
        v.overlay_selected = 0;
        v.overlay_window_rows = kind.window_rows();
        match contract {
            SurfaceContract::Faceted => {
                v.overlay_lens = vec![("All".into(), false), ("Facet".into(), true)];
                v.overlay_hint = kind.hint();
            }
            SurfaceContract::Flat => v.overlay_hint = kind.hint(),
            SurfaceContract::Contextual => {
                v.overlay_spell = Some((0, 0, 3));
                v.overlay_hint.clear();
                v.overlay_title = "".to_string();
            }
        }
        p.set_view(&v);
        p.prepare(&device, &queue, w, h).unwrap();

        let card = p.overlay_card_rect().expect("every kind draws a card");

        let header_rows = match contract {
            SurfaceContract::Contextual => 0usize,
            SurfaceContract::Flat => 1,
            SurfaceContract::Faceted => 2,
        };
        // WHICH shaped line carries candidate 0 is production's own answer, not
        // a count of header rows: a flat card's BEAT takes its own glyph-free
        // line between the query field and the candidates.
        let first_candidate_line = p.overlay_geometry(w).shaped_first_row_line();

        // TITLE/QUERY → optional FACET → CANDIDATES → FOOTER, each region's
        // own law asked in drawn order and threading the running
        // `previous_bottom` the next region orders against.
        let previous_bottom = assert_title_query_region(&p, kind, card, header_rows);
        let previous_bottom = assert_facet_region(&p, kind, contract, previous_bottom);
        let last_candidate_bottom =
            assert_candidate_rows(&p, kind, first_candidate_line, previous_bottom);
        assert_footer_region(
            &p,
            kind,
            contract,
            card,
            first_candidate_line,
            last_candidate_bottom,
        );
    }

    theme::set_active(theme::DEFAULT_THEME);
}

/// The hint's REAL INK band inside `y_rect`, plus its mass-weighted vertical
/// centroid, as `(top, bottom, centroid)` in canvas pixels.
///
/// Scanned across the CARD'S OWN width rather than the shaped line's x-span:
/// `overlay_line_glyph_box` reports the run's box in the panel buffer's own
/// coordinates, which a right-ALIGNED world (Magpie) draws somewhere else
/// entirely — its y is always right, its x is not, and an x-scoped probe reads
/// blank card there and reports no hint at all. The background is re-derived
/// PER ROW (the modal colour of that row inside the card) so a world's own
/// vertical gradient or plate is absorbed instead of counted as ink, and the
/// scan is inset from the card's edges so a footer plate's own border is not
/// mistaken for a glyph.
fn hint_ink_band(
    pixels: &[[u8; 4]],
    width: u32,
    height: u32,
    y_rect: [f32; 4],
    card: [f32; 4],
) -> Option<(i32, i32, f32)> {
    use std::collections::HashMap;
    const EDGE_INSET: f32 = 6.0;
    const INK_DELTA: u8 = 24;
    const MIN_INK_PX: usize = 4;
    let x0 = (card[0] + EDGE_INSET).floor().max(0.0) as u32;
    let x1 = (card[0] + card[2] - EDGE_INSET)
        .ceil()
        .clamp(0.0, width as f32) as u32;
    let y0 = y_rect[1].floor().max(0.0) as u32;
    let y1 = (y_rect[1] + y_rect[3]).ceil().clamp(0.0, height as f32) as u32;
    let (mut top, mut bottom) = (i32::MAX, i32::MIN);
    let (mut mass, mut moment) = (0.0f64, 0.0f64);
    for y in y0..y1 {
        let mut hist: HashMap<[u8; 3], usize> = HashMap::new();
        for x in x0..x1 {
            let p = pixels[(y * width + x) as usize];
            *hist.entry([p[0], p[1], p[2]]).or_default() += 1;
        }
        let Some(bg) = hist.iter().max_by_key(|(_, n)| **n).map(|(c, _)| *c) else {
            continue;
        };
        let n = (x0..x1)
            .filter(|&x| {
                let p = pixels[(y * width + x) as usize];
                p[0].abs_diff(bg[0])
                    .max(p[1].abs_diff(bg[1]))
                    .max(p[2].abs_diff(bg[2]))
                    > INK_DELTA
            })
            .count();
        if n >= MIN_INK_PX {
            top = top.min(y as i32);
            bottom = bottom.max(y as i32);
        }
        mass += n as f64;
        moment += n as f64 * y as f64;
    }
    (top <= bottom).then(|| (top, bottom, (moment / mass.max(1.0)) as f32))
}

/// A picker-shaped fixture for the instruction-band law: `n` candidates with
/// the selection on the last one (the row nearest the band), an authored
/// `hint`, and — under `grouped` — the lens strip and contiguous section runs
/// that route the card through the GROUPED geometry owner instead of the flat
/// one.
fn band_view(n: usize, hint: &str, grouped: bool) -> ViewState {
    let mut v = view("hello\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = "themes".to_string();
    v.overlay_items = (0..n).map(|i| format!("Candidate {i}")).collect();
    v.overlay_selected = n.saturating_sub(1);
    v.overlay_hint = hint.to_string();
    if grouped {
        v.overlay_lens = vec![("All".into(), true), ("File".into(), false)];
        v.overlay_sections = (0..n)
            .map(|i| match i * 3 / n.max(1) {
                0 => "Alpha".to_string(),
                1 => "Beta".to_string(),
                _ => "Gamma".to_string(),
            })
            .collect();
    }
    v
}

/// THE INSTRUCTION BAND LAW — the foot hint's own box is BALANCED and SHORT.
///
/// The band the eye reads as the instruction box runs from
/// `OverlayRowPlan::footer_top` (where the candidate band ends, and where a
/// plated world seats the footer plate) to the card's own bottom edge. Two
/// claims about it, each with the companion that stops it being satisfied by
/// the subject going missing:
///
/// * **CENTRED.** The hint's drawn INK sits at the band's own centre, within a
///   fraction of a row. Measured as the ink's mass-weighted centroid rather
///   than its extents: an extent is set by a single ascender or descender
///   pixel and moves with the hint string's own glyph set (`↵`, `⌫` and `⇥`
///   are drawn at their own heights per face), while the centroid is stable
///   across faces, strings and antialiasing. A second, independently-noisy
///   form of the same claim reads the shaped LINE BOX's own seat in the band,
///   so the two arms cannot both be fooled by the same artefact.
/// * **SHORT.** The band stays under one and two-thirds of a row.
///
/// This REPLACES the retired "clear air above, trim the chin, reject the old
/// dials" law, which asserted the opposite balance — a separator sized well
/// above the chin — and so was satisfied by the top-heavy band the product
/// actually shipped. Its `0.62`-row / `5px` counterfactual went with it: those
/// two retirements shift the gap and the chin by the SAME constant and say
/// nothing about which of them is larger.
///
/// Enrolment is the whole shipped world roster (derived, never named), swept
/// over four window geometries including one narrow enough to make the hint
/// yield its explanation and one short enough to clamp the list, both card
/// families, two hint lengths, and both a few-candidate and a
/// window-clamped-many candidate shape.
/// Tolerances, every one a fraction of the world's OWN row pitch — never of an
/// authored constant, and never of a pixel count measured on this host.
const BAND_CENTRE_TOL: f32 = 0.18;
const BAND_BOX_SEAT_TOL: f32 = 0.20;
const BAND_MAX_ROWS: f32 = 1.65;
const BAND_MIN_ROWS: f32 = 1.20;
const BAND_PAD_MIN: f32 = 0.25;
const BAND_INK_MIN: f32 = 0.18;

/// Grade ONE cell of the instruction-band sweep against an already-rendered
/// frame: the presence companions first, then the two centring arms and the
/// compactness claim.
fn grade_instruction_band(p: &TextPipeline, pixels: &[[u8; 4]], w: u32, h: u32, ctx: &str) {
    let geom = p.overlay_geometry(w);
    let plan = p.overlay_row_plan(&geom);
    let band_top = plan.footer_top();
    let card = p
        .overlay_card_rect()
        .unwrap_or_else(|| panic!("{ctx}: the picker card must be placed"));
    let band_bottom = card[1] + card[3];
    let band = band_bottom - band_top;
    let lh = p.overlay_lh();
    let hint_line = p
        .overlay_hint_line()
        .unwrap_or_else(|| panic!("{ctx}: this fixture always sets a hint"));
    let hbox = p
        .overlay_line_glyph_box(hint_line)
        .unwrap_or_else(|| panic!("{ctx}: the hint's own line must shape"));
    let (ink_top, ink_bottom, centroid) =
        hint_ink_band(pixels, w, h, hbox, card).unwrap_or_else(|| {
            panic!(
                "{ctx}: no hint ink drawn anywhere across the card at the hint's own line \
                 ({hbox:?}) — the band has no subject"
            )
        });
    let above = ink_top as f32 - band_top;
    let below = band_bottom - ink_bottom as f32;
    let ink_h = (ink_bottom - ink_top + 1) as f32;

    // PRESENCE. Both claims below get happier as the hint fades or the band
    // collapses, so pin the subject shut first — each floor set well under the
    // tightest value the shipped roster actually produces.
    assert!(
        ink_h >= lh * BAND_INK_MIN,
        "{ctx}: the hint's own ink is only {ink_h}px tall at {lh:.1}px row pitch — a centred \
         band would be satisfied by the hint disappearing"
    );
    assert!(
        above >= lh * BAND_PAD_MIN && below >= lh * BAND_PAD_MIN,
        "{ctx}: the hint's ink has no real air on both sides (above {above:.1}px, below \
         {below:.1}px, floor {:.1}px at {lh:.1}px row pitch)",
        lh * BAND_PAD_MIN
    );
    assert!(
        band >= lh * BAND_MIN_ROWS,
        "{ctx}: the instruction band is only {band:.1}px at {lh:.1}px row pitch — the SHORT \
         claim below would be satisfied by the band collapsing rather than by the separator \
         being trimmed"
    );

    // CENTRED, arm 1: real ink, mass-weighted.
    let centre_off = centroid - (band_top + band_bottom) * 0.5;
    assert!(
        centre_off.abs() <= lh * BAND_CENTRE_TOL,
        "{ctx}: the hint's ink centres {centre_off:.1}px off the instruction band's own centre \
         (tolerance {:.1}px at {lh:.1}px row pitch; ink [{ink_top}, {ink_bottom}] in band \
         [{band_top:.1}, {band_bottom:.1}]) — the band is {} and reads as a mistake rather \
         than a considered pause",
        lh * BAND_CENTRE_TOL,
        if centre_off > 0.0 {
            "top-heavy"
        } else {
            "bottom-heavy"
        }
    );
    // CENTRED, arm 2: the shaped line box's own seat. A different statistic
    // with different noise, so the pair cannot share an artefact.
    let box_above = hbox[1] - band_top;
    let box_below = band_bottom - (hbox[1] + hbox[3]);
    assert!(
        (box_above - box_below).abs() <= lh * BAND_BOX_SEAT_TOL,
        "{ctx}: the hint's own line box is seated {box_above:.1}px below the content band but \
         {box_below:.1}px above the card's edge (tolerance {:.1}px at {lh:.1}px row pitch)",
        lh * BAND_BOX_SEAT_TOL
    );

    // SHORT.
    assert!(
        band <= lh * BAND_MAX_ROWS,
        "{ctx}: the instruction band is {band:.1}px — {:.2} rows at {lh:.1}px row pitch, past \
         the {BAND_MAX_ROWS} rows the authored outcome allows",
        band / lh
    );
}

#[test]
fn the_hint_card_centres_its_line_in_a_band_close_to_one_row() {
    let _g = crate::testlock::serial();
    let geometries = [
        ("canonical 1200x800@1", 1200u32, 800u32, 1.0f32),
        ("retina 2400x1600@2", 2400, 1600, 2.0),
        ("narrow 900x600@1", 900, 600, 1.0),
        ("short 1100x460@1", 1100, 460, 1.0),
    ];
    let hints = [
        ("short", "↵ keep   esc revert"),
        (
            "long",
            "type to filter   ↵ keep   ⌫ clear   ⇥ lens   esc revert",
        ),
    ];
    let worlds = crate::theme::world_names();
    assert!(
        worlds.len() > 8,
        "the world roster must supply this law's enrolment, got {}",
        worlds.len()
    );
    let mut graded = 0usize;
    let mut per_axis: std::collections::BTreeMap<String, usize> = Default::default();
    for (gname, w, h, dpi) in geometries {
        let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
            eprintln!(
                "skipping the_hint_card_centres_its_line_in_a_band_close_to_one_row: \
                 no wgpu adapter"
            );
            return;
        };
        for world in &worlds {
            theme::set_active_by_name(world).unwrap();
            p.sync_theme();
            p.set_dpi(dpi);
            p.set_size(w as f32, h as f32);
            for grouped in [false, true] {
                for (hname, hint) in hints {
                    for (sname, n) in [("few", 3usize), ("clamped-many", 60)] {
                        let family = if grouped { "grouped" } else { "flat" };
                        let ctx = format!("{world} / {gname} / {family} / {hname} hint / {sname}");
                        p.set_view(&band_view(n, hint, grouped));
                        p.prepare(&device, &queue, w, h).unwrap();
                        let pixels = pixeldiff::render_frame(&mut p, &device, &queue, w, h);
                        grade_instruction_band(&p, &pixels, w, h, &ctx);
                        graded += 1;
                        for key in [
                            gname.to_string(),
                            family.to_string(),
                            format!("{hname} hint"),
                            sname.to_string(),
                        ] {
                            *per_axis.entry(key).or_default() += 1;
                        }
                    }
                }
            }
        }
        p.set_dpi(1.0);
    }
    theme::set_active(theme::DEFAULT_THEME);
    assert!(
        graded > 400,
        "the instruction-band sweep must actually run, got {graded} cells"
    );
    for (axis, n) in &per_axis {
        assert!(
            *n > 20,
            "axis value {axis:?} was reached only {n} times — the sweep did not cover it \
             (graded {graded} cells: {per_axis:?})"
        );
    }
}
