//! PRINTED-CARD LAW SUITE — Quokka's `CardTexture::HalftoneDots` and the
//! shared `CardShape::Chamfered` cap Quokka and Cassowary author. Structural
//! rosters first (every OTHER texture stays `Flat`; every other shape stays
//! `Rectangular`, with no-wildcard matches over both closed enums),
//! then real-pixel proofs (the Wagtail tripwire: appearance is arithmetic
//! over the PNG, never inferred from state) — a chamfered corner reads as a
//! genuine 45° cut distinguishable from the pre-existing small rounded
//! corner, and the dot texture rolls off toward the left content side.
//!
//! (Bowerbird briefly extended this suite with its own woven
//! `CardTexture::JaggedWave` cap + its own rolloff/legibility/silhouette
//! tests; retiring the variant returned Bowerbird's cards
//! to plain flat — so those Bowerbird-specific tests were removed along with
//! it. `bowerbird_card_corner_is_not_chamfered` survives as a plain roster
//! member of the "every other world" sweep below, now unremarkable.)

use super::super::*;
use super::{headless_dqp, pixeldiff};

// --- structural rosters --------------------------------------------------

/// EXHAUSTIVE ROSTER: every world but Quokka (`HalftoneDots`) carries the
/// byte-identical `Flat` `CardTexture` default; Quokka and Cassowary author
/// the shared chamfer while every other world stays rectangular — a
/// no-wildcard match so a newly added `CardTexture`/`CardShape` variant
/// can't silently dodge this sweep. (Bowerbird's `JaggedWave` woven card
/// texture was retired outright; Bowerbird now
/// falls into the plain `Flat`/`Rectangular` bucket like every other
/// non-Quokka world.)
#[test]
fn card_caps_rosters_are_exact_for_every_world() {
    for t in theme::THEMES {
        let is_flat = match t.render_caps.card_texture {
            theme::CardTexture::Flat => true,
            theme::CardTexture::HalftoneDots { .. } => false,
        };
        let is_rect = match t.render_caps.card_shape {
            theme::CardShape::Rectangular => true,
            theme::CardShape::Chamfered { .. } => false,
        };
        match t.name {
            "Quokka" => {
                assert!(!is_flat, "Quokka must assign a non-default CardTexture");
                assert!(!is_rect, "Quokka must assign a non-default CardShape");
            }
            "Cassowary" => {
                assert!(
                    is_flat,
                    "Cassowary's scanlines are a summoned material, not CardTexture"
                );
                assert!(
                    !is_rect,
                    "Cassowary authors the shared chamfered console shape"
                );
            }
            _ => {
                assert!(
                    is_flat,
                    "{} must keep CardTexture::Flat (Quokka is the only carrier)",
                    t.name
                );
                assert!(
                    is_rect,
                    "{} must keep CardShape::Rectangular (Quokka and Cassowary are the carriers)",
                    t.name
                );
            }
        }
    }
}

/// Quokka's authored dials sit inside the round's own spec bands: dot angle
/// 15-20°, chamfer cut 10-12 logical px, a non-degenerate density/cell.
#[test]
fn quokka_card_caps_are_within_the_rounds_authored_spec() {
    let caps = theme::QUOKKA.render_caps;
    match caps.card_texture {
        theme::CardTexture::HalftoneDots {
            angle_deg,
            cell_px,
            density,
        } => {
            assert!(
                (15.0..=20.0).contains(&angle_deg),
                "angle {angle_deg} outside 15-20°"
            );
            assert!(cell_px > 0.0, "cell_px must be positive");
            assert!(
                density > 0.0 && density <= 1.0,
                "density {density} outside (0,1]"
            );
        }
        other => panic!("Quokka must ship HalftoneDots, got {other:?}"),
    }
    match caps.card_shape {
        theme::CardShape::Chamfered {
            top_cut_px,
            bottom_cut_px,
        } => {
            assert!(
                (10.0..=12.0).contains(&top_cut_px),
                "top_cut_px {top_cut_px} outside 10-12px"
            );
            assert!(
                (10.0..=12.0).contains(&bottom_cut_px),
                "bottom_cut_px {bottom_cut_px} outside 10-12px"
            );
            assert_eq!(
                top_cut_px, bottom_cut_px,
                "Quokka's own identity is the all-four-corner chamfer — top and \
                 bottom must agree"
            );
        }
        theme::CardShape::Rectangular => panic!("Quokka must ship Chamfered"),
    }
}

