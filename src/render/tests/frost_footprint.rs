//! THE CRISP PICKER'S OWN FOOTPRINT, MEASURED IN PIXELS.
//!
//! On a world whose list composition backs its rows with nothing — no card panel, no
//! row plates — the theme and caret pickers used to draw their rows straight over live
//! prose, and the two interleaved glyph-for-glyph. Frost was a property of the PLATE,
//! and those compositions draw none. The fix frosts the card's OWN BOX and nothing
//! outside it (`blur::Frost::Footprint`), so the page keeps the live colours the
//! picker exists to preview.
//!
//! # WHAT "NO DOCUMENT GLYPH SURVIVES AS TEXT" IS, NUMERICALLY
//!
//! The naive measurement — count sharp edges inside the card — measures the CARD'S OWN
//! rows, which are sharp by design and are not the subject. So the subject is isolated
//! first: render the SAME picker over a DENSE document and over an EMPTY one. The card
//! is pixel-for-pixel the same in both (its geometry and its rows are the world roster,
//! not the document), so the per-pixel luma DIFFERENCE is the DOCUMENT'S OWN
//! CONTRIBUTION and nothing else — the residue.
//!
//! Text is then distinguished from a defocus of text by the residue's LOCAL LUMA
//! GRADIENT. A glyph stem is a step: it moves tens of luma units across one pixel
//! boundary at any size or DPI. A Gaussian whose reach is wider than a stem cannot
//! leave such a step anywhere — its output is a weighted average of a whole
//! neighbourhood, so successive pixels differ by a fraction of the neighbourhood's own
//! range. Measured on this tree: the same document's residue carries a peak gradient of
//! ~5/255 inside the footprint and ~190/255 outside it, a 38× separation, so the
//! [`STRONG_GRADIENT`] threshold sits in a very wide valley rather than on a tuned edge.
//!
//! Three things keep the law from measuring something adjacent to its subject:
//!
//! * **The card's own ink is masked out**, and the mask is DERIVED rather than drawn:
//!   the empty-document frame's backdrop is a blur of a blank page, which is smooth by
//!   construction, so any strong local gradient in THAT frame is the card's own ink.
//!   (Card glyphs are alpha-blended over the backdrop, so their anti-aliased edges
//!   modulate the residue at exactly the scale a glyph edge lives at — unmasked, they
//!   are indistinguishable from the thing being tested.)
//! * **A PRESENCE floor** runs beside the smoothness floor. A smoothness floor alone
//!   gets HAPPIER as the document vanishes: an opaque card, or a frost dimmed to the
//!   flat base, would satisfy it perfectly while destroying the very thing the crisp
//!   pickers exist for. The residue must still reach [`PRESENCE_FLOOR`] — the document
//!   is defocused, not deleted.
//! * **The same statistic is asked on BOTH sides of the condition.** A collar just
//!   outside the footprint must be SHARP by the same measurement. That is what makes
//!   this a confinement law: a fullscreen frost fails the collar, an absent frost fails
//!   the interior, and neither failure can hide in the other's green.
//!
//! Swept over every enrolled world (derived from the roster, never named) × 1× and 2×.

use super::super::*;
use super::dither::{offscreen, read_pixels};
use super::frost_card_ink::{CardInk, luma, row_ink_vetoes, step};
use super::{headless_dqp, view_md};

/// A local luma step (of 255) that only an EDGE produces. Set in the empty middle of
/// the measured valley: this tree's frosted residue peaks near 5 and its sharp residue
/// near 190, so the threshold is not load-bearing to within a factor of four either way.
const STRONG_GRADIENT: f32 = 24.0;

/// How much of the document must still REACH through the frost, in luma. The companion
/// to the smoothness floor: without it, deleting the subject passes.
const PRESENCE_FLOOR: f32 = 12.0;

/// Prose dense enough to put real glyph structure under the card at every geometry the
/// sweep uses, markdown so the heading ladder and the list rows vary the row heights.
const DENSE: &str = concat!(
    "# The frosted footprint\n\n",
    "Prose is the product, and the prose is what a summoned picker draws over.\n",
    "This paragraph exists so that dense glyph structure sits under the card's\n",
    "box at every geometry the sweep visits, at both device scales.\n\n",
    "Sharp text is a step function: a stem edge moves tens of luma units across\n",
    "a single pixel boundary. A defocus whose reach exceeds a stem cannot leave\n",
    "one anywhere.\n\n",
    "- a list row with several short words in it\n",
    "- another list row, similar in shape\n",
    "- a third row, so the block has height\n\n",
    "Every line of this document sits under the picker's own card, which is the\n",
    "whole point of the measurement this file performs.\n",
);

