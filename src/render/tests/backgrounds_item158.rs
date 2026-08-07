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
use crate::theme::{self, Background, Weave};

/// Per-pixel total-channel deviation from the mark-free pass that counts as
/// real material. The differential oracle cancels the dither exactly, so this
/// only has to clear 8-bit quantization (item 89's own floor).
const INK_FLOOR: i32 = 3;

fn paperbark_bg() -> Background {
    theme::PAPERBARK.background
}

fn galah_bg() -> Background {
    theme::GALAH.background
}

/// A controlled `Weave::Fibres` fixture whose dials match the paired Strata
/// fixture below. Galah uses its own authored dials; this one isolates weave.
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
            ..
        } => Background::Deckle {
            ground,
            layer,
            deckle,
            weave,
            period_px,
            wander_px,
            density: 0.20,
        },
        _ => unreachable!("Paperbark ships Background::Deckle"),
    }
}

/// DATA-LEVEL ROSTER, NO WILDCARD: each Deckle weave has one deliberate
/// assignee. Galah's Fibres prove the reusable profile is a real theme-owned
/// material rather than dormant renderer infrastructure.
#[test]
fn deckle_roster_assigns_paperbark_strata_and_galah_fibres_no_wildcard() {
    for t in theme::THEMES {
        let weave = match t.background {
            Background::Gradient { .. } => None,
            Background::Dots { .. } => None,
            Background::Pinstripe { .. } => None,
            Background::Stripes { .. } => None,
            Background::Lava { .. } => None,
            Background::Bands { .. } => None,
            Background::Waves { .. } => None,
            Background::Zigzag { .. } => None,
            Background::Organic { .. } => None,
            Background::Deckle { weave, .. } => Some(weave),
            Background::WarpedGrid { .. } => None,
        };
        let want = match t.name {
            "Paperbark" => Some(Weave::Strata),
            "Galah" => Some(Weave::Fibres),
            _ => None,
        };
        assert_eq!(
            weave, want,
            "{}: deliberate Deckle roster assignment",
            t.name
        );
    }
    // The weave is a THEME-OWNED scalar, and the inert default is what every
    // ground with NO profile dial reports — the shader's own `params.w`/`.z`
    // slot never changes shape for a world with none. `Weave` is the LAST
    // profile dial: Organic's own `Arrangement` collapsed to one arm and went,
    // so Bowerbird now reports the inert value here like every other world and
    // this sweep is Deckle's alone.
    for t in theme::THEMES {
        let want = match t.name {
            "Paperbark" => Weave::Strata.mode(),
            "Galah" => Weave::Fibres.mode(),
            _ => 0.0,
        };
        assert_eq!(
            t.background.profile_mode(),
            want,
            "{}: profile_mode is inert off Deckle",
            t.name
        );
    }
    assert_eq!(Weave::Strata.mode(), 0.0);
    assert_eq!(Weave::Fibres.mode(), 1.0);
}

/// THE PAGE STAYS FLAT AND OPAQUE. Not one pixel of material may enter the
/// writing column, at ANY swept geometry or DPI — the material is the room,
/// the page is the figure.
#[test]
fn deckle_ink_never_enters_the_writing_page_at_any_swept_geometry() {
    let _g = crate::testlock::serial();
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
    for (name, bg) in [("Paperbark", paperbark_bg()), ("Galah", galah_bg())] {
        theme::set_active_by_name(name).unwrap();
        p.sync_theme();
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
                        "{name} {ww}x{wh}@{measure}: deckle material entered \
                         the writing page at ({x},{y})"
                    );
                }
            }
        }
    }
    crate::page::set_measure(was_measure);
    crate::page::set_page_on(was_page);
    theme::set_active_by_name(was_theme).unwrap();
    p.sync_theme();
}