/// `narrowed_chamfer_px` never grows the authored cut and shrinks it once
/// the card's own smaller dimension gets tight — the "narrow layouts reduce
/// the chamfer before it steals text room" rule, pure function.
#[test]
fn narrowed_chamfer_never_exceeds_the_authored_cut_and_shrinks_on_a_small_card() {
    use crate::render::chrome::narrowed_chamfer_px;
    // A generously sized card: no reduction.
    assert_eq!(narrowed_chamfer_px(11.0, 400.0, 300.0), 11.0);
    // A tiny card (well under a genuine popup's usual size): reduced, never
    // negative, never larger than the authored cut.
    let small = narrowed_chamfer_px(11.0, 20.0, 15.0);
    assert!(
        (0.0..11.0).contains(&small),
        "small-card chamfer {small} out of [0,11)"
    );
    // A short-but-ordinary query bar (Split Pane's upper surface, ~500x50)
    // must NOT be reduced — 40% of its own 50px height (20px) still clears
    // the 11px authored cut. Only a genuinely tiny surface shrinks.
    let query_bar = narrowed_chamfer_px(11.0, 500.0, 50.0);
    assert_eq!(
        query_bar, 11.0,
        "an ordinary short query bar must keep its full chamfer"
    );
    // Monotone: a smaller card never yields a LARGER chamfer than a bigger one.
    let mid = narrowed_chamfer_px(11.0, 120.0, 90.0);
    assert!(
        small <= mid,
        "chamfer should shrink monotonically with card size"
    );
}

// --- real-pixel proofs ----------------------------------------------------

