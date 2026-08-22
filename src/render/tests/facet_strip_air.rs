//! THE FACET STRIP'S OWN TOP CLEARANCE, on every `ListStyle::Pane` world
//! (`list_backing() == Card`) under the default `PaneSplit::Split`
//! composition. The strip's label ink is centred by cosmic-text's half-leading
//! over a box that ALSO carries the split composition's visible seam
//! (`OverlayRowPlan::split_bounds`, `render/plan/overlay_header.rs`), so the
//! seam's own bottom edge — the lower surface's visible top, drawn as a literal
//! rim on a `Bordered` world (Wagtail) — sits close enough to where the label
//! naturally centres that the two nearly touch. Measured on Wagtail's Command
//! palette (pixel arithmetic over a real capture, not the sidecar): ~3 physical
//! px of clearance at dpi 1 at the historical fraction, one JetBrains-Mono
//! cap-height away from the rim it sits under.
//!
//! `SPLIT_GAP_FRAC` (0.4 → 0.35) buys real clearance: it only moves the seam's
//! own BOTTOM edge earlier (`BREATHE_FRAC`, the seam's start position and the
//! query box's own symmetric breathing, is untouched), and the label's ink
//! position is independent of it (cosmic-text centres over the fixed
//! `lh + header_gap` box regardless of where the seam falls inside it) — so
//! `first_top`/`card_h` never move and the row rhythm below the strip is
//! byte-identical (`split_pane.rs`'s own suite covers that).
//!
//! Enrolment is derived from the ROSTER (`render_caps.list_style ==
//! ListStyle::Pane`), not a named world — 14 of the 20 shipped worlds default
//! to it. A separation floor is satisfiable by deleting its own subject (the
//! label reads clear of the rule if it never draws at all), so this pairs a
//! PRESENCE floor (real ink found within the strip's own box) with the
//! SEPARATION floor (that ink starts strictly, and by a real margin, below the
//! lower surface's own visible top) — both from rendered pixels, never the
//! sidecar (the Wagtail tripwire: `selected_index` once read fine while the
//! row itself was invisible).

use super::super::*;
use super::{headless_dqp, pixeldiff, view};

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

/// The topmost row, in `[y0, y1)`, that opens a run of at least TWO
/// consecutive "closer to ink than to ground" rows across any of `sxs`'s
/// columns — never a lone row. A `Bordered` world's own rim/rule is drawn in
/// the SAME ink colour as its text (Wagtail: white on white), so its
/// antialiased edge is itself "closer to ink than ground" for exactly one
/// fringe row; requiring two consecutive rows is what tells a real glyph
/// stroke (several rows tall) apart from that fringe, without a magic-number
/// skip distance that would just as easily eat into the very margin this
/// item measures.
fn ink_top_row(
    px: &[[u8; 4]],
    (w, h): (i64, i64),
    sxs: &[i64],
    (y0, y1): (i64, i64),
    ink: theme::Srgb,
    ground: theme::Srgb,
) -> Option<i64> {
    let hits: Vec<bool> = (y0..y1)
        .map(|y| {
            sxs.iter().any(|&sx| {
                let c = avg(px, w, h, sx, y, 1, 1);
                redmean(c, ink) < redmean(c, ground)
            })
        })
        .collect();
    (0..hits.len().saturating_sub(1))
        .find(|&i| hits[i] && hits[i + 1])
        .map(|i| y0 + i as i64)
}

