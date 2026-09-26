//! KITE's WARPED-GRID laws — the ground's own test module.
//!
//! Two rules shape everything here.
//!
//! **Data claims sweep exhaustively.** Roster membership, the inert default and
//! the non-assigned-world identity ride no-wildcard matches over the whole
//! `Background` enum and the whole `THEMES` roster, so a future variant or world
//! cannot dodge them without a compile error.
//!
//! **Appearance claims are arithmetic over real GPU pixels, measured
//! DIFFERENTIALLY.** Every one renders the world as authored minus the same
//! world with `density: 0.0` (the `mark_field` oracle), so the flat ground
//! cancels exactly and what remains is the field alone. `density: 0.0` collapsing
//! this ground to its flat `ground` tone EXACTLY is what makes that possible, and
//! it is asserted directly.
//!
//! Nothing here trusts the sidecar for how the field LOOKS — CAPTURE.md's
//! "state oracle, not an appearance oracle" tripwire.

use super::bands_waves::{bg_desc_for, headless_dq};
use super::zigzag_ground::margins;
use crate::background::BgDesc;
use crate::theme;
use crate::warpgrid;

/// The canonical wide-Retina-ish scan surface: a real 1600x1000 gallery canvas
/// with the app's own adaptive column at measure 66 (the geometry
/// `scripts/capture-worlds.sh` shoots every world at).
pub(super) const W: u32 = 1600;
pub(super) const H: u32 = 1000;
pub(super) const COL_LEFT: f32 = 324.0;
pub(super) const COL_W: f32 = 950.0;

/// Total-channel deviation below this is 8-bit quantization, not a mark.
pub(super) const INK_FLOOR: i32 = 3;

pub(super) fn kite() -> theme::Background {
    theme::KITE.background
}

fn with_density(bg: theme::Background, density: f32) -> theme::Background {
    match bg {
        theme::Background::WarpedGrid {
            ground,
            minor,
            major,
            tunnel,
            spacing_px,
            fold,
            twist,
            forward_drift,
            ribs,
            ..
        } => theme::Background::WarpedGrid {
            ground,
            minor,
            major,
            tunnel,
            spacing_px,
            density,
            fold,
            twist,
            forward_drift,
            ribs,
        },
        other => other,
    }
}

pub(super) fn with_tunnel(bg: theme::Background, tunnel: theme::Tunnel) -> theme::Background {
    match bg {
        theme::Background::WarpedGrid {
            ground,
            minor,
            major,
            spacing_px,
            density,
            fold,
            twist,
            forward_drift,
            ribs,
            ..
        } => theme::Background::WarpedGrid {
            ground,
            minor,
            major,
            tunnel,
            spacing_px,
            density,
            fold,
            twist,
            forward_drift,
            ribs,
        },
        other => other,
    }
}

/// The room-owned axis every pre-roaming law in this file was written
/// against — dead centre, which makes the roaming BEND an exact no-op
/// (mixing two identical points) so these laws keep grading exactly what
/// they always graded. Roaming-specific laws live in `warp_roam.rs` and
/// supply their own axis via [`render_travel_axis`].
pub(super) const AXIS_ROOM: (f32, f32) = (0.5, 0.5);

/// Render one background pass at a real forward-travel phase, at the
/// room-owned rest axis.
#[allow(clippy::too_many_arguments)]
fn render(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    desc: BgDesc,
    w: u32,
    h: u32,
    col_left: f32,
    col_w: f32,
    phase: f32,
) -> Vec<[u8; 4]> {
    render_travel(
        device,
        queue,
        desc,
        w,
        h,
        col_left,
        col_w,
        warpgrid::forward_cells(phase, kite().forward_drift()),
    )
}

/// The same pass driven by EXPLICIT travel — the seam a lattice-periodicity
/// claim needs, because the phase resolver wraps at the loop boundary.
#[allow(clippy::too_many_arguments)]
fn render_travel(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    desc: BgDesc,
    w: u32,
    h: u32,
    col_left: f32,
    col_w: f32,
    warp_travel: f32,
) -> Vec<[u8; 4]> {
    render_travel_axis(
        device,
        queue,
        desc,
        w,
        h,
        col_left,
        col_w,
        warp_travel,
        AXIS_ROOM,
    )
}

