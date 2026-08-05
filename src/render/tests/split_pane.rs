//! THE SPLIT-PANE COMPOSITION round's law suite (queue item 50). A summoned
//! [`theme::ListStyle::Pane`] world's takeover card composes as TWO surfaces —
//! the title/query INPUT above, a visible strip of the world's own background
//! BETWEEN, then ONE lower result ROOM (facets/section-headers + candidate rows +
//! footer) — the DEFAULT for every Pane world; Cassowary opts back to the
//! historical UNIFIED single room as one-line DATA. A [`theme::ListStyle::Bars`]
//! world keeps its per-row plates (the split is inert), and the contextual spell
//! popup is never split.
//!
//! The decision is DATA through the ONE owner
//! [`crate::render::effective_pane_split`] (the `theme_caps_law` grep-law bans a
//! world name in `src/render/`); this file proves the OUTCOME over real pixels
//! (the Wagtail tripwire — appearance is asserted by pixel arithmetic, never the
//! sidecar/count alone): real ground shows across the FULL inter-surface gap, and
//! NO lower glyph escapes above its lower surface. The geometry owner
//! The plan's own `split_bounds` is unit-tested pure; the exhaustive
//! surface-roster law sweeps every world with a no-wildcard cap match.

use super::super::*;
use super::{headless_dqp, pixeldiff, view};

// --- pure geometry: the plan's own `split_bounds` -----------------------------

/// The gap sits ENTIRELY inside the query beat's negative space, above the first
/// candidate row, for both the flat and faceted header layouts — so no text moves
/// and (proven by the pixel laws below) no glyph falls in it.
#[test]
fn split_bounds_carve_the_query_beat_above_the_first_row() {
    let (text_top, lh, hg) = (64.0_f32, 27.2_f32, 35.0_f32);
    // No header (spell popup) or a zero beat → never split.
    let plan = crate::render::plan::test_header_plan;
    assert_eq!(plan(text_top, 0, hg, lh).split_bounds(), None);
    assert_eq!(plan(text_top, 1, 0.0, lh).split_bounds(), None);

    for header_rows in [1usize, 2] {
        let (gt, gb) = plan(text_top, header_rows, hg, lh).split_bounds().unwrap();
        // The gap is a real, positive band.
        assert!(gb > gt, "gap is non-degenerate (header_rows={header_rows})");
        // It is `SPLIT_GAP_FRAC` (0.4) of the beat tall — a strip of ground, not
        // the whole beat.
        assert!(
            (gb - gt - hg * 0.4).abs() < 1e-3,
            "gap height is 0.4 of the query beat (got {})",
            gb - gt
        );
        // The whole gap sits ABOVE the first candidate row's top (nothing below
        // the beat is touched) and BELOW the query line's own top.
        let first_row_top = crate::render::plan::test_row_top(text_top, header_rows, hg, 0, lh);
        assert!(
            gb <= first_row_top + 1e-3,
            "gap ends at/above the first row"
        );
        assert!(
            gt >= text_top + lh - 1e-3,
            "gap starts at/below the query line"
        );
        assert!(gt > text_top, "the upper surface is non-empty");
    }
}

