//! ITEM 158 — REAL-PIXEL laws for `Background::Deckle`, the handmade-paper
//! material field, and for Paperbark, its one production assignee.
//!
//! Every claim here is DIFFERENTIAL: the world as authored MINUS the same
//! world at `density: 0.0`, through the shared `mark_field` oracle item 86
//! introduced. That is why the shader's `density == 0.0` arm was designed to
//! collapse both weaves to an exactly flat ground — the gradient, the ordered
//! dither and the 8-bit quantization cancel, and what remains is the material
//! alone.
//!
//! **THE AXIS THIS FILE SWEEPS is the MARGIN'S OWN WIDTH.** Deckle's Strata
//! weave indexes its lanes on DISTANCE FROM THE PAGE COLUMN, so how many lanes
//! a margin can show is a property of that margin, not of the window: at a
//! fixed 1400px window a 60-char measure leaves ~236px of margin (two and a
//! half lanes) and a 100-char measure leaves ~30px (a fraction of one). The
//! failure that hides at a hand-picked size is therefore the FLAT one — a
//! margin narrower than a lane renders as a single seeded tone, which is
//! precisely "collapses toward Bilby's quiet gradient", the outcome the
//! world's own brief forbids. So the geometry sweep is the whole point, the
//! bound is derived from the field's own pitch rather than assumed, and the
//! DPI axis rides along because `period_px` is PHYSICAL pixels: at 2x DPI the
//! same margin holds twice as many, half as large.
//!
//! Skips (with a printed note, not a failure) on a machine with no wgpu
//! adapter, like every other GPU-backed render test in this tree.

use super::backgrounds_item69::{bg_desc_for, headless_dq, render_bg};
use super::backgrounds_item89::{SWEEP, margins, mark_field};
use super::{headless_dqp, view};
use crate::theme::{self, Background, DeckleAnchor, Weave};

/// Per-pixel total-channel deviation from the mark-free pass that counts as
/// real material. The differential oracle cancels the dither exactly, so this
/// only has to clear 8-bit quantization (item 89's own floor).
const INK_FLOOR: i32 = 3;

fn paperbark_bg() -> Background {
    theme::PAPERBARK.background
}

/// The DORMANT `Weave::Fibres` profile under test here. Item 159 gives it a
/// world; until then this literal is what keeps the arm honest — a variant
/// nothing constructs is a variant nobody has proven draws anything.
fn fibres_bg() -> Background {
    with_weave(Weave::Fibres, 88.0, 25.0)
}

/// Paperbark's ground with one field swapped — so a law can isolate the WEAVE
/// from the dials instead of measuring both at once.
fn with_weave(weave: Weave, period_px: f32, wander_px: f32) -> Background {
    match paperbark_bg() {
        Background::Deckle {
            ground,
            layer,
            deckle,
            anchor,
            ..
        } => Background::Deckle {
            ground,
            layer,
            deckle,
            weave,
            anchor,
            period_px,
            wander_px,
            density: 0.20,
        },
        _ => unreachable!("Paperbark ships Background::Deckle"),
    }
}

fn with_anchor(anchor: DeckleAnchor) -> Background {
    match paperbark_bg() {
        Background::Deckle {
            ground,
            layer,
            deckle,
            weave,
            period_px,
            wander_px,
            density,
            ..
        } => Background::Deckle {
            ground,
            layer,
            deckle,
            weave,
            anchor,
            period_px,
            wander_px,
            density,
        },
        _ => unreachable!("Paperbark ships Background::Deckle"),
    }
}

fn with_weave_and_anchor(weave: Weave, anchor: DeckleAnchor) -> Background {
    match paperbark_bg() {
        Background::Deckle {
            ground,
            layer,
            deckle,
            period_px,
            wander_px,
            density,
            ..
        } => Background::Deckle {
            ground,
            layer,
            deckle,
            weave,
            anchor,
            period_px,
            wander_px,
            density,
        },
        _ => unreachable!("Paperbark ships Background::Deckle"),
    }
}

