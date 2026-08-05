//! THE DOCUMENT-SELECTION LEGIBILITY LAW — the band that covers your text.
//!
//! Every other ink-adjacent token in the theme model is held to a floor: the
//! syntax roles to 4.5:1 against the ground (`syntax_roles`), picker-row ink to
//! 3.0:1 with a fallback that flips the ink when the band crowds it
//! (`theme::derive::SELECTED_ROW_INK_CONTRAST_FLOOR`), the spell underline to
//! its own check. The `selection` token had none — it is handed to the
//! document-selection and search-match pipelines raw, with no floor and no
//! fallback, so a world could author a wash that eats its own prose and nothing
//! would say so.
//!
//! ⚠️ **THE PICKER-ROW FLOOR IS A DIFFERENT SURFACE AND DOES NOT COVER THIS
//! ONE.** `selected_row_ink` protects a band fed by `surface_step_band`, which
//! is derived from `base_200`/`base_300`. The DOCUMENT band is `theme::selection`
//! composited over the page by the GPU. Two bands, two owners; only one of them
//! had a law.
//!
//! ⚠️ **THE COMPOSITE IS NOT MODELLED IN THE HOST, AND MUST NOT BE.** Two
//! separate things defeat a host-side `selection`-over-`base_100` calculation,
//! and each one alone is enough:
//!
//! * **The substrate the prose sits on is not `base_100`'s bytes.** Tawny's
//!   `base_100` is `#16181D`; the substrate under its prose renders `#53565F`.
//!   The relation is exact and reproducible — those bytes are `base_100` treated
//!   as LINEAR and re-encoded by the sRGB target (`enc(0x16/255) * 255 == 83`,
//!   and likewise on every channel of every world checked). Whether that
//!   round-trip is intended is a question for the world authors and not for this
//!   law; what matters here is that it is what ships, so it is what the reader
//!   sees and what a legibility floor has to be measured against.
//! * **The wash is blended in LINEAR space, not straight alpha.** Mulga's old
//!   near-white wash over its own substrate composites to `#A6A275`; the
//!   straight-alpha arithmetic a host would reach for predicts `#8A...`, a whole
//!   value step darker, in the SAFE direction. A host model would have quietly
//!   under-reported the very defect this law exists to catch.
//!
//! So every number here is arithmetic over bytes that came back off the GPU.
//!
//! **THE ORACLE.** Render one selected line of ordinary prose, take the strict
//! interior of the row's own selection rect, and split its pixel population two
//! ways by area rather than by any recomputation of the rule under test:
//!
//! * the BAND is the modal exact colour — the flat field is most of the rect;
//! * the INK is the extreme-luminance colour, on whichever side of the band the
//!   glyphs fall, that still holds REAL AREA (`INK_AREA_FLOOR` pixels of one
//!   exact value). A single antialiased fleck cannot be the ink.
//!
//! Both are then checked against what the ONE owner `Theme::highlight_treatment`
//! says they should be, so a law that sampled the wrong pixels fails loudly
//! instead of quietly reporting a contrast between two arbitrary colours.
//!
//! NO-WILDCARD on `HighlightTreatment`: a new treatment variant fails to compile
//! here until someone decides what its band and its ink are.
//!
//! **WHAT THIS LAW DOES NOT COVER, stated so nobody reads it as wider than it
//! is.** The ink here is prose ink — `base_content`, the colour the overwhelming
//! majority of selected characters are. Dimmer document inks sit under the same
//! band and are not measured: the `muted` rung a code comment renders in is the
//! quietest of them, and it starts life much closer to the band than
//! `base_content` does. Extending this oracle to the syntax roles under
//! selection is a second law, not a wider sweep of this one — the fixture, the
//! `highlight_treatment` expectation and the floor would all differ.

use super::super::*;
use super::dither::{offscreen, read_pixels};
use super::{headless_dqp, view};

