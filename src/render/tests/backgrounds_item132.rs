//! KITE's WARPED-GRID laws (item 132) — the ground's own test module.
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
//! world with `density: 0.0` (item 86's `mark_field` oracle), so the flat ground
//! cancels exactly and what remains is the field alone. `density: 0.0` collapsing
//! this ground to its flat `ground` tone EXACTLY is what makes that possible, and
//! it is asserted directly.
//!
//! Nothing here trusts the sidecar for how the field LOOKS — CAPTURE.md's
//! "state oracle, not an appearance oracle" tripwire. The sidecar's steering pose
//! is used only to say WHICH pose was asked for.

use super::backgrounds_item69::{bg_desc_for, headless_dq};
use super::backgrounds_item89::{SWEEP, margins};
use crate::background::BgDesc;
use crate::theme;
use crate::warpgrid;

/// The canonical wide-Retina-ish scan surface: a real 1600x1000 gallery canvas
/// with the app's own adaptive column at measure 66 (the geometry
/// `scripts/capture-worlds.sh` shoots every world at).
const W: u32 = 1600;
const H: u32 = 1000;
const COL_LEFT: f32 = 324.0;
const COL_W: f32 = 950.0;

/// Total-channel deviation below this is 8-bit quantization, not a mark.
const INK_FLOOR: i32 = 3;

fn kite() -> theme::Background {
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
            curvature,
            ..
        } => theme::Background::WarpedGrid {
            ground,
            minor,
            major,
            tunnel,
            spacing_px,
            curvature,
            density,
        },
        other => other,
    }
}

fn with_tunnel(bg: theme::Background, tunnel: theme::Tunnel) -> theme::Background {
    match bg {
        theme::Background::WarpedGrid {
            ground,
            minor,
            major,
            spacing_px,
            curvature,
            density,
            ..
        } => theme::Background::WarpedGrid {
            ground,
            minor,
            major,
            tunnel,
            spacing_px,
            curvature,
            density,
        },
        other => other,
    }
}

/// Render one background pass at a real steering pose. The pose reaches the
/// shader through the SAME `Globals.pose` row production uses, resolved by the
/// SAME `warpgrid::route_pose` owner — the test never invents a pose shape.
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
    let p = warpgrid::route_pose(phase);
    render_pose(
        device,
        queue,
        desc,
        w,
        h,
        col_left,
        col_w,
        [p.yaw, p.pitch, p.forward_cells],
    )
}