/// DATA-LEVEL ROSTER, NO WILDCARD: a future ground variant must decide its
/// Deckle story here, and a second Deckle world must be a deliberate edit.
#[test]
fn deckle_is_paperbarks_alone_and_fibres_has_no_assignee_no_wildcard() {
    for t in theme::THEMES {
        let weave = match t.background {
            Background::Gradient { .. } => None,
            Background::Dots { .. } => None,
            Background::Starfield { .. } => None,
            Background::Pinstripe { .. } => None,
            Background::Stripes { .. } => None,
            Background::Lava { .. } => None,
            Background::Bands { .. } => None,
            Background::Waves { .. } => None,
            Background::Zigzag { .. } => None,
            Background::Organic { .. } => None,
            Background::Deckle { weave, .. } => Some(weave),
        };
        assert_eq!(
            weave.is_some(),
            t.name == "Paperbark",
            "{}: Deckle is Paperbark's alone",
            t.name
        );
        assert_ne!(
            weave,
            Some(Weave::Fibres),
            "{}: Weave::Fibres is reusable infrastructure with NO assignee until \
             a world deliberately claims it (item 159) — this is the `Bands` / \
             `Dots {{ edge: true }}` shape, and claiming it is an edit, not a drift",
            t.name
        );
    }
    // The weave is a THEME-OWNED scalar, and the inert default is what every
    // other ground reports — the shader's own `params.w` slot never changes
    // shape for a world that has no weave.
    for t in theme::THEMES {
        let want = if t.name == "Paperbark" {
            Weave::Strata.mode()
        } else {
            0.0
        };
        assert_eq!(
            t.background.weave_mode(),
            want,
            "{}: weave_mode is inert off Deckle",
            t.name
        );
    }
    assert_eq!(Weave::Strata.mode(), 0.0);
    assert_eq!(Weave::Fibres.mode(), 1.0);
    for t in theme::THEMES {
        let want = if t.name == "Paperbark" {
            DeckleAnchor::Viewport.mode()
        } else {
            0.0
        };
        assert_eq!(
            t.background.deckle_anchor_mode(),
            want,
            "{}: Deckle anchor is inert outside its one assignee",
            t.name
        );
    }
}

/// THE PAGE STAYS FLAT AND OPAQUE. Not one pixel of material may enter the
/// writing column, at ANY swept geometry or DPI — the material is the room,
/// the page is the figure.
#[test]
fn deckle_ink_never_enters_the_writing_page_at_any_swept_geometry() {
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping deckle_ink_never_enters_the_writing_page: no wgpu adapter");
        return;
    };
    let _g = crate::testlock::serial();
    let (was_page, was_measure, was_theme) = (
        crate::page::page_on(),
        crate::page::measure(),
        theme::active().name,
    );
    crate::page::set_page_on(true);
    theme::set_active_by_name("Paperbark").unwrap();
    p.sync_theme();

    let bg = paperbark_bg();
    for (ww, wh, measure) in SWEEP {
        crate::page::set_measure(measure);
        p.set_size(ww as f32, wh as f32);
        p.set_view(&view("some plain prose here, no headings at all\n", 0, 0));
        let (col_left, col_w) = (p.column_left(), p.column_width());
        let field = mark_field(&device, &queue, bg, ww, wh, col_left, col_w);
        let x0 = col_left.max(0.0).ceil() as u32;
        let x1 = ((col_left + col_w).floor() as u32).min(ww);
        for y in 0..wh {
            for x in x0..x1 {
                assert_eq!(
                    field[(y * ww + x) as usize],
                    0,
                    "{ww}x{wh}@{measure}: deckle material entered the writing page at ({x},{y})"
                );
            }
        }
    }
    crate::page::set_measure(was_measure);
    crate::page::set_page_on(was_page);
    theme::set_active_by_name(was_theme).unwrap();
    p.sync_theme();
}

/// A margin's own material statistics: how many distinct strata VALUES a
/// vertical scan crosses, and the total tonal range the margin spans. Both are
/// read off the differential field, so they measure the MATERIAL, never the
/// gradient underneath it.
struct MarginStats {
    /// Peak absolute deviation anywhere in the margin.
    peak: i32,
    /// Number of lane BOUNDARIES a mid-height horizontal scan crosses — a
    /// run-length count of sign-stable bands, so an antialiased edge is one
    /// crossing, not fifty.
    bands: usize,
    /// The tonal SPREAD the margin shows: max deviation minus min deviation
    /// across the whole margin. A margin that collapsed to one flat lane has
    /// a spread near zero however dark that lane is.
    spread: i32,
    /// How many DISTINCT tones the lane INTERIORS take. `spread` and `bands`
    /// are both satisfied by the deckled boundary alone, so a field whose
    /// lanes all drew one seeded tone — layered paper degraded to a ruled
    /// grid — would slip past them. This counts the long runs only (a boundary
    /// feather is short), so it measures the layering itself.
    lane_tones: usize,
}

fn margin_stats(field: &[i32], w: u32, h: u32, mx0: u32, mx1: u32) -> MarginStats {
    let at = |x: u32, y: u32| field[(y * w + x) as usize];
    let mut peak = 0;
    let mut lo = i32::MAX;
    let mut hi = i32::MIN;
    for y in 0..h {
        for x in mx0..mx1 {
            let v = at(x, y);
            peak = peak.max(v.abs());
            lo = lo.min(v);
            hi = hi.max(v);
        }
    }
    // Band count on the ACROSS-LANE axis. Strata lanes run parallel to the
    // page edge, so the axis that crosses them is x — the same "scan the axis
    // the marks travel across, never the screen axis that looks natural"
    // lesson item 89's blank-lane law was built on.
    let mut bands = 0usize;
    let mut samples = 0usize;
    let mut tones: std::collections::HashSet<i32> = std::collections::HashSet::new();
    for y in (h / 8..h.saturating_sub(h / 8)).step_by((h as usize / 12).max(1)) {
        let mut last: Option<i32> = None;
        let mut run = 0u32;
        let mut row = 0usize;
        let close = |v: i32, len: u32, tones: &mut std::collections::HashSet<i32>| {
            // A LANE INTERIOR, not a boundary feather: the deckled edge is a
            // few percent of a lane, so anything this long is lane body.
            if len >= 15 {
                tones.insert(v);
            }
        };
        for x in mx0..mx1 {
            // Quantize to the field's own coarse steps so antialias feathering
            // inside one lane is not counted as a boundary.
            let q = at(x, y) / 6;
            match last {
                Some(l) if l == q => run += 1,
                Some(l) => {
                    close(l, run, &mut tones);
                    row += 1;
                    run = 1;
                }
                None => run = 1,
            }
            last = Some(q);
        }
        if let Some(l) = last {
            close(l, run, &mut tones);
        }
        bands += row;
        samples += 1;
    }
    MarginStats {
        peak,
        bands: bands / samples.max(1),
        spread: if lo <= hi { hi - lo } else { 0 },
        lane_tones: tones.len(),
    }
}