/// THE ROSTER, measured through the oracle below at every swept cell — the
/// selected prose's contrast against its own band, worst first:
///
/// ```text
///   Mangrove  3.25   Mulga     4.19   Potoroo   4.26   Tawny     4.98
///   Cassowary 5.39   Mopoke    5.51   Firetail  5.66   Bowerbird 5.76
///   Bombora   6.82   Currawong 9.11   Brolga    9.97   Paperbark 11.01
///   Kite     11.28   Bilby    11.62   Gumtree  11.91   Galah    12.16
///   Quokka   12.37   Saltpan  12.48   Magpie   13.41   Wagtail  21.00
/// ```
///
/// **THE TWO WORLDS NEAREST THE FLOOR ARE MANGROVE (3.25) AND POTOROO (4.26),
/// AND BOTH ARE DELIBERATELY ACCEPTED RATHER THAN RE-AUTHORED.** They clear the
/// floor as authored, and a world's selection colour is a taste decision that
/// belongs to its author; re-tinting a passing world to buy headroom would be
/// this law overreaching into a judgement it is not the oracle for. What the
/// figures buy instead is a tripwire: Mangrove holds only 8% margin, so any
/// future move to its page, its ink or its wash lands it under the floor and
/// fails HERE, by name, with the number — which is the outcome that matters and
/// the reason the table is written down rather than left implicit in a passing
/// test.
///
/// The split is structural rather than sloppy. A translucent wash over a DARK
/// page rises toward mid-grey while the ink stays near-white, compressing the
/// two together; over a LIGHT page the band stays light and the ink stays
/// near-black. That is why every world under 7:1 is a dark one and every light
/// world clears 9.9:1 — and why the failure this law exists to catch can only
/// ever appear on the dark half of the roster.
///
/// THE FLOOR. 3.0:1, the same number the picker-row band already answers to
/// (`SELECTED_ROW_INK_CONTRAST_FLOOR`). It is the same question asked of the
/// same kind of surface — ink sitting on a selected band — and a second number
/// for the document band would need a reason the surfaces differ that nobody
/// has. 3.0:1 is also WCAG's large-text grade, which is the honest grade for a
/// band that is a transient state rather than a permanent page colour.
const SELECTION_INK_CONTRAST_FLOOR: f32 = 3.0;

/// How many pixels of one exact colour it takes to be the INK rather than an
/// antialiasing artifact. Glyph stems at the capture font size render dozens of
/// fully-covered pixels; a fringe pixel is unique or near it.
const INK_AREA_FLOOR: usize = 8;

/// HOW PRESENT the band must be against the page beside it (see `presence`).
///
/// ⚠️ THIS FLOOR EXISTS BECAUSE THE CONTRAST FLOOR ALONE IS SATISFIABLE BY NOT
/// DRAWING A BAND. Lowering a wash's alpha moves the band toward the page, which
/// on EVERY world moves it AWAY from the ink — so a world can buy any contrast
/// figure it likes by fading its selection into invisibility, and the legibility
/// law would applaud. Measured: dropping one world's selection alpha to `0x04`
/// left a band four bytes from the page in a single channel and RAISED its
/// reported contrast. A legibility law that can be satisfied by deleting the
/// feature is not a law, so presence is asserted alongside it.
///
/// Set below the tightest value the shipped roster actually holds (Bombora,
/// 0.049) and far above what a faded-out band reaches (~0.013).
const BAND_PRESENCE_FLOOR: f32 = 0.03;

/// The share of the selection rect the flat band must hold for the modal-colour
/// reading to mean "the band". Measured worst across the roster is far above
/// this; a rect that does not clear it is not a band with text on it and every
/// number taken over it would be about something else.
const BAND_SHARE_FLOOR: f32 = 0.25;