/// [`render_travel`], with an explicit roaming-axis fraction instead of the
/// room-owned rest point.
#[allow(clippy::too_many_arguments)]
pub(super) fn render_travel_axis(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    desc: BgDesc,
    w: u32,
    h: u32,
    col_left: f32,
    col_w: f32,
    warp_travel: f32,
    warp_axis: (f32, f32),
) -> Vec<[u8; 4]> {
    render_travel_axis_dpi(
        device,
        queue,
        desc,
        w,
        h,
        col_left,
        col_w,
        warp_travel,
        warp_axis,
        1.0,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn render_travel_axis_dpi(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    desc: BgDesc,
    w: u32,
    h: u32,
    col_left: f32,
    col_w: f32,
    warp_travel: f32,
    warp_axis: (f32, f32),
    dpi: f32,
) -> Vec<[u8; 4]> {
    let mut bg = crate::background::BackgroundPipeline::new(device, super::dither::FMT, desc);
    bg.prepare(
        queue,
        w,
        h,
        col_left,
        col_w,
        crate::background::AmbientUpload {
            warp_travel,
            warp_axis,
            ..Default::default()
        },
        dpi,
    );
    let (texture, tview) = super::dither::offscreen(device, w, h);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("awl warped-grid encoder"),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("awl warped-grid pass"),
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
    super::dither::read_pixels(device, queue, &texture, w, h)
}

/// The DIFFERENTIAL field: per-pixel total-channel deviation between the ground
/// as authored and the same ground with its coverage zeroed. Isolates the grid
/// from the flat ground exactly, with no host colour mirror.
#[allow(clippy::too_many_arguments)]
pub(super) fn field(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    bg: theme::Background,
    w: u32,
    h: u32,
    col_left: f32,
    col_w: f32,
    phase: f32,
) -> Vec<i32> {
    let a = render(device, queue, bg_desc_for(bg), w, h, col_left, col_w, phase);
    let b = render(
        device,
        queue,
        bg_desc_for(with_density(bg, 0.0)),
        w,
        h,
        col_left,
        col_w,
        phase,
    );
    a.iter()
        .zip(b.iter())
        .map(|(p, q)| {
            (0..3)
                .map(|k| (p[k] as i32 - q[k] as i32).abs())
                .sum::<i32>()
        })
        .collect()
}

/// The two page margins at the canonical geometry.
fn canon_margins() -> [(u32, u32); 2] {
    margins(W, COL_LEFT, COL_W)
}

// ---------------------------------------------------------------------------
// DATA: roster, the inert default, and exact non-assigned-world identity.
// ---------------------------------------------------------------------------

/// KITE ALONE wears the warped grid, and every OTHER world's tunnel scalar is
/// EXACTLY the inert `0.0` — so the shader's `params.w` slot never changes shape
/// for a world that has no tunnel. Exhaustive over the enum (a new variant is a
/// compile error here) and over the roster.
#[test]
fn warped_grid_is_kites_alone_no_wildcard() {
    for t in theme::THEMES {
        let tunnel = match t.background {
            theme::Background::Gradient { .. } => None,
            theme::Background::Dots { .. } => None,
            theme::Background::Pinstripe { .. } => None,
            theme::Background::Stripes { .. } => None,
            theme::Background::Lava { .. } => None,
            theme::Background::Bands { .. } => None,
            theme::Background::Waves { .. } => None,
            theme::Background::Zigzag { .. } => None,
            theme::Background::Organic { .. } => None,
            theme::Background::Deckle { .. } => None,
            theme::Background::WarpedGrid { tunnel, .. } => Some(tunnel),
        };
        let want = (t.name == "Kite").then_some(theme::Tunnel::Fixed);
        assert_eq!(
            tunnel, want,
            "{}: deliberate warped-grid assignment",
            t.name
        );
        assert_eq!(
            t.background.is_warped_grid(),
            t.name == "Kite",
            "{}: is_warped_grid",
            t.name
        );
        let want_mode = if t.name == "Kite" {
            theme::Tunnel::Fixed.mode()
        } else {
            0.0
        };
        assert_eq!(
            t.background.tunnel_mode(),
            want_mode,
            "{}: tunnel_mode must be inert off the warped grid",
            t.name
        );
        // The shader id is the OTHER thing a new ground could disturb.
        assert_eq!(
            t.background.shader_id() == 10,
            t.name == "Kite",
            "{}: only the warped grid dispatches shader 10",
            t.name
        );
    }
    assert_eq!(theme::Tunnel::Fixed.mode(), 0.0);
    assert_ne!(theme::Tunnel::Fixed.mode(), theme::Tunnel::Reversed.mode());
}

/// NO OTHER WORLD'S UPLOAD CHANGED, proven over rendered BYTES rather than over
/// the descriptor's field list. If another ground read Kite's travel scalar,
/// its pixels would move between these two phases.
#[test]
fn no_other_worlds_ground_can_see_kites_travel() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    let mid = 150.0f32;
    assert_ne!(
        warpgrid::forward_cells(mid, kite().forward_drift()),
        warpgrid::forward_cells(warpgrid::FROZEN_PHASE, kite().forward_drift()),
        "the probe phase must actually differ from the settled one"
    );
    let mut checked = 0usize;
    for t in theme::THEMES {
        if t.background.is_warped_grid() {
            continue;
        }
        let d = bg_desc_for(t.background);
        assert_eq!(d.tunnel, 0.0, "{}: inert tunnel scalar", t.name);
        let a = render(
            &device,
            &queue,
            d,
            640,
            400,
            160.0,
            320.0,
            warpgrid::FROZEN_PHASE,
        );
        let b = render(&device, &queue, d, 640, 400, 160.0, 320.0, mid);
        assert_eq!(
            a, b,
            "{}: a non-warped ground must be byte-identical at every travel phase",
            t.name
        );
        checked += 1;
    }
    assert_eq!(
        checked,
        theme::THEMES.len() - 1,
        "every world but Kite must be checked"
    );
}

/// `density: 0.0` must collapse the field to its flat `ground` tone EXACTLY.
/// This is the precondition for every differential law in this file — without
/// it the oracle silently measures the ground's own dither too.
#[test]
fn zero_density_is_an_exact_flat_ground_reference() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    let flat = render(
        &device,
        &queue,
        bg_desc_for(with_density(kite(), 0.0)),
        W,
        H,
        COL_LEFT,
        COL_W,
        warpgrid::FROZEN_PHASE,
    );
    let want = kite().from().rgba_bytes();
    for (x0, x1) in canon_margins() {
        for y in (0..H).step_by(37) {
            for x in (x0..x1).step_by(29) {
                let px = flat[(y * W + x) as usize];
                assert_eq!(
                    [px[0], px[1], px[2]],
                    [want[0], want[1], want[2]],
                    "zeroed density must be the flat authored ground at ({x},{y})"
                );
            }
        }
    }
    // And the authored field is genuinely NOT flat (non-vacuity).
    let f = field(
        &device,
        &queue,
        kite(),
        W,
        H,
        COL_LEFT,
        COL_W,
        warpgrid::FROZEN_PHASE,
    );
    assert!(
        f.iter().filter(|v| **v > INK_FLOOR).count() > 10_000,
        "the authored field must carry real ink"
    );
}