/// ITEM 83 — THE FACETED QUERY OPTICAL-CENTERING LAW: the query text/caret
/// never move (pinned to `text_top`; `overlay_query_center`'s own contract), but
/// the DRAWN upper surface historically carried the card's `pad` (12px)
/// breathing room ABOVE them and NONE below (`[card_y, text_top + line_height]`
/// exactly) — so the query read bottom-heavy inside its own strip (Quokka's
/// command palette, the reported specimen: the eye reads generous headroom
/// above "commands › |" and almost none below, before the visible gap). Proven
/// NON-VACUOUS: the pre-fix offset is reconstructed inline from the historical
/// formula (not read back from the fix) and asserted to be the bigger one.
#[test]
fn faceted_query_strip_is_optically_centered_not_bottom_heavy() {
    let pad = 12.0_f32;
    let (line_height, header_gap) = (27.2_f32, 35.0_f32);
    let card_y = 100.0_f32;
    let text_top = card_y + pad;
    let header_rows = 2;

    let (gap_top, gap_bottom) =
        crate::render::plan::test_header_plan(text_top, header_rows, header_gap, line_height)
            .split_bounds()
            .unwrap();
    assert!(gap_bottom > gap_top, "a real, non-degenerate gap");

    // The query text's own (UNMOVED) vertical center — `overlay_query_center`'s
    // exact formula for the faceted (plain, uninflated query-line) case.
    let text_center = text_top + line_height * 0.5;

    // The drawn upper surface is `overlay_pane_fills`'s exact rect for this
    // branch: `[card_y, gap_top]`.
    let surface_center = (card_y + gap_top) * 0.5;
    let fixed_offset = (text_center - surface_center).abs();

    // NON-VACUOUS: the PRE-FIX surface (`[card_y, text_top + line_height]` — zero
    // breathing below the query box, reconstructed from the documented
    // pre-item-83 formula, not from the fix under test) centred here instead.
    let pre_fix_bottom = text_top + line_height;
    let pre_fix_center = (card_y + pre_fix_bottom) * 0.5;
    let pre_fix_offset = (text_center - pre_fix_center).abs();
    assert!(
        (pre_fix_offset - pad / 2.0).abs() < 1e-3,
        "sanity: the historical offset is exactly pad/2 ({pre_fix_offset} vs {})",
        pad / 2.0
    );

    assert!(
        fixed_offset < pre_fix_offset * 0.5,
        "the query must read centred, not bottom-heavy: fixed offset ({fixed_offset}) \
         should be well under half the pre-fix skew ({pre_fix_offset})"
    );

    // The added breathing margin stays inside the split gap's own proven-safe
    // budget (`SPLIT_GAP_FRAC`'s own doc): the gap band itself is UNCHANGED in
    // width (still exactly `0.4 * header_gap`) — only its position shifted.
    assert!(
        (gap_bottom - gap_top - header_gap * 0.4).abs() < 1e-3,
        "the visible gap band keeps its historical width, only its position moved"
    );
}

// --- `overlay_pane_fills`: the fill rects the card draws ----------------------

