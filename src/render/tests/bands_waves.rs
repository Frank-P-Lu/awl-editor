//! REAL-PIXEL proofs for `Background::Bands` (three broad
//! tone-on-tone diagonal bands) and Bombora's wave-TIERS
//! (`Background::Waves`), the two grounds that replaced Gumtree's old
//! uniform Dots grid and Bombora's old scattered-star ground. Mirrors `dither.rs`'s
//! pattern — drive `BackgroundPipeline` directly (the purest reachable seam,
//! no text/markdown involved) and read the real GPU output back.
//!
//! **The Bands laws below drive a SYNTHETIC literal, and that is deliberate.**
//! Gumtree's own ground moved from Bands to a repeating chevron
//! `Background::Zigzag` (see `zigzag_ground.rs` for that ground's own
//! real-pixel laws), leaving Bands unworn for a time; the laws were repointed
//! at an explicit literal — Gumtree's old tones/angle, preserved verbatim —
//! rather than deleted, because the geometry/continuity/determinism properties
//! a variant must hold are about the SHAPE, never about which world wears it.
//!
//! **Bands is worn again — Magpie took it — and the literal STAYS anyway.**
//! Repointing these laws at Magpie was considered and rejected on two grounds,
//! one structural and one measured. Structurally, a shape law pinned to a live
//! world becomes a hostage: Magpie's angle and tones are taste, a future taste
//! round will move them, and a geometry proof must not go red because a world
//! was redecorated — that coupling is precisely what moving to a literal
//! removed. Measured, these laws classify each pixel to its NEAREST tone, so
//! their strength is the tone separation they run at: the literal's rungs are
//! 21 and 24 apart on their dominant channel, Magpie's ladder rungs are 10 and
//! 13, so running the shape proof on Magpie would halve the classifier's margin
//! to buy nothing. Magpie's own instance is proved separately, at real pixels
//! and at that narrow separation, by `magpie_bands` — the two files ask
//! different questions and both answers are wanted.
//!
//! Per the project tripwire (the sidecar is a STATE oracle, never an
//! APPEARANCE oracle — it once reported a selected row that rendered fully
//! invisible), every "exactly three bands/tiers" claim here is proven by
//! PIXEL arithmetic over the rendered bytes, not by inspecting the `Background`
//! data alone (that data-level check lives separately in `theme::tests`).
//!
//! Skips (with a printed note, not a failure) on a machine with no wgpu
//! adapter, exactly like every other GPU-backed render test in this tree.

use crate::background::BgDesc;
use crate::theme;

pub(super) fn headless_dq() -> Option<(wgpu::Device, wgpu::Queue)> {
    crate::test_gpu::shared_device_queue()
}

/// Flatten a world's [`theme::Background`] into a [`BgDesc`] the SAME way
/// `render::background_desc` does for the live/headless renderer — one owner
/// of the accessor call sequence, reused here so the test drives the exact
/// same upload shape production code does.
pub(super) fn bg_desc_for(bg: theme::Background) -> BgDesc {
    BgDesc {
        from: bg.from().rgba_bytes(),
        to: bg.to().rgba_bytes(),
        dir: bg.dir(),
        shader: bg.shader_id(),
        tint: bg.tint().rgb_bytes(),
        edge: bg.edge(),
        angle: bg.angle(),
        period_px: bg.period_px(),
        amplitude_px: bg.amplitude_px(),
        density: bg.density(),
        banded: bg.zigzag_banded(),
        profile: bg.profile_mode(),
        tunnel: bg.tunnel_mode(),
    }
}

/// Draw a `BackgroundPipeline` covering a `width`x`height` canvas, with a page
/// column hole at `[col_left, col_left+col_w)` (pass `col_w = 0.0` for NO hole
/// — the whole canvas is margin, the purest scan surface for the band/tier
/// count laws), and a WAVES phase `drift` in radians — inert
/// (`0.0`) for every non-Waves ground (Zigzag callers pass `0.0`).
/// Mirrors `dither.rs::render_background`, generalized with the column + drift
/// params this file's laws and its sibling module need.
// Test rendering passes the real GPU and background controls explicitly to mirror production setup.
#[allow(clippy::too_many_arguments)]
pub(super) fn render_bg(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    desc: BgDesc,
    width: u32,
    height: u32,
    col_left: f32,
    col_w: f32,
    drift: f32,
) -> Vec<[u8; 4]> {
    render_bg_scaled(
        device, queue, desc, width, height, col_left, col_w, drift, 1.0,
    )
}