// ---------------------------------------------------------------------------
// APPEARANCE: the page, the margins, and the composition.
// ---------------------------------------------------------------------------

/// The retired whole-page maximum compared unrelated depths and line
/// intersections. Its replacement samples the same projected major-section
/// family at known page and open-margin landmarks, proving that the veil is
/// present, materially quieter, and able to reject an unmasked-page mutation.
#[test]
fn the_writing_page_carries_a_quiet_projected_veil() {
    super::warp_projection::projected_page_veil_is_present_and_quiet();
}

fn sampled_phases() -> [f32; 5] {
    // Forward travel no longer wraps at a fixed loop length (the roaming
    // vanishing point retired it — see `warpgrid::forward_cells`'s own
    // doc), so these are just a spread of real elapsed seconds.
    [warpgrid::FROZEN_PHASE, 69.0, 134.0, 235.5, 329.0]
}

/// The retired arbitrary-sliver area floor is superseded by real projected
/// pixels in both margins across the supported viewports, DPIs, rest corners,
/// and transit. Its blank-margin mutation proves presence is load-bearing.
#[test]
fn both_supported_margins_carry_the_projected_surface() {
    super::warp_projection::projected_surface_marks_both_supported_margins();
}

/// THE LINE HIERARCHY IS TWO MEASURABLE RUNGS, every fifth line the strong one.
/// The population of marked pixels must split into a broad quiet band and a
/// distinctly stronger band; a single-rung field (hierarchy lost) has no gap.
#[test]
fn the_major_minor_hierarchy_reads_as_two_distinct_rungs() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    let f = field(
        &device,
        &queue,
        kite(),
        W,
        H,
        COL_LEFT,
        COL_W,
        warpgrid::FROZEN_PHASE,
    );
    let mut vals: Vec<i32> = f.iter().copied().filter(|v| *v > INK_FLOOR).collect();
    vals.sort_unstable();
    assert!(
        vals.len() > 10_000,
        "need a populated field, got {}",
        vals.len()
    );
    let q = |frac: f64| vals[((vals.len() - 1) as f64 * frac) as usize];
    let minor = q(0.35);
    let major = q(0.995);
    assert!(
        major >= minor * 3,
        "the strong rung must be unmistakably stronger than the quiet one \
         (minor p35 {minor}, major p99.5 {major})"
    );
    // Both rungs are genuinely POPULATED — a hierarchy with three major pixels
    // is not a hierarchy.
    let strong = vals.iter().filter(|v| **v >= major / 2).count();
    assert!(
        strong > 500,
        "the strong rung must be populated, got {strong} pixels"
    );
    let quiet = vals.iter().filter(|v| **v < major / 2).count();
    assert!(quiet > 5_000, "the quiet rung must dominate, got {quiet}");
}