/// One rendered frame's pixels.
fn render_frame(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    p: &mut TextPipeline,
    w: u32,
    h: u32,
) -> Vec<[u8; 4]> {
    p.prepare(device, queue, w, h).unwrap();
    let (texture, tview) = offscreen(device, w, h);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("awl frost footprint encoder"),
    });
    p.render(&mut encoder, &tview).unwrap();
    queue.submit(Some(encoder.finish()));
    read_pixels(device, queue, &texture, w, h)
}

/// The THEME picker, open over `text` — the crisp picker with the widest card, and the
/// one the defect was reported on. Its rows are the world roster, so they do not vary
/// with the document: that independence is what makes the residue below the document's
/// own contribution.
fn theme_picker(text: &str) -> ViewState {
    let mut v = view_md(text, 0, 0);
    v.overlay_active = true;
    v.overlay_crisp = true;
    v.overlay_items = crate::theme::THEMES.iter().map(|t| t.name.into()).collect();
    v.overlay_sections = vec![String::new(); v.overlay_items.len()];
    v.overlay_selected = 11;
    v.overlay_title = "themes".to_string();
    v.overlay_hint = "type to filter   ↵ keep   esc revert".to_string();
    v
}

/// What one region's measurement says.
#[derive(Debug, Clone, Copy)]
struct Sharpness {
    /// Pixels measured (card ink excluded).
    measured: usize,
    /// How many carried a step at or over [`STRONG_GRADIENT`] — an EDGE.
    edges: usize,
    /// The largest step seen.
    peak_step: f32,
    /// The largest residue AMPLITUDE seen: how far the document reaches through.
    peak_amplitude: f32,
}

/// Measure the document's residue over the pixels of `field` that `keep` admits and the
/// card-ink veto does not exclude.
fn measure(
    residue: &[f32],
    ink: &CardInk,
    w: i64,
    h: i64,
    keep: impl Fn(i64, i64) -> bool,
) -> Sharpness {
    let mut out = Sharpness {
        measured: 0,
        edges: 0,
        peak_step: 0.0,
        peak_amplitude: 0.0,
    };
    for y in 0..h - 1 {
        for x in 0..w - 1 {
            if ink.vetoes(x, y) || !keep(x, y) {
                continue;
            }
            out.measured += 1;
            let s = step(residue, w, h, x, y);
            out.peak_step = out.peak_step.max(s);
            out.peak_amplitude = out.peak_amplitude.max(residue[(y * w + x) as usize].abs());
            if s >= STRONG_GRADIENT {
                out.edges += 1;
            }
        }
    }
    out
}

/// Every world whose composition enrols in the footprint frost, taken from the ROSTER
/// rather than named — a world that changes its list style changes what this sweeps.
fn enrolled_worlds() -> Vec<&'static str> {
    crate::theme::THEMES
        .iter()
        .filter(|t| crate::render::blur::footprint_frost_applies(t.render_caps.list_style))
        .map(|t| t.name)
        .collect()
}