/// [`render_bg`] with the item-186 device ratio (PHYSICAL px per LOGICAL px)
/// spelled out — `1.0` is the 1:1 screen every pre-186 law renders at, and the
/// ground-space sweep drives the same seam at `2.0` on a doubled canvas.
#[allow(clippy::too_many_arguments)]
pub(super) fn render_bg_scaled(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    desc: BgDesc,
    width: u32,
    height: u32,
    col_left: f32,
    col_w: f32,
    drift: f32,
    scale: f32,
) -> Vec<[u8; 4]> {
    render_bg_ambient(
        device,
        queue,
        desc,
        width,
        height,
        col_left,
        col_w,
        crate::background::AmbientUpload {
            drift,
            ..Default::default()
        },
        scale,
    )
}

/// [`render_bg_scaled`] generalized to the FULL [`crate::background::AmbientUpload`]
/// — the direct-injection seam every ambient consumer's own real-pixel
/// wrap-continuity law needs, since `render_bg`/`render_bg_scaled` only ever
/// expose the single `drift` (radians) field. `render_bg_scaled` itself is
/// now a thin wrapper over this.
#[allow(clippy::too_many_arguments)]
pub(super) fn render_bg_ambient(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    desc: BgDesc,
    width: u32,
    height: u32,
    col_left: f32,
    col_w: f32,
    ambient: crate::background::AmbientUpload,
    scale: f32,
) -> Vec<[u8; 4]> {
    let mut bg = crate::background::BackgroundPipeline::new(device, super::dither::FMT, desc);
    bg.prepare(queue, width, height, col_left, col_w, ambient, scale);
    let (texture, tview) = super::dither::offscreen(device, width, height);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("awl item69-bg-test encoder"),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("awl item69-bg-test pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &tview,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        bg.draw(&mut pass);
    }
    queue.submit(Some(encoder.finish()));
    super::dither::read_pixels(device, queue, &texture, width, height)
}

/// Classify a pixel to the nearest of the world's three authored tones (exact
/// byte match expected away from a boundary's ~3px antialiased feather; the
/// nearest-tone fallback only matters IN that feather, where it still resolves
/// unambiguously to whichever tone is closer). Returns `0`, `1`, or `2`.
fn classify(px: [u8; 4], tones: [[u8; 4]; 3]) -> usize {
    let d2 = |a: [u8; 4], b: [u8; 4]| {
        (0..3)
            .map(|i| {
                let d = a[i] as i32 - b[i] as i32;
                d * d
            })
            .sum::<i32>()
    };
    (0..3).min_by_key(|&i| d2(px, tones[i])).unwrap()
}

/// The ascending-x (or ascending-y) run-length-encoded LABEL sequence of a
/// scanline, e.g. `[0,0,0,1,1,2]` -> `[0,1,2]` — the transition count is
/// `runs.len() - 1`. Exactly what "N broad bands/tiers" needs to assert
/// without being tripped up by the few antialiased pixels at each boundary.
fn runs(labels: &[usize]) -> Vec<usize> {
    let mut out: Vec<usize> = Vec::new();
    for &l in labels {
        if out.last() != Some(&l) {
            out.push(l);
        }
    }
    out
}

/// The SYNTHETIC `Background::Bands` value under regression test here —
/// Gumtree's old tones/angle, preserved verbatim as a literal so this file's
/// geometry laws keep proving the variant's own shape rather than a live
/// world's current pick. See this module's header for why it stayed a literal
/// after Magpie adopted the ground.
fn bands_test_bg() -> theme::Background {
    theme::Background::Bands {
        tones: [
            theme::Srgb::rgb(0xE4, 0xF8, 0xE2),
            theme::Srgb::rgb(0xCF, 0xF3, 0xCC),
            theme::Srgb::rgb(0xB7, 0xEF, 0xB4),
        ],
        angle: 0.56,
    }
}