/// The retired fixed-strip mean could miss a curved contour on one side of the
/// edge while measuring it on the other. Its replacement follows actual
/// projected major-section segments through both boundaries and rejects a
/// deliberately severed margin.
#[test]
fn projected_sections_continue_across_both_page_edges() {
    super::warp_projection::projected_major_sections_cross_both_page_edges();
}

/// NO HIGH-FREQUENCY ALIASING, swept over DPI, canvas and phase. A converging
/// lattice that reaches sub-pixel pitch turns into moire; the shader bounds its
/// own projected pitch (a SOFTENED RADIUS — ring pitch grows as
/// `ln2*(spacing+r)/k`, rail pitch as `pi*r/k`, so a floor under `r` is a floor
/// under both) and fades the minor rung out before the alias band.
///
/// THE SIGNATURE IS LOCAL SATURATION, not isolated pixels — and finding that out
/// is why this law is in its second shape. The first cut counted marked pixels
/// with no marked horizontal neighbour, and it went GREEN over a deliberate
/// removal of the radius floor: `warp_line` draws every line at a constant PIXEL
/// width, so an over-dense lattice does not scatter into speckle, it MERGES into
/// solid patches. A tile of margin that is almost entirely ink is a patch with no
/// resolvable structure left — exactly what shimmers on a Retina or WebGL2
/// rasteriser as the field moves under it — so the bound is on the densest tile
/// the field produces anywhere.
#[test]
fn the_lattice_never_saturates_a_patch_of_margin_at_any_scale() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    // The tile is a few line pitches across: big enough that a legitimately
    // tight-but-resolvable region still shows its gaps, small enough that a
    // genuinely saturated patch is not diluted by open margin around it.
    const TILE: u32 = 14;
    // Two ~2px lines crossing a 14px tile plus their antialiased skirts is well
    // under half; a tile past this has lost its structure.
    const MAX_TILE_COVERAGE: f64 = 0.70;
    // 1x, 2x Retina, two zoom-like scalings — AND the geometry that actually
    // exercises the radius floor, including ultrawide rooms with a narrow page
    // where a large share of the converging field is exposed.
    for (ww, wh, col_left, col_w) in [
        (W, H, COL_LEFT, COL_W),
        (W * 2, H * 2, COL_LEFT * 2.0, COL_W * 2.0),
        (1280, 800, 260.0, 760.0),
        (2560, 1440, 520.0, 1520.0),
        (2560, 1000, 930.0, 700.0), // ultrawide + narrow measure: the VP enters a margin
        (3440, 1200, 1370.0, 700.0), // and further still
    ] {
        for phase in sampled_phases() {
            let f = field(&device, &queue, kite(), ww, wh, col_left, col_w, phase);
            let mut worst = 0.0f64;
            let mut worst_at = (0u32, 0u32);
            let mut marked = 0usize;
            for (x0, x1) in margins(ww, col_left, col_w) {
                if x1.saturating_sub(x0) < TILE {
                    continue;
                }
                for ty in (0..wh.saturating_sub(TILE)).step_by(TILE as usize) {
                    for tx in (x0..x1.saturating_sub(TILE)).step_by(TILE as usize) {
                        let mut ink = 0usize;
                        for y in ty..ty + TILE {
                            for x in tx..tx + TILE {
                                if f[(y * ww + x) as usize] > INK_FLOOR {
                                    ink += 1;
                                }
                            }
                        }
                        marked += ink;
                        let cov = ink as f64 / (TILE * TILE) as f64;
                        if cov > worst {
                            worst = cov;
                            worst_at = (tx, ty);
                        }
                    }
                }
            }
            assert!(marked > 1_000, "{ww}x{wh} @{phase}: need a populated field");
            assert!(
                worst <= MAX_TILE_COVERAGE,
                "{ww}x{wh} @{phase}: a {TILE}x{TILE} tile at {worst_at:?} is {:.1}% ink — \
                 the lattice has packed past resolvable and will shimmer",
                100.0 * worst
            );
        }
    }
}