/// THE HEADLINE LAW. Inside the crisp picker's footprint the document survives as a
/// defocus and NOT as text; in a collar just outside it, the same document is sharp;
/// and it is still THERE inside, not painted over. Swept over every enrolled world at
/// 1× and 2×.
#[test]
fn the_footprint_frost_unmakes_the_document_as_text_and_confines_itself_to_the_card() {
    let _g = crate::testlock::serial();
    // The AMBIENT world, captured rather than assumed: this law switches worlds, and
    // the guard requires it to exit on the one it entered on. A `DEFAULT_THEME`
    // restore would be a guess about the process's state.
    let entry = crate::theme::active_index();
    let worlds = enrolled_worlds();
    assert!(
        !worlds.is_empty(),
        "no world enrols in the footprint frost — this law has no subject"
    );
    for world in &worlds {
        for (dpi, w, h) in [(1.0f32, 1200u32, 800u32), (2.0, 2400, 1600)] {
            let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
                eprintln!("skipping the frost-footprint sweep: no wgpu adapter");
                return;
            };
            crate::theme::set_active_by_name(world).unwrap();
            p.set_dpi(dpi);

            // A: the picker over dense prose. B: the SAME picker over an empty
            // document — identical card, so A - B is the document alone.
            p.set_view(&theme_picker(DENSE));
            let a = render_frame(&device, &queue, &mut p, w, h);
            let rect = p
                .overlay_card_rect()
                .expect("the crisp picker is open, so it has a card box");
            let row_ink = p.overlay_row_ink_probe();
            p.set_view(&theme_picker(""));
            let b = render_frame(&device, &queue, &mut p, w, h);

            let (wi, hi) = (w as i64, h as i64);
            let residue: Vec<f32> = a
                .iter()
                .zip(b.iter())
                .map(|(pa, pb)| luma(*pa) - luma(*pb))
                .collect();
            let ink = CardInk::derive(&b, wi, hi, dpi);
            // A `Bars` plate/scrim authored close to the world's own ground can sit
            // under `CardInk`'s own `INK_GRADIENT`, leaving it blind to real, opaque
            // row ink — `row_ink_vetoes` is the geometric backstop for that blind spot.
            let row_veto = |x: i64, y: i64| row_ink_vetoes(&row_ink, dpi, x, y);

            // The footprint, and a collar outside it. The interior's inset stays small:
            // the frost's mask is FULL STRENGTH on and inside the card's faces (the
            // feather ramps entirely outward), so the feather edge does not move
            // this half of the law at all.
            //
            // ⚠️ THE COLLAR'S INNER BOUNDARY IS THE BOUNDARY'S OWN WIDTH, and the
            // boundary is no longer one pixel: it is the feather, plus the shear's own
            // reach on a leaning composition. Read off the frost's `footprint_bound` —
            // the same arithmetic the composite's scissor uses — rather than a second
            // authored inset, so a retuned feather moves this with it instead of
            // quietly turning a confinement claim into a reading of the skirt.
            //
            // ⚠️ AND THE INTERIOR IS THE PARALLELOGRAM, NOT THE CARD'S BOX. The frost's
            // shape leans, so the box's two OFF-RAKE corners are deliberately unfrosted
            // and the document there is deliberately sharp — that is the intended
            // shape, and a region pinned to the box reads those corners as this
            // law's own failure. Both the rect and the shear come from the frost the
            // frame SENT THE SHADER, so this follows the shape that was drawn rather
            // than a second derivation of it.
            let pad = 4.0 * dpi;
            let collar = 24.0 * dpi;
            let (frect, shear) = match p.frost_mode() {
                Some(crate::render::blur::Frost::Footprint(f)) => (f.rect, f.shear),
                other => panic!("{world}: expected the footprint arm, got {other:?}"),
            };
            let [rx, ry, rw, rh] = frect;
            let skirt = crate::render::blur::footprint_skirt_px(p.frost_mode(), dpi);
            let lean = |py: f32| shear * (py - (ry + rh * 0.5));
            let inside = |x: i64, y: i64| {
                let (fx, fy) = (x as f32, y as f32);
                fx >= rx + lean(fy) + pad
                    && fx < rx + rw + lean(fy) - pad
                    && fy >= ry + pad
                    && fy < ry + rh - pad
            };
            let collar_only = |x: i64, y: i64| {
                let (fx, fy) = (x as f32, y as f32);
                let outer = fx >= rx - skirt - collar
                    && fx < rx + rw + skirt + collar
                    && fy >= ry - skirt - collar
                    && fy < ry + rh + skirt + collar;
                let inner = fx >= rx - skirt
                    && fx < rx + rw + skirt
                    && fy >= ry - skirt
                    && fy < ry + rh + skirt;
                outer && !inner
            };

            let within = measure(&residue, &ink, wi, hi, |x, y| {
                inside(x, y) && !row_veto(x, y)
            });
            let beyond = measure(&residue, &ink, wi, hi, collar_only);
            let label = format!("{world} @ {dpi}x ({w}x{h}), card {rect:?}");
            eprintln!("MEASURED {label}: within={within:?} beyond={beyond:?}");

            assert!(
                within.measured > 1000 && beyond.measured > 1000,
                "{label}: too few pixels to measure (inside {}, collar {}) — the \
                 regions, not the product, are what failed",
                within.measured,
                beyond.measured
            );
            // 1) THE DEFECT: not one glyph edge of the document survives inside.
            assert_eq!(
                within.edges, 0,
                "{label}: {} of {} pixels inside the footprint carry a document EDGE \
                 (step >= {STRONG_GRADIENT}, peak {:.1}) — the document is still \
                 drawing as TEXT under the picker's rows. The same document in the \
                 collar just outside: {} edges, peak {:.1}",
                within.edges, within.measured, within.peak_step, beyond.edges, beyond.peak_step,
            );
            // 2) THE PRESENCE FLOOR: it is defocused, not deleted or flattened.
            assert!(
                within.peak_amplitude >= PRESENCE_FLOOR,
                "{label}: the document reaches only {:.1} luma through the frost \
                 (floor {PRESENCE_FLOOR}) — a smoothness floor is satisfied by an \
                 opaque card or a frost dimmed to the flat base, and this is the \
                 companion that refuses both",
                within.peak_amplitude,
            );
            // 3) CONFINEMENT: the same document, the same statistic, one collar out.
            assert!(
                beyond.edges > 100 && beyond.peak_step >= 4.0 * STRONG_GRADIENT,
                "{label}: the collar {collar} px outside the card carries only {} \
                 edges (peak step {:.1}) — the frost is not confined to the \
                 footprint, so the page has lost the live colours the crisp picker \
                 exists to preview",
                beyond.edges,
                beyond.peak_step,
            );
        }
    }
    crate::theme::set_active(entry);
}