fn bands_tones() -> [[u8; 4]; 3] {
    match bands_test_bg() {
        theme::Background::Bands { tones, .. } => [
            tones[0].rgba_bytes(),
            tones[1].rgba_bytes(),
            tones[2].rgba_bytes(),
        ],
        _ => unreachable!(),
    }
}

fn bombora_tones() -> [[u8; 4]; 3] {
    match theme::BOMBORA.background {
        theme::Background::Waves { tones } => [
            tones[0].rgba_bytes(),
            tones[1].rgba_bytes(),
            tones[2].rgba_bytes(),
        ],
        _ => panic!("Bombora must ship Background::Waves"),
    }
}

/// A horizontal scanline at `y` across the WHOLE canvas (no page hole),
/// classified into the three-tone label sequence, then run-length-collapsed.
fn scan_row(pixels: &[[u8; 4]], w: u32, y: u32, tones: [[u8; 4]; 3]) -> Vec<usize> {
    let labels: Vec<usize> = (0..w)
        .map(|x| classify(pixels[(y * w + x) as usize], tones))
        .collect();
    runs(&labels)
}

/// A vertical scanline at `x` down the WHOLE canvas height, same shape as
/// [`scan_row`].
fn scan_col(pixels: &[[u8; 4]], w: u32, h: u32, x: u32, tones: [[u8; 4]; 3]) -> Vec<usize> {
    let labels: Vec<usize> = (0..h)
        .map(|y| classify(pixels[(y * w + x) as usize], tones))
        .collect();
    runs(&labels)
}

/// Per-region PIXEL COUNT of a classified scanline (not just the run-length
/// order) — the "broad enough to read as three authored shapes, not a sliver"
/// half of the law.
fn region_widths(pixels: &[[u8; 4]], w: u32, y: u32, tones: [[u8; 4]; 3]) -> [u32; 3] {
    let mut counts = [0u32; 3];
    for x in 0..w {
        counts[classify(pixels[(y * w + x) as usize], tones)] += 1;
    }
    counts
}

// ---------------------------------------------------------------------------
// GUMTREE — Bands
// ---------------------------------------------------------------------------

/// THE CORE LAW: Gumtree's canonical mid-field witness crosses EXACTLY THREE
/// broad diagonal bands. A horizontal scanline through the canvas midline, at
/// the canonical capture canvas size (`capture::CANVAS_WIDTH`/`HEIGHT`), run-
/// length-collapses to exactly `[0,1,2]` or `[2,1,0]` (three ordered regions,
/// two transitions) — never more (a repeating stripe-tile) and never fewer (a
/// degenerate flat fill).
#[test]
fn bands_canonical_mid_field_crosses_exactly_three_bands() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        eprintln!(
            "skipping bands_canonical_mid_field_crosses_exactly_three_bands: no wgpu adapter"
        );
        return;
    };
    let _g = crate::testlock::serial();

    let (w, h) = (crate::capture::CANVAS_WIDTH, crate::capture::CANVAS_HEIGHT);
    let desc = bg_desc_for(bands_test_bg());
    let pixels = render_bg(&device, &queue, desc, w, h, 0.0, 0.0, 0.0);
    let tones = bands_tones();

    let seq = scan_row(&pixels, w, h / 2, tones);
    assert_eq!(
        seq.len(),
        3,
        "expected exactly 3 bands crossed, got run sequence {seq:?}"
    );
    assert_eq!(
        seq.iter().collect::<std::collections::HashSet<_>>().len(),
        3,
        "the three runs must be the three DISTINCT tones (no tone repeats), got {seq:?}"
    );

    // BROAD, not a sliver: each band takes a real share of the scanline.
    let widths = region_widths(&pixels, w, h / 2, tones);
    for (i, &wpx) in widths.iter().enumerate() {
        assert!(
            wpx as f32 / w as f32 > 0.15,
            "band {i} is only {wpx}px of {w} — too thin to read as an authored shape"
        );
    }
}