/// A NARROW margin SIMPLIFIES rather than miniaturising: the minor rung retires
/// and the major scaffold carries the world alone. DESIGN.md §8's contraction
/// order, and the item's "never squeeze a tiny illegible tunnel behind the page".
#[test]
fn a_narrow_margin_simplifies_to_the_major_scaffold_alone() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    let wide = field(
        &device,
        &queue,
        kite(),
        W,
        H,
        340.0,
        900.0,
        warpgrid::FROZEN_PHASE,
    );
    // A 70px margin — narrow, but wider than the edge-quiet band.
    let narrow = field(
        &device,
        &queue,
        kite(),
        W,
        H,
        70.0,
        1460.0,
        warpgrid::FROZEN_PHASE,
    );
    let quiet_share = |f: &[i32], x0: u32, x1: u32| {
        let mut quiet = 0usize;
        let mut all = 0usize;
        for y in 0..H {
            for x in x0..x1 {
                let v = f[(y * W + x) as usize];
                if v > INK_FLOOR {
                    all += 1;
                    if v < 60 {
                        quiet += 1;
                    }
                }
            }
        }
        (quiet as f64 / all.max(1) as f64, all)
    };
    let (wide_quiet, wide_n) = quiet_share(&wide, 0, 340);
    let (narrow_quiet, narrow_n) = quiet_share(&narrow, 0, 70);
    assert!(wide_n > 1_000 && narrow_n > 200, "both margins must draw");
    assert!(
        narrow_quiet < wide_quiet,
        "a narrow margin must shed its QUIET rung and read as the major scaffold: \
         quiet share {narrow_quiet:.3} narrow vs {wide_quiet:.3} wide"
    );
}

/// FIGURE/GROUND: the field lives inside the world's own ground value band, so
/// the margins read as recessive ground at every pose, and the prose ink clears
/// the field's strongest pixel by a wide contrast margin.
#[test]
fn the_field_stays_inside_the_grounds_value_band_and_the_ink_clears_it() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    fn rel_lum(p: [u8; 4]) -> f64 {
        let ch = |u: u8| {
            let s = u as f64 / 255.0;
            if s <= 0.03928 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * ch(p[0]) + 0.7152 * ch(p[1]) + 0.0722 * ch(p[2])
    }
    let th = theme::KITE;
    // The field mixes ground -> minor -> major, so `mix()` bounds it by its
    // endpoints; the darkest reachable pixel is the authored `major`.
    let mut darkest = [255u8; 4];
    for phase in sampled_phases() {
        let px = render(
            &device,
            &queue,
            bg_desc_for(kite()),
            W,
            H,
            COL_LEFT,
            COL_W,
            phase,
        );
        for (x0, x1) in canon_margins() {
            for y in 0..H {
                for x in x0..x1 {
                    let p = px[(y * W + x) as usize];
                    if rel_lum(p) < rel_lum(darkest) {
                        darkest = p;
                    }
                }
            }
        }
    }
    // On a LIGHT world the ground band runs base_100 (lightest) down to the
    // darkest authored ground rung; the field may not go darker than its own
    // authored major tone.
    let major = kite().to();
    assert!(
        rel_lum(darkest) >= rel_lum([major.r, major.g, major.b, 255]) - 0.002,
        "the darkest field pixel {darkest:?} went past the authored major tone \
         {major:?} — the ground must stay inside its own value band"
    );
    // The prose ink is unmistakably the figure against that worst-case pixel.
    let ink = th.base_content;
    let cr = {
        let (a, b) = (rel_lum([ink.r, ink.g, ink.b, 255]), rel_lum(darkest));
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    };
    assert!(
        cr >= 3.0,
        "prose ink must clear the field's strongest pixel (contrast {cr:.2}:1)"
    );
}