/// `(window_w, window_h, page measure, page on)` — the configurations the law
/// sweeps.
///
/// ⚠️ **BOTH OF THESE AXES WERE MEASURED FLAT, AND SAYING SO IS THE POINT.** A
/// check runs in one configuration and that configuration is itself a
/// hypothesis, so this file tried the two that could plausibly move a composite
/// and reports what they did. Three window geometries returned BYTE-IDENTICAL
/// band, ink and substrate colours; so did both states of the PAGE toggle, which
/// was the more promising of the two (the layout genuinely moves — the glyph-ink
/// pixel counts change — but the prose keeps landing on the same substrate
/// either way, so the composite does not budge).
///
/// The cells are kept rather than collapsed for two reasons. Their invariance is
/// itself asserted below, so it stays a MEASURED fact that decays loudly instead
/// of an assumption a later reader inherits; and one of the geometries is the
/// gallery shape `scripts/capture-worlds.sh` shoots every world at, which is
/// where a human actually judges a world.
///
/// **The axis that does move these numbers is the WORLD**, which is swept
/// exhaustively and with no wildcard. What is NOT swept, and would be the
/// natural next cell: DPI. Every cell here runs at the harness's own scale, and
/// this tree has already shipped a defect that only the other scale could see.
const SWEEP: [(u32, u32, usize, bool); 4] = [
    (1200, 800, 70, true),  // the canonical capture canvas, page on.
    (1600, 1000, 66, true), // THE GALLERY GEOMETRY every world is judged at.
    (1200, 800, 70, false), // page off.
    (900, 600, 40, false),  // page off at a narrow measure.
];

/// A line of ordinary prose, deliberately dense in vertical stems so the glyph
/// cores land plenty of fully-covered pixels for the ink reading — and short
/// enough to fit the NARROWEST swept measure without wrapping, so the selected
/// line stays exactly one visual row on every shape.
const PROSE: &str = "Bill will fulfil all mill bills.";

/// WCAG relative luminance of an sRGB byte triple.
fn rel_lum(px: [u8; 4]) -> f32 {
    fn lin(u: u8) -> f32 {
        let s = u as f32 / 255.0;
        if s <= 0.04045 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    }
    0.2126 * lin(px[0]) + 0.7152 * lin(px[1]) + 0.0722 * lin(px[2])
}