/// RESPONSIVE CROP/SCALE, never a periodic tile, at three representative page
/// widths (narrow/canonical/wide, all SQUARE canvases so the diagonal
/// projection's aspect ratio is held fixed and the boundary fraction is
/// analytically exact rather than aspect-confounded): the field shows EXACTLY
/// three bands at every size (never more stripes — the direct disproof of
/// "periodic tile": a fixed ~13px period, Stripes' own period, would cross
/// ~50-140 bands at these sizes, not three), and the boundary's fraction of
/// the canvas is the SAME at every size (a periodic tile would instead put
/// the boundary at a near-constant PIXEL offset, so its FRACTION would shrink
/// as the canvas grows — the opposite of what a scaled field does).
#[test]
fn bands_narrow_canonical_wide_still_show_exactly_three_bands_that_scale_not_tile() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        eprintln!(
            "skipping bands_narrow_canonical_wide_still_show_exactly_three_bands_that_scale_not_tile: no wgpu adapter"
        );
        return;
    };
    let _g = crate::testlock::serial();

    let desc = bg_desc_for(bands_test_bg());
    let tones = bands_tones();

    let mut first_boundary_x = Vec::new();
    for side in [700u32, 1200, 1800] {
        let (w, h) = (side, side);
        let pixels = render_bg(&device, &queue, desc, w, h, 0.0, 0.0, 0.0);
        let labels: Vec<usize> = (0..w)
            .map(|x| classify(pixels[(h / 2 * w + x) as usize], tones))
            .collect();
        let seq = runs(&labels);
        assert_eq!(
            seq.len(),
            3,
            "side={side}: expected exactly 3 bands, got {seq:?}"
        );
        // The first x where the label changes away from labels[0].
        let boundary = labels.iter().position(|&l| l != labels[0]).unwrap() as f32;
        first_boundary_x.push((side, boundary));
    }
    // Proportional scaling check: on a SQUARE canvas the diagonal projection is
    // homogeneous of degree 1 in (x, side), so boundary/side is EXACTLY the
    // same fraction at every size — a periodic tile would instead put the
    // boundary at a near-fixed PIXEL offset, which fails this ratio check hard.
    let fractions: Vec<f32> = first_boundary_x
        .iter()
        .map(|&(side, b)| b / side as f32)
        .collect();
    let lo = fractions.iter().cloned().fold(f32::INFINITY, f32::min);
    let hi = fractions.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    assert!(
        hi - lo < 0.01,
        "boundary fraction drifted across sizes {first_boundary_x:?} -> fractions {fractions:?} \
         (spread {:.4}) — looks like a fixed-period TILE, not a scaled field",
        hi - lo
    );
}

/// AUDIT REGRESSION (Fable): at the CANONICAL capture
/// canvas (a wide ~1200x800 aspect, not square like the test above), a
/// full-height 16px-wide margin sliver on EITHER side of the page column must
/// show at least two of the three tones — i.e. it catches a band edge
/// in-viewport. Before the center-anchored re-scale in `bands_rgb`, the wide
/// aspect ratio pushed BOTH margins' entire vertical extent into one flat
/// corner of the field (left always tone0, right always tone2), so the
/// default window size silently degraded the whole grass-band idea to two
/// flat tones even though the mid-field scanline still read as three bands.
#[test]
fn bands_canonical_margin_slivers_each_catch_a_band_edge() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        eprintln!(
            "skipping bands_canonical_margin_slivers_each_catch_a_band_edge: no wgpu adapter"
        );
        return;
    };
    let _g = crate::testlock::serial();

    let (w, h) = (crate::capture::CANVAS_WIDTH, crate::capture::CANVAS_HEIGHT);
    let desc = bg_desc_for(bands_test_bg());
    let pixels = render_bg(&device, &queue, desc, w, h, 0.0, 0.0, 0.0);
    let tones = bands_tones();

    // A representative column near each edge, well inside a 16px margin
    // sliver (the same order of magnitude a page-mode margin actually is).
    for (label, x) in [("left", 8u32), ("right", w - 8)] {
        let seq = scan_col(&pixels, w, h, x, tones);
        assert!(
            seq.len() >= 2,
            "{label} margin sliver at x={x} shows only ONE tone top-to-bottom \
             (run sequence {seq:?}) — the band field degenerated to a flat fill \
             at the canonical viewport size"
        );
    }
}