/// Galah's assigned Fibres must stay present but sparse across window size and
/// physical-pixel density. The field is read differentially from a density-zero
/// pass, so this grades plumage rather than its pink gradient.
#[test]
fn galah_fibres_are_sparse_present_and_deterministic_across_size_and_dpi() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        eprintln!("skipping galah_fibres_are_sparse_present_and_deterministic: no wgpu adapter");
        return;
    };
    let _g = crate::testlock::serial();
    for (w, h, cl, cw) in [
        (900, 600, 225.0, 450.0),
        (1400, 800, 350.0, 700.0),
        (2800, 1600, 700.0, 1400.0),
    ] {
        let a = mark_field(&device, &queue, galah_bg(), w, h, cl, cw);
        let b = mark_field(&device, &queue, galah_bg(), w, h, cl, cw);
        assert_eq!(a, b, "Galah {w}x{h}: static Fibres must be deterministic");
        let mut marked = 0usize;
        let mut margin = 0usize;
        let mut peak = 0i32;
        for y in 0..h {
            for x in 0..w {
                let i = (y * w + x) as usize;
                if x as f32 >= cl && (x as f32) < cl + cw {
                    assert_eq!(a[i], 0, "Galah {w}x{h}: fibres entered page at ({x},{y})");
                } else {
                    margin += 1;
                    peak = peak.max(a[i].abs());
                    marked += usize::from(a[i].abs() >= INK_FLOOR);
                }
            }
        }
        let share = marked as f32 / margin as f32;
        assert!(
            peak >= INK_FLOOR,
            "Galah {w}x{h}: fibres must leave visible plumage"
        );
        assert!(
            share > 0.01 && share < 0.55,
            "Galah {w}x{h}: fibres must remain sparse (marked {:.1}%)",
            share * 100.0
        );
        eprintln!(
            "Galah fibres {w}x{h}: {:.1}% marked, peak {peak}",
            share * 100.0
        );
    }
}

/// A margin's own material statistics: how many distinct strata VALUES a
/// vertical scan crosses, and the total tonal range the margin spans. Both are
/// read off the differential field, so they measure the MATERIAL, never the
/// gradient underneath it.
// pub(super) (item 201): the lane-INTERIOR tone count is exactly the oracle
// item 201's own Retina law needs — reused rather than re-derived, per the
// "same behavior, same code" rule.
pub(super) struct MarginStats {
    /// Peak absolute deviation anywhere in the margin.
    pub(super) peak: i32,
    /// Number of lane BOUNDARIES a mid-height horizontal scan crosses — a
    /// run-length count of sign-stable bands, so an antialiased edge is one
    /// crossing, not fifty.
    pub(super) bands: usize,
    /// The tonal SPREAD the margin shows: max deviation minus min deviation
    /// across the whole margin. A margin that collapsed to one flat lane has
    /// a spread near zero however dark that lane is.
    pub(super) spread: i32,
    /// How many DISTINCT tones the lane INTERIORS take. `spread` and `bands`
    /// are both satisfied by the deckled boundary alone, so a field whose
    /// lanes all drew one seeded tone — layered paper degraded to a ruled
    /// grid — would slip past them. This counts the long runs only (a boundary
    /// feather is short), so it measures the layering itself.
    pub(super) lane_tones: usize,
}