/// Open the theme picker on `world`, render one settled frame, and return
/// `(pixels, canvas_w, canvas_h, card_rect)`.
// Pixels, canvas geometry, and card rect are returned together for the pixel-law fixture.
#[allow(clippy::type_complexity)]
fn render_theme_picker(world: &str) -> Option<(Vec<[u8; 4]>, i64, i64, [f32; 4])> {
    let (device, queue, mut p) = headless_dqp(1200.0, 800.0)?;
    let _g = crate::testlock::serial();
    theme::set_active_by_name(world).unwrap();
    p.sync_theme();
    let mut v = super::view("hello world\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = "themes";
    v.overlay_items = theme::world_names().iter().map(|s| s.to_string()).collect();
    p.set_view(&v);
    p.prepare(&device, &queue, 1200, 800).unwrap();
    let card = p
        .overlay_card_rect()
        .expect("theme picker card must be open");
    let pixels = pixeldiff::render_frame(&mut p, &device, &queue, 1200, 800);
    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    Some((pixels, 1200, 800, card))
}

fn px_at(pixels: &[[u8; 4]], w: i64, x: i64, y: i64) -> [u8; 4] {
    pixels[(y * w + x) as usize]
}

/// THE CHAMFER DISCRIMINATOR: at a point `(ex, ey)` inward from a corner
/// (measuring distance from each of the two nearest straight edges), a
/// `chamfer=c` octagon is OUTSIDE the fill iff `ex + ey < c` — a plain small
/// rounded corner (`r ~= 2.5px`, every non-Quokka world) is INSIDE well
/// before that (`ex=ey=5` clears a 2.5px radius easily). So sampling `(5,5)`
/// inward from Quokka's own card corner must land on the WORLD'S PAGE
/// BACKGROUND (the corner is cut away), while the identical offset on a
/// Rectangular-shaped world's card must land on the CARD'S OWN fill.
#[test]
fn quokka_card_top_left_corner_is_genuinely_chamfered() {
    let _g = crate::testlock::serial();
    let Some((pixels, w, _h, card)) = render_theme_picker("Quokka") else {
        eprintln!("skipping quokka_card_top_left_corner_is_genuinely_chamfered: no wgpu adapter");
        return;
    };
    let [cx, cy, _cw, _ch] = card;
    let card_fill = theme::QUOKKA.base_300.rgba_bytes();
    // 5px inward from the corner on BOTH axes: inside a 2.5px round, outside
    // an 11px chamfer (5+5=10 < 11).
    let corner_5 = px_at(&pixels, w, (cx + 5.0) as i64, (cy + 5.0) as i64);
    let near = |a: [u8; 4], b: [u8; 4]| (0..3).all(|k| (a[k] as i16 - b[k] as i16).abs() <= 4);
    assert!(
        !near(corner_5, card_fill),
        "5px inward from Quokka's card corner still reads as card fill {card_fill:?} \
         (got {corner_5:?}) — the chamfer isn't cutting the corner"
    );
    // Well past the chamfer (60 + 8 inward, sum 68 >> 11): must be filled —
    // the cut is a CORNER treatment, not a shrunk card. Sampled ALONG THE TOP
    // EDGE rather than down the diagonal: a flat picker's query glyphs ride
    // their own line's centre, which the (25, 25) diagonal probe runs into.
    let deep = px_at(&pixels, w, (cx + 60.0) as i64, (cy + 8.0) as i64);
    assert!(
        near(deep, card_fill),
        "25px inward from the corner must be the card fill (got {deep:?})"
    );
}

/// The SAME corner probe on a plain `Rectangular` world (Bombora, a Pane/
/// Split dark world) must find the 5px-inward point ALREADY filled — proving
/// the discriminator actually distinguishes chamfer from the pre-existing
/// small rounded corner, and that Rectangular worlds are untouched.
#[test]
fn non_quokka_card_corner_is_not_chamfered() {
    let _g = crate::testlock::serial();
    let Some((pixels, w, _h, card)) = render_theme_picker("Bombora") else {
        eprintln!("skipping non_quokka_card_corner_is_not_chamfered: no wgpu adapter");
        return;
    };
    let [cx, cy, _cw, _ch] = card;
    let card_fill = theme::BOMBORA.base_300.rgba_bytes();
    let corner_5 = px_at(&pixels, w, (cx + 5.0) as i64, (cy + 5.0) as i64);
    let near = |a: [u8; 4], b: [u8; 4]| (0..3).all(|k| (a[k] as i16 - b[k] as i16).abs() <= 4);
    assert!(
        near(corner_5, card_fill),
        "Bombora's card corner must stay the pre-existing small rounded corner (filled at \
         5px inward), got {corner_5:?} vs fill {card_fill:?}"
    );
}

/// THE ROLLOFF LAW: sampling a fixed row through the card's PLAIN interior
/// (below the header, above the footer, off any text glyph or the selected
/// band) at the LEFT edge of the content column vs a point near the card's
/// own RIGHT edge, more pixels differ from the flat card-fill color on the
/// right than on the left — "strongest at the far/right decorative side,
/// rolling off before the left-aligned content-heavy side".
#[test]
fn quokka_halftone_rolls_off_toward_the_left_content_side() {
    let _g = crate::testlock::serial();
    let Some((pixels, w, _h, card)) = render_theme_picker("Quokka") else {
        eprintln!(
            "skipping quokka_halftone_rolls_off_toward_the_left_content_side: no wgpu adapter"
        );
        return;
    };
    let [cx, cy, cw, ch] = card;
    let card_fill = theme::QUOKKA.base_300.rgba_bytes();
    let differs = |px: [u8; 4]| (0..3).any(|k| (px[k] as i16 - card_fill[k] as i16).abs() > 2);
    // Sample a band of rows across the card's lower half (well clear of the
    // header/query row and any single text row), a column near the LEFT
    // content edge (a few px in) and one near the RIGHT decorative edge.
    let y0 = (cy + ch * 0.55) as i64;
    let y1 = (cy + ch * 0.90) as i64;
    let left_x = (cx + cw * 0.06) as i64;
    let right_x = (cx + cw * 0.94) as i64;
    let mut left_hits = 0usize;
    let mut right_hits = 0usize;
    let mut total = 0usize;
    for y in y0..y1 {
        total += 1;
        if differs(px_at(&pixels, w, left_x, y)) {
            left_hits += 1;
        }
        if differs(px_at(&pixels, w, right_x, y)) {
            right_hits += 1;
        }
    }
    assert!(
        right_hits > 0,
        "the right decorative edge should show SOME dot texture (0/{total})"
    );
    assert!(
        right_hits > left_hits,
        "dot texture should be stronger at the right edge ({right_hits}/{total}) than the \
         left content edge ({left_hits}/{total})"
    );
}

/// TEXT/CARD CONTRAST: the selected-row band (drawn over the halftone card)
/// still carries plenty of high-contrast ink pixels — the dot texture never
/// washes out the row text. A regression that let the dots overdraw glyphs
/// would collapse this count toward zero.
#[test]
fn quokka_selected_row_text_stays_legible_over_the_dot_texture() {
    let _g = crate::testlock::serial();
    let Some((pixels, w, h, card)) = render_theme_picker("Quokka") else {
        eprintln!(
            "skipping quokka_selected_row_text_stays_legible_over_the_dot_texture: no wgpu adapter"
        );
        return;
    };
    let [cx, cy, cw, ch] = card;
    let ink = theme::QUOKKA.base_content.rgba_bytes();
    let near_ink = |px: [u8; 4]| (0..3).all(|k| (px[k] as i16 - ink[k] as i16).abs() <= 24);
    let mut ink_pixels = 0usize;
    let x0 = cx.max(0.0) as i64;
    let x1 = ((cx + cw).min(w as f32)) as i64;
    let y0 = cy.max(0.0) as i64;
    let y1 = ((cy + ch).min(h as f32)) as i64;
    for y in y0..y1 {
        for x in x0..x1 {
            if near_ink(px_at(&pixels, w, x, y)) {
                ink_pixels += 1;
            }
        }
    }
    assert!(
        ink_pixels >= 200,
        "expected a healthy floor of real ink pixels (row text) over Quokka's textured \
         card, found only {ink_pixels}"
    );
}

/// Bowerbird's card corner stays the pre-existing small rounded corner
/// (`CardShape::Rectangular`, the default every non-Quokka world carries) —
/// a named regression pin for the world that once gave a non-default
/// `CardTexture`: this proves Bowerbird's card is now
/// unremarkable, distinguishing it from Quokka's chamfer just like
/// [`non_quokka_card_corner_is_not_chamfered`] does generically.
#[test]
fn bowerbird_card_corner_is_not_chamfered() {
    let _g = crate::testlock::serial();
    let Some((pixels, w, _h, card)) = render_theme_picker("Bowerbird") else {
        eprintln!("skipping bowerbird_card_corner_is_not_chamfered: no wgpu adapter");
        return;
    };
    let [cx, cy, _cw, _ch] = card;
    let card_fill = theme::BOWERBIRD.base_300.rgba_bytes();
    let corner_5 = px_at(&pixels, w, (cx + 5.0) as i64, (cy + 5.0) as i64);
    let near = |a: [u8; 4], b: [u8; 4]| (0..3).all(|k| (a[k] as i16 - b[k] as i16).abs() <= 4);
    assert!(
        near(corner_5, card_fill),
        "Bowerbird's card corner must stay the pre-existing small rounded corner (filled at \
         5px inward), got {corner_5:?} vs fill {card_fill:?}"
    );
}

// --- the bar-scrim chamfer leak -------------------------------------------
//
// `panel_card` is not just the CARD-backing pipeline the tests above probe —
// it is also the one `overlay_prepare_bar_scrims` reuses to draw every
// `ListStyle::Bars` world's row scrim (`overlay_row_ink_probe`). `chamfer` is
// a field on the pipeline struct, uploaded fresh only when something calls
// `set_chamfer` that frame; a `BarePlates` world's `ListBacking` never runs
// the CARD branch (`card_shape_texture`) that would derive it from the ACTIVE
// world, so a value a previous frame's chamfered card left behind survives
// into a Bars world's scrim unless the scrim path resets it itself — the same
// "reset on the world that doesn't own this treatment" contract
// `SelectionPipeline::set_dither`'s own doc already states for the dither
// field. A cold, single-world capture can never see this: it never puts two
// worlds through the same pipeline in one process, which is exactly what a
// live theme switch does.

/// Every world whose `list_style` is `ListStyle::Bars`, derived from the
/// roster rather than named, so a newly authored `Bars` world is swept for
/// free.
fn bars_worlds() -> Vec<&'static str> {
    theme::THEMES
        .iter()
        .filter(|t| matches!(t.render_caps.list_style, theme::ListStyle::Bars))
        .map(|t| t.name)
        .collect()
}