/// THE MEAN COLOUR OF TWO FRAMES over the pixels of the card's box that the frost
/// FULLY reaches and the card-ink veto does not exclude — accumulated in LINEAR light
/// and converted to Lab once, at the end.
///
/// A blur preserves the mean of LINEAR radiance; it does not preserve the mean of any
/// nonlinear transform of it, so averaging L\*a\*b\* per pixel and comparing the averages
/// reports a chroma shift that is an artifact of Jensen's inequality rather than
/// anything the shader did.
///
/// ⚠️ THE REGION IS THE FROST'S OWN SHAPE, READ THROUGH THE SHIPPING POLICY'S MIRROR,
/// not the card's box. On a leaning composition the box's two off-rake corners are
/// deliberately unfrosted, and the live page showing through them has the live page's
/// exact colour — so admitting them pulls the "frosted" mean toward the "live" one it is
/// being compared against, and the bound below gets easier the more of the card's box
/// the frost fails to cover. Measured on this tree that dilution is 2.3–5.4% of the box
/// at the roster's current rake; it grows without bound with the shear. Excluding it
/// also keeps the veto inside the region where its own premise holds, which is a
/// property of the frost's reach rather than of the card.
struct MeanPair {
    /// The frosted patch's mean colour, and the live page's, both as L\*a\*b\*.
    frosted: (f64, f64, f64),
    live: (f64, f64, f64),
    /// Pixels averaged: the PRESENCE guard's own subject, since the veto's surplus is a
    /// property of the world's ground rather than of the frost.
    n: f64,
    /// Pixels of the card's box the frost does not fully reach, and so excluded.
    short: u64,
}

fn frosted_and_live_mean_lab(
    frames: (&[[u8; 4]], &[[u8; 4]]),
    ink: &CardInk,
    row_ink: &[[f32; 4]],
    card: [f32; 4],
    frost: crate::render::blur::Frost,
    (dpi, w): (f32, i64),
) -> MeanPair {
    let (open, closed) = frames;
    let [rx, ry, rw, rh] = card;
    let mut n = 0.0f64;
    let mut short = 0u64;
    let mut acc = [[0.0f64; 3]; 2];
    // Same blind spot as `CardInk` above: a `Bars` plate/scrim authored close to the
    // world's own ground can sit under `INK_GRADIENT` at its own edge, so its opaque
    // fill would otherwise drag this mean toward the plate's own colour rather than the
    // page's. `overlay_row_ink_probe` is the production owner of that ink.
    let row_veto = |x: i64, y: i64| row_ink_vetoes(row_ink, dpi, x, y);
    for y in (ry + 8.0) as i64..(ry + rh - 8.0) as i64 {
        for x in (rx + 8.0) as i64..(rx + rw - 8.0) as i64 {
            if ink.vetoes(x, y) || row_veto(x, y) {
                continue;
            }
            if crate::render::blur::footprint_mask_for(frost, dpi, x as f32, y as f32) < 1.0 {
                short += 1;
                continue;
            }
            let i = (y * w + x) as usize;
            for c in 0..3 {
                acc[0][c] += crate::theme::srgb_channel_to_linear(open[i][c]);
                acc[1][c] += crate::theme::srgb_channel_to_linear(closed[i][c]);
            }
            n += 1.0;
        }
    }
    let to_lab = |a: [f64; 3]| {
        let enc = |v: f64| {
            let v = (v / n).clamp(0.0, 1.0);
            let s = if v <= 0.003_130_8 {
                v * 12.92
            } else {
                1.055 * v.powf(1.0 / 2.4) - 0.055
            };
            (s * 255.0).round() as u8
        };
        super::pixeldiff::lab([enc(a[0]), enc(a[1]), enc(a[2]), 255])
    };
    MeanPair {
        frosted: to_lab(acc[0]),
        live: to_lab(acc[1]),
        n,
        short,
    }
}