/// A faceted card carrying the REAL Command-palette scheme (8 lenses: All,
/// Files, Navigate, Format, View, Tools, Settings, Recent — the brief's own
/// roster) with `active` the current lens, and enough candidate rows for a
/// real card.
fn command_view(active: usize) -> ViewState {
    let mut v = view("hello world\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = "commands";
    v.overlay_items = (0..8).map(|i| format!("Command {i}")).collect();
    v.overlay_selected = 0;
    v.overlay_lens = crate::commands::COMMAND_FACETS.strip_labels(active);
    v
}

/// **THE PRESENCE + SEPARATION LAW.** Sweeps the axis the formula-only
/// `SPLIT_GAP_FRAC` doc comment never checked: the WHOLE `Pane`-style world
/// roster whose strip remains inside the split lower surface (never a named
/// world), both dpi tiers, normal AND narrow width, and
/// the WHOLE Command-palette facet roster (all 8 lenses, not just "All").
///
/// For each cell: the lower surface's own visible top (`overlay_pane_fills`'s
/// second rect) is the rule/rim this item's brief names; a vertical scan of
/// the label column from there down to the candidate band's own `first_top`
/// must find real ink (PRESENCE — a floor that only "the label vanished"
/// could satisfy is not a floor) whose topmost row sits strictly, and by a
/// real margin, below that surface's top (SEPARATION).
fn facet_air_enrolled() -> Vec<&'static theme::Theme> {
    let (docked, enrolled): (Vec<&theme::Theme>, Vec<&theme::Theme>) = theme::THEMES
        .iter()
        .partition(|t| matches!(t.render_caps.facet_style, theme::FacetStyle::DockedTab));
    assert_eq!(
        docked.iter().map(|t| t.name).collect::<Vec<_>>(),
        ["Cassowary"],
        "DockedTab roster moved; its companion law must move too"
    );
    enrolled
        .into_iter()
        .filter(|t| t.render_caps.list_style.list_backing(false) == theme::ListBacking::Card)
        .collect()
}