/// THE HEADLINE LAW — A REAL MARGIN NEVER COLLAPSES TO ONE FLAT TONE.
///
/// Swept over item 89's twelve `(window, measure)` shapes at the app's OWN
/// adaptive column owner, AND over 1x/2x DPI (which halves the field's
/// apparent scale, because `period_px` is physical pixels), on every margin
/// wide enough to hold one authored lane pitch. Each such margin must show:
///
///   * real material at all (`peak` clears the ink floor), and
///   * a genuine tonal SPREAD, not one seeded lane painted edge to edge, and
///   * at least one lane boundary crossed by a horizontal scan.
///
/// The bound is DERIVED from the field's own dials — a margin is only held to
/// this once it is at least `period_px` wide — rather than assumed from a
/// hand-picked window size. A margin narrower than a lane is the one case the
/// geometry genuinely cannot show strata in, and this law says so out loud
/// instead of quietly passing everywhere.
#[test]
fn deckle_strata_never_collapses_to_one_flat_tone_in_a_margin_that_can_hold_a_lane() {
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!("skipping deckle_strata_never_collapses_to_one_flat_tone: no wgpu adapter");
        return;
    };
    let _g = crate::testlock::serial();
    let (was_page, was_measure, was_theme) = (
        crate::page::page_on(),
        crate::page::measure(),
        theme::active().name,
    );
    crate::page::set_page_on(true);
    theme::set_active_by_name("Paperbark").unwrap();
    p.sync_theme();

    let bg = paperbark_bg();
    let pitch = bg.period_px();
    let mut graded = 0usize;
    let mut tightest_spread = i32::MAX;
    for dpi in [1u32, 2u32] {
        for (ww, wh, measure) in SWEEP {
            crate::page::set_measure(measure);
            p.set_size(ww as f32, wh as f32);
            p.set_view(&view("some plain prose here, no headings at all\n", 0, 0));
            let (col_left, col_w) = (p.column_left(), p.column_width());
            let (pw, ph) = (ww * dpi, wh * dpi);
            let (cl, cw) = (col_left * dpi as f32, col_w * dpi as f32);
            let field = mark_field(&device, &queue, bg, pw, ph, cl, cw);
            for (mi, (mx0, mx1)) in margins(pw, cl, cw).iter().enumerate() {
                let width = mx1.saturating_sub(*mx0);
                if (width as f32) < pitch {
                    continue;
                }
                let s = margin_stats(&field, pw, ph, *mx0, *mx1);
                let where_ = format!("{ww}x{wh}@{measure} dpi{dpi} margin{mi} ({width}px)");
                assert!(
                    s.peak >= INK_FLOOR,
                    "{where_}: the deckle field must reach real material (peak {})",
                    s.peak
                );
                assert!(
                    s.spread >= 8,
                    "{where_}: the margin renders as ONE flat tone (spread {}, peak {}) — \
                     a margin at least a lane wide must show its strata, or Paperbark \
                     collapses toward the quiet gradient its own brief forbids",
                    s.spread,
                    s.peak
                );
                assert!(
                    s.bands >= 2,
                    "{where_}: a horizontal scan crossed {} tonal bands — a margin at \
                     least a lane wide must cross a lane boundary",
                    s.bands
                );
                // Two lanes' worth of margin must show two LAYERS, not one tone
                // ruled by a boundary. `spread` and `bands` are both satisfied
                // by the deckled edge alone; this is the claim about the paper.
                if (width as f32) >= pitch * 2.0 {
                    assert!(
                        s.lane_tones >= 2,
                        "{where_}: the lane interiors take only {} tone(s) — the strata \
                         degraded to a ruled grid, which is layered paper losing its layers",
                        s.lane_tones
                    );
                }
                tightest_spread = tightest_spread.min(s.spread);
                graded += 1;
            }
        }
    }
    assert!(
        graded >= 20,
        "the sweep must actually grade margins (graded {graded})"
    );
    eprintln!("deckle collapse law: {graded} margins graded, tightest spread {tightest_spread}");
    crate::page::set_measure(was_measure);
    crate::page::set_page_on(was_page);
    theme::set_active_by_name(was_theme).unwrap();
    p.sync_theme();
}