// ---------------------------------------------------------------------------
// BOMBORA — Waves
// ---------------------------------------------------------------------------

/// THE CORE LAW: Bombora exposes EXACTLY THREE non-overlapping wave tiers at a
/// fixed vertical scanline through the canonical capture canvas.
/// Run-length-collapses to exactly 3 distinct labels top-to-bottom
/// (non-overlapping: were the tiers to interleave, the run count would exceed
/// 3 — a `[0,1,0,2]`-shaped sequence, which this asserts against directly).
#[test]
fn bombora_canonical_mid_field_exposes_exactly_three_wave_tiers() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        eprintln!(
            "skipping bombora_canonical_mid_field_exposes_exactly_three_wave_tiers: no wgpu adapter"
        );
        return;
    };
    let _g = crate::testlock::serial();

    let (w, h) = (crate::capture::CANVAS_WIDTH, crate::capture::CANVAS_HEIGHT);
    let desc = bg_desc_for(theme::BOMBORA.background);
    let pixels = render_bg(&device, &queue, desc, w, h, 0.0, 0.0, 0.0);
    let tones = bombora_tones();

    let seq = scan_col(&pixels, w, h, w / 2, tones);
    assert_eq!(
        seq,
        vec![0, 1, 2],
        "expected exactly the three tiers top to bottom in order, got {seq:?}"
    );
}

/// FIXED PHASE/GEOMETRY, horizontally phase-offset: sampling the tier
/// boundaries at several x columns still shows exactly three non-overlapping
/// tiers at EVERY column (the geometry is a fixed shader constant, not
/// per-column noise), while the two boundary ROWS themselves visibly MOVE
/// (and move by DIFFERENT amounts) across columns — the "wide scalloped
/// crests, phase-offset so they layer" claim, arithmetically: if the two
/// boundaries moved in lockstep the middle tier's thickness would stay
/// constant across x, which is exactly the "a grid, not layered" failure mode
/// this rules out.
#[test]
fn bombora_wave_boundaries_are_phase_offset_scallops_not_a_grid() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        eprintln!(
            "skipping bombora_wave_boundaries_are_phase_offset_scallops_not_a_grid: no wgpu adapter"
        );
        return;
    };
    let _g = crate::testlock::serial();

    let (w, h) = (1200u32, 800u32);
    let desc = bg_desc_for(theme::BOMBORA.background);
    let pixels = render_bg(&device, &queue, desc, w, h, 0.0, 0.0, 0.0);
    let tones = bombora_tones();

    let mut middle_thickness = Vec::new();
    for x in [80u32, 400, 700, 1050] {
        let labels: Vec<usize> = (0..h)
            .map(|y| classify(pixels[(y * w + x) as usize], tones))
            .collect();
        let seq = runs(&labels);
        assert_eq!(
            seq,
            vec![0, 1, 2],
            "x={x}: expected exactly the three tiers top to bottom, got {seq:?}"
        );
        let thickness = labels.iter().filter(|&&l| l == 1).count();
        middle_thickness.push((x, thickness));
    }
    let lo = middle_thickness.iter().map(|&(_, t)| t).min().unwrap();
    let hi = middle_thickness.iter().map(|&(_, t)| t).max().unwrap();
    assert!(
        hi - lo > 4,
        "the middle tier's thickness barely varies across x ({middle_thickness:?}) — \
         the two boundaries look locked in lockstep (a grid), not phase-offset scallops"
    );
}

// ---------------------------------------------------------------------------
// CONTINUITY THROUGH THE HIDDEN PAGE (both worlds)
// ---------------------------------------------------------------------------