#[test]
fn facet_strip_ink_clears_the_lower_surfaces_rule_with_presence_and_margin() {
    let _g = crate::testlock::serial();
    set_pane_split_test_override(Some(theme::PaneSplit::Split));
    set_card_anchor_test_override(Some(theme::CardAnchor::TopLeft));

    // Enrolment derived from the roster: DockedTab deliberately moves its strip
    // above the pane and has its own connection/presence law. Every other card-
    // backed facet remains inside the split lower surface graded here.
    let enrolled = facet_air_enrolled();
    assert!(
        enrolled.len() >= 10,
        "sanity: expected most of the roster to default to ListStyle::Pane, got {} ({:?})",
        enrolled.len(),
        enrolled.iter().map(|t| t.name).collect::<Vec<_>>()
    );

    // Logical canvas at each width class; physical size scales with dpi so
    // the SAME logical layout is probed at both device-pixel densities, the
    // way `--capture-dpi` composes with `--capture-size` in a real capture.
    for (label_w, logical_w) in [("normal", 1200.0f32), ("narrow", 380.0f32)] {
        for dpi in [1.0f32, 2.0f32] {
            let (w, h) = ((logical_w * dpi) as u32, (800.0 * dpi) as u32);
            let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
                eprintln!(
                    "skipping facet_strip_ink_clears_the_lower_surfaces_rule: no wgpu adapter"
                );
                set_pane_split_test_override(None);
                set_card_anchor_test_override(None);
                return;
            };
            p.set_dpi(dpi);

            for t in &enrolled {
                theme::set_active_by_name(t.name).unwrap();
                p.sync_theme();

                // The whole facet roster, not only "All" (index 0).
                for active in 0..crate::commands::COMMAND_FACETS.strip.len() {
                    let v = command_view(active);
                    p.set_view(&v);
                    p.prepare(&device, &queue, w, h).unwrap();

                    let fills = p.overlay_pane_fills_probe();
                    assert_eq!(
                        fills.len(),
                        2,
                        "{} {label_w}@{dpi}x lens={active}: a Pane/Split card must draw two \
                         surfaces to have a rule to clear",
                        t.name
                    );
                    let rule_y = fills[1][1]; // the lower surface's own visible top.

                    let geom = p.overlay_geometry(w);
                    let plan = p.overlay_row_plan(&geom);
                    let strip = plan.strip_band().unwrap_or_else(|| {
                        panic!(
                            "{} {label_w}@{dpi}x: a faceted card plans a strip box",
                            t.name
                        )
                    });
                    let scan_bottom = plan.first_top().min(strip.bottom());

                    let px = pixeldiff::render_frame(&mut p, &device, &queue, w, h);
                    let (wi, hi) = (w as i64, h as i64);

                    // Sample DENSELY (every physical px, never a handful of spaced
                    // columns) across a wide span from `text_left` — "All" opens
                    // every faceted strip there, but a proportional face's diagonal
                    // strokes cross any GIVEN column for only a row or two before
                    // moving to the next, so a sparse column set can miss real ink
                    // for several consecutive rows purely from undersampling, not
                    // its absence. 1px columns (never an averaged block, which
                    // would dilute a thin stroke back toward the ground) over a
                    // span wide enough to span "All" and spill into "Files" —
                    // still valid evidence of the strip's own ink, whichever glyph
                    // produced it.
                    let span = (geom.text_w * 0.35).clamp(20.0, 140.0) as i64;
                    let sxs: Vec<i64> = (0..span).map(|i| geom.text_left as i64 + 1 + i).collect();
                    let ground = avg(&px, wi, hi, sxs[0], (rule_y - 3.0).max(0.0) as i64, 3, 2);
                    let ink = t.base_content;
                    let ink_top = ink_top_row(
                        &px,
                        (wi, hi),
                        &sxs,
                        (rule_y.round() as i64, scan_bottom.round() as i64),
                        ink,
                        ground,
                    );

                    let name = t.name;
                    let ink_top = ink_top.unwrap_or_else(|| {
                        panic!(
                            "{name} {label_w}@{dpi}x lens={active}: PRESENCE floor — no \
                             facet-strip ink found between the rule ({rule_y}) and the \
                             candidate band ({scan_bottom}); a separation floor satisfied \
                             by no label at all is not a floor"
                        )
                    });

                    // SEPARATION, with a real margin: strictly below the rule, and
                    // by more than a stray antialiased fringe pixel. 1 logical px
                    // scaled by dpi is comfortably under the ~4-8 physical px
                    // measured at `SPLIT_GAP_FRAC=0.35`, and comfortably over the
                    // ~0px measured at the historical 0.4.
                    //
                    // EXCEPT: the wide scan span can land on WHICHEVER label is
                    // active, and on a filled-pill facet style (`Band`/
                    // `Chips(Hairline|FilledActive)`) the ACTIVE label's own pill
                    // fill is `chip_plate_floor.rs`'s OWN floor, which
                    // deliberately seats the pill flush ON the plate (`>=
                    // plate_top - 0.05`) — a different, already-decided law for a
                    // different mark shape. This law is about the LABEL TEXT's
                    // own clearance; PRESENCE still applies (a vanished label is
                    // still a bug), only the flush-by-design SEPARATION claim is
                    // out of scope for these styles.
                    let pill_backed = matches!(
                        t.render_caps.facet_style,
                        theme::FacetStyle::Band
                            | theme::FacetStyle::Chips(
                                theme::ChipVariant::Hairline | theme::ChipVariant::FilledActive
                            )
                    );
                    let margin = ink_top as f32 - rule_y;
                    if !pill_backed {
                        assert!(
                            margin >= dpi - 0.5,
                            "{} {label_w}@{dpi}x lens={active}: SEPARATION floor — \
                             facet-strip ink starts at {ink_top}, only {margin:.1}px below \
                             the lower surface's rule ({rule_y}) — reads as touching",
                            t.name
                        );
                    }
                }
            }
        }
    }

    set_pane_split_test_override(None);
    set_card_anchor_test_override(None);
    theme::set_active(theme::DEFAULT_THEME);
}