/// THE WALLPAPER LAW (item 175): a page-width drag changes only the opaque page
/// mask. Any screen point exposed before AND after the drag is the SAME fixed
/// Room wallpaper pixel. The `Page` mutation arm restores the old near-edge
/// sampling and must produce a named mismatch, proving this is not a vacuous
/// byte-identity assertion over a wholly-covered region.
#[test]
fn deckle_strata_stays_fixed_at_exposed_viewport_points_across_page_width_drags() {
    let Some((device, queue)) = headless_dq() else {
        eprintln!("skipping deckle_strata_mirror_across_the_writing_page: no wgpu adapter");
        return;
    };
    let _g = crate::testlock::serial();
    let geometry = [
        (1600u32, 900u32, (400.0f32, 800.0f32), (300.0f32, 1000.0f32)),
        (1200, 800, (350.0, 500.0), (210.0, 780.0)),
    ];
    for (w, h, first_col, second_col) in geometry {
        let stable = with_anchor(DeckleAnchor::Viewport);
        let a = render_bg(
            &device,
            &queue,
            bg_desc_for(stable),
            w,
            h,
            first_col.0,
            first_col.1,
            0.0,
        );
        let b = render_bg(
            &device,
            &queue,
            bg_desc_for(stable),
            w,
            h,
            second_col.0,
            second_col.1,
            0.0,
        );
        let exposed = |x: f32| {
            (x < first_col.0 || x >= first_col.0 + first_col.1)
                && (x < second_col.0 || x >= second_col.0 + second_col.1)
        };
        let mut checked = 0usize;
        for (i, (left, right)) in a.iter().zip(&b).enumerate() {
            let x = (i as u32 % w) as f32;
            if exposed(x) {
                assert_eq!(
                    left,
                    right,
                    "{w}x{h}: exposed wallpaper moved at ({x},{}) across page-width drag",
                    i as u32 / w
                );
                checked += 1;
            }
        }
        assert!(
            checked > (w * h / 4) as usize,
            "{w}x{h}: intersection must be substantial"
        );

        let legacy = with_anchor(DeckleAnchor::Page);
        let legacy_a = render_bg(
            &device,
            &queue,
            bg_desc_for(legacy),
            w,
            h,
            first_col.0,
            first_col.1,
            0.0,
        );
        let legacy_b = render_bg(
            &device,
            &queue,
            bg_desc_for(legacy),
            w,
            h,
            second_col.0,
            second_col.1,
            0.0,
        );
        let moved = legacy_a
            .iter()
            .zip(&legacy_b)
            .enumerate()
            .filter(|(i, (left, right))| exposed((*i as u32 % w) as f32) && left != right)
            .count();
        assert!(
            moved > (w * h / 100) as usize,
            "mutation witness at {w}x{h}: page-relative Deckle moved only {moved} exposed pixels"
        );
    }
}

/// Deckle's packed mode is total over the two independent controls: Fibres is
/// a screen-space material regardless of anchor, while Strata alone selects a
/// coordinate owner. The exhaustive match is deliberately wildcard-free, so a
/// future Weave or DeckleAnchor variant cannot silently miss this law.
#[test]
fn deckle_mode_sweeps_every_weave_anchor_combination() {
    let Some((device, queue)) = headless_dq() else {
        eprintln!("skipping deckle_mode_sweeps_every_weave_anchor_combination: no wgpu adapter");
        return;
    };
    let _g = crate::testlock::serial();
    let (w, h, cl, cw) = (1400, 800, 350.0, 700.0);
    let strata_viewport = render_bg(
        &device,
        &queue,
        bg_desc_for(with_weave_and_anchor(Weave::Strata, DeckleAnchor::Viewport)),
        w,
        h,
        cl,
        cw,
        0.0,
    );
    let strata_page = render_bg(
        &device,
        &queue,
        bg_desc_for(with_weave_and_anchor(Weave::Strata, DeckleAnchor::Page)),
        w,
        h,
        cl,
        cw,
        0.0,
    );
    let fibres_viewport = render_bg(
        &device,
        &queue,
        bg_desc_for(with_weave_and_anchor(Weave::Fibres, DeckleAnchor::Viewport)),
        w,
        h,
        cl,
        cw,
        0.0,
    );
    let fibres_page = render_bg(
        &device,
        &queue,
        bg_desc_for(with_weave_and_anchor(Weave::Fibres, DeckleAnchor::Page)),
        w,
        h,
        cl,
        cw,
        0.0,
    );

    for ((weave, anchor), pixels) in [
        ((Weave::Strata, DeckleAnchor::Viewport), &strata_viewport),
        ((Weave::Strata, DeckleAnchor::Page), &strata_page),
        ((Weave::Fibres, DeckleAnchor::Viewport), &fibres_viewport),
        ((Weave::Fibres, DeckleAnchor::Page), &fibres_page),
    ] {
        match (weave, anchor) {
            (Weave::Strata, DeckleAnchor::Viewport) => assert_ne!(
                pixels, &strata_page,
                "viewport Strata must not fall through to page-relative Strata"
            ),
            (Weave::Strata, DeckleAnchor::Page) => assert_ne!(
                pixels, &strata_viewport,
                "page-relative Strata must select its own coordinate owner"
            ),
            (Weave::Fibres, DeckleAnchor::Viewport) => {
                assert_eq!(pixels, &fibres_page, "Fibres must ignore its anchor")
            }
            (Weave::Fibres, DeckleAnchor::Page) => {
                assert_eq!(pixels, &fibres_viewport, "Fibres must ignore its anchor")
            }
        }
    }
}