/// WCAG contrast RATIO between two rendered pixels.
fn contrast(a: [u8; 4], b: [u8; 4]) -> f32 {
    let (la, lb) = (rel_lum(a), rel_lum(b));
    let (hi, lo) = if la >= lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

fn theme_px(c: theme::Srgb) -> [u8; 4] {
    [c.r, c.g, c.b, 0xFF]
}

/// HOW PRESENT a band is against the page beside it. Neither luminance nor hue
/// alone will do: a band can announce itself purely by VALUE (every light
/// world's does) or almost purely by CHROMA at nearly equal luminance (Tawny's
/// blue wash over its grey substrate does), and a floor phrased in one of them
/// scores the other at zero. So presence is the larger of the two — a normalized
/// luminance step and a normalized channel-spread step.
fn presence(band: [u8; 4], page: [u8; 4]) -> f32 {
    let dl = (rel_lum(band) - rel_lum(page)).abs();
    // The chroma arm: how far the band's channel BALANCE moved, which a pure
    // hue shift at constant luminance registers on and `dl` does not.
    let spread = |px: [u8; 4]| {
        let m = (px[0] as f32 + px[1] as f32 + px[2] as f32) / 3.0;
        [px[0] as f32 - m, px[1] as f32 - m, px[2] as f32 - m]
    };
    let (a, b) = (spread(band), spread(page));
    let dc = ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt() / 255.0;
    dl.max(dc)
}

/// Squared RGB distance between two rendered pixels.
fn dist2(a: [u8; 4], b: [u8; 4]) -> i32 {
    let d = |i: usize| (a[i] as i32 - b[i] as i32).pow(2);
    d(0) + d(1) + d(2)
}

/// What one world's selected row actually rendered.
struct BandProbe {
    /// The flat selection band, read as the modal colour of the rect.
    band: [u8; 4],
    /// That mode's share of the rect.
    band_share: f32,
    /// The glyph ink: the extreme-luminance colour with real area.
    ink: [u8; 4],
    /// How many pixels carry exactly that ink colour.
    ink_area: usize,
    /// The page immediately outside the band, on the same row — the LIFTED
    /// plane, sampled rather than modelled.
    page: [u8; 4],
}

/// Render one world with one line selected and read its band, its ink and the
/// page around it. `None` on a machine with no wgpu adapter.
///
/// ⚠️ Pins every global that can put ink inside the page column. A whole-frame
/// law is at the mercy of every toggle that draws, and the shared serial guard
/// restores world/page/spellcheck and the render overrides but not the panel
/// toggles — a sweep elsewhere in the suite has already left the debug panel on
/// and made a neighbouring law measure its readout as if it were ground.
fn probe(world: &'static str, w: u32, h: u32, measure: usize, page_on: bool) -> Option<BandProbe> {
    let cell = format!("{world} {w}x{h} m{measure} page={page_on}");
    let (device, queue, mut p) = headless_dqp(w as f32, h as f32)?;
    let was_page_on = crate::page::page_on();
    let was_measure = crate::page::measure();
    let was_debug = crate::debug::debug_on();
    let was_outline = crate::outline::outline_on();
    let was_nits = crate::nits::nits_on();
    let was_typewriter = crate::typewriter::typewriter_on();
    let was_spell = crate::spell::spellcheck_on();
    crate::page::set_page_on(page_on);
    crate::page::set_measure(measure);
    crate::debug::set_debug_on(false);
    crate::outline::set_outline_on(false);
    crate::nits::set_nits_on(false);
    crate::typewriter::set_typewriter_on(false);
    // A red squiggle under selected prose is ink that is not `base_content`,
    // and on a world whose band crowds it, the squiggle — not the glyph — could
    // become the extreme-luminance reading.
    crate::spell::set_spellcheck_on(false);

    theme::set_active_by_name(world).unwrap_or_else(|| panic!("{world} must be a real world"));
    p.sync_theme();

    // Line 1 is selected; line 0 and line 2 are unselected controls, and the
    // caret sits on line 2 so no caret quad lands inside the sampled row.
    let text = format!("{PROSE}\n{PROSE}\n{PROSE}\n");
    let mut v = view(&text, 2, 0);
    // Strictly inside line 1: an endpoint AT the line's own length normalizes
    // onto the head of line 2 and yields a second row rect.
    v.selection = Some(((1, 0), (1, PROSE.chars().count() - 1)));
    p.set_view(&v);

    let rects = p.selection_rects();
    let pixels = {
        p.prepare(&device, &queue, w, h).unwrap();
        let (texture, tview) = offscreen(&device, w, h);
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("awl selection contrast encoder"),
        });
        p.render(&mut encoder, &tview).unwrap();
        queue.submit(Some(encoder.finish()));
        read_pixels(&device, &queue, &texture, w, h)
    };

    crate::page::set_page_on(was_page_on);
    crate::page::set_measure(was_measure);
    crate::debug::set_debug_on(was_debug);
    crate::outline::set_outline_on(was_outline);
    crate::nits::set_nits_on(was_nits);
    crate::typewriter::set_typewriter_on(was_typewriter);
    crate::spell::set_spellcheck_on(was_spell);

    assert_eq!(
        rects.len(),
        1,
        "{cell}: one selected line must yield exactly one row rect (got {rects:?}) — the \
         fixture wrapped, and a wrapped tail row is a different population from the one \
         this law reads"
    );
    let r = rects[0];
    // The STRICT interior — shrink inward, never the `floor`/`ceil`-expanded
    // bounding box, so every sampled pixel is provably under the quad rather
    // than a hair outside its exact float boundary.
    let left = r[0].ceil().max(0.0) as i32;
    let right = (r[0] + r[2]).floor().min(w as f32) as i32;
    let top = r[1].ceil().max(0.0) as i32;
    let bottom = (r[1] + r[3]).floor().min(h as f32) as i32;
    assert!(
        right - left > 100 && bottom - top > 4,
        "{cell}: the selected row rect is too small to hold a population claim ({}x{} px)",
        right - left,
        bottom - top
    );

    let mut hist: std::collections::HashMap<[u8; 4], usize> = std::collections::HashMap::new();
    for y in top..bottom {
        for x in left..right {
            *hist.entry(pixels[(y * w as i32 + x) as usize]).or_default() += 1;
        }
    }
    let total = ((right - left) * (bottom - top)) as f32;
    let (&band, &band_n) = hist
        .iter()
        .max_by_key(|&(_, n)| *n)
        .expect("the rect holds pixels");
    let band_share = band_n as f32 / total;

    // The ink is on whichever side of the band the glyphs fall — which side is
    // not knowable in advance (a dark world's ink is brighter than its band, a
    // light world's is darker), so BOTH extremes with real area are found and
    // the one further from the band in luminance wins.
    let with_area = |pick: fn(f32, f32) -> bool| -> Option<[u8; 4]> {
        hist.iter().filter(|&(_, &n)| n >= INK_AREA_FLOOR).fold(
            None,
            |acc: Option<[u8; 4]>, (&px, _)| match acc {
                Some(best) if !pick(rel_lum(px), rel_lum(best)) => Some(best),
                _ => Some(px),
            },
        )
    };
    let darkest = with_area(|a, b| a < b).expect("some colour holds real area");
    let brightest = with_area(|a, b| a > b).expect("some colour holds real area");
    let ink = if contrast(darkest, band) >= contrast(brightest, band) {
        darkest
    } else {
        brightest
    };
    let ink_area = hist[&ink];

    // The page on the same row, just outside the band's right edge — the lifted
    // plane, sampled. The rect ends at the end of the prose, and the column
    // runs on past it.
    let page_x = ((right + 6).min(w as i32 - 1)).max(0);
    let page_y = (top + bottom) / 2;
    let page = pixels[(page_y * w as i32 + page_x) as usize];

    Some(BandProbe {
        band,
        band_share,
        ink,
        ink_area,
        page,
    })
}