/// LEFT/RIGHT CONTINUITY: the field is ONE continuous function of the
/// ABSOLUTE pixel position, occluded (not restarted) by the page hole. Render
/// the SAME desc twice at the SAME canvas size — once with no hole (the
/// reference field) and once with a page column punched in the middle — and
/// prove every visible margin pixel (left AND right of the hole) matches the
/// reference field byte-for-byte at that same absolute coordinate.
fn assert_left_right_continuity_through_the_page(
    bg: theme::Background,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) {
    let (w, h) = (1200u32, 800u32);
    let desc = bg_desc_for(bg);
    let reference = render_bg(device, queue, desc, w, h, 0.0, 0.0, 0.0);
    let (col_left, col_w) = (350.0f32, 500.0f32); // a representative centered page column
    let occluded = render_bg(device, queue, desc, w, h, col_left, col_w, 0.0);

    // Sanity: the page column itself DOES differ from the unoccluded reference
    // somewhere (the hole is actually punched, not a no-op render) — checked
    // once at the column's own center, well clear of any boundary pixel.
    let mid_idx = ((h / 2) * w + (col_left + col_w / 2.0) as u32) as usize;
    assert_ne!(
        occluded[mid_idx], reference[mid_idx],
        "the page column must actually occlude the field"
    );

    let mut checked = 0usize;
    for y in (0..h).step_by(37) {
        for x in (0..w).step_by(11) {
            let is_page = (x as f32) >= col_left && (x as f32) < col_left + col_w;
            if is_page {
                continue;
            }
            let idx = (y * w + x) as usize;
            assert_eq!(
                occluded[idx], reference[idx],
                "({x},{y}) margin pixel diverges from the unoccluded reference field — \
                 the page hole must OCCLUDE the field, never restart/re-origin it"
            );
            checked += 1;
        }
    }
    assert!(
        checked > 500,
        "sanity: too few margin pixels checked ({checked})"
    );
}

#[test]
fn bands_are_continuous_left_and_right_of_the_hidden_page() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        eprintln!(
            "skipping bands_are_continuous_left_and_right_of_the_hidden_page: no wgpu adapter"
        );
        return;
    };
    let _g = crate::testlock::serial();
    assert_left_right_continuity_through_the_page(bands_test_bg(), &device, &queue);
}

#[test]
fn bombora_waves_are_continuous_left_and_right_of_the_hidden_page() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        eprintln!(
            "skipping bombora_waves_are_continuous_left_and_right_of_the_hidden_page: no wgpu adapter"
        );
        return;
    };
    let _g = crate::testlock::serial();
    assert_left_right_continuity_through_the_page(theme::BOMBORA.background, &device, &queue);
}

// ---------------------------------------------------------------------------
// DPI / ZOOM INDEPENDENCE (both worlds)
// ---------------------------------------------------------------------------

/// DPI/ZOOM INDEPENDENCE: the shader's ONLY spatial inputs are the PHYSICAL-
/// pixel viewport + column bounds (no separate DPI uniform exists at all —
/// `background.rs`'s `Globals` has no such field), so a @2x physical canvas
/// must show the identical field SCALED exactly 2x, not a shifted or
/// differently-shaped one: the first band/tier boundary at 2x canvas size
/// lands within a couple of AA pixels of exactly double its 1x position.
fn assert_boundary_scales_with_resolution(
    bg: theme::Background,
    tones: [[u8; 4]; 3],
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    vertical: bool,
) {
    let (w1, h1) = (600u32, 400u32);
    let (w2, h2) = (w1 * 2, h1 * 2);
    let desc = bg_desc_for(bg);

    let find_boundary = |pixels: &[[u8; 4]], w: u32, h: u32| -> f32 {
        if vertical {
            let labels: Vec<usize> = (0..h)
                .map(|y| classify(pixels[(y * w + w / 2) as usize], tones))
                .collect();
            labels.iter().position(|&l| l != labels[0]).unwrap() as f32
        } else {
            let labels: Vec<usize> = (0..w)
                .map(|x| classify(pixels[(h / 2 * w + x) as usize], tones))
                .collect();
            labels.iter().position(|&l| l != labels[0]).unwrap() as f32
        }
    };

    let p1 = render_bg(device, queue, desc, w1, h1, 0.0, 0.0, 0.0);
    let p2 = render_bg(device, queue, desc, w2, h2, 0.0, 0.0, 0.0);
    let b1 = find_boundary(&p1, w1, h1);
    let b2 = find_boundary(&p2, w2, h2);

    assert!(
        (b2 - 2.0 * b1).abs() <= 4.0,
        "boundary at 2x resolution ({b2}) is not ~2x the 1x boundary ({b1}, expected ~{}) \
         — the field is not scaling proportionally with physical resolution",
        2.0 * b1
    );
}

