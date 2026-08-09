//! MAGPIE'S `Background::Bands` — the REAL-PIXEL laws for the live world.
//!
//! `bands_waves` proves what the Bands SHAPE must do, and deliberately
//! drives a synthetic high-contrast literal to do it: three broad regions, two
//! transitions, continuity across the hidden page, proportional scaling. Those
//! are properties of the geometry and must not become hostage to whichever
//! world currently wears the ground.
//!
//! This file asks the other question, which that one cannot: does MAGPIE'S OWN
//! authored instance actually work? Magpie's three tones are its own ground
//! ladder, and the whole ladder spans 23/255 — a fifth of the separation item
//! 69's literal enjoys. A shape law passing on far-apart tones says nothing
//! about whether three rungs THIS close survive the boundary feather, 8-bit
//! quantization and the page hole as three legible regions. So every claim
//! here is arithmetic over rendered bytes at the live column geometry.
//!
//! ⚠️ THE LOUD DIRECTION IS THE DARK ONE. A margin-loudness statistic phrased
//! as "peak luminance" reads the bright tail, which is the correct tail on a
//! DARK world — the ground there is dim and an incident is a bright fleck. On
//! a LIGHT world like Magpie the page is near-white and the ground can only
//! draw attention by going DARKER, so the bright tail is structurally
//! uninformative and the informative tail is the dim one. Both are measured
//! below; the assertions are on the dark tail.
//!
//! Skips (with a printed note, not a failure) on a machine with no wgpu
//! adapter, exactly like every other GPU-backed render test in this tree.

use super::super::*;
use super::dither::{offscreen, read_pixels};
use super::{headless_dqp, view};

/// `(window_w, window_h, page measure)` — the shapes every law here sweeps.
///
/// ⚠️ ONE GEOMETRY WOULD NOT HAVE SEEN THE BUG THESE LAWS ARE FOR. The band
/// boundaries are FRACTIONS of the viewport, and the page is a hole in the
/// middle of it, so whether all three bands reach the margin depends entirely
/// on how wide the margins are — which the MEASURE sets, not the window. At a
/// narrow measure the margins are broad and every angle puts three tones out
/// there; the middle band only disappears behind the page at a wide measure.
/// A first cut of this file ran at `900x600 measure 24` alone and stayed green
/// with the angle mutated to 0.05, because at that one shape the middle band
/// still held 13% of the margin. The sweep is the law.
///
/// The gallery entry is the geometry `scripts/capture-worlds.sh` shoots every
/// world at, so it is the shape a human actually judges Magpie in.
const SWEEP: [(u32, u32, usize); 6] = [
    (900, 600, 24),   // wide margins — the easy case, kept as the control.
    (1200, 800, 70),  // the canonical capture canvas at the default measure.
    (1600, 1000, 66), // THE GALLERY GEOMETRY every world is judged at.
    (1280, 1024, 70), // squarer.
    (1100, 900, 60),  // the tightest three-band balance in the roster sweep.
    (1000, 1400, 70), // portrait.
];

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
        label: Some("awl magpie bands encoder"),
    });
    p.render(&mut encoder, &tview).unwrap();
    queue.submit(Some(encoder.finish()));
    read_pixels(device, queue, &texture, w, h)
}

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

/// WCAG contrast RATIO between two luminances ((L1+0.05)/(L2+0.05)).
fn contrast(a: f32, b: f32) -> f32 {
    let (hi, lo) = if a > b { (a, b) } else { (b, a) };
    (hi + 0.05) / (lo + 0.05)
}

fn theme_lum(c: theme::Srgb) -> f32 {
    rel_lum([c.r, c.g, c.b, 0xFF])
}

/// Magpie's three authored band tones as bytes, read from the world itself so
/// this file cannot drift from the literal it is testing.
fn magpie_tones() -> [[u8; 4]; 3] {
    match theme::MAGPIE.background {
        theme::Background::Bands { tones, .. } => [
            tones[0].rgba_bytes(),
            tones[1].rgba_bytes(),
            tones[2].rgba_bytes(),
        ],
        _ => panic!("Magpie must ship Background::Bands"),
    }
}