/// Every world authoring a non-default `CardShape` — the chamfered-card
/// family — derived from the roster rather than named.
fn chamfered_worlds() -> Vec<&'static str> {
    theme::THEMES
        .iter()
        .filter(|t| !matches!(t.render_caps.card_shape, theme::CardShape::Rectangular))
        .map(|t| t.name)
        .collect()
}

/// Prepare an overlay frame for `after` — optionally PRIMING the SAME
/// pipeline with a `before` world's own frame first, the shared-pipeline
/// shape a live theme switch takes (`before: None` is the control: a
/// pipeline that has never drawn anything else, so `panel_card` still
/// carries its construction-default `(0.0, 0.0)` chamfer). Returns `before`'s
/// own `panel_card` chamfer pair where primed (a non-vacuity check: `before`
/// must actually have chamfered it), `after`'s post-frame `panel_card`
/// chamfer pair, `after`'s scrim rects, and the rendered pixels of the
/// `after` frame.
#[allow(clippy::type_complexity)]
fn render_bars_scrim(
    before: Option<&str>,
    after: &str,
) -> Option<(
    (f32, f32),
    (f32, f32),
    Vec<[f32; 4]>,
    Vec<[u8; 4]>,
    i64,
    i64,
)> {
    let _g = crate::testlock::serial();
    let (device, queue, mut p) = headless_dqp(1200.0, 800.0)?;
    let mut v = super::view("hello world\n", 0, 0);
    v.overlay_active = true;
    v.overlay_title = "commands";
    v.overlay_items = (0..8).map(|i| format!("Command {i}")).collect();

    let mut before_chamfer = (0.0f32, 0.0f32);
    if let Some(before) = before {
        theme::set_active_by_name(before).unwrap();
        p.sync_theme();
        p.set_view(&v);
        p.prepare(&device, &queue, 1200, 800).unwrap();
        before_chamfer = p.panel_card.chamfer();
    }

    theme::set_active_by_name(after).unwrap();
    p.sync_theme();
    p.set_view(&v);
    p.prepare(&device, &queue, 1200, 800).unwrap();
    let scrims = p.overlay_row_ink_probe();
    let after_chamfer = p.panel_card.chamfer();
    let pixels = pixeldiff::render_frame(&mut p, &device, &queue, 1200, 800);

    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
    Some((before_chamfer, after_chamfer, scrims, pixels, 1200, 800))
}