#[test]
fn bands_boundary_scales_proportionally_with_physical_resolution() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        eprintln!(
            "skipping bands_boundary_scales_proportionally_with_physical_resolution: no wgpu adapter"
        );
        return;
    };
    let _g = crate::testlock::serial();
    assert_boundary_scales_with_resolution(bands_test_bg(), bands_tones(), &device, &queue, false);
}

/// Bombora's tier BASELINE (the viewport-relative 1/3, 2/3 split) scales with
/// the canvas exactly like Gumtree's band boundary does — proven separately
/// below. Its scallop WOBBLE, though, is `waves_rgb`'s own FIXED-PHYSICAL-
/// PIXEL constant (`WAVE_AMP`, deliberately NOT viewport-relative — the doc
/// on `Background::Waves` calls this out: "tier geometry is FIXED shader
/// math, not per-world data"), so DPI/zoom independence for Bombora reads
/// differently than for Gumtree's pure-diagonal Bands: the wobble's own PIXEL
/// magnitude (max boundary row - min boundary row, sampled across x) must
/// stay the SAME fixed number of physical pixels at a small canvas and at a
/// @2x one — never silently doubling just because the canvas grew (the same
/// "no hidden DPI multiplier" law Dots' 24px cell already holds, generalized
/// to a wobble amplitude instead of a grid period).
#[test]
fn bombora_wave_wobble_is_a_fixed_logical_pixel_amplitude_not_canvas_scaled() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        eprintln!(
            "skipping bombora_wave_wobble_is_a_fixed_logical_pixel_amplitude_not_canvas_scaled: \
             no wgpu adapter"
        );
        return;
    };
    let _g = crate::testlock::serial();

    let desc = bg_desc_for(theme::BOMBORA.background);
    let tones = bombora_tones();
    let wobble_amplitude = |w: u32, h: u32| -> u32 {
        let pixels = render_bg(&device, &queue, desc, w, h, 0.0, 0.0, 0.0);
        let mut rows = Vec::new();
        for x in (0..w).step_by(20) {
            let labels: Vec<usize> = (0..h)
                .map(|y| classify(pixels[(y * w + x) as usize], tones))
                .collect();
            // The tier-0/tier-1 boundary row at this column.
            rows.push(labels.iter().position(|&l| l != labels[0]).unwrap() as u32);
        }
        rows.iter().max().unwrap() - rows.iter().min().unwrap()
    };

    let small = wobble_amplitude(600, 400);
    let large = wobble_amplitude(1200, 800);
    // Both should land near 2*WAVE_AMP (44px, the boundary's own sin() peak-
    // to-peak range) regardless of canvas size — NOT roughly double at the
    // larger canvas, which is what a wobble authored as a FRACTION of the
    // viewport would show. It sweeps CANVAS SIZE at
    // one device ratio and always has, so "physical pixel" was never what it
    // proved. `WAVE_AMP` is a fixed LOGICAL px constant — the device-ratio half
    // of the claim is `ground_space`'s to make.)
    for (label, amp) in [("small", small), ("large", large)] {
        assert!(
            (30..60).contains(&amp),
            "{label} canvas wobble amplitude {amp}px escaped the fixed ~44px band \
             (2*WAVE_AMP) — the scallop should be a fixed authored constant, not a \
             fraction of the canvas"
        );
    }
    assert!(
        (small as i32 - large as i32).abs() < 20,
        "wobble amplitude small={small}px vs large={large}px diverged too far — \
         looks canvas-scaled, not the fixed authored constant it's meant to be"
    );
}

// ---------------------------------------------------------------------------
// DETERMINISM (both worlds) — static, no time/randomness
// ---------------------------------------------------------------------------

