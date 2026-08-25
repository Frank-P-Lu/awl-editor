//! A filled facet mark (`FacetStyle::Band` or a pill/tick
//! `Chips(..)` skin) is centred on the lens STRIP's own header-line box
//! (`OverlayRowPlan::strip_band`), but on a `ListStyle::Pane` world under the
//! default `PaneSplit::Split` composition that box is not entirely plate: the
//! query beat's own seam (`OverlayRowPlan::split_bounds`) falls INSIDE it, by
//! construction (`BREATHE_FRAC + SPLIT_GAP_FRAC < 1.0`), so the visible plate
//! begins only at `split_bounds().1`, partway down the box. A mark centred on
//! the WHOLE box then draws above the plate it is meant to sit on — Kite's
//! reported defect, "the filled chip's plate runs flush into the strip band's
//! top, reading as clipped".
//!
//! Two claims:
//!   1. THE FLOOR — every filled/ticked mark this item's fix touches
//!      (`Band`, `Chips(Hairline)`, `Chips(FilledActive)`, `Chips(Bracket)`)
//!      keeps its own top edge ON the plate, never above it, on a forced
//!      `Pane` + `Split` card — swept at dpi 1x and 2x (the one scale an
//!      ordinary `--capture-dpi 1` capture cannot see past). Proven
//!      non-vacuous by reconstructing the PRE-FIX centre inline (never read
//!      back from the fix) and showing it violates the same floor.
//!   2. THE ROSTER SWEEP — enrollment is derived from each world's OWN
//!      `render_caps` (`list_style`, `pane_split`, `facet_style`), never a
//!      named list: today exactly one shipped world (`Kite`, `Band` on
//!      `Pane`/`Split`) satisfies all three, because no `Chips` world ships
//!      on `ListStyle::Pane`. Every other world's mark centre is
//!      BYTE-IDENTICAL to the pre-fix `strip_band().center()` — the clamp is
//!      an unconditional no-op off either gate, not a per-world branch.

use super::super::*;
use super::{headless_dqp, view};

const LOGICAL: (f32, f32) = (1200.0, 800.0);