/// **THE LEAK LAW.** Every `Bars` world, prepared right after every
/// chamfered-card world on the same pipeline, must still carry a ZERO
/// `panel_card` chamfer pair — a no-wildcard roster PRODUCT (every `Bars`
/// world × every chamfered world), so neither roster can silently dodge the
/// sweep by growing a new member.
#[test]
fn bar_scrim_chamfer_does_not_leak_from_a_chamfered_world() {
    let bars = bars_worlds();
    let chamfered = chamfered_worlds();
    assert!(
        !bars.is_empty(),
        "the Bars roster is empty — this law would sweep nothing"
    );
    assert!(
        !chamfered.is_empty(),
        "the chamfered-card roster is empty — this law would sweep nothing"
    );
    let mut graded = 0usize;
    for &before in &chamfered {
        for &after in &bars {
            let Some((before_chamfer, after_chamfer, scrims, ..)) =
                render_bars_scrim(Some(before), after)
            else {
                eprintln!(
                    "skipping bar_scrim_chamfer_does_not_leak_from_a_chamfered_world: \
                     no wgpu adapter"
                );
                return;
            };
            // A per-half chamfer axis lets a world cut only ONE half
            // (Cassowary: top 0.0, bottom > 0.0) — so "a real chamfer" means
            // EITHER half is nonzero, not both.
            assert!(
                before_chamfer.0 > 0.0 || before_chamfer.1 > 0.0,
                "{before}: its own card carried a {before_chamfer:?} chamfer, not a real \
                 one — this fixture isn't exercising the chamfered state this law depends on"
            );
            assert!(
                !scrims.is_empty(),
                "{after} after {before}: the scrim probe drew nothing — this fixture isn't \
                 exercising the scrim path this law grades"
            );
            assert_eq!(
                after_chamfer,
                (0.0, 0.0),
                "{after}'s row scrim carries a {after_chamfer:?} chamfer left over from \
                 {before}'s own chamfered card — a Bars world's scrim must reset it every frame"
            );
            graded += 1;
        }
    }
    assert_eq!(
        graded,
        bars.len() * chamfered.len(),
        "must grade the full Bars x chamfered-world roster product"
    );
}