// ---------------------------------------------------------------------------
// MOTION: determinism, the invisible wrap, and the composed still.
// ---------------------------------------------------------------------------

/// Recycling a depth slot must preserve each moving section's emphasis.
/// Probe both travel directions at the actual slot boundaries, where a
/// slot-based major/minor classification would jump to the neighboring ring.
#[test]
fn projected_linear_z_travel_is_continuous_at_real_pixels() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    let step_z = (10.8 - 0.72) / 58.0;
    for dpi in [1.0, 2.0] {
        for boundary in [-6.0, -1.0, 1.0, 2.0, 5.0, 6.0] {
            let draw = |offset: f32| {
                render_travel_axis_dpi(
                    &device,
                    &queue,
                    bg_desc_for(kite()),
                    (W as f32 * dpi) as u32,
                    (H as f32 * dpi) as u32,
                    COL_LEFT * dpi,
                    COL_W * dpi,
                    (boundary * step_z + offset) * warpgrid::FORWARD_SPEED_SCALE / 1.7,
                    (0.80, 0.24),
                    dpi,
                )
            };
            let before = draw(-0.00001);
            let after = draw(0.00001);
            let changed = before
                .iter()
                .zip(&after)
                .filter(|(p, q)| (0..3).any(|k| (p[k] as i32 - q[k] as i32).abs() > 12))
                .count();
            let fraction = changed as f32 / before.len() as f32;
            assert!(
                fraction < 0.001,
                "dpi={dpi} boundary={boundary}: emphasis jumped on {fraction:.5} of pixels"
            );
        }
    }
}

/// EVERY CALM PATH RESOLVES TO THE ONE COMPOSED STILL, driven through the
/// REAL resolver (`warpgrid::resolved_render`) rather than a hand-picked
/// axis/travel pair — this is the law that would catch a calm resolution
/// that quietly reads the stored phase or the seed after all. Reduce
/// Motion and `ambient_motion` off both resolve to the SAME
/// `crate::warpgrid::calm_requested` axis, so testing the `calm: true`
/// argument covers both.
#[test]
fn every_calm_path_renders_the_one_composed_still() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    let profile = warpgrid::WarpProfile::from_background(&kite()).expect("Kite is WarpedGrid");
    let resolve = |stored: f32, seed: u64| {
        let mut cursor = warpgrid::RoamCursor::start();
        warpgrid::resolved_render(&mut cursor, &profile, stored, seed, true)
    };
    let render_pose = |r: &warpgrid::WarpRender| {
        render_travel_axis(
            &device,
            &queue,
            bg_desc_for(kite()),
            W,
            H,
            COL_LEFT,
            COL_W,
            r.travel_cells,
            r.axis_frac,
        )
    };
    let still = render_pose(&resolve(0.0, warpgrid::DEFAULT_SEED));
    // Every stored phase and every seed, calm ALWAYS TRUE: the same composed
    // still — the accessibility promise and byte-determinism are one fact.
    for stored in [
        0.0f32,
        91.3,
        500.7,
        warpgrid::wrap_seconds(profile.forward_drift) * 3.2,
    ] {
        for seed in [0u64, 7, 0xDEAD_BEEF] {
            let r = resolve(stored, seed);
            assert!(r.calm, "resolved_render must report calm:true when asked");
            let frame = render_pose(&r);
            assert_eq!(
                frame, still,
                "calm must render the composed still whatever the clock/seed hold \
                 (stored={stored} seed={seed})"
            );
        }
    }
    // The SAME stored phase, calm OFF: must NOT match the still (mid-transit
    // stays mid-transit) — the discriminator that proves calm is a genuine
    // resolution switch, not a stored-phase coincidence.
    let mid_transit = warpgrid::roam::DWELL_SECONDS + warpgrid::roam::TRANSIT_SECONDS * 0.5;
    let live = {
        let mut cursor = warpgrid::RoamCursor::start();
        warpgrid::resolved_render(&mut cursor, &profile, mid_transit, 7, false)
    };
    assert!(!live.calm);
    assert!(!live.holding, "must be mid-transit at this phase");
    // Kite arms the shared ambient tick and its freeze conditions, exactly like
    // every other moving ground — inherited, not re-implemented.
    assert!(theme::KITE.has_ambient_motion(), "Kite is an ambient world");
    assert!(theme::KITE.has_ambient_tick(), "Kite arms the shared tick");
    for (ambient_on, reduced, focused, paused) in [
        (false, false, true, false),
        (true, true, true, false),
        (true, false, false, false),
        (true, false, true, true),
    ] {
        assert!(
            !crate::lava::lava_should_tick(true, ambient_on, reduced, focused, paused),
            "ambient_on={ambient_on} reduced={reduced} focused={focused} paused={paused} \
             must schedule zero frames"
        );
    }
    assert!(crate::lava::lava_should_tick(
        true, true, false, true, false
    ));
}