/// THE MATERIAL WHISPERS AND NEVER COMPETES WITH INK. Two bounds, both in real
/// pixels over the fully composited margin (not the differential — the reader's
/// eye sees the composite):
///
///   * every margin pixel stays clearly LIGHTER than the world's own prose ink,
///     so the page-mode gutter label and the margin outline stay readable over
///     it; and
///   * the material's own peak deviation stays under a ceiling, so it reads as
///     paper rather than as a pattern demanding attention.
#[test]
fn deckle_material_whispers_and_keeps_clear_of_the_prose_ink() {
    let Some((device, queue)) = headless_dq() else {
        eprintln!("skipping deckle_material_whispers: no wgpu adapter");
        return;
    };
    let _g = crate::testlock::serial();
    let (w, h, cl, cw) = (1600u32, 900u32, 400.0f32, 800.0f32);
    let bg = paperbark_bg();
    let composited = render_bg(&device, &queue, bg_desc_for(bg), w, h, cl, cw, 0.0);
    let field = mark_field(&device, &queue, bg, w, h, cl, cw);

    let ink = theme::PAPERBARK.base_content;
    let lum = |c: [u8; 4]| 0.2126 * c[0] as f32 + 0.7152 * c[1] as f32 + 0.0722 * c[2] as f32;
    let ink_lum = 0.2126 * ink.r as f32 + 0.7152 * ink.g as f32 + 0.0722 * ink.b as f32;

    let mut darkest = f32::MAX;
    let mut peak = 0i32;
    for (mx0, mx1) in margins(w, cl, cw) {
        for y in 0..h {
            for x in mx0..mx1 {
                darkest = darkest.min(lum(composited[(y * w + x) as usize]));
                peak = peak.max(field[(y * w + x) as usize].abs());
            }
        }
    }
    assert!(
        darkest - ink_lum >= 80.0,
        "the darkest deckle pixel (luma {darkest:.1}) must stay well clear of the prose \
         ink (luma {ink_lum:.1}) — the margin carries the gutter label and the outline"
    );
    assert!(
        peak >= INK_FLOOR * 3,
        "the material must be visible at all (peak {peak})"
    );
    assert!(
        peak <= 120,
        "the material must stay a whisper, not a pattern (peak {peak})"
    );
}

/// NO MOIRE AT THE SHADER'S OWN FLOOR. Deckle's deckled edge is a FRACTION of
/// a lane, so a lane finer than a few pixels puts that edge under one pixel and
/// the field aliases. The shader CLAMPS the pitch to `DECKLE_MIN_PITCH_PX`
/// rather than trusting the dial (item 89's abutment lesson: coverage is a
/// property of the shader, not of a dial pair), so this law drives an
/// absurdly fine authored pitch straight at it and asserts the result is still
/// smooth — measured as the fraction of horizontally adjacent margin pixels
/// that swing by more than a lane's whole tonal range, which is what a moire
/// looks like numerically.
#[test]
fn deckle_pitch_floor_holds_the_field_smooth_below_its_own_minimum() {
    let Some((device, queue)) = headless_dq() else {
        eprintln!("skipping deckle_pitch_floor_holds_the_field_smooth: no wgpu adapter");
        return;
    };
    let _g = crate::testlock::serial();
    let (w, h, cl, cw) = (1400u32, 800u32, 400.0f32, 600.0f32);
    let mut worst = 0.0f32;
    // Sweep DOWN through and PAST the authored floor, including values a
    // careless future dial could hold, at both DPI scales.
    for pitch in [
        theme::DECKLE_MAX_PERIOD_PX,
        94.0,
        theme::DECKLE_MIN_PERIOD_PX,
        12.0,
        3.0,
        0.5,
    ] {
        let bg = match paperbark_bg() {
            Background::Deckle {
                ground,
                layer,
                deckle,
                weave,
                anchor,
                wander_px,
                density,
                ..
            } => Background::Deckle {
                ground,
                layer,
                deckle,
                weave,
                anchor,
                period_px: pitch,
                wander_px,
                density,
            },
            _ => unreachable!(),
        };
        let field = mark_field(&device, &queue, bg, w, h, cl, cw);
        let mut jumps = 0usize;
        let mut total = 0usize;
        for (mx0, mx1) in margins(w, cl, cw) {
            for y in 0..h {
                for x in mx0..mx1.saturating_sub(1) {
                    let a = field[(y * w + x) as usize];
                    let b = field[(y * w + x + 1) as usize];
                    if (a - b).abs() > 40 {
                        jumps += 1;
                    }
                    total += 1;
                }
            }
        }
        let frac = jumps as f32 / total.max(1) as f32;
        worst = worst.max(frac);
        assert!(
            frac < 0.02,
            "pitch {pitch}: {:.2}% of adjacent margin pixels swing hard — the field is \
             aliasing, which is what DECKLE_MIN_PITCH_PX exists to prevent",
            frac * 100.0
        );
    }
    eprintln!(
        "deckle moire law: worst adjacent-swing fraction {:.4}%",
        worst * 100.0
    );
}