/// THE FROST IS A DEFOCUS, NOT A WASH — measured as CHROMA.
///
/// The footprint arm dims by nothing, so the frosted patch must carry the page's OWN
/// colour, defocused. What a wash does — the neutral grey scrim this whole mechanism
/// replaced, or a heavy dim toward the flat base — is collapse the (a\*, b\*) chroma
/// while leaving L\* roughly where it was, which is exactly what a value-only oracle
/// (a contrast ratio, a |ΔY|) cannot see. So the tight bound goes on chroma.
///
/// Measured over the FULLY FROSTED, card-ink-free pixels of the card's box (see
/// [`frosted_and_live_mean_lab`] for why both qualifiers are load-bearing), at 1× and
/// 2×: the veto's dilation is a physical length, so a mask derived at the wrong scale
/// under-swallows a glyph's skirt on retina and lets the card's own chroma into a
/// measurement of the page's.
#[test]
fn the_footprint_frost_keeps_the_pages_own_hue() {
    let _g = crate::testlock::serial();
    let entry = crate::theme::active_index();
    for world in enrolled_worlds() {
        for (dpi, w, h) in [(1.0f32, 1200u32, 800u32), (2.0, 2400, 1600)] {
            let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
                eprintln!("skipping the_footprint_frost_keeps_the_pages_own_hue: no wgpu adapter");
                return;
            };
            crate::theme::set_active_by_name(world).unwrap();
            p.set_dpi(dpi);

            p.set_view(&theme_picker(DENSE));
            let open = render_frame(&device, &queue, &mut p, w, h);
            let rect = p.overlay_card_rect().expect("the picker is open");
            let frost = p.frost_mode().expect("an enrolled world reaches the frost");
            let row_ink = p.overlay_row_ink_probe();
            // The same picker over an empty document: the card-ink oracle.
            p.set_view(&theme_picker(""));
            let empty = render_frame(&device, &queue, &mut p, w, h);
            // The same document with NO picker: the live page, at its own hue.
            let mut plain = view_md(DENSE, 0, 0);
            plain.overlay_active = false;
            p.set_view(&plain);
            let closed = render_frame(&device, &queue, &mut p, w, h);

            let ink = CardInk::derive(&empty, w as i64, h as i64, dpi);
            let m = frosted_and_live_mean_lab(
                (&open, &closed),
                &ink,
                &row_ink,
                rect,
                frost,
                (dpi, w as i64),
            );
            let ((fl, fa, fb), (ll, la, lb), n, short) = (m.frosted, m.live, m.n, m.short);
            let label = format!("{world} @ {dpi}x ({w}x{h})");
            assert!(
                n > 1000.0,
                "{label}: only {n} fully-frosted card-ink-free pixels inside the card's \
                 box ({short} more fell short of full frost) — the region, not the \
                 product, is what failed"
            );
            eprintln!(
                "MEASURED {label} hue: frosted L*{fl:.2} a*{fa:.2} b*{fb:.2} \
                 live L*{ll:.2} a*{la:.2} b*{lb:.2} over {n} px, {short} px of the \
                 card's box short of full frost and excluded"
            );
            assert!(
                (fa - la).abs() <= 1.0 && (fb - lb).abs() <= 1.0,
                "{label}: the frosted footprint's mean chroma (a* {fa:.2}, b* {fb:.2}) \
                 departs from the live page's (a* {la:.2}, b* {lb:.2}) — the footprint arm \
                 dims by NOTHING, so it must be a DEFOCUS of the page's own colour, not a \
                 wash toward anything. (L*: frosted {fl:.2}, live {ll:.2}.)"
            );
            // …and it is genuinely a DIFFERENT image (a frost that drew nothing, or one
            // flattened to something the page already was, satisfies a hue test perfectly).
            let differing = open
                .iter()
                .zip(closed.iter())
                .filter(|(a, b)| a != b)
                .count();
            assert!(
                differing > (w * h / 20) as usize,
                "{label}: only {differing} pixels differ between picker-open and \
                 picker-closed — either nothing drew, or the footprint was flattened to \
                 something the page already was"
            );
        }
    }
    crate::theme::set_active(entry);
}