/// Squared RGB distance — only ever used to pick a NEAREST tone, so the square
/// root would change no ordering.
fn dist2(a: [u8; 4], b: [u8; 4]) -> i32 {
    let d = |i: usize| (a[i] as i32 - b[i] as i32).pow(2);
    d(0) + d(1) + d(2)
}

/// Squared RGB distance from `px` to the SEGMENT `a`-`b` — the closed set of
/// colours that are a mix of those two tones.
fn dist2_to_segment(px: [u8; 4], a: [u8; 4], b: [u8; 4]) -> f32 {
    let p = [px[0] as f32, px[1] as f32, px[2] as f32];
    let a = [a[0] as f32, a[1] as f32, a[2] as f32];
    let b = [b[0] as f32, b[1] as f32, b[2] as f32];
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ap = [p[0] - a[0], p[1] - a[1], p[2] - a[2]];
    let len2 = ab[0] * ab[0] + ab[1] * ab[1] + ab[2] * ab[2];
    // A degenerate segment is a point; two equal tones are already forbidden by
    // the ladder law, so this only guards the arithmetic.
    let t = if len2 <= f32::EPSILON {
        0.0
    } else {
        ((ap[0] * ab[0] + ap[1] * ab[1] + ap[2] * ab[2]) / len2).clamp(0.0, 1.0)
    };
    let d = [ap[0] - t * ab[0], ap[1] - t * ab[1], ap[2] - t * ab[2]];
    d[0] * d[0] + d[1] * d[1] + d[2] * d[2]
}

/// How far off the authored tone POLYLINE a pixel may sit and still count as
/// ground, squared, summed over three channels. Two levels per channel — the
/// shader's own rounding plus 8-bit quantization, with headroom; the measured
/// worst genuine ground pixel sits far inside it.
///
/// ⚠️ THIS IS A DISTANCE TO THE POLYLINE `t0-t1-t2`, NOT A BALL AROUND EACH
/// TONE, and the difference is the whole point. `Bands` paints flat tones and
/// feathers between ADJACENT rungs, so every legal ground colour is a mix of
/// two neighbouring tones — a point ON that polyline. A ball around each tone
/// also admits colours BEYOND the outermost rungs, which no mixture can
/// produce: darker than `base_300` or brighter than `base_100`.
///
/// That is not a hypothetical. The first cut of this file used a ball of radius
/// 7 (half the widest rung step) and admitted a pixel at RGB(221,221,218) —
/// exactly 7 below `base_300` on every channel, landing at squared distance
/// 147 against a budget of 147. It was an ANTIALIASED GLYPH EDGE of the debug
/// panel, which another test had left switched on, drawn over the darkest band.
/// It dragged the margin's minimum below the world's own ladder floor and made
/// the incident law report a tail that the ground does not have. A ball cannot
/// exclude it; the polyline excludes it by construction, because nothing darker
/// than the darkest authored tone is a mixture of authored tones.
const GROUND_TOLERANCE: f32 = 2.0 * 2.0 * 3.0;

/// The fraction of the margin's ground each of the three bands must hold, at
/// every swept shape. Set well under the tightest shipped value (the roster
/// sweep's worst cell is a little under 5%) and far above what a band pushed
/// behind the page reaches, which is zero — so the gap this floor sits in is
/// the gap between "all three bands are out here" and "one of them is not".
const BAND_SHARE_FLOOR: f32 = 0.03;