/// **THE APPEARANCE PROOF.** The state law above catches the leaked NUMBER;
/// this catches what it actually DRAWS. A `Bars` world's scrim corner region
/// must render PIXEL-IDENTICAL whether or not a chamfered world happened to
/// prepare a frame on the same pipeline first — CONTROL (`before: None`, a
/// pipeline that has drawn nothing else) against TEST (primed with a
/// representative chamfered world). This never has to guess the scrim's own
/// fill color (whether the 5px-inward point lands in the scrim's own halo or
/// the plate it backs is exactly the kind of detail this sidesteps): if a
/// chamfer leaks, SOMETHING in that corner box changes, whatever it is drawn
/// over. Both representatives are the rosters' own first member (still
/// roster-derived, not a name pinned in the law), so a roster reordering
/// cannot silently point this at nothing.
#[test]
fn bar_scrim_corner_pixels_are_unchanged_by_a_preceding_chamfered_world() {
    let bars = bars_worlds();
    let chamfered = chamfered_worlds();
    let (Some(&after), Some(&before)) = (bars.first(), chamfered.first()) else {
        eprintln!(
            "skipping bar_scrim_corner_pixels_are_unchanged_by_a_preceding_chamfered_world: \
             an empty roster"
        );
        return;
    };
    let Some((_, _, control_scrims, control_pixels, w, _h)) = render_bars_scrim(None, after) else {
        eprintln!(
            "skipping bar_scrim_corner_pixels_are_unchanged_by_a_preceding_chamfered_world: \
             no wgpu adapter"
        );
        return;
    };
    let Some((before_chamfer, after_chamfer, scrims, pixels, ..)) =
        render_bars_scrim(Some(before), after)
    else {
        eprintln!(
            "skipping bar_scrim_corner_pixels_are_unchanged_by_a_preceding_chamfered_world: \
             no wgpu adapter"
        );
        return;
    };
    assert!(
        before_chamfer.0 > 0.0 || before_chamfer.1 > 0.0,
        "{before} must carry a real card chamfer, got {before_chamfer:?}"
    );
    assert_eq!(
        after_chamfer,
        (0.0, 0.0),
        "{after}'s panel_card carries a leaked {after_chamfer:?} chamfer from {before}"
    );
    assert_eq!(
        scrims, control_scrims,
        "{after}'s scrim geometry differs depending on whether {before} rendered first — a \
         differing rect would make the pixel comparison below meaningless"
    );
    let Some(&[sx, sy, sw, sh]) = scrims.first() else {
        panic!("{after} after {before}: no scrim rect to grade");
    };
    assert!(
        sw > 12.0 && sh > 12.0,
        "the scrim [{sx}, {sy}, {sw}, {sh}] is too small to probe an 8x8 corner box"
    );
    let mut differing = 0usize;
    for dy in 0..8i64 {
        for dx in 0..8i64 {
            let (x, y) = (sx as i64 + dx, sy as i64 + dy);
            if px_at(&pixels, w, x, y) != px_at(&control_pixels, w, x, y) {
                differing += 1;
            }
        }
    }
    assert_eq!(
        differing, 0,
        "{after}'s scrim corner has {differing}/64 pixels that differ depending on whether \
         {before} rendered first on the same pipeline — a chamfer leaking from {before} changes \
         what gets drawn there"
    );
}