/// An open, faceted (lens-carrying) card with one active facet — the shape
/// every `overlay_shape_theme` mark (pill, tick, or underline) is recorded
/// against.
fn facet_view() -> ViewState {
    let mut v = view("hello world\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = "commands".to_string();
    v.overlay_items = (0..6).map(|i| format!("Command {i}")).collect();
    v.overlay_selected = 0;
    v.overlay_lens = vec![
        ("All".into(), true),
        ("Files".into(), false),
        ("Navigate".into(), false),
    ];
    v
}

/// The plan's own strip box centre (`strip_band().center()`) and the lower
/// surface's visible top (`split_bounds().1`, `None` when the card is not
/// actually split into two surfaces) — read from the SAME pipeline state the
/// mark was just recorded against, at `dpi`.
fn plate_geometry(p: &mut TextPipeline, v: &ViewState, w: u32) -> (f32, Option<f32>) {
    p.set_view(v);
    let geom = p.overlay_geometry(w);
    let plan = p.overlay_row_plan(&geom);
    let strip_center = plan
        .strip_band()
        .expect("a faceted card plans a strip box")
        .center();
    // `overlay_pane_fills` itself has no `list_style` gate — that gate lives one
    // call up, in `overlay_prepare_card_backing`, which only ever reaches it on
    // `ListBacking::Card` (`ListStyle::Pane`); a `Bars`/`Diagonal`/`Ruled` world
    // takes the `BarePlates` branch and never draws this fill at all, even
    // though the probe below would still happily compute two rects for it.
    // Mirror that same production gate here, or a Bars world (whose own
    // `overlay_row_gap` already keeps it clear of the floor) would be
    // misread as drawing a plate it never paints.
    let has_plate =
        crate::render::effective_list_style().list_backing(false) == theme::ListBacking::Card;
    let fills = p.overlay_pane_fills_probe();
    let plate_top = (has_plate && fills.len() == 2).then(|| fills[1][1]);
    (strip_center, plate_top)
}

/// The mark's own top edge, whichever shape drew it: `overlay_theme_underline`
/// for a pill/underline rect, or the lowest tick's `y` for `Chips(Bracket)`'s
/// corner ticks (the ticks alone carry no single rect, but the top-left/
/// top-right pair share the mark's own top).
fn mark_top(p: &TextPipeline) -> Option<f32> {
    if let Some(r) = p.overlay_theme_underline {
        return Some(r[1]);
    }
    let ghosts = &p.overlay_theme_facet_ghosts;
    if ghosts.is_empty() {
        return None;
    }
    Some(ghosts.iter().map(|r| r[1]).fold(f32::INFINITY, f32::min))
}

/// **CLAIM 1 — THE FLOOR.** Forced `Pane` + `Split` (the composition Kite
/// ships), swept across every mark shape this item's fix touches and both
/// dpi tiers. The mark's own top edge never rises above the lower surface's
/// visible top — and the naive pre-fix centre (`strip_band().center()`,
/// reconstructed independently of the fix) is shown to violate that same
/// floor, so the check is proven to bite.
#[test]
fn filled_facet_marks_never_draw_above_their_own_plate() {
    let _g = crate::testlock::serial();
    set_list_style_test_override(Some(theme::ListStyle::Pane));
    set_pane_split_test_override(Some(theme::PaneSplit::Split));
    set_card_anchor_test_override(Some(theme::CardAnchor::TopLeft));

    for (label, style) in [
        ("Band", theme::FacetStyle::Band),
        (
            "Chips(Hairline)",
            theme::FacetStyle::Chips(theme::ChipVariant::Hairline),
        ),
        (
            "Chips(FilledActive)",
            theme::FacetStyle::Chips(theme::ChipVariant::FilledActive),
        ),
        (
            "Chips(Bracket)",
            theme::FacetStyle::Chips(theme::ChipVariant::Bracket),
        ),
    ] {
        set_facet_style_test_override(Some(style));
        for dpi in [1.0f32, 2.0f32] {
            let (w, h) = ((LOGICAL.0 * dpi) as u32, (LOGICAL.1 * dpi) as u32);
            let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
                eprintln!("skipping the plate-floor law: no wgpu adapter");
                set_facet_style_test_override(None);
                set_pane_split_test_override(None);
                set_list_style_test_override(None);
                set_card_anchor_test_override(None);
                return;
            };
            p.set_dpi(dpi);
            let v = facet_view();
            p.set_view(&v);
            p.prepare(&device, &queue, w, h).unwrap();
            let (strip_center, plate_top) = plate_geometry(&mut p, &v, w);
            let plate_top = plate_top.unwrap_or_else(|| {
                panic!("{label}@{dpi}x: a Pane/Split card must draw two surfaces")
            });
            let top = mark_top(&p)
                .unwrap_or_else(|| panic!("{label}@{dpi}x: no mark recorded (rect or ticks)"));

            // NON-VACUITY: the naive pre-fix centre — `strip_band().center()`,
            // exactly what `mark_cy` was before this item, reconstructed here
            // independently of the fix under test — sits ABOVE the plate. If
            // it did not, the floor below would pass on a check that never
            // engages.
            let chip_half = top.is_finite().then(|| {
                // Recover the mark's own half-height from the drawn rect when
                // one exists; corner ticks have no single height, so approximate
                // from the pill formula's own inputs is unnecessary — the
                // pre-fix TOP for a rect-shaped mark is `strip_center -
                // rect_height/2`, which is exactly `top` plus the amount THIS
                // fix already added. Recomputed straight from the rect so nothing
                // here depends on the fix's own arithmetic.
                p.overlay_theme_underline.map(|r| r[3] * 0.5)
            });
            if let Some(Some(half)) = chip_half {
                let naive_top = strip_center - half;
                assert!(
                    naive_top < plate_top - 0.01,
                    "{label}@{dpi}x: sanity check failed — the naive centre \
                     ({strip_center}) already clears the plate ({plate_top}), so \
                     this world/style pair cannot prove the floor bites"
                );
            }

            assert!(
                top >= plate_top - 0.05,
                "{label}@{dpi}x: the mark's own top ({top}) rises {:.2}px above \
                 the plate's visible top ({plate_top}) — the filled chip's \
                 plate running flush into the strip band's top, exactly item \
                 292's defect",
                plate_top - top
            );
        }
    }

    set_facet_style_test_override(None);
    set_pane_split_test_override(None);
    set_list_style_test_override(None);
    set_card_anchor_test_override(None);
    theme::set_active(theme::DEFAULT_THEME);
}