/// The same pass driven by an EXPLICIT pose — the seam a lattice-periodicity
/// claim needs, because `route_pose` wraps its phase and so cannot express "one
/// whole loop of travel further on".
#[allow(clippy::too_many_arguments)]
fn render_pose(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    desc: BgDesc,
    w: u32,
    h: u32,
    col_left: f32,
    col_w: f32,
    pose: [f32; 3],
) -> Vec<[u8; 4]> {
    let mut bg = crate::background::BackgroundPipeline::new(device, super::dither::FMT, desc);
    bg.prepare(
        queue,
        w,
        h,
        col_left,
        col_w,
        crate::background::AmbientUpload {
            pose,
            ..Default::default()
        },
        1.0,
    );
    let (texture, tview) = super::dither::offscreen(device, w, h);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("awl item132-warp encoder"),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("awl item132-warp pass"),
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
fn field(
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

/// Marked pixels inside `[x0, x1)` over the whole height.
fn ink_in(f: &[i32], w: u32, h: u32, x0: u32, x1: u32) -> usize {
    (0..h)
        .flat_map(|y| (x0..x1).map(move |x| (y, x)))
        .filter(|&(y, x)| f[(y * w + x) as usize] > INK_FLOOR)
        .count()
}

/// How many times a horizontal scan at `y` crosses INTO a mark inside
/// `[x0, x1)` — the field's projected line count along that scanline, read with
/// a Schmitt trigger so an antialiased ramp counts once.
fn crossings(f: &[i32], w: u32, y: u32, x0: u32, x1: u32, peak: i32) -> usize {
    let mut n = 0;
    let mut inside = false;
    for x in x0..x1 {
        let v = f[(y * w + x) as usize];
        if !inside && v >= peak {
            inside = true;
            n += 1;
        } else if inside && v <= INK_FLOOR {
            inside = false;
        }
    }
    n
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
            theme::Background::Starfield { .. } => None,
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
        let want = (t.name == "Kite").then_some(theme::Tunnel::Shared);
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
            theme::Tunnel::Shared.mode()
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
    assert_eq!(theme::Tunnel::Shared.mode(), 0.0);
    assert_eq!(theme::Tunnel::PerMargin.mode(), 1.0);
    // The MUTATION arm ships and is reachable, so the item-194 coherence
    // laws can be proven capable of failing.
    assert_ne!(
        theme::Tunnel::Shared.mode(),
        theme::Tunnel::PerMargin.mode(),
        "the two profiles must be distinguishable to the shader"
    );
}

/// NO OTHER WORLD'S UPLOAD CHANGED, proven over rendered BYTES rather than over
/// the descriptor's field list. The route's steering pose arrives in a new
/// `Globals` row; if any other ground could read it, that ground's pixels would
/// move when the pose does. Every non-Kite world is rendered at the frozen pose
/// and at a hard mid-route pose and required to be byte-identical.
#[test]
fn no_other_worlds_ground_can_see_the_route_pose() {
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    let mid = warpgrid::ROUTE_LEG_SECONDS * 1.5;
    assert_ne!(
        warpgrid::route_pose(mid),
        warpgrid::route_pose(warpgrid::FROZEN_PHASE),
        "the probe pose must actually differ from the settled one"
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
            "{}: a non-warped ground must be byte-identical at every route pose",
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

/// THE WRITING PAGE STAYS FLAT AND OPAQUE — not one pixel of grid inside the
/// column, at every swept geometry AND every named pose. The item's own first
/// constraint, and the one a "distort the curvature outward" mechanism is most
/// able to break.
#[test]
fn the_grid_never_enters_the_writing_page_at_any_geometry_or_pose() {
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    let _g = crate::testlock::serial();
    let mut cells = 0usize;
    let restore = crate::page::measure();
    for (ww, wh, measure) in SWEEP {
        let Some(p) = super::headless_dqp(ww as f32, wh as f32) else {
            return;
        };
        let (_d, _q, mut pipe) = p;
        crate::page::set_measure(measure);
        pipe.set_view(&super::view("hello", 0, 0));
        let col_left = pipe.column_left();
        let col_w = pipe.column_width();
        for phase in named_phases() {
            let f = field(&device, &queue, kite(), ww, wh, col_left, col_w, phase);
            let x0 = col_left.max(0.0) as u32;
            let x1 = ((col_left + col_w).ceil() as u32).min(ww);
            let inside = ink_in(&f, ww, wh, x0, x1);
            assert_eq!(
                inside, 0,
                "{ww}x{wh}/m{measure} @phase {phase}: {inside} grid pixels inside the \
                 writing column [{x0},{x1}) — the page must stay flat and opaque"
            );
            cells += 1;
        }
    }
    crate::page::set_measure(restore);
    assert!(
        cells >= 60,
        "the sweep must actually grade cells, got {cells}"
    );
}

fn named_phases() -> [f32; 5] {
    let hold = |leg: f32| warpgrid::ROUTE_LEG_SECONDS * (leg + warpgrid::ROUTE_HOLD_FRAC * 0.5);
    [
        warpgrid::FROZEN_PHASE,
        hold(1.0), // left
        hold(2.0), // climb
        hold(4.0), // right
        hold(5.0), // descent
    ]
}

/// BOTH MARGINS carry a real field at every swept geometry — the composition is
/// two slices, never one live margin and one blank one.
#[test]
fn both_margins_carry_a_real_field_at_every_swept_geometry() {
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    let _g = crate::testlock::serial();
    let restore = crate::page::measure();
    for (ww, wh, measure) in SWEEP {
        let Some((_d, _q, mut pipe)) = super::headless_dqp(ww as f32, wh as f32) else {
            return;
        };
        crate::page::set_measure(measure);
        pipe.set_view(&super::view("hello", 0, 0));
        let (col_left, col_w) = (pipe.column_left(), pipe.column_width());
        let f = field(
            &device,
            &queue,
            kite(),
            ww,
            wh,
            col_left,
            col_w,
            warpgrid::FROZEN_PHASE,
        );
        for (i, (x0, x1)) in margins(ww, col_left, col_w).into_iter().enumerate() {
            if x1.saturating_sub(x0) < 24 {
                continue; // a sliver narrower than the edge-quiet band is allowed to hold nothing
            }
            let area = ((x1 - x0) * wh) as f64;
            let ink = ink_in(&f, ww, wh, x0, x1) as f64;
            assert!(
                ink / area >= 0.02,
                "{ww}x{wh}/m{measure} margin {i} [{x0},{x1}): only {:.3}% of it carries \
                 field ink — a margin wide enough to draw must read as a slice of the tunnel",
                100.0 * ink / area
            );
        }
    }
    crate::page::set_measure(restore);
}

/// ONE JOURNEY, TWO SLICES: during a LEFT turn the left margin's field
/// COMPRESSES and the right margin's OPENS, and a RIGHT turn mirrors it exactly.
///
/// "Compresses" is measured as the slice's own INK DENSITY — the share of its
/// pixels the lattice occupies. That is the reading the eye actually makes, and
/// it is the one the geometry guarantees: ring pitch grows as `ln2*(spacing+r)/k`
/// and rail pitch as `pi*r/k`, so the margin whose radii shrink draws tighter
/// lines and the margin whose radii grow draws looser ones, from ONE shared
/// vanishing point.
///
/// A COUNT of lines crossed along one scanline is deliberately NOT the metric,
/// and finding that out is the reason this law exists in this shape: the number
/// of lattice periods a margin's own x-extent spans FALLS on both sides during a
/// turn (measured 8 -> 6 left and 8 -> 5 right on the left bend), because the
/// margin maps to a narrower range of the log-radial coordinate even as the
/// lines inside it get tighter. A count law would have read "both margins
/// opened" and been green on a field that visibly does the right thing.
///
/// This is the law the item's "rather than two unrelated animations" clause
/// names, and `Tunnel::Centred` is what proves it can fail (see the mutation
/// law).
#[test]
fn a_turn_compresses_one_margin_and_opens_the_other_coherently() {
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    let density = |phase: f32| {
        let f = field(&device, &queue, kite(), W, H, COL_LEFT, COL_W, phase);
        canon_margins().map(|(x0, x1)| ink_in(&f, W, H, x0, x1) as f64 / ((x1 - x0) * H) as f64)
    };
    let hold = |leg: f32| warpgrid::ROUTE_LEG_SECONDS * (leg + warpgrid::ROUTE_HOLD_FRAC * 0.5);
    let straight = density(warpgrid::FROZEN_PHASE);
    let left = density(hold(1.0)); // the long LEFT bend
    let right = density(hold(4.0)); // the long RIGHT bend

    // Straight travel is symmetric by construction — the common reference.
    assert!(
        (straight[0] - straight[1]).abs() < 0.02,
        "straight travel must read symmetrically ({:.4} vs {:.4})",
        straight[0],
        straight[1]
    );
    assert!(
        straight[0] > 0.03,
        "the reference pose must carry a real field ({:.4})",
        straight[0]
    );
    assert!(
        left[0] > straight[0] && left[1] < straight[1],
        "a LEFT bend must COMPRESS the left slice and OPEN the right: \
         left {:.4} -> {:.4}, right {:.4} -> {:.4}",
        straight[0],
        left[0],
        straight[1],
        left[1]
    );
    assert!(
        right[1] > straight[1] && right[0] < straight[0],
        "a RIGHT bend must mirror it: right {:.4} -> {:.4}, left {:.4} -> {:.4}",
        straight[1],
        right[1],
        straight[0],
        right[0]
    );
    // The two bends lean OPPOSITE ways — not merely "both busier".
    assert!(
        (left[0] - left[1]).signum() == -(right[0] - right[1]).signum(),
        "the bends must lean opposite ways (left bend {:.4}/{:.4}, right bend {:.4}/{:.4})",
        left[0],
        left[1],
        right[0],
        right[1]
    );
}

/// CLIMB and DESCENT produce COHERENT OPPOSING VERTICAL FLOW: the field's
/// weight moves to opposite halves of the canvas, and it does so the SAME way in
/// BOTH margins (one camera, not two).
#[test]
fn climb_and_descent_shift_the_field_in_opposite_halves_of_both_margins() {
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    let hold = |leg: f32| warpgrid::ROUTE_LEG_SECONDS * (leg + warpgrid::ROUTE_HOLD_FRAC * 0.5);
    // Ink balance = (top half ink - bottom half ink) / total, per margin.
    let balance = |phase: f32| {
        let f = field(&device, &queue, kite(), W, H, COL_LEFT, COL_W, phase);
        canon_margins().map(|(x0, x1)| {
            let mut top = 0i64;
            let mut bot = 0i64;
            for y in 0..H {
                for x in x0..x1 {
                    if f[(y * W + x) as usize] > INK_FLOOR {
                        if y < H / 2 {
                            top += 1;
                        } else {
                            bot += 1;
                        }
                    }
                }
            }
            (top - bot) as f64 / (top + bot).max(1) as f64
        })
    };
    let up = balance(hold(2.0)); // climb
    let down = balance(hold(5.0)); // descent
    for i in 0..2 {
        assert!(
            up[i] * down[i] < 0.0,
            "margin {i}: climb and descent must push the field into OPPOSITE halves \
             (climb balance {:.3}, descent {:.3})",
            up[i],
            down[i]
        );
        assert!(
            up[i].abs() > 0.02 && down[i].abs() > 0.02,
            "margin {i}: the vertical shift must be measurable (climb {:.3}, descent {:.3})",
            up[i],
            down[i]
        );
    }
    // COHERENCE: both margins lean the SAME way on a given climb/descent — one
    // camera. Two independent animations would have no reason to agree.
    assert!(
        up[0] * up[1] > 0.0,
        "a climb must move both margins the same way ({:.3} vs {:.3})",
        up[0],
        up[1]
    );
    assert!(
        down[0] * down[1] > 0.0,
        "a descent must move both margins the same way ({:.3} vs {:.3})",
        down[0],
        down[1]
    );
}

/// THE LINE HIERARCHY IS TWO MEASURABLE RUNGS, every fifth line the strong one.
/// The population of marked pixels must split into a broad quiet band and a
/// distinctly stronger band; a single-rung field (hierarchy lost) has no gap.
#[test]
fn the_major_minor_hierarchy_reads_as_two_distinct_rungs() {
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

/// THE FIELD QUIETS BESIDE THE PAGE. Nothing may compete with prose at the
/// boundary the eye reads across, so the band immediately outside the column
/// carries far less ink than the open margin further out.
#[test]
fn the_field_fades_toward_the_page_edge() {
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
    let col_right = (COL_LEFT + COL_W).ceil() as u32;
    let mean = |x0: u32, x1: u32| {
        let mut sum = 0f64;
        for y in 0..H {
            for x in x0..x1 {
                sum += f[(y * W + x) as usize] as f64;
            }
        }
        sum / ((x1 - x0) * H) as f64
    };
    for (label, near, far) in [
        (
            "left",
            mean(COL_LEFT as u32 - 8, COL_LEFT as u32),
            mean(COL_LEFT as u32 - 90, COL_LEFT as u32 - 50),
        ),
        (
            "right",
            mean(col_right, col_right + 8),
            mean(col_right + 50, col_right + 90),
        ),
    ] {
        assert!(
            near * 4.0 < far,
            "{label} page edge: the 8px beside the page carries mean {near:.2} against \
             {far:.2} out in the open margin — the field must recede at the edge"
        );
    }
}

/// NO HIGH-FREQUENCY ALIASING, swept over DPI, canvas and pose. A converging
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
    // exercises the radius floor. THE AXIS THE FIRST CUT MISSED: on a WIDE
    // canvas at a NARROW measure the page column is small relative to the
    // window, so a committed bend swings the shared vanishing point PAST the
    // page edge and INTO a margin. The proportional distance floor scales with
    // distance from that point, so it offers nothing there — only the softened
    // radius keeps the lattice resolvable. Every other swept geometry keeps the
    // vanishing point behind the page, where the proportional floor alone is
    // already enough, which is exactly why they all stayed green over a
    // deliberate removal of the radius floor.
    for (ww, wh, col_left, col_w) in [
        (W, H, COL_LEFT, COL_W),
        (W * 2, H * 2, COL_LEFT * 2.0, COL_W * 2.0),
        (1280, 800, 260.0, 760.0),
        (2560, 1440, 520.0, 1520.0),
        (2560, 1000, 930.0, 700.0), // ultrawide + narrow measure: the VP enters a margin
        (3440, 1200, 1370.0, 700.0), // and further still
    ] {
        for phase in named_phases() {
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
    for phase in named_phases() {
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

/// THE ROUTE'S REPEAT IS INVISIBLE IN REAL PIXELS, and the claim is made where
/// it can actually fail.
///
/// A first cut compared the frame at `phase == ROUTE_LOOP_SECONDS` with the one
/// at `phase == 0`. That is VACUOUS: `route_pose` wraps its input, so the two
/// calls are literally the same call, and the law stayed green over
/// `FORWARD_CELLS_PER_LOOP: 64` — the very value whose wrap rotates which lines
/// are major. The load-bearing statement is about the POSE, not the phase: a
/// whole loop of forward travel must land the field back on its own lattice AND
/// its own line hierarchy, which happens if and only if the travel is a multiple
/// of the major modulus.
#[test]
fn a_whole_loop_of_forward_travel_is_byte_identical_at_real_pixels() {
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    let at = |forward: f32| {
        render_pose(
            &device,
            &queue,
            bg_desc_for(kite()),
            W,
            H,
            COL_LEFT,
            COL_W,
            [0.0, 0.0, forward],
        )
    };
    let start = at(0.0);
    // The comparison is a per-pixel BOUND, not byte equality. The ring
    // coordinate is `depth*k - forward`, so a whole loop of travel is an exact
    // integer shift of a lattice whose period divides it — the lattice and the
    // hierarchy are mathematically unmoved — but `fwidth` of a coordinate offset
    // by 65 loses a few f32 bits, so the antialiased edge of a line can land one
    // 8-bit step away. One step is not a visible seam; a shifted lattice is, and
    // the two are orders of magnitude apart (measured 2 against 96 below).
    let worst = |a: &[[u8; 4]], b: &[[u8; 4]]| {
        a.iter()
            .zip(b.iter())
            .map(|(p, q)| {
                (0..3)
                    .map(|k| (p[k] as i32 - q[k] as i32).abs())
                    .max()
                    .unwrap_or(0)
            })
            .max()
            .unwrap_or(0)
    };
    let loop_delta = worst(&at(warpgrid::FORWARD_CELLS_PER_LOOP), &start);
    assert!(
        loop_delta <= 3,
        "a whole loop of forward travel ({} cells) must land the field back on its own \
         lattice and hierarchy — worst channel delta {loop_delta}, which is a moved line, \
         not rounding: the several-minute repeat would carry a visible one-cell jump",
        warpgrid::FORWARD_CELLS_PER_LOOP
    );
    // NON-VACUITY, both ways: a travel that is a whole number of MINOR cells but
    // not of MAJOR ones rotates the hierarchy, and a fractional one moves the
    // lattice itself. Both must be far outside the rounding bound.
    let off_by_one = worst(&at(warpgrid::FORWARD_CELLS_PER_LOOP - 1.0), &start);
    assert!(
        off_by_one > 20,
        "one cell short of a loop must rotate the hierarchy visibly (delta {off_by_one})"
    );
    let fractional = worst(&at(0.4), &start);
    assert!(
        fractional > 20,
        "a fractional travel must move the lattice visibly (delta {fractional})"
    );
    // And the phase-level wrap agrees, which is what the App actually drives.
    assert_eq!(
        render(
            &device,
            &queue,
            bg_desc_for(kite()),
            W,
            H,
            COL_LEFT,
            COL_W,
            0.0
        ),
        render(
            &device,
            &queue,
            bg_desc_for(kite()),
            W,
            H,
            COL_LEFT,
            COL_W,
            warpgrid::ROUTE_LOOP_SECONDS
        ),
        "the App-driven phase wrap must be exactly periodic"
    );
    assert_ne!(
        render(
            &device,
            &queue,
            bg_desc_for(kite()),
            W,
            H,
            COL_LEFT,
            COL_W,
            warpgrid::ROUTE_LEG_SECONDS * 1.5
        ),
        start,
        "the route must actually move the field"
    );
}

/// EVERY FREEZE PATH RESOLVES TO THE ONE COMPOSED STILL, and the headless
/// capture shares it. Reduce Motion, `ambient_motion` off (which hard-freezes
/// the accumulator) and a capture that never ticks the clock all render the same
/// frame — the accessibility promise and byte-determinism are one fact.
#[test]
fn every_freeze_path_renders_the_one_composed_still() {
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    let still = render(
        &device,
        &queue,
        bg_desc_for(kite()),
        W,
        H,
        COL_LEFT,
        COL_W,
        warpgrid::FROZEN_PHASE,
    );
    // Reduce Motion pins the phase whatever the accumulator holds.
    for stored in [0.0f32, 91.3, warpgrid::ROUTE_LOOP_SECONDS * 0.77] {
        let phase = warpgrid::phase_for(stored, true, None);
        let frame = render(
            &device,
            &queue,
            bg_desc_for(kite()),
            W,
            H,
            COL_LEFT,
            COL_W,
            phase,
        );
        assert_eq!(
            frame, still,
            "Reduce Motion must render the composed still whatever the clock holds ({stored})"
        );
    }
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

/// THE SHADER'S STRUCTURAL REPAIRS ARE PINNED, and it names no world.
///
/// Every derived bound above rests on two expressions the shader must keep: a
/// PROPORTIONAL floor under the pulled distance, and a SOFTENED radius. The
/// first cut's constant floor is pinned as ABSENT, so reverting to it fails
/// here as well as in the aliasing law. The every-fifth-line modulus is held in
/// lockstep with the Rust constant `FORWARD_CELLS_PER_LOOP` must respect, or the
/// seamless-wrap proof silently stops being about the shipped field.
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
        // ITEM 194 — the nearest-wall solve. ONE bracketed bisection, which is
        // what makes the projection unconditional at every bend.
        "if (length(q - bend / mid) - mid > 0.0) { lo = mid; } else { hi = mid; }",
        "let w = q - bend / max(d, core);",
        // The section is derived from the PAGE, never authored: the page hides
        // one third of the cross-section by construction.
        "let anchor = WARP_SECTION_PAGE_RATIO * page_half;",
        // ONE bend vector for the whole picture — not one per margin.
        "let bend = WARP_BEND_GAIN * curvature * anchor * anchor * steer;",
        // The radius floor: what bounds both lattices' projected density.
        "let u = max(u_raw, core);",
        // Both families retire into the far end rather than crowding into a knot.
        "let core_fade = smoothstep(core * WARP_CORE_FADE_LO, core * WARP_CORE_FADE_HI, u_raw);",
        // The mutation arm, and the ONLY place a side test may reach the camera.
        "if (g.params.w >= WARP_TUNNEL_PER_MARGIN) {",
    ] {
        assert!(
            wgsl.contains(expr),
            "shaders/background.wgsl must hold `{expr}`"
        );
    }
    // THE DEFECT ITEM 194 REPAIRED. The outward pull re-planted the tunnel's own
    // vanishing region at each page edge, which is what made the two margins read
    // as separately cropped circles; nothing may bring it back.
    for gone in ["WARP_PULL_FRAC", "WARP_PULL_KEEP", "WARP_TUNNEL_CENTRED"] {
        assert!(
            !wgsl.contains(gone),
            "the per-margin outward pull is back (`{gone}`) — it re-plants the \
             vanishing region at each page edge and the margins stop being one \
             cylinder"
        );
    }
    // The route lives in Rust alone: no leg table, no loop length, no easing here.
    for absent in ["leg_seconds", "loop_seconds", "warp_route_pose"] {
        assert!(
            !wgsl.contains(absent),
            "the route must not be mirrored into WGSL (`{absent}` found) — the host \
             resolves the pose and there is nothing here to drift"
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