/// What the ONE owner says this world's selected band and its ink ARE — the
/// no-wildcard match over `HighlightTreatment`, so both arms are covered without
/// a branch on any world's name.
///
/// `ValueBand` is a translucent wash: the band's rendered colour is the
/// composite, which only the GPU knows, so only the INK is predictable here (the
/// prose keeps its own `base_content`). `InverseFill` flips both: the band
/// becomes `base_content` and the ink `base_300`, and both are opaque, so both
/// are predictable.
fn expected(th: &theme::Theme) -> (Option<[u8; 4]>, [u8; 4]) {
    match th.highlight_treatment(th.selection_document) {
        theme::HighlightTreatment::ValueBand(_) => (None, theme_px(th.base_content)),
        theme::HighlightTreatment::InverseFill { band, ink } => {
            (Some(theme_px(band)), theme_px(ink))
        }
    }
}

/// THE LAW: on every world in `THEMES`, at every swept geometry, the ink inside
/// a document selection clears `SELECTION_INK_CONTRAST_FLOOR` against the band
/// that covers it — measured off the rendered pixels, never modelled.
///
/// The oracle is proven before it is trusted. Each probe first asserts that the
/// ink it found IS the ink the `highlight_treatment` owner names, and (on an
/// inverse-video world, where the band is opaque and therefore predictable) that
/// the band is the band. A law that sampled the wrong pixels would fail there
/// rather than reporting a contrast between two colours nobody chose.
#[test]
fn every_world_keeps_selected_prose_legible_inside_its_own_band() {
    let _g = crate::testlock::serial();
    if !crate::test_gpu::adapter_present() {
        eprintln!(
            "skipping every_world_keeps_selected_prose_legible_inside_its_own_band: no wgpu adapter"
        );
        return;
    }
    let _world = theme::WorldPin::snapshot();
    // world -> the ratio its FIRST swept cell produced, for the invariance
    // clause at the bottom of the loop.
    let mut first_cell: std::collections::HashMap<&'static str, f32> =
        std::collections::HashMap::new();

    for (w, h, measure, page_on) in SWEEP {
        for th in theme::THEMES.iter() {
            let Some(pr) = probe(th.name, w, h, measure, page_on) else {
                return;
            };
            let (want_band, want_ink) = expected(th);
            let ratio = contrast(pr.band, pr.ink);
            let cell = format!("{} {w}x{h} m{measure} page={page_on}", th.name);
            eprintln!(
                "{:>10} m{measure} page={page_on}: band {:?} ({:.0}% of rect) ink {:?} ({} px) \
                 page {:?} -> ink/band {ratio:.2}:1  band/page step {:.4}",
                th.name,
                pr.band,
                pr.band_share * 100.0,
                pr.ink,
                pr.ink_area,
                pr.page,
                presence(pr.band, pr.page)
            );

            // THE ORACLE'S OWN PRECONDITIONS — the sampling is right before the
            // number it produced is believed.
            assert!(
                pr.band_share >= BAND_SHARE_FLOOR,
                "{cell}: the modal colour holds only {:.1}% of the selected row — that is \
                 not a flat band with text on it, so the reading is about something else",
                pr.band_share * 100.0
            );
            assert!(
                dist2(pr.ink, want_ink) <= 12,
                "{cell}: the extreme-area colour inside the band is {:?}, but \
                 `highlight_treatment` says the ink is {want_ink:?} — this law is sampling \
                 the wrong pixels",
                pr.ink
            );
            if let Some(want) = want_band {
                assert!(
                    dist2(pr.band, want) <= 12,
                    "{cell}: the band rendered {:?}, but `highlight_treatment` says an \
                     inverse-video band is {want:?}",
                    pr.band
                );
            }
            // THE BAND IS ACTUALLY THERE — asserted on every world, including the
            // inverse-video one, because the contrast floor below is satisfiable
            // by fading the selection out entirely (see `BAND_PRESENCE_FLOOR`).
            let step = presence(pr.band, pr.page);
            assert!(
                step >= BAND_PRESENCE_FLOOR,
                "{cell}: the selected band {:?} stands only {step:.4} off the page beside \
                 it {:?}, under the {BAND_PRESENCE_FLOOR} presence floor — a selection \
                 nobody can see passes a legibility floor trivially, so this is a failure \
                 even though the contrast reads high",
                pr.band,
                pr.page
            );

            assert!(
                ratio >= SELECTION_INK_CONTRAST_FLOOR,
                "{cell}: selected prose reads at only {ratio:.2}:1 against its own band \
                 (band {:?}, ink {:?}) — under the {SELECTION_INK_CONTRAST_FLOOR}:1 floor \
                 the selection is eating the text it covers",
                pr.band,
                pr.ink
            );

            // ONE WORLD, ONE NUMBER. Every configuration swept above measured
            // the same composite, which is what lets the roster be quoted as a
            // single figure per world. Asserted rather than assumed: if some
            // future change makes the substrate configuration-dependent — a
            // page plane that really does lift, a per-DPI wash — this says so
            // in the cell where it happened, instead of letting a table that
            // has quietly become an average keep reading like a measurement.
            let first = *first_cell.entry(th.name).or_insert(ratio);
            assert!(
                (ratio - first).abs() < 0.05,
                "{cell}: reads {ratio:.2}:1 here but {first:.2}:1 in this world's first \
                 swept cell — the selected band's composite has become configuration- \
                 dependent, so one number per world is no longer the whole story and this \
                 law's roster table needs a column, not a row"
            );
        }
    }
}