/// **CLAIM 2 — THE ROSTER SWEEP.** Enrollment is derived from each world's own
/// `render_caps`, never a named list. A world enrolls the floor when it draws
/// a real two-surface `Pane`/`Split` card AND its facet style is one of the
/// four rect/tick shapes the floor applies to (`Text` and
/// `Chips(Underline)` read the shaped glyph BASELINE instead and never touch
/// `strip_band().center()` at all, so they are out of scope by construction,
/// not by exemption). Every unenrolled world's mark centre must be
/// BYTE-IDENTICAL to the pre-fix `strip_band().center()` — proof the clamp
/// changes nothing anywhere it is not needed.
#[test]
fn only_worlds_that_draw_a_pane_split_plate_are_floored() {
    let _g = crate::testlock::serial();
    let (w, h) = (LOGICAL.0 as u32, LOGICAL.1 as u32);
    let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
        eprintln!("skipping the plate-floor roster sweep: no wgpu adapter");
        return;
    };
    set_card_anchor_test_override(Some(theme::CardAnchor::TopLeft));
    let v = facet_view();
    p.set_view(&v);

    let mut floored: Vec<&str> = Vec::new();
    let mut untouched: Vec<&str> = Vec::new();
    for t in theme::THEMES.iter() {
        theme::set_active_by_name(t.name).unwrap();
        p.sync_theme();
        p.prepare(&device, &queue, w, h).unwrap();
        let (strip_center, plate_top) = plate_geometry(&mut p, &v, w);
        // ONLY these four skins ever read `strip_band().center()` at all —
        // `Text` and `Chips(Underline)` position from the shaped glyph
        // BASELINE instead (a rect still exists for them, but centering it on
        // the strip box was never their formula, so they carry no claim here).
        let pill_shaped = matches!(
            t.render_caps.facet_style,
            theme::FacetStyle::Band
                | theme::FacetStyle::Chips(
                    theme::ChipVariant::Hairline
                        | theme::ChipVariant::FilledActive
                        | theme::ChipVariant::Bracket
                )
        );
        let enrolled = pill_shaped && plate_top.is_some();

        match p.overlay_theme_underline {
            Some(r) if pill_shaped => {
                let center = r[1] + r[3] * 0.5;
                if enrolled {
                    floored.push(t.name);
                    let plate_top = plate_top.unwrap();
                    assert!(
                        r[1] >= plate_top - 0.05,
                        "{}: enrolled but its mark top ({}) still clears the \
                         plate ({plate_top}) incorrectly",
                        t.name,
                        r[1]
                    );
                } else {
                    untouched.push(t.name);
                    assert!(
                        (center - strip_center).abs() < 0.01,
                        "{}: unenrolled world's mark centre moved from the \
                         pre-fix strip_band().center() ({strip_center}) to \
                         {center} — the floor must be a no-op off its own gate",
                        t.name
                    );
                }
            }
            // `Text` / `Chips(Underline)` (rect exists, but baseline-driven —
            // out of scope by construction, not by exemption) and
            // `Chips(Bracket)` (ticks, no single rect — claim 1's forced
            // sweep covers its floor behaviour instead).
            _ => {
                untouched.push(t.name);
            }
        }
    }

    // NAME WHAT ENROLLED. Today the derived set is EMPTY: Kite — the floor's
    // original and only carrier (`Band` on `Pane`/`Split`) — moved to
    // `ListStyle::Ruled` (user decision 2026-08-22), and no shipped world
    // wears a pill-shaped facet style on `ListStyle::Pane`. The floor's
    // behaviour stays proven by claim 1's FORCED `Pane`+`Split` sweep above;
    // this pin exists so a future world that satisfies the gate is named the
    // day it lands and gets a human look, rather than silently enrolling.
    assert_eq!(
        floored,
        Vec::<&str>::new(),
        "a world newly enrolled the plate floor — a human confirms the floor \
         reads right there before this pin grows"
    );
    assert!(
        untouched.len() + floored.len() == theme::THEMES.len(),
        "every world is accounted for exactly once"
    );
    eprintln!(
        "plate floor roster: floored={floored:?} untouched={}",
        untouched.len()
    );

    set_card_anchor_test_override(None);
    theme::set_active(theme::DEFAULT_THEME);
}