/// The COMPARISON GEOMETRY the done-clause law measures all three worlds at.
const CMP: (u32, u32, f32, f32) = (1600, 900, 400.0, 800.0);

/// Mark POSITIONS along the one axis all three grounds are crossed by (x:
/// Deckle's lanes run parallel to the page edge, Saltpan's pinstripes are
/// vertical rules). A mark is a local luminance MINIMUM with real prominence,
/// so dither and 8-bit quantization cannot invent one.
fn marks_in_row(px: &[[u8; 4]], y: u32, x0: u32, x1: u32) -> Vec<f32> {
    let w = CMP.0;
    let lum = |x: u32| {
        let c = px[(y * w + x) as usize];
        0.2126 * c[0] as f32 + 0.7152 * c[1] as f32 + 0.0722 * c[2] as f32
    };
    let mut out: Vec<f32> = Vec::new();
    for x in (x0 + 3)..x1.saturating_sub(3) {
        let here = lum(x);
        let around = (x - 3..=x + 3).map(lum).fold(f32::MIN, f32::max);
        let is_min = (x - 3..=x + 3).all(|k| lum(k) >= here - 0.01);
        if is_min && around - here >= 3.0 && !out.last().is_some_and(|p| x as f32 - p < 4.0) {
            out.push(x as f32);
        }
    }
    out
}

/// The rows and the left-margin span every measurement below shares.
fn cmp_rows() -> (Vec<u32>, u32, u32) {
    let (_, h, cl, _) = CMP;
    ((80..h - 80).step_by(17).collect(), 8, cl as u32 - 8)
}

/// HOW BROAD: the mean gap between boundary marks, and how many were found at
/// all. A layered handmade sheet lays down wide lanes; a ledger rule is fine
/// and frequent.
fn mean_gap(px: &[[u8; 4]]) -> (f32, usize) {
    let (rows, x0, x1) = cmp_rows();
    let mut gaps = Vec::new();
    let mut seen = 0usize;
    for y in rows {
        let m = marks_in_row(px, y, x0, x1);
        seen += m.len();
        gaps.extend(m.windows(2).map(|p| p[1] - p[0]));
    }
    let g = if gaps.is_empty() {
        0.0
    } else {
        gaps.iter().sum::<f32>() / gaps.len() as f32
    };
    (g, seen)
}

/// HOW STRAIGHT: track the boundary nearest the margin's midpoint down the rows
/// and measure how far its x wanders. THIS is "deckled" as a number — a torn
/// edge moves, a printed rule does not.
fn boundary_wander(px: &[[u8; 4]]) -> f32 {
    let (rows, x0, x1) = cmp_rows();
    let mid = (x0 + x1) as f32 * 0.5;
    let xs: Vec<f32> = rows
        .iter()
        .filter_map(|&y| {
            marks_in_row(px, y, x0, x1)
                .into_iter()
                .min_by(|a, b| (a - mid).abs().total_cmp(&(b - mid).abs()))
        })
        .collect();
    if xs.len() < 8 {
        return 0.0;
    }
    let mean = xs.iter().sum::<f32>() / xs.len() as f32;
    (xs.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / xs.len() as f32).sqrt()
}