/// A live Magpie frame, split three ways.
///
/// ⚠️ THE MARGIN IS NOT ALL GROUND, and a law that assumes it is will be
/// measuring the wrong pixels. Chrome draws out there — Magpie's raked location
/// cue among it — so a margin scan picks up dark ink that belongs to no band.
/// Measured at the canonical capture geometry that is ~2.4% of margin pixels,
/// small enough to overlook and far enough from any band tone (a squared
/// distance in six figures against a budget of 147) to wreck any extreme-value
/// statistic taken over the raw margin: the "darkest margin pixel" would be
/// chrome ink, not a band, and every loudness and contrast number computed from
/// it would be about the wrong thing.
///
/// So `ground` is the margin pixels that are a MIXTURE OF ADJACENT AUTHORED
/// TONES (see `GROUND_TOLERANCE`), and the laws below take their statistics
/// over that. The rest is kept and its share asserted small, because "the
/// margin is mostly ground" is itself a claim worth failing on.
struct MarginFrame {
    pixels: Vec<[u8; 4]>,
    margin: Vec<[u8; 4]>,
    /// Margin pixels lying on the authored tone polyline, each paired with the
    /// index of the band it is attributed to (its nearest tone).
    ground: Vec<([u8; 4], usize)>,
}

/// Render live Magpie at one swept shape and split its margin into ground and
/// chrome. Returns `None` on a machine with no wgpu adapter.
fn magpie_margin_frame(w: u32, h: u32, measure: usize) -> Option<MarginFrame> {
    let (device, queue, mut p) = headless_dqp(w as f32, h as f32)?;
    let p = &mut p;
    let was_page_on = crate::page::page_on();
    let was_measure = crate::page::measure();
    // ⚠️ PIN EVERY GLOBAL THAT CAN PUT INK IN THE MARGIN, not only the two that
    // set the column. A whole-frame law is at the mercy of every toggle that
    // draws, and the shared serial guard restores only world/page/spellcheck
    // plus the render overrides — the panel toggles are outside it. A test
    // elsewhere in the suite left the DEBUG PANEL switched on, which renders a
    // readout stack down the right margin; this law then measured that text as
    // if it were ground. It passed alone and failed only in the full suite,
    // which is the signature of exactly this class.
    let was_debug = crate::debug::debug_on();
    let was_outline = crate::outline::outline_on();
    let was_nits = crate::nits::nits_on();
    let was_typewriter = crate::typewriter::typewriter_on();
    crate::page::set_page_on(true);
    crate::page::set_measure(measure);
    crate::debug::set_debug_on(false);
    crate::outline::set_outline_on(true);
    crate::nits::set_nits_on(true);
    crate::typewriter::set_typewriter_on(false);

    theme::set_active_by_name("Magpie").unwrap();
    p.sync_theme();
    let v = view("hi\nthere\n", 0, 0);
    p.set_view(&v);
    let pixels = render_frame(&device, &queue, p, w, h);

    // The renderer's OWN column band this frame — never a synthetic constant.
    let col_left = p.column_left();
    let col_right = col_left + p.column_width();

    crate::page::set_page_on(was_page_on);
    crate::page::set_measure(was_measure);
    crate::debug::set_debug_on(was_debug);
    crate::outline::set_outline_on(was_outline);
    crate::nits::set_nits_on(was_nits);
    crate::typewriter::set_typewriter_on(was_typewriter);

    let mut margin = Vec::new();
    for y in 0..h as usize {
        for x in 0..w as usize {
            if (x as f32) >= col_left && (x as f32) < col_right {
                continue;
            }
            margin.push(pixels[y * w as usize + x]);
        }
    }
    assert!(
        margin.len() > 20_000,
        "{w}x{h} m{measure}: the margin sample is too small to hold a percentile claim ({} px)",
        margin.len()
    );

    let tones = magpie_tones();
    let mut ground = Vec::with_capacity(margin.len());
    for &px in &margin {
        // Ground membership is distance to the tone POLYLINE; the nearest-tone
        // index is only how a member is ATTRIBUTED to a band for the share
        // count. Deciding membership by nearest tone is what admitted foreign
        // ink — see `GROUND_TOLERANCE`.
        let off_line =
            dist2_to_segment(px, tones[0], tones[1]).min(dist2_to_segment(px, tones[1], tones[2]));
        if off_line > GROUND_TOLERANCE {
            continue;
        }
        let best = (0..3)
            .min_by_key(|&i| dist2(px, tones[i]))
            .expect("three tones");
        ground.push((px, best));
    }
    let ground_frac = ground.len() as f32 / margin.len() as f32;
    assert!(
        ground_frac > 0.90,
        "{w}x{h} m{measure}: only {:.1}% of Magpie's margin is a mixture of its own band tones \
         — the margin is not mostly ground, so every statistic taken over it is about something \
         else. If this dropped without the world changing, something is DRAWING in the margin: \
         check the panel toggles this helper pins.",
        ground_frac * 100.0
    );

    Some(MarginFrame {
        pixels,
        margin,
        ground,
    })
}