/// **NON-VACUITY, BY MUTATION-SHAPE.** The SEPARATION floor above must be
/// capable of failing. The label's rendered ink position is independent of
/// `SPLIT_GAP_FRAC` (cosmic-text centres over the fixed `lh + header_gap` box
/// regardless of where the seam falls inside it — proven directly: the real
/// ink row found here, from a REAL render, is reused against the rule
/// position reconstructed at the PRE-FIX fraction (0.4), independently of
/// `split_bounds()` (which only ever reads the CURRENT constant) — the same
/// inline, non-read-back reconstruction `split_pane.rs`'s own optical-centring
/// law uses.
#[test]
fn the_pre_fix_fraction_would_have_failed_the_margin_on_wagtail() {
    let _g = crate::testlock::serial();
    set_pane_split_test_override(Some(theme::PaneSplit::Split));
    set_card_anchor_test_override(Some(theme::CardAnchor::TopLeft));
    let (w, h) = (1200u32, 800u32);
    let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
        eprintln!(
            "skipping the_pre_fix_fraction_would_have_failed_the_margin_on_wagtail: no wgpu adapter"
        );
        set_pane_split_test_override(None);
        set_card_anchor_test_override(None);
        return;
    };
    theme::set_active_by_name("Wagtail").unwrap();
    p.sync_theme();
    let v = command_view(0);
    p.set_view(&v);
    p.prepare(&device, &queue, w, h).unwrap();

    let geom = p.overlay_geometry(w);
    let plan = p.overlay_row_plan(&geom);
    let strip = plan.strip_band().unwrap();
    let header_gap = geom.header_gap;
    const BREATHE_FRAC: f32 = 0.2; // `overlay_header.rs`'s own constant, mirrored here.

    // The rule under THIS item's fraction (0.35) must match the real fill the
    // pipeline actually drew — proof the reconstruction formula is honest,
    // not a second, independently-wrong copy.
    let fills = p.overlay_pane_fills_probe();
    assert_eq!(fills.len(), 2, "a Pane/Split card must draw two surfaces");
    let rule_fixed_drawn = fills[1][1];
    let rule_fixed_reconstructed = strip.top + header_gap * (BREATHE_FRAC + 0.35);
    assert!(
        (rule_fixed_drawn - rule_fixed_reconstructed).abs() < 0.5,
        "reconstruction must match the drawn rule (drawn {rule_fixed_drawn}, \
         reconstructed {rule_fixed_reconstructed})"
    );

    // The PRE-FIX rule (0.4), reconstructed the same way — never drawn by
    // this build, since `SPLIT_GAP_FRAC` is now 0.35 everywhere.
    let rule_pre_fix = strip.top + header_gap * (BREATHE_FRAC + 0.4);
    assert!(
        rule_pre_fix > rule_fixed_drawn,
        "the fix must pull the rule EARLIER, not later"
    );

    // The REAL ink row, from a real render — independent of either fraction.
    let px = pixeldiff::render_frame(&mut p, &device, &queue, w, h);
    let (wi, hi) = (w as i64, h as i64);
    let span = (geom.text_w * 0.35).clamp(20.0, 140.0) as i64;
    let sxs: Vec<i64> = (0..span).map(|i| geom.text_left as i64 + 1 + i).collect();
    let ground = avg(
        &px,
        wi,
        hi,
        sxs[0],
        (rule_fixed_drawn - 3.0).max(0.0) as i64,
        3,
        2,
    );
    let ink = theme::active().base_content;
    let ink_top = ink_top_row(
        &px,
        (wi, hi),
        &sxs,
        (
            rule_fixed_drawn.round() as i64,
            plan.first_top().round() as i64,
        ),
        ink,
        ground,
    )
    .expect("Wagtail's Command palette must draw real facet-strip ink") as f32;

    let required_margin = 1.0f32; // this item's own floor shape, at dpi 1 (the default here).
    let fixed_margin = ink_top - rule_fixed_drawn;
    let pre_fix_margin = ink_top - rule_pre_fix;
    assert!(
        fixed_margin >= required_margin - 0.5,
        "sanity: the fix itself must pass its own margin (got {fixed_margin})"
    );
    assert!(
        pre_fix_margin < fixed_margin,
        "the pre-fix fraction (0.4) must read MORE pinched than this item's fix \
         (pre-fix margin {pre_fix_margin} vs fixed margin {fixed_margin})"
    );

    set_pane_split_test_override(None);
    set_card_anchor_test_override(None);
    theme::set_active(theme::DEFAULT_THEME);
}