/// THE DONE CLAUSE, AS PIXELS. Paperbark must not read as Saltpan's regular
/// pinstripes or Bilby's quiet gradient.
///
/// The differential `mark_field` oracle CANNOT be used for the comparison
/// worlds: `density` gates Deckle's, Zigzag's and Organic's marks, but
/// Pinstripe's coverage never reads it, so a "bare" Saltpan still draws every
/// stripe and the differential comes out empty. Measuring the reference that
/// way would make this law vacuously true, which is exactly the shape of a
/// green law hiding a real defect. So all three worlds are measured the same
/// honest way instead — over their COMPOSITED margin pixels, by locating the
/// dark boundary marks a reader actually sees:
///
///   * BILBY (a gradient) must show none at all;
///   * SALTPAN's must be REGULAR — its mark gaps have a small coefficient of
///     variation, because a ruled pinstripe lands on a fixed period;
///   * PAPERBARK's must be several times more irregular, because its lanes are
///     seeded per layer and torn by the wander profile. A handmade sheet, not
///     a printed rule.
#[test]
fn paperbark_reads_as_neither_saltpans_pinstripes_nor_bilbys_gradient() {
    let Some((device, queue)) = headless_dq() else {
        eprintln!("skipping paperbark_reads_as_neither: no wgpu adapter");
        return;
    };
    let _g = crate::testlock::serial();
    let (w, h, cl, cw) = CMP;

    let shot = |bg| render_bg(&device, &queue, bg_desc_for(bg), w, h, cl, cw, 0.0);
    let (pb, sp, bi) = (
        shot(paperbark_bg()),
        shot(theme::SALTPAN.background),
        shot(theme::BILBY.background),
    );
    let (pb_gap, pb_marks) = mean_gap(&pb);
    let (sp_gap, sp_marks) = mean_gap(&sp);
    let (_, bi_marks) = mean_gap(&bi);
    let (pb_wander, sp_wander) = (boundary_wander(&pb), boundary_wander(&sp));

    // NOT BILBY: a gradient carries no boundary at all.
    assert_eq!(
        bi_marks, 0,
        "Bilby's gradient must carry NO boundary marks (found {bi_marks}) — if it does, \
         this whole comparison is measuring noise"
    );
    assert!(
        pb_marks > 40,
        "Paperbark's margin must carry real lane boundaries (found {pb_marks})"
    );
    assert!(
        sp_marks > 400,
        "the Saltpan reference must actually draw its pinstripes (found {sp_marks}) — \
         a reference that draws nothing proves nothing"
    );

    // NOT SALTPAN, on two independent axes.
    assert!(
        pb_gap > sp_gap * 5.0,
        "Paperbark's lanes ({pb_gap:.1}px apart) must be far broader than Saltpan's \
         ruled pinstripes ({sp_gap:.1}px) — layers of a sheet, not a ledger"
    );
    assert!(
        sp_wander <= 1.0,
        "the Saltpan reference must actually be a STRAIGHT rule (its boundary wanders \
         {sp_wander:.2}px) — otherwise the deckle claim below is meaningless"
    );
    assert!(
        pb_wander >= 6.0,
        "Paperbark's lane boundary must WANDER down the margin (it moves {pb_wander:.2}px, \
         Saltpan's {sp_wander:.2}px) — a deckled torn edge is the whole material claim, \
         and a straight one is a recolored pinstripe"
    );
    eprintln!(
        "done-clause law: paperbark gap {pb_gap:.1}px wander {pb_wander:.2}px ({pb_marks} \
         marks) vs saltpan gap {sp_gap:.1}px wander {sp_wander:.2}px ({sp_marks} marks) \
         vs bilby {bi_marks} marks"
    );
}

/// THE DORMANT ARM IS REAL. `Weave::Fibres` has no world, so nothing else in
/// the suite would ever draw it — and a variant nobody draws is a variant that
/// silently rots until item 159 tries to use it. This law renders it through
/// the SAME pipeline and asserts it produces a genuinely DIFFERENT field from
/// `Strata` at identical tones and dials: the weave is a real profile pick,
/// not a slot that happens to be read.
#[test]
fn weave_fibres_draws_a_real_field_distinct_from_strata() {
    let Some((device, queue)) = headless_dq() else {
        eprintln!("skipping weave_fibres_draws_a_real_field: no wgpu adapter");
        return;
    };
    let _g = crate::testlock::serial();
    let (w, h, cl, cw) = (1400u32, 900u32, 400.0f32, 600.0f32);
    let fibres = mark_field(&device, &queue, fibres_bg(), w, h, cl, cw);
    // THE WEAVE ISOLATED: identical tones AND identical dials, so the only
    // thing that can move a pixel is the profile pick itself. Without this the
    // comparison would be measuring the dials as much as the weave, and a
    // weave that silently fell through to Strata could still look "different".
    let same_dials_strata = mark_field(
        &device,
        &queue,
        with_weave(Weave::Strata, 88.0, 25.0),
        w,
        h,
        cl,
        cw,
    );
    let strata = mark_field(&device, &queue, paperbark_bg(), w, h, cl, cw);

    let mut fib_peak = 0i32;
    let mut differing = 0usize;
    let mut total = 0usize;
    for (mx0, mx1) in margins(w, cl, cw) {
        for y in 0..h {
            for x in mx0..mx1 {
                let i = (y * w + x) as usize;
                fib_peak = fib_peak.max(fibres[i].abs());
                if (fibres[i] - same_dials_strata[i]).abs() > 2 {
                    differing += 1;
                }
                total += 1;
            }
        }
    }
    assert!(
        fib_peak >= INK_FLOOR,
        "Weave::Fibres must draw real material (peak {fib_peak}) — a dormant arm that \
         renders nothing is not reusable infrastructure, it is dead code"
    );
    let frac = differing as f32 / total.max(1) as f32;
    assert!(
        frac > 0.50,
        "the two weaves must be genuinely different profiles at IDENTICAL dials \
         ({:.1}% of margin pixels differ) — a weave that falls through to its \
         sibling is a dial item 159 cannot use",
        frac * 100.0
    );
    // And the dials still do their own work on top of the weave.
    let dialled: usize = margins(w, cl, cw)
        .iter()
        .flat_map(|(a, b)| (0..h).flat_map(move |y| (*a..*b).map(move |x| (y * w + x) as usize)))
        .filter(|&i| (fibres[i] - strata[i]).abs() > 2)
        .count();
    assert!(
        dialled * 4 > total,
        "the authored dials must move the field too ({dialled} of {total} pixels)"
    );
    // And it stays off the page, exactly like Strata.
    for y in 0..h {
        for x in (cl.ceil() as u32)..((cl + cw).floor() as u32) {
            assert_eq!(
                fibres[(y * w + x) as usize],
                0,
                "Weave::Fibres material entered the writing page at ({x},{y})"
            );
        }
    }
}