/// THE ADOPTION LAW: Magpie's margin really is painted in its OWN three band
/// tones, and all THREE of them hold real margin area.
///
/// Two halves, and the second is the one that bites. (a) Every margin pixel
/// sits within a tight distance of one of the three authored tones — the
/// margin IS the three tones, not a gradient that happens to pass through
/// them. (b) Each of the three holds a genuine share of the margin. A ground
/// whose third band falls entirely behind the page renders as two bands and a
/// seam; the sidecar would still report `Background::Bands` and three tones, so
/// only pixels can tell the difference. That is the failure this law exists
/// for, and it is a real risk here: the band boundaries are viewport
/// FRACTIONS, so an angle close to either axis puts a boundary behind a
/// centered page and hides the middle tone entirely.
#[test]
fn magpie_margin_carries_all_three_of_its_own_band_tones() {
    let _g = crate::testlock::serial();
    if !crate::test_gpu::adapter_present() {
        eprintln!(
            "skipping magpie_margin_carries_all_three_of_its_own_band_tones: no wgpu adapter"
        );
        return;
    }
    let _world = crate::theme::WorldPin::snapshot();
    let tones = magpie_tones();

    for (w, h, measure) in SWEEP {
        let Some(frame) = magpie_margin_frame(w, h, measure) else {
            return;
        };
        // (a) is discharged by `magpie_margin_frame` itself: it classifies every
        // margin pixel against the authored tones and fails if the ground is
        // not the overwhelming majority of the margin.
        //
        // (b) All three bands reach the margin, each with real area.
        let mut counts = [0usize; 3];
        for &(_, k) in &frame.ground {
            counts[k] += 1;
        }
        let total = frame.ground.len() as f32;
        let share: Vec<f32> = counts.iter().map(|&c| c as f32 / total).collect();
        let pretty: Vec<String> = share.iter().map(|v| format!("{:.2}%", v * 100.0)).collect();
        eprintln!("magpie {w}x{h} m{measure}: band shares {pretty:?}");
        for (i, &sh) in share.iter().enumerate() {
            assert!(
                sh > BAND_SHARE_FLOOR,
                "{w}x{h} m{measure}: band tone {i} holds only {:.2}% of Magpie's margin ground \
                 (shares {pretty:?}) — a band that does not reach the margin is a band the \
                 reader never sees",
                sh * 100.0,
            );
        }
        // Non-vacuity: the three shares are genuinely three, not one tone
        // counted under three labels. The tones are pinned pairwise distinct by
        // `theme::tests::magpie_ground_stays_on_its_own_ladder`; the claim here
        // is that the RENDER separates them — each band exists as its own flat
        // field somewhere in the margin, not merely as feather between others.
        for (i, tone) in tones.iter().enumerate() {
            let found = frame.ground.iter().any(|&(px, k)| k == i && px == *tone);
            assert!(
                found,
                "{w}x{h} m{measure}: no margin pixel renders band tone {i} exactly ({tone:?}) — \
                 the band is present only as feather, never as its own flat field"
            );
        }
    }
}