/// Build an open Pane picker view. `faceted` adds the lens strip; `n` candidate
/// rows (0 = the empty-state message row).
fn picker(faceted: bool, n: usize) -> ViewState {
    let mut v = view("hello world this is the page behind the card\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = "themes";
    v.overlay_items = (0..n).map(|i| format!("Command {i}")).collect();
    v.overlay_selected = if n == 0 { 0 } else { (n - 1).min(3) };
    if faceted {
        v.overlay_lens = vec![
            ("All".into(), true),
            ("File".into(), false),
            ("Recent".into(), false),
        ];
    }
    v
}

/// SPLIT (the default) draws TWO fill rects; UNIFIED draws ONE full-card rect —
/// and the DECISION is the cap (data): forcing the override flips the count with
/// nothing else changed. The two split surfaces are non-overlapping, both inside
/// the card, and leave exactly the `overlay_split_bounds` gap between them.
#[test]
fn split_draws_two_surfaces_unified_draws_one() {
    let _g = crate::testlock::serial();
    let (w, h) = (1200u32, 800u32);
    let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
        eprintln!("skipping split_draws_two_surfaces: no wgpu adapter");
        return;
    };
    let _g = crate::testlock::serial();
    set_card_anchor_test_override(Some(theme::CardAnchor::TopLeft));

    for faceted in [false, true] {
        let v = picker(faceted, 8);

        // UNIFIED: one fill == the whole card rect.
        set_pane_split_test_override(Some(theme::PaneSplit::Unified));
        p.set_view(&v);
        p.prepare(&device, &queue, w, h).unwrap();
        let rect = p.overlay_card_rect().expect("card rect");
        let uni = p.overlay_pane_fills_probe();
        assert_eq!(uni.len(), 1, "Unified draws ONE fill (faceted={faceted})");
        assert_eq!(uni[0], rect, "Unified fill == the whole card rect");
        assert_eq!(
            p.panel_card.instance_count(),
            1,
            "Unified uploads one card quad (faceted={faceted})"
        );

        // SPLIT: two fills, a real gap between, both inside the card.
        set_pane_split_test_override(Some(theme::PaneSplit::Split));
        p.set_view(&v);
        p.prepare(&device, &queue, w, h).unwrap();
        let fills = p.overlay_pane_fills_probe();
        assert_eq!(fills.len(), 2, "Split draws TWO fills (faceted={faceted})");
        assert_eq!(
            p.panel_card.instance_count(),
            2,
            "Split uploads two card quads (faceted={faceted})"
        );
        let [ux, uy, uw, uh] = fills[0];
        let [lx, ly, lw, lh_] = fills[1];
        // Same column, full card width.
        assert!((ux - rect[0]).abs() < 1e-3 && (uw - rect[2]).abs() < 1e-3);
        assert!((lx - rect[0]).abs() < 1e-3 && (lw - rect[2]).abs() < 1e-3);
        // The upper surface starts at the card top; the lower ends at the card
        // bottom; both have positive height; they do NOT overlap.
        assert!((uy - rect[1]).abs() < 1e-3, "upper starts at card top");
        assert!(
            ((ly + lh_) - (rect[1] + rect[3])).abs() < 1e-3,
            "lower ends at card bottom"
        );
        assert!(uh > 0.0 && lh_ > 0.0, "both surfaces have positive height");
        let gap_top = uy + uh;
        let gap_bottom = ly;
        assert!(
            gap_bottom > gap_top + 1.0,
            "a real background gap sits between the surfaces (faceted={faceted})"
        );
        // The gap matches the ONE geometry owner — a plan built here from the
        // pipeline's own header metrics, never the plan the fills were cut from.
        let (gt, gb) = crate::render::plan::test_header_plan(
            rect[1] + 12.0, // text_top = card_y + pad
            if faceted { 2 } else { 1 },
            p.overlay_header_gap(),
            p.overlay_lh(),
        )
        .split_bounds()
        .unwrap();
        assert!(
            (gap_top - gt).abs() < 1e-3 && (gap_bottom - gb).abs() < 1e-3,
            "the drawn gap == the plan's split_bounds (faceted={faceted})"
        );
    }

    set_pane_split_test_override(None);
    set_card_anchor_test_override(None);
}

/// NARROW WIDTH and ZERO RESULTS still form TWO valid, unclipped surfaces: both
/// rects have positive height, sit inside the (narrow / fill-regime) card, and
/// leave a real gap. The pathological arm (a card too short to seat both) falls
/// back to the unified room rather than a zero-height/inverted fill.
#[test]
fn split_stays_valid_narrow_and_empty() {
    let _g = crate::testlock::serial();
    set_card_anchor_test_override(Some(theme::CardAnchor::TopLeft));
    set_pane_split_test_override(Some(theme::PaneSplit::Split));

    // Narrow window (the card enters its fill regime) × zero and non-zero rows ×
    // flat/faceted — pure geometry, no GPU needed.
    for (w, hh) in [(360u32, 800u32), (300, 800)] {
        let Some((device, queue, mut p)) = headless_dqp(w as f32, hh as f32) else {
            eprintln!("skipping split_stays_valid: no wgpu adapter");
            set_pane_split_test_override(None);
            set_card_anchor_test_override(None);
            return;
        };
        for faceted in [false, true] {
            for n in [0usize, 1, 8] {
                let v = picker(faceted, n);
                p.set_view(&v);
                p.prepare(&device, &queue, w, hh).unwrap();
                let rect = p.overlay_card_rect().expect("narrow card rect");
                let fills = p.overlay_pane_fills_probe();
                let label = format!("w={w} faceted={faceted} n={n}");
                for f in &fills {
                    assert!(
                        f[2] > 0.0 && f[3] > 0.0,
                        "{label}: every fill is non-degenerate ({f:?})"
                    );
                    // Fully inside the card bounds (unclipped surface).
                    assert!(
                        f[0] >= rect[0] - 1e-3 && f[1] >= rect[1] - 1e-3,
                        "{label}: fill inside card top-left"
                    );
                    assert!(
                        f[0] + f[2] <= rect[0] + rect[2] + 1e-3
                            && f[1] + f[3] <= rect[1] + rect[3] + 1e-3,
                        "{label}: fill inside card bottom-right ({f:?} vs {rect:?})"
                    );
                }
                // A card that CAN split keeps a real gap; a degenerate one stays
                // unified — either way at least one valid, unclipped surface.
                if fills.len() == 2 {
                    let gap = fills[1][1] - (fills[0][1] + fills[0][3]);
                    assert!(gap > 1.0, "{label}: split cards keep a real gap ({gap})");
                } else {
                    assert_eq!(fills.len(), 1, "{label}: else the unified fallback");
                }
            }
        }
    }

    set_pane_split_test_override(None);
    set_card_anchor_test_override(None);
}