/// THE SCISSOR IS PASS STATE, NOT DRAW STATE.
///
/// The footprint composite, the card, the world's own wordmark — which anchors to a
/// CANVAS corner, not to the card — and the whole chrome tail all share ONE render
/// pass. A scissor left set after the composite clips every one of them to the card's
/// box. Nothing in this tree noticed when the reset was deleted: 1007 render tests
/// stayed green, because the card's own rows are inside the box already and the visible
/// loss is exactly the chrome that is not.
///
/// The subject here is a sticky NOTICE, which draws at the foot of the page column and
/// so is reliably outside a picker's card box on every world: with the reset it draws,
/// without it there is nothing there at all.
#[test]
fn the_footprint_scissor_does_not_clip_what_the_frame_draws_after_it() {
    let _g = crate::testlock::serial();
    let entry = crate::theme::active_index();
    for world in enrolled_worlds() {
        let (w, h) = (1200u32, 800u32);
        let Some((device, queue, mut p)) = headless_dqp(w as f32, h as f32) else {
            eprintln!("skipping the scissor-reset law: no wgpu adapter");
            return;
        };
        crate::theme::set_active_by_name(world).unwrap();

        let mut quiet = theme_picker(DENSE);
        quiet.notice = String::new();
        p.set_view(&quiet);
        let without = render_frame(&device, &queue, &mut p, w, h);
        let rect = p.overlay_card_rect().expect("the picker is open");

        let mut loud = theme_picker(DENSE);
        loud.notice = "the file was written".to_string();
        loud.notice_kind = crate::actions::NoticeKind::Sticky;
        p.set_view(&loud);
        let with = render_frame(&device, &queue, &mut p, w, h);
        let (drawn, _) = p
            .notice_report()
            .expect("a notice was set, so the toast has content");
        assert!(!drawn.is_empty(), "{world}: the notice shaped to nothing");

        let [rx, ry, rw, rh] = rect;
        let mut outside = 0usize;
        for y in 0..h as i64 {
            for x in 0..w as i64 {
                let (fx, fy) = (x as f32, y as f32);
                if fx >= rx && fx < rx + rw && fy >= ry && fy < ry + rh {
                    continue;
                }
                let i = (y * w as i64 + x) as usize;
                if with[i] != without[i] {
                    outside += 1;
                }
            }
        }
        assert!(
            outside > 200,
            "{world}: only {outside} pixels OUTSIDE the card box {rect:?} differ when a \
             toast is added — the frost's scissor is still set when the chrome tail \
             draws, so everything the frame paints after the card's backdrop is being \
             clipped to the card"
        );
    }
    crate::theme::set_active(entry);
}

/// A world whose card BACKS ITSELF takes no frost at all under a crisp picker: its own
/// panel (or its row plates) already covers what it sits on, so the frame stays on the
/// unchanged crisp path, byte-for-byte. Asked of every non-enrolled world in the
/// roster, so the excluded set is swept rather than sampled.
#[test]
fn a_self_backing_composition_takes_no_frost_under_a_crisp_picker() {
    let _g = crate::testlock::serial();
    let entry = crate::theme::active_index();
    let Some(mut p) = super::headless_pipeline() else {
        eprintln!("skipping the self-backing exclusion law: no wgpu adapter");
        return;
    };
    let mut excluded = 0;
    for t in crate::theme::THEMES.iter() {
        if crate::render::blur::footprint_frost_applies(t.render_caps.list_style) {
            continue;
        }
        excluded += 1;
        crate::theme::set_active_by_name(t.name).unwrap();
        p.set_view(&theme_picker(DENSE));
        assert!(
            !p.backdrop_blur(),
            "{}: a crisp picker over a self-backing composition must reach NO frost \
             at all — the frame stays on the unchanged path",
            t.name
        );
        // …and the same world's NON-crisp overlay still frosts, so the exclusion is
        // about the crisp arm and not about the world losing its frost.
        let mut full = theme_picker(DENSE);
        full.overlay_crisp = false;
        p.set_view(&full);
        let one_bit = t.render_caps.backdrop == crate::theme::Backdrop::Flat;
        assert_eq!(
            p.backdrop_blur(),
            !one_bit,
            "{}: a full-takeover overlay's frost is unaffected by the footprint arm \
             (one_bit={one_bit})",
            t.name
        );
    }
    assert!(
        excluded > 0,
        "every world enrols — the byte-identical arm has no subject"
    );
    crate::theme::set_active(entry);
}