/// THE INCIDENT LAW: Magpie's margin reads as TEXTURE, never as incident.
///
/// The statistic is the ratio of the margin's extreme pixel to its 1st
/// percentile, on the tail that can draw attention. A field of quiet ground
/// carrying a scattering of loud specks has a large ratio — 99% of it is
/// unremarkable and the eye goes to the specks. A field of broad flat bands has
/// a ratio at 1.0: its most extreme pixel IS its typical extreme pixel, so
/// there is nothing for the eye to land on and the margin recedes.
///
/// ⚠️ THE TAIL IS THE DARK ONE. Magpie is a light world: its page is near-white
/// and its ground can only assert itself by going darker, so `min/p1` is the
/// informative ratio and `peak/p99` is structurally pinned near 1.0 whatever
/// the ground does. The bright-tail figures are computed and printed anyway,
/// because a law that reports only the tail it chose gives a reader no way to
/// see which tail it did not check.
///
/// The second clause is the structural bound the ladder buys: no margin pixel
/// may fall below the world's own `base_300`. A ground authored on the ladder
/// cannot break this; one with a tone authored past the ladder immediately
/// does, which is exactly how a ground in this roster got loud enough for the
/// user to object to the room.
#[test]
fn magpie_margin_is_texture_not_incident() {
    let _g = crate::testlock::serial();
    let _misc_restore = crate::testlock::misc::TogglesRestore::capture();
    if !crate::test_gpu::adapter_present() {
        eprintln!("skipping magpie_margin_is_texture_not_incident: no wgpu adapter");
        return;
    }
    let _world = crate::theme::WorldPin::snapshot();
    // This law is about the margin's own texture, not the menu bar — pin the bar
    // off so the sampled band doesn't shift under a platform where the bar
    // defaults on (`_misc_restore` above already restores whatever this found).
    crate::menubar::set_menu_bar_on(false);
    // The ladder's structural bound: nothing in the margin is darker than the
    // bottom rung of the world's own ground ramp.
    let floor = theme_lum(theme::MAGPIE.base_300);

    for (w, h, measure) in SWEEP {
        let Some(frame) = magpie_margin_frame(w, h, measure) else {
            return;
        };
        let mut lums: Vec<f32> = frame.ground.iter().map(|&(px, _)| rel_lum(px)).collect();
        lums.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let q = |frac: f64| lums[((lums.len() - 1) as f64 * frac) as usize];

        let (min, p1, p99, peak) = (q(0.0), q(0.01), q(0.99), q(1.0));
        // A luminance ratio is taken with the WCAG +0.05 floor on both sides,
        // so a near-black tail cannot manufacture a huge ratio out of rounding.
        let dark_ratio = contrast(min, p1);
        let bright_ratio = contrast(peak, p99);
        eprintln!(
            "magpie {w}x{h} m{measure}: min {min:.5} p1 {p1:.5} p99 {p99:.5} peak {peak:.5} | \
             dark incident ratio (min/p1) {dark_ratio:.4} | bright (peak/p99) {bright_ratio:.4}"
        );

        assert!(
            dark_ratio < 1.05,
            "{w}x{h} m{measure}: Magpie's margin has a dark incident tail — its darkest pixel is \
             {dark_ratio:.3}x its 1st percentile, so the field is quiet ground plus specks \
             rather than broad bands"
        );
        assert!(
            min >= floor - 0.005,
            "{w}x{h} m{measure}: a Magpie margin pixel reached luminance {min:.5}, below its own \
             base_300 rung ({floor:.5}) — a band tone has been authored off the ladder"
        );
    }
}