// ---------------------------------------------------------------------------
// STRUCTURE: the WGSL tripwire.
// ---------------------------------------------------------------------------

/// The shader keeps one fixed framing, direct bounded projected geometry, the
/// forward sign, and no dormant inverse/steering machinery.
#[test]
fn the_warped_grid_wgsl_holds_its_repairs_and_names_no_world() {
    let wgsl = include_str!("../../../shaders/background.wgsl");
    let want = format!("const WARP_MAJOR_EVERY: f32 = {:?};", warpgrid::MAJOR_EVERY);
    assert!(
        wgsl.contains(&want),
        "shaders/background.wgsl must declare `{want}` — the host's own major \
         modulus and the GPU's have drifted, and the seamless-wrap arithmetic \
         depends on them being the same number"
    );
    for expr in [
        "@vertex\nfn vs_tunnel(",
        "const WARP_RING_SEGMENTS: u32 = 128u;",
        "const WARP_RING_SLOTS: u32 = 65u;",
        "const WARP_RAIL_SEGMENTS: u32 = 92u;",
        "const WARP_RAIL_SLOTS: u32 = 24u;",
        "let rings = clamp(round(g.warp_shape.z), 1.0, f32(WARP_RING_SLOTS - 1u));",
        "let radius = warp_radius(theta, world_z, fold, twist);",
        "warp_path(world_z)",
        "warp_roll(world_z, spin)",
        "fn warp_depth_alpha(z: f32) -> f32 {",
        "clamp((0.90 - z * 0.060) / 0.58, 0.35, 1.0)",
        "if (in.family == 1u || in.major == 0u) {",
        "mask *= margin_only;",
    ] {
        assert!(
            wgsl.contains(expr),
            "shaders/background.wgsl must hold `{expr}`"
        );
    }
    for gone in [
        "WARP_PULL_FRAC",
        "WARP_BEND_GAIN",
        "WARP_SOLVE_STEPS",
        "warp_surface_coord",
        "warped_grid_inverse_rgba",
        "per_margin",
        "g.pose",
        // THE PER-MARGIN WINDOW PLACEMENT AND THE INSET THAT SIZED IT. These ARE
        // the two tunnels: an axis owner taking `on_right` could only ever hand
        // each margin its own vanishing point. Named here so the shape cannot be
        // reintroduced by hand.
        "warp_window_axis",
        "WARP_WINDOW_INSET",
    ] {
        assert!(
            !wgsl.contains(gone),
            "obsolete warped-grid machinery remains: `{gone}`"
        );
    }
    // No world name in the branch's CODE (prose comments are fine).
    let start = wgsl
        .find("// --- 10: WARPED GRID")
        .expect("the warped-grid section must be findable");
    let end = start
        + wgsl[start..]
            .find("// BANDING KILL")
            .expect("the section must end at its neighbour");
    let code: String = wgsl[start..end]
        .lines()
        .map(|l| l.split("//").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n");
    for t in theme::THEMES {
        assert!(
            !code.contains(t.name),
            "the warped-grid shader CODE names the world {:?} — grounds are data",
            t.name
        );
    }
}