// --- exhaustive surface roster / no-world-branch -----------------------------

/// THE EXHAUSTIVE SURFACE-ROSTER LAW (no world branch): sweep EVERY shipped
/// world. A Pane world's fill count follows its OWN `pane_split` cap
/// (Split → 2, Unified → 1); a Bars world takes the bare-plate path (the split is
/// inert — the card fill is the per-plate scrims, never the 1/2 split). And the
/// decision is DATA, not identity: forcing the `pane_split` override to `Split`
/// makes EVERY Pane world draw two surfaces, and to `Unified` makes every Pane
/// world draw one — the same flip on all of them, no per-world code path.
#[test]
fn every_world_splits_by_its_cap_never_by_identity() {
    let _g = crate::testlock::serial();
    let (w, h) = (1200u32, 800u32);
    let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
        eprintln!("skipping every_world_splits_by_its_cap: no wgpu adapter");
        return;
    };
    let _g = crate::testlock::serial();
    set_card_anchor_test_override(Some(theme::CardAnchor::TopLeft));

    for t in theme::THEMES.iter() {
        theme::set_active_by_name(t.name).unwrap();
        p.sync_theme();

        // (1) The world's OWN data governs (no override).
        set_pane_split_test_override(None);
        let v = picker(false, 8);
        p.set_view(&v);
        p.prepare(&device, &queue, w, h).unwrap();
        let fills = p.overlay_pane_fills_probe();
        match t.render_caps.list_style {
            theme::ListStyle::Pane => {
                let want = match t.render_caps.pane_split {
                    theme::PaneSplit::Split => 2,
                    theme::PaneSplit::Unified => 1,
                };
                assert_eq!(
                    fills.len(),
                    want,
                    "{}: Pane fill count follows pane_split",
                    t.name
                );
            }
            theme::ListStyle::Bars => {
                let plates = p.overlay_bars.instance_count() + p.overlay_rows.instance_count();
                assert!(plates > 0, "{}: Bars floats plates", t.name);
                assert_eq!(
                    p.panel_card.instance_count(),
                    plates,
                    "{}: Bars scrims are its card fill",
                    t.name
                );
            }
            theme::ListStyle::Diagonal(_) => {
                assert_eq!(
                    p.overlay_bars.instance_count() + p.overlay_rows.instance_count(),
                    0,
                    "{}: diagonal has no bars",
                    t.name
                );
                assert_eq!(
                    p.panel_card.instance_count(),
                    0,
                    "{}: diagonal has no card fill",
                    t.name
                );
                assert_eq!(
                    p.overlay_spine.instance_count(),
                    1,
                    "{}: diagonal keeps one spine",
                    t.name
                );
            }
        }

        // (2) The decision is DATA: the override flips it on EVERY world the same
        // way. (On a Bars world the Card arm is not reached, so the override is
        // inert there — assert it stays plate-backed, never 1/2.)
        for (forced, want) in [
            (theme::PaneSplit::Split, 2usize),
            (theme::PaneSplit::Unified, 1),
        ] {
            set_pane_split_test_override(Some(forced));
            p.set_view(&v);
            p.prepare(&device, &queue, w, h).unwrap();
            let fills = p.overlay_pane_fills_probe();
            match t.render_caps.list_style {
                theme::ListStyle::Pane => assert_eq!(
                    fills.len(),
                    want,
                    "{}: forcing pane_split={forced:?} sets Pane fill count",
                    t.name
                ),
                theme::ListStyle::Bars => {}
                theme::ListStyle::Diagonal(_) => assert_eq!(
                    p.overlay_bars.instance_count() + p.overlay_rows.instance_count(),
                    0,
                    "{}: pane split override cannot turn diagonal into bars",
                    t.name
                ),
            }
        }
    }

    set_pane_split_test_override(None);
    set_card_anchor_test_override(None);
    theme::set_active(theme::DEFAULT_THEME);
}