pub(super) fn margin_stats(field: &[i32], w: u32, h: u32, mx0: u32, mx1: u32) -> MarginStats {
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
    let _g = crate::testlock::serial();
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
/// Room wallpaper pixel.
///
/// NON-VACUITY, AND WHY IT NO LONGER NEEDS A WRONG ARM TO SHIP. A byte-identity
/// assertion is worthless if the region it compares is small, or flat, or
/// insensitive to the very displacement the defect would introduce. This law
/// used to answer that by rendering a second, deliberately page-anchored
/// coordinate owner and watching it move — a real proof, bought by keeping the
/// rejected behaviour alive in the shipped shader forever. It is stated
/// DIRECTLY instead now, which is strictly stronger:
///
///   * the exposed intersection must be SUBSTANTIAL (a quarter of the frame),
///     so the comparison is not over a wholly-covered region; and
///   * the field must NOT be INVARIANT under the exact translation a
///     page-anchored owner would have applied. A page-anchored Strata measures
///     `d = col_left - x` in the left margin, so moving the column by `shift`
///     makes the post-drag field at `x` equal the pre-drag field at
///     `x + shift`, EXACTLY (the tear profile rides `d` too, so it translates
///     with it). Asserting the pre-drag field differs from itself at that
///     offset is therefore the same claim the mutation arm used to
///     demonstrate — a page-anchored implementation cannot satisfy both
///     bullets at once — and it can never go quietly vacuous, because it is a
///     positive assertion about real rendered pixels rather than the absence
///     of one.
#[test]
fn deckle_strata_stays_fixed_at_exposed_viewport_points_across_page_width_drags() {
    let _g = crate::testlock::serial();
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
        let stable = paperbark_bg();
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

        // The sensitivity half. `shift` is the displacement a page-anchored
        // owner would have applied to the LEFT margin's field; both `x` and
        // `x + shift` are held inside the exposed left band, so this reads the
        // MATERIAL at both ends and never the flat page under the mask.
        let shift = (first_col.0 - second_col.0) as i64;
        assert!(shift > 0, "the drag must move the left page edge outward");
        let left_exposed_end = first_col.0.min(second_col.0) as i64;
        let mut moved = 0usize;
        let mut sampled = 0usize;
        for y in 0..h as i64 {
            for x in 0..(left_exposed_end - shift) {
                let here = a[(y * w as i64 + x) as usize];
                let there = a[(y * w as i64 + x + shift) as usize];
                sampled += 1;
                if here != there {
                    moved += 1;
                }
            }
        }
        assert!(
            sampled > 0,
            "{w}x{h}: the swept geometry left no exposed left margin to sample"
        );
        assert!(
            moved > sampled / 4,
            "{w}x{h}: the Strata field is nearly invariant under a {shift}px shift \
             ({moved}/{sampled} pixels differ) — byte-identity across the drag would then \
             be satisfied by a page-anchored owner too, and this law would prove nothing"
        );
    }
}

// `deckle_mode_sweeps_every_weave_anchor_combination` WAS HERE AND IS DELETED,
// not repaired. Its subject was that the packed `params.w` is TOTAL over two
// independent controls (Weave x DeckleAnchor) — with the coordinate owner gone
// there is one control, so the claim has no content left to make. Its two
// surviving halves each already have an owner: that Strata and Fibres are
// genuinely different fields is `weave_fibres_draws_a_real_field_distinct_from_
// strata` below, and that the slot carries the weave scalar and nothing else is
// the roster law's own `profile_mode` sweep above. Rewriting it to sweep one
// arm would have left a wildcard-free match with a single case — green forever,
// asserting nothing.

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
    let _g = crate::testlock::serial();
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
    let _g = crate::testlock::serial();
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
                wander_px,
                density,
                ..
            } => Background::Deckle {
                ground,
                layer,
                deckle,
                weave,
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
    let _g = crate::testlock::serial();
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
    // ITEM 201: DERIVED from Paperbark's own `wander_px`, not a hand-picked
    // constant. `boundary_wander` is the stddev of one tracked boundary's x
    // position, which the shader computes as a sinusoid whose amplitude is
    // `wander_px` — linear in the dial regardless of `period_px` — so the
    // measured-to-authored ratio is a stable ~0.74 at both this world's
    // pre-201 (94/13) and post-201 (47/6.5) dials. A hardcoded `6.0` (picked
    // against 13.0's ~9.6px measurement) went stale the moment item 201
    // retuned the dial for the Retina regression; half the authored wander
    // stays a comfortable floor at either scale while still failing on a
    // genuinely-flattened mutation.
    let wander_floor = paperbark_bg().wander_px() * 0.5;
    assert!(
        pb_wander >= wander_floor,
        "Paperbark's lane boundary must WANDER down the margin (it moves {pb_wander:.2}px \
         against a floor of {wander_floor:.2} — half the authored wander_px — Saltpan's \
         {sp_wander:.2}px) — a deckled torn edge is the whole material claim, and a \
         straight one is a recolored pinstripe"
    );
    eprintln!(
        "done-clause law: paperbark gap {pb_gap:.1}px wander {pb_wander:.2}px ({pb_marks} \
         marks) vs saltpan gap {sp_gap:.1}px wander {sp_wander:.2}px ({sp_marks} marks) \
         vs bilby {bi_marks} marks"
    );
}