/// MEASUREMENT REPORT, not a law — the twenty-world table in one place, sorted
/// worst-first, so the roster's margin above the floor is a thing a human can
/// read rather than infer from a passing test. `#[ignore]`d by default (same
/// spirit as the gallery generators):
/// `cargo test --bin awl selection_contrast_report -- --ignored --nocapture`
#[test]
#[ignore]
fn selection_contrast_report() {
    let _g = crate::testlock::serial();
    if !crate::test_gpu::adapter_present() {
        eprintln!("skipping selection_contrast_report: no wgpu adapter");
        return;
    }
    let _world = theme::WorldPin::snapshot();
    let (w, h, measure, page_on) = SWEEP[0];
    let mut rows: Vec<(f32, String)> = Vec::new();
    for th in theme::THEMES.iter() {
        let Some(pr) = probe(th.name, w, h, measure, page_on) else {
            return;
        };
        let ratio = contrast(pr.band, pr.ink);
        rows.push((
            ratio,
            format!(
                "{:>10} {ratio:6.2}:1  band {:?}  ink {:?}  page {:?}  selection token {:?}",
                th.name,
                pr.band,
                pr.ink,
                pr.page,
                th.selection_document.rgba_bytes()
            ),
        ));
    }
    rows.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    eprintln!("\n=== document-selection band-vs-ink, {w}x{h} m{measure} page{page_on} ===");
    for (_, line) in &rows {
        eprintln!("{line}");
    }
}