// --- PIXELS: real ground across the FULL gap; no glyph escapes ----------------

fn avg(px: &[[u8; 4]], w: i64, h: i64, x: i64, y: i64, rw: i64, rh: i64) -> theme::Srgb {
    let (x0, y0) = (x.max(0), y.max(0));
    let (x1, y1) = ((x + rw).min(w), (y + rh).min(h));
    let mut s = [0u64; 3];
    let mut n = 0u64;
    for yy in y0..y1 {
        for xx in x0..x1 {
            let p = px[(yy * w + xx) as usize];
            s[0] += p[0] as u64;
            s[1] += p[1] as u64;
            s[2] += p[2] as u64;
            n += 1;
        }
    }
    assert!(n > 0, "empty sample");
    theme::Srgb::rgb((s[0] / n) as u8, (s[1] / n) as u8, (s[2] / n) as u8)
}

fn redmean(a: theme::Srgb, b: theme::Srgb) -> f32 {
    let rbar = (a.r as f32 + b.r as f32) * 0.5;
    let dr = a.r as f32 - b.r as f32;
    let dg = a.g as f32 - b.g as f32;
    let db = a.b as f32 - b.b as f32;
    ((2.0 + rbar / 256.0) * dr * dr + 4.0 * dg * dg + (2.0 + (255.0 - rbar) / 256.0) * db * db)
        .sqrt()
}