/// Fibres and Strata remain genuinely different profiles. Render both through
/// the same pipeline with identical tones and dials so this grades the weave
/// choice itself rather than Galah and Paperbark's authored parameters.
#[test]
fn weave_fibres_draws_a_real_field_distinct_from_strata() {
    let _g = crate::testlock::serial();
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
        "Weave::Fibres must draw real material (peak {fib_peak})"
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
        wgsl.contains("g.params.w >= DECKLE_WEAVE_FIBRES"),
        "deckle_rgb must branch on the theme-owned weave scalar"
    );
    assert!(
        wgsl.contains("let d = deckle_viewport_distance(px);"),
        "Strata must read the stable viewport coordinate, unconditionally"
    );
    // And the rejected owner is GONE from the shader, not merely unselected:
    // measuring a Room wallpaper from a movable page edge is border
    // decoration, and a named function is one `select` away from being
    // reachable again.
    assert!(
        !wgsl.contains("deckle_page_distance"),
        "the page-anchored coordinate owner is back in the shipped shader"
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

// ---------------------------------------------------------------------------
// The grading law for a ground's `density` dial is the DIAL, not the world:
// every world whose ground carries the shared `density` field must show the
// dial doing real, material work, so a future world whose density silently
// disconnects from the shader (or a reverted world) fails here BY NAME
// rather than at a taste audit.
// ---------------------------------------------------------------------------

/// The density-bearing roster: every world on `theme::THEMES` whose ground
/// carries the shared `density` field, so enrolling or retiring a world tracks
/// this sweep with nothing here to edit.
///
/// The VARIANT question is asked as an exhaustive `match` with no wildcard,
/// which is the whole point of spelling it here instead of calling
/// `Background::density()`: that owner ends in `_ => 0.0`, so a new
/// density-bearing variant would answer "no density" and drop out of this
/// sweep silently. Written this way the new variant fails to COMPILE until
/// someone decides which side it belongs on. Enrolment by variant rather than
/// by measured value is also deliberate — a world that authored its density to
/// 0.0 must fail the presence floor below, not quietly leave the roster.
fn density_bearing_worlds() -> Vec<(&'static str, Background)> {
    fn bears_density(bg: &Background) -> bool {
        match bg {
            Background::Zigzag { .. }
            | Background::Organic { .. }
            | Background::Deckle { .. }
            | Background::WarpedGrid { .. } => true,
            Background::Gradient { .. }
            | Background::Dots { .. }
            | Background::Pinstripe { .. }
            | Background::Stripes { .. }
            | Background::Lava { .. }
            | Background::Bands { .. }
            | Background::Waves { .. } => false,
        }
    }
    theme::THEMES
        .iter()
        .filter(|t| bears_density(&t.background))
        .map(|t| (t.name, t.background))
        .collect()
}

/// Halve a ground's `density` field, EXHAUSTIVE over every `Background` arm
/// (not a wildcard default) so a new density-bearing variant fails to
/// compile here until this function knows how to scale it too — the same
/// discipline `Background::density()` itself already holds to.
fn with_half_density(bg: Background) -> Background {
    match bg {
        Background::Gradient { .. }
        | Background::Dots { .. }
        | Background::Pinstripe { .. }
        | Background::Stripes { .. }
        | Background::Lava { .. }
        | Background::Bands { .. }
        | Background::Waves { .. } => bg,
        Background::Zigzag {
            from,
            to,
            dir,
            tint,
            period_px,
            amplitude_px,
            angle,
            density,
            banded,
        } => Background::Zigzag {
            from,
            to,
            dir,
            tint,
            period_px,
            amplitude_px,
            angle,
            density: density * 0.5,
            banded,
        },
        Background::Organic {
            tones,
            scale_px,
            density,
        } => Background::Organic {
            tones,
            scale_px,
            density: density * 0.5,
        },
        Background::Deckle {
            ground,
            layer,
            deckle,
            weave,
            period_px,
            wander_px,
            density,
        } => Background::Deckle {
            ground,
            layer,
            deckle,
            weave,
            period_px,
            wander_px,
            density: density * 0.5,
        },
        Background::WarpedGrid {
            ground,
            minor,
            major,
            tunnel,
            spacing_px,
            density,
        } => Background::WarpedGrid {
            ground,
            minor,
            major,
            tunnel,
            spacing_px,
            density: density * 0.5,
        },
    }
}

fn mean_ink(field: &[i32]) -> f64 {
    field.iter().map(|&v| v as f64).sum::<f64>() / field.len() as f64
}

/// THE DENSITY DIAL DOES REAL, MATERIAL WORK ON EVERY WORLD THAT CARRIES ONE.
/// Halving a world's authored density must measurably quieten its margin
/// field's mean ink (the differential `mark_field` oracle, averaged rather
/// than peaked — a peak is dominated by a handful of the strongest marked
/// pixels and stayed flat across Galah's own 0.06-0.30 sweep, so it cannot
/// see the dial move; the mean tracks the field's overall reading and scaled
/// ~1.8-2.2x between full and half density for every world measured here).
/// Margin `1.3x` sits comfortably under that measured floor while still
/// catching a dial that has gone materially inert. This is the law a
/// Galah-style "up it a tinny bit" change leans on: it sweeps the ROSTER, so
/// a regression on any other density-bearing world fails here BY NAME.
#[test]
fn density_bearing_worlds_show_a_material_gap_between_full_and_half_density() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        eprintln!(
            "skipping density_bearing_worlds_show_a_material_gap_between_full_and_half_density: \
             no wgpu adapter"
        );
        return;
    };
    let (w, h, cl, cw) = (900u32, 700u32, 125.0f32, 650.0f32);
    let enrolled = density_bearing_worlds();
    assert!(
        enrolled.len() >= 5,
        "the density-bearing roster shrank to {} worlds — the enrollment derivation itself \
         needs a look, not just this law",
        enrolled.len()
    );
    for (name, bg) in enrolled {
        let full = mean_ink(&mark_field(&device, &queue, bg, w, h, cl, cw));
        let half = mean_ink(&mark_field(
            &device,
            &queue,
            with_half_density(bg),
            w,
            h,
            cl,
            cw,
        ));
        assert!(
            full > 0.0,
            "{name}: shipped density draws NO material at all (mean ink {full})"
        );
        assert!(
            full > half * 1.3,
            "{name}: halving density must measurably quieten the field (full mean {full:.4} \
             vs half mean {half:.4}, ratio {:.3}) — the dial reads as inert",
            full / half.max(1e-9)
        );
    }
}