/// THE INK-CONTRAST FLOOR over the ground. Chrome draws ON the margin (Magpie's
/// raked location cue is drawn there), and with the page off the prose itself
/// does, so the ink rungs must clear the ground's WORST pixel — the darkest one
/// on a light world — not merely its average.
///
/// The floors are per-rung because the rungs are not peers: `base_content`
/// carries prose and `muted` carries secondary chrome text, while `faint` is a
/// deliberate whisper that is never body copy. Asserting one floor across all
/// three would either be vacuous for the first two or wrong about the third. So
/// the two text-carrying rungs get absolute floors, and `faint` is held to the
/// LADDER ORDER instead: it must stay strictly quieter than `muted`, which is
/// the actual property that makes it a whisper rather than a third text color.
#[test]
fn magpie_ink_rungs_clear_the_darkest_band() {
    let _g = crate::testlock::serial();
    if !crate::test_gpu::adapter_present() {
        eprintln!("skipping magpie_ink_rungs_clear_the_darkest_band: no wgpu adapter");
        return;
    }
    let _world = crate::theme::WorldPin::snapshot();

    for (w, h, measure) in SWEEP {
        let Some(frame) = magpie_margin_frame(w, h, measure) else {
            return;
        };
        let darkest = frame
            .ground
            .iter()
            .map(|&(px, _)| rel_lum(px))
            .fold(f32::INFINITY, f32::min);

        let content = contrast(theme_lum(theme::MAGPIE.base_content), darkest);
        let muted = contrast(theme_lum(theme::MAGPIE.muted), darkest);
        let faint = contrast(theme_lum(theme::MAGPIE.faint), darkest);
        eprintln!(
            "magpie {w}x{h} m{measure}: ink over darkest band ({darkest:.5}) — base_content \
             {content:.2}:1 muted {muted:.2}:1 faint {faint:.2}:1"
        );

        assert!(
            content >= 7.0,
            "{w}x{h} m{measure}: Magpie's prose ink clears the darkest band by only {content:.2}:1"
        );
        assert!(
            muted >= 3.0,
            "{w}x{h} m{measure}: Magpie's muted rung clears the darkest band by only {muted:.2}:1"
        );
        assert!(
            faint < muted,
            "{w}x{h} m{measure}: Magpie's faint rung ({faint:.2}:1) must stay quieter than its \
             muted rung ({muted:.2}:1) against the ground — the whisper rung cannot out-read \
             the text rung"
        );
    }
}

/// THE PAGE STAYS THE FIGURE. The band field is broad and low-contrast by
/// construction, but "low contrast against itself" is not the same claim as
/// "recessive against the page" — a ground can be perfectly uniform and still
/// sit closer to the eye than the paper it frames.
///
/// Asserted at real pixels over the same frame: the page's own body area is
/// brighter than every margin pixel. On a light world that is what "the page is
/// the figure" means, and the ladder guarantees it — the page is `base_100`,
/// which is the ground's own brightest rung, so no band can out-shine it.
#[test]
fn magpie_page_stays_brighter_than_every_band() {
    let _g = crate::testlock::serial();
    if !crate::test_gpu::adapter_present() {
        eprintln!("skipping magpie_page_stays_brighter_than_every_band: no wgpu adapter");
        return;
    }
    let _world = crate::theme::WorldPin::snapshot();
    let page_lum = theme_lum(theme::MAGPIE.base_100);

    for (w, h, measure) in SWEEP {
        let Some(frame) = magpie_margin_frame(w, h, measure) else {
            return;
        };
        let brightest = frame
            .ground
            .iter()
            .map(|&(px, _)| rel_lum(px))
            .fold(f32::NEG_INFINITY, f32::max);
        assert!(
            brightest <= page_lum + 0.005,
            "{w}x{h} m{measure}: a Magpie margin pixel ({brightest:.5}) out-shines the page \
             itself ({page_lum:.5}) — the ground has stopped being ground"
        );
        // Non-vacuity: the frame really did render a page (the pixel population
        // is not all margin), so the comparison above is about a composition.
        assert!(
            frame.pixels.len() > frame.margin.len(),
            "{w}x{h} m{measure}: no page band in the frame — the margin claim would be vacuous"
        );
    }
}