/// STATIC: two independent renders of the SAME desc at the SAME size are
/// byte-for-byte identical — no clock, no randomness, exactly the determinism
/// every other background ground already holds (a headless capture stays
/// byte-stable).
#[test]
fn bands_and_waves_render_byte_identically_across_two_independent_draws() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        eprintln!(
            "skipping bands_and_waves_render_byte_identically_across_two_independent_draws: no wgpu adapter"
        );
        return;
    };
    let _g = crate::testlock::serial();

    for bg in [bands_test_bg(), theme::BOMBORA.background] {
        let desc = bg_desc_for(bg);
        let a = render_bg(&device, &queue, desc, 900, 600, 200.0, 400.0, 0.0);
        let b = render_bg(&device, &queue, desc, 900, 600, 200.0, 400.0, 0.0);
        assert_eq!(
            a,
            b,
            "{}: two draws of the identical desc diverged",
            bg.as_str()
        );
    }
}

// ---------------------------------------------------------------------------
// BYTE-IDENTITY OF EVERY UNRELATED WORLD (roster sweep)
// ---------------------------------------------------------------------------

/// Every shipping world's authored shader assignment stays pinned while new
/// shared background capabilities join the roster. Currawong's separate
/// ambient lifecycle stars are not part of this pipeline; its base ground
/// remains a Gradient.
#[test]
fn every_world_reports_its_authored_shader_id() {
    // Pins the EXACT roster of shader ids that could plausibly have
    // disturbed, one line per world, so a future accidental edit to any of
    // these worlds' `background` field fails HERE first. Quokka and Gumtree
    // are EXCLUDED (both use `Zigzag`, id 7 — asserted
    // separately below, alongside Bombora's own item-69 id).
    let expected: &[(&str, u32)] = &[
        ("Potoroo", 4),   // Stripes — untouched
        ("Mulga", 3),     // Pinstripe — Mulga's ground since the star retirement
        ("Currawong", 0), // Gradient (+ separate ambient stars, unaffected)
        ("Bilby", 0),
        ("Magpie", 5), // Bands — the ground's one live assignee
        ("Saltpan", 3),
        ("Galah", 9), // Deckle/Fibres
        ("Mopoke", 1),
        ("Bowerbird", 8), // Organic cutout field
        ("Brolga", 0),
        ("Mangrove", 0), // Lava degrades to 0 for the base margin pass
        ("Tawny", 1),
        ("Wagtail", 0),
        ("Firetail", 0), // Lava degrades to 0 for the base margin pass
        ("Cassowary", 3),
    ];
    for &(name, want) in expected {
        let t = theme::THEMES
            .iter()
            .find(|t| t.name == name)
            .unwrap_or_else(|| panic!("world {name} not found"));
        assert_eq!(t.background.shader_id(), want, "{name}: shader id drifted");
    }
    // The item-69 Waves world carries its OWN id and never recycled the one
    // it migrated off. That id — 2, the retired scattered-star ground's — is
    // now permanently VACANT (see `Background::shader_id`), and this assertion
    // is part of why: it is only evidence that the migration did not reuse the
    // number for as long as the number stays unissued. No world may report 2.
    assert_eq!(
        theme::BOMBORA.background.shader_id(),
        6,
        "Bombora must carry its own Waves id (6), never the id it migrated off"
    );
    for t in theme::THEMES {
        assert_ne!(
            t.background.shader_id(),
            2,
            "{}: shader id 2 is retired and must stay unissued — a ground that \
             claims it inherits the vacated branch, and this file's Bombora \
             assertion silently stops meaning anything",
            t.name
        );
    }
    // The item-86 Zigzag worlds carry their NEW id, never their old ones
    // (Quokka was Dots/1, Gumtree was Bands/5 — see `zigzag_ground.rs`).
    assert_eq!(
        theme::QUOKKA.background.shader_id(),
        7,
        "Quokka must be Zigzag (7), not the old Dots (1)"
    );
    assert_eq!(
        theme::GUMTREE.background.shader_id(),
        7,
        "Gumtree must be Zigzag (7), not Bands (5)"
    );
    // Bands' id 5 is LIVE, and exactly one world reports it. The pairing with
    // the Gumtree assertion just above is the point: 5 is not vacant like 2,
    // it moved houses, and the roster must show it at exactly one address.
    let bands: Vec<&str> = theme::THEMES
        .iter()
        .filter(|t| t.background.shader_id() == 5)
        .map(|t| t.name)
        .collect();
    assert_eq!(bands, ["Magpie"], "shader id 5 (Bands) is Magpie's alone");
}