/// HOST/WGSL LOCKSTEP. The four Deckle constants the laws above reason with
/// are mirrors of numbers that actually live in `shaders/background.wgsl`.
/// This is a structural tripwire, not a rendering claim: if the shader's copy
/// is retuned without the host's, every derived bound above becomes a lie that
/// still passes.
#[test]
fn deckle_shader_constants_match_their_host_mirrors() {
    let wgsl = include_str!("../../../shaders/background.wgsl");
    for (name, value) in [
        ("DECKLE_MIN_PITCH_PX", theme::DECKLE_MIN_PERIOD_PX),
        ("DECKLE_MID", theme::DECKLE_MID),
        ("DECKLE_SPREAD_GAIN", theme::DECKLE_SPREAD_GAIN),
    ] {
        let want = format!("const {name}: f32 = {value:?};");
        assert!(
            wgsl.contains(&want),
            "shaders/background.wgsl must declare `{want}` — the host mirror and the GPU \
             have drifted"
        );
    }
    // The weave threshold, and the fact the dispatcher clamps the pitch rather
    // than trusting the dial.
    assert!(
        wgsl.contains("let pitch = max(g.params.x, DECKLE_MIN_PITCH_PX);"),
        "deckle_rgb must CLAMP the authored pitch to its own floor — coverage is a \
         property of the shader, not of a dial pair"
    );
    assert!(
        wgsl.contains("g.params.w >= DECKLE_WEAVE_FIBRES && g.params.w < 1.5"),
        "deckle_rgb must branch on the theme-owned weave scalar while reserving the legacy \
         page-relative arm for this law's mutation proof"
    );
    assert!(
        wgsl.contains("deckle_viewport_distance(px), deckle_page_distance(px), g.params.w >= 1.5"),
        "Strata must default to stable viewport coordinates; page-relative sampling belongs only \
         to the explicit mutation arm"
    );
    // And no world name reaches the DECKLE branch. (The file's older sections
    // name worlds in prose comments; this is the new ground's own contract.)
    let start = wgsl
        .find("// --- 9: DECKLE")
        .expect("the deckle section must be findable");
    let end = wgsl[start..]
        .find("// ITEM 69 FOLLOW-UP")
        .expect("the deckle section must end at its neighbour");
    // ...in its CODE. The prose above `deckle_rgb` names its assignee, exactly
    // as every other ground's comments do; what must never happen is a world
    // name reaching an expression.
    let code: String = wgsl[start..start + end]
        .lines()
        .map(|l| l.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n");
    for t in theme::THEMES {
        assert!(
            !code.contains(t.name),
            "the deckle shader CODE names the world {:?} — grounds are data",
            t.name
        );
    }
}

/// PAPERBARK'S OWN DIALS STAY INSIDE THE FAMILY'S AUTHORED BOUNDS, and the
/// world is STATIC — no ambient capability, so it schedules no frames.
#[test]
fn paperbark_dials_are_in_bounds_and_the_world_is_static() {
    let bg = paperbark_bg();
    assert!(
        (theme::DECKLE_MIN_PERIOD_PX..=theme::DECKLE_MAX_PERIOD_PX).contains(&bg.period_px()),
        "Paperbark's lane pitch {} is outside the authored Deckle bounds",
        bg.period_px()
    );
    assert!(
        bg.wander_px() > 0.0 && bg.wander_px() < bg.period_px() * 0.5,
        "the wander must tear the lanes without letting neighbours cross ({} vs pitch {})",
        bg.wander_px(),
        bg.period_px()
    );
    assert!(
        bg.density() > 0.0 && bg.density() <= 0.35,
        "the density dial is a whisper multiplier ({})",
        bg.density()
    );
    assert!(
        !theme::PAPERBARK.has_ambient_tick() && !theme::PAPERBARK.has_ambient_motion(),
        "Paperbark is a STATIC treatment — no clock, no drift, no raking light"
    );
    assert!(
        !bg.is_lava() && !bg.is_waves() && !bg.is_organic(),
        "Deckle is its own ground, not a re-labelled animated one"
    );
}