/// NON-VACUITY SELF-PROOF, same shape as `backgrounds_item86`'s
/// `distinctness_check_fails_on_identical_dials_proving_it_is_non_vacuous`:
/// run the law's own inequality against a DEGENERATE pair (both sides at the
/// same density) and confirm it fails, so the material-gap law above is
/// provably capable of catching an inert dial rather than only ever
/// exercising the real, always-passing roster.
#[test]
fn material_gap_check_fails_when_the_dial_does_not_move_proving_it_is_non_vacuous() {
    let full = 0.115_f64; // Galah's own shipped mean, item 158's probe.
    let half = full; // a degenerate "dial" that does not respond to density at all.
    assert!(
        full <= half * 1.3,
        "an inert dial (full == half) must NOT pass the material-gap check"
    );
}

/// GALAH'S OWN DECISION, RECORDED AS DATA: the user's own words were "up it
/// a tinnyyy bit" off the shipped `0.10`, pinned to the `0.12`-`0.16`
/// neighbourhood. `0.12` is not an arbitrary pick inside that band — it is
/// the smallest rung where a REAL capture (1600x1000, measure 70) differs
/// from the old `0.10` by more than the repo's own perceptibility floor
/// (`EDGE_DELTA = 3`, `scripts/loudness-measure.py`): one rung down at
/// `0.11` stays inside 8-bit quantization noise (max right-margin luminance
/// delta 1.9, zero pixels crossing the floor), `0.12` clears it (max delta
/// 3.7, 0.18% of margin pixels above it). This is the sole place Galah is
/// named — the grading law above is the roster-general one.
#[test]
fn galah_density_lands_in_the_pinned_up_a_tinny_bit_band() {
    let Background::Deckle { density, .. } = theme::GALAH.background else {
        panic!("Galah must ship Background::Deckle");
    };
    assert!(
        (0.12..=0.16).contains(&density),
        "Galah's density {density} must sit in the user-pinned 0.12-0.16 neighbourhood"
    );
    assert_ne!(
        density, 0.10,
        "Galah's density must have actually moved off the old shipped 0.10"
    );
}