/// THE GROUND-VISIBLE + NO-ESCAPE PIXEL LAW, over chromatic Pane worlds (where
/// `base_100` ground reads distinct from the `base_300` card): the split shows
/// REAL BACKGROUND across the FULL inter-surface gap, and NO lower/query glyph
/// escapes into it.
///   - GROUND ACROSS THE FULL GAP: sample a surface column (left of the text, no
///     glyphs) at EVERY row of the gap — each row reads the ground, distinct from
///     the card fill sampled just inside each surface.
///   - NO GLYPH ESCAPES: sample the TEXT column across the gap — it stays the
///     ground (no ink pokes up from the lower surface's first row nor down from
///     the query line).
#[test]
fn split_shows_ground_across_the_gap_and_no_glyph_escapes() {
    let _g = crate::testlock::serial();
    let (w, h) = (1200u32, 800u32);
    let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
        eprintln!("skipping split_shows_ground_across_the_gap: no wgpu adapter");
        return;
    };
    let _g = crate::testlock::serial();
    set_card_anchor_test_override(Some(theme::CardAnchor::TopLeft));
    set_pane_split_test_override(Some(theme::PaneSplit::Split));
    let (wi, hi) = (w as i64, h as i64);

    // Chromatic Pane worlds: a dark room (Currawong), a warm light (Bilby), a
    // cool light (Gumtree), and Bowerbird (a plain dark Pane world since item
    // 86 retired its item-71 woven `CardTexture::JaggedWave` card fill).
    // Wagtail (1-bit, ground == card == black) is covered by the
    // border-count roster law instead — its gap reads by the rims.
    for world in ["Currawong", "Bilby", "Gumtree", "Bowerbird"] {
        theme::set_active_by_name(world).unwrap();
        p.sync_theme();
        for faceted in [false, true] {
            // A query with DESCENDERS ("gjpqy") stresses the no-escape floor of
            // the query line; real candidate rows stress the lower surface's top.
            let mut v = picker(faceted, 8);
            v.overlay_query = "gjpqy".into();
            p.set_view(&v);
            p.prepare(&device, &queue, w, h).unwrap();
            let fills = p.overlay_pane_fills_probe();
            assert_eq!(fills.len(), 2, "{world}/{faceted}: a split card");
            let rect = p.overlay_card_rect().unwrap();
            let up = fills[0];
            let lo = fills[1];
            let gap_top = up[1] + up[3];
            let gap_bottom = lo[1];
            let px = pixeldiff::render_frame(&mut p, &device, &queue, w, h);

            // Card fill sampled just inside each surface, at a glyph-free surface
            // column (card_x + 9, left of text_left = card_x + 12).
            let sx = (rect[0] + 9.0) as i64;
            let fill_up = avg(&px, wi, hi, sx, (up[1] + up[3] * 0.5) as i64, 3, 4);
            let fill_lo = avg(&px, wi, hi, sx, (lo[1] + lo[3] - 8.0) as i64, 3, 4);
            // The two surfaces are the SAME card value.
            assert!(
                redmean(fill_up, fill_lo) < 8.0,
                "{world}/{faceted}: both surfaces are the one card fill ({fill_up:?} vs {fill_lo:?})"
            );

            // GROUND ACROSS THE FULL GAP: every gap row (surface column) reads a
            // value distinct from the card fill — the world's background, not the
            // surface. Inset 2px each edge to clear the surfaces' AA + the 1px
            // raised BORDER rim (Bordered worlds peek their rim 1px into the gap).
            let g0 = (gap_top + 2.0) as i64;
            let g1 = (gap_bottom - 2.0) as i64;
            assert!(g1 > g0, "{world}/{faceted}: the gap has interior rows");
            for gy in g0..g1 {
                let ground = avg(&px, wi, hi, sx, gy, 3, 1);
                let d = redmean(ground, fill_up);
                assert!(
                    d >= 12.0,
                    "{world}/{faceted}: gap row {gy} shows REAL ground {ground:?} distinct from the card fill {fill_up:?} (redmean {d:.1})"
                );
            }

            // NO GLYPH ESCAPES: the TEXT column across the gap carries no GLYPH
            // INK — no query descender pokes down from the upper surface, no
            // candidate/strip glyph pokes up from the lower surface. A glyph is
            // the world's `base_content` ink; the ground and the raised border rim
            // are NOT (each stays far closer to the ground than to the ink). So
            // every gap text-column pixel must read more GROUND-LIKE than INK-LIKE
            // — a partial (antialiased) escape would tip a pixel toward the ink.
            let ground_ref = avg(&px, wi, hi, sx, (gap_top + gap_bottom) as i64 / 2, 3, 2);
            let ink = theme::base_content();
            let tx0 = (rect[0] + 12.0) as i64;
            let tx1 = (rect[0] + rect[2] - 12.0) as i64;
            for gy in (g0 + 1)..(g1) {
                for tx in (tx0..tx1).step_by(2) {
                    let c = avg(&px, wi, hi, tx, gy, 1, 1);
                    assert!(
                        redmean(c, ground_ref) <= redmean(c, ink),
                        "{world}/{faceted}: a glyph escaped into the gap at ({tx},{gy}): {c:?} is closer to the ink {ink:?} than the ground {ground_ref:?}"
                    );
                }
            }
        }
    }

    set_pane_split_test_override(None);
    set_card_anchor_test_override(None);
    theme::set_active(theme::DEFAULT_THEME);
}
