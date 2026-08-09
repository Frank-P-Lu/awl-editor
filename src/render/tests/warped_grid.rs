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
//! "state oracle, not an appearance oracle" tripwire.

use super::bands_waves::{bg_desc_for, headless_dq};
use super::zigzag_ground::{SWEEP, margins};
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
            ..
        } => theme::Background::WarpedGrid {
            ground,
            minor,
            major,
            tunnel,
            spacing_px,
            density,
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
            ..
        } => theme::Background::WarpedGrid {
            ground,
            minor,
            major,
            tunnel,
            spacing_px,
            density,
        },
        other => other,
    }
}

/// Render one background pass at a real forward-travel phase.
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
        warpgrid::forward_cells(phase),
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
    let mut bg = crate::background::BackgroundPipeline::new(device, super::dither::FMT, desc);
    bg.prepare(
        queue,
        w,
        h,
        col_left,
        col_w,
        crate::background::AmbientUpload {
            warp_travel,
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

/// Marked pixels inside `[x0, x1)` over the whole height.
fn ink_in(f: &[i32], w: u32, h: u32, x0: u32, x1: u32) -> usize {
    (0..h)
        .flat_map(|y| (x0..x1).map(move |x| (y, x)))
        .filter(|&(y, x)| f[(y * w + x) as usize] > INK_FLOOR)
        .count()
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
    let mid = warpgrid::LOOP_SECONDS * 0.37;
    assert_ne!(
        warpgrid::forward_cells(mid),
        warpgrid::forward_cells(warpgrid::FROZEN_PHASE),
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

/// THE WRITING PAGE CARRIES THE FIELD, AT ONE CONSTANT VEIL, AT EVERY GEOMETRY.
///
/// ⚠️ THIS LAW REPLACES `the_grid_never_enters_the_writing_page_at_any_geometry_
/// _or_phase`, WHICH ASSERTED THE OPPOSITE, and the reversal is a decision
/// rather than a regression. When the tunnel had an axis in each margin,
/// punching the page away was right — there was nothing under it to see. With
/// ONE axis at the room's centre the vanishing point IS under the page, and two
/// flanks with a hard hole between them read as two pictures, which is exactly
/// the defect the user reported. The crossing is what makes them one.
///
/// THE STATISTIC IS THE STRONGEST MARK, AND CHOOSING IT WAS THE WHOLE DESIGN OF
/// THIS LAW. Two earlier drafts graded the wrong quantity and both passed while
/// measuring nothing. Counting marked PIXELS grades the geometry: in a short
/// wide room the page holds whole concentric rings while the margins are tangent
/// slivers, so the margin loses on count though every one of its marks is far
/// stronger. Averaging over marked pixels grades the ANTIALIAS population: the
/// margin's dense minor lattice contributes a huge tail of barely-inked pixels
/// that drags its mean below the page's few strong arcs. Only the peak grades
/// the veil.
///
/// The three clauses are deliberately different in kind. That the page is inked
/// is the crossing existing at all. That its peak is CONSTANT across twelve
/// geometries and five phases is the strong one — it says the page column can
/// only MASK this field and can never rescale it, which is the same invariant
/// the shipping profile's placement claims, checked from the other side. And
/// that the peak sits far under an open margin's is prose keeping figure/ground.
#[test]
fn the_writing_page_carries_the_field_at_one_constant_veil_at_every_geometry() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    // The strongest mark inside `[a, b)`, over the whole height. Both bands are
    // inset off the column boundary: `fs_main` decides the page in PHYSICAL
    // pixels against a fractional column edge, so the boundary pixel belongs to
    // neither band — read uninset, it once made the page look STRONGER than the
    // margin and would have hidden the real ratio entirely.
    let peak = |f: &[i32], w: u32, h: u32, a: u32, b: u32| -> i32 {
        (0..h)
            .flat_map(|y| (a..b).map(move |x| (y, x)))
            .map(|(y, x)| f[(y * w + x) as usize])
            .max()
            .unwrap_or(0)
    };

    // THE FULL-STRENGTH REFERENCE is taken once, from an OPEN margin at the
    // canonical geometry — never from the local margin, which at some swept
    // shapes is a sliver lying entirely inside the page-edge ramp and is
    // therefore not at full strength either.
    let canon = field(
        &device,
        &queue,
        kite(),
        W,
        H,
        COL_LEFT,
        COL_W,
        warpgrid::FROZEN_PHASE,
    );
    let open_margin = canon_margins()
        .into_iter()
        .map(|(a, b)| peak(&canon, W, H, a, b.saturating_sub(4)))
        .max()
        .unwrap_or(0);
    assert!(
        open_margin > 100,
        "the open-margin reference must be a real full-strength field, got {open_margin}"
    );

    let mut seen: Vec<(String, i32)> = Vec::new();
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
        for phase in sampled_phases() {
            let f = field(&device, &queue, kite(), ww, wh, col_left, col_w, phase);
            let x0 = col_left.max(0.0) as u32 + 4;
            let x1 = ((col_left + col_w).ceil() as u32).min(ww).saturating_sub(4);
            let page = peak(&f, ww, wh, x0, x1);
            assert!(
                page > INK_FLOOR,
                "{ww}x{wh}/m{measure} @phase {phase}: the page carries NO field — the two \
                 flanks are then two pictures with a hole between them, which is the \
                 two-tunnels read item 268 exists to remove"
            );
            assert!(
                page * 3 <= open_margin,
                "{ww}x{wh}/m{measure} @phase {phase}: the page's strongest mark is {page} \
                 against an open margin's {open_margin} — the crossing must stay a whisper \
                 under prose, not a second margin"
            );
            seen.push((format!("{ww}x{wh}/m{measure}@{phase}"), page));
        }
    }
    crate::page::set_measure(restore);
    assert!(
        seen.len() >= 60,
        "the sweep must actually grade cells, got {}",
        seen.len()
    );
    let (ref lo_name, lo) = *seen.iter().min_by_key(|(_, v)| *v).unwrap();
    let (ref hi_name, hi) = *seen.iter().max_by_key(|(_, v)| *v).unwrap();
    assert_eq!(
        lo, hi,
        "the under-page veil is not one constant: {lo} at {lo_name} against {hi} at \
         {hi_name}. The page column may MASK this field and may never rescale it — a veil \
         that reads the page is the same class of defect as an axis that read the margin"
    );
}

fn sampled_phases() -> [f32; 5] {
    [
        warpgrid::FROZEN_PHASE,
        warpgrid::LOOP_SECONDS * 0.17,
        warpgrid::LOOP_SECONDS * 0.33,
        warpgrid::LOOP_SECONDS * 0.58,
        warpgrid::LOOP_SECONDS * 0.81,
    ]
}

/// BOTH MARGINS carry a real field at every swept geometry — the composition is
/// two slices, never one live margin and one blank one.
#[test]
fn both_margins_carry_a_real_field_at_every_swept_geometry() {
    let _g = crate::testlock::serial();
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
            let span = x1.saturating_sub(x0);
            if span < 24 {
                continue; // a sliver narrower than the edge-quiet band is allowed to hold nothing
            }
            let area = (span * wh) as f64;
            let ink = ink_in(&f, ww, wh, x0, x1) as f64;
            // THE FLOOR IS THE WORLD'S OWN NARROW-MARGIN BAND, not one number.
            // `WARP_NARROW_LO_PX`..`_HI_PX` (84..210) already retires the MINOR
            // lattice as a margin narrows — shipped, deliberate simplification —
            // so a margin inside that band is DESIGNED to be quiet and holding it
            // to the open-margin figure would assert against the design. It is
            // also where the centred axis costs most: a thin strip far
            // from the axis runs nearly TANGENT to every ring, so its crossings
            // are short. Measured worst in this sweep is 1400x700/m86's 80px
            // flanks at 0.571%; every margin at or above the band's top clears 5%.
            let floor = if span >= 210 { 0.02 } else { 0.003 };
            assert!(
                ink / area >= floor,
                "{ww}x{wh}/m{measure} margin {i} [{x0},{x1}) ({span}px): only {:.3}% of it \
                 carries field ink against a {:.1}% floor — a margin wide enough to draw \
                 must read as a slice of the tunnel",
                100.0 * ink / area,
                100.0 * floor
            );
        }
    }
    crate::page::set_measure(restore);
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

/// THE FIELD QUIETS BESIDE THE PAGE — BUT NO LONGER TO NOTHING, AND BOTH HALVES
/// OF THAT ARE ASSERTED. Nothing may compete with prose at the boundary the eye
/// reads across, so the band immediately outside the column carries materially
/// less ink than the open margin further out. What the crossing changed is the FLOOR
/// of that recession: it used to be zero, because the page edge was the end of
/// the world; it is now `WARP_PAGE_VEIL`, because the field continues across. A
/// ramp that still touched zero here would break every ring at exactly the one
/// boundary a reader can check the two flanks against each other. So the near
/// band must be quieter than the open margin AND must not be empty, and the two
/// clauses fail on opposite mistakes.
#[test]
fn the_field_fades_toward_the_page_edge() {
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
    // THE CROSSING ITSELF: the 8px band just OUTSIDE the page edge against the
    // 8px band just INSIDE it. A stroke that leaves one flank must arrive in the
    // other, and the only place that can be checked directly is the boundary. If
    // the ramp still fell to zero at the page edge these two would differ by
    // everything; at the veil they are the same field seen from either side.
    for (label, out_band, in_band) in [
        (
            "left",
            mean(COL_LEFT as u32 - 8, COL_LEFT as u32),
            mean(COL_LEFT as u32 + 2, COL_LEFT as u32 + 10),
        ),
        (
            "right",
            mean(col_right, col_right + 8),
            mean(col_right - 10, col_right - 2),
        ),
    ] {
        let (lo, hi) = (out_band.min(in_band), out_band.max(in_band));
        assert!(
            lo > 0.05 && hi < lo * 3.0,
            "{label} page edge: {out_band:.2} just outside against {in_band:.2} just \
             inside — the field must CROSS the boundary at one strength, not break at it"
        );
    }
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
            near * 2.0 < far,
            "{label} page edge: the 8px beside the page carries mean {near:.2} against \
             {far:.2} out in the open margin — the field must recede at the edge"
        );
        assert!(
            near > 0.05,
            "{label} page edge: the 8px beside the page carries mean {near:.2} — the field \
             must recede to the VEIL, never to nothing, or every ring breaks at the one \
             boundary a reader can check the two flanks against each other"
        );
    }
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

/// THE LOOP REPEAT IS INVISIBLE IN REAL PIXELS, and the claim is made where
/// it can actually fail.
///
/// Comparing the resolved frames at `phase == LOOP_SECONDS` and `phase == 0`
/// would be vacuous because the phase resolver wraps its input. The two
/// calls are literally the same call, and the law stayed green over
/// `FORWARD_CELLS_PER_LOOP: 64` — the very value whose wrap rotates which lines
/// are major. The load-bearing statement is about explicit travel, not phase: a
/// whole loop of forward travel must land the field back on its own lattice AND
/// its own line hierarchy, which happens if and only if the travel is a multiple
/// of the major modulus.
#[test]
fn a_whole_loop_of_forward_travel_is_byte_identical_at_real_pixels() {
    let _g = crate::testlock::serial();
    let Some((device, queue)) = headless_dq() else {
        return;
    };
    let at = |forward: f32| {
        render_travel(
            &device,
            &queue,
            bg_desc_for(kite()),
            W,
            H,
            COL_LEFT,
            COL_W,
            forward,
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
            warpgrid::LOOP_SECONDS
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
            warpgrid::LOOP_SECONDS * 0.37
        ),
        start,
        "the travel clock must actually move the field"
    );
}

/// EVERY FREEZE PATH RESOLVES TO THE ONE COMPOSED STILL, and the headless
/// capture shares it. Reduce Motion, `ambient_motion` off (which hard-freezes
/// the accumulator) and a capture that never ticks the clock all render the same
/// frame — the accessibility promise and byte-determinism are one fact.
#[test]
fn every_freeze_path_renders_the_one_composed_still() {
    let _g = crate::testlock::serial();
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
    for stored in [0.0f32, 91.3, warpgrid::LOOP_SECONDS * 0.77] {
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

/// The shader keeps one fixed framing, direct straight-tube geometry, the
/// forward sign, and no dormant steering machinery.
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
        "var anchor = WARP_SECTION_ROOM_FRAC * max(vp.y, 1.0);",
        // ONE axis owner, and its SIGNATURE is the proof: no side argument, so
        // the shader cannot give the two margins different vanishing points.
        "fn warp_room_axis(vp_x: f32) -> f32 {",
        "return vp_x * 0.5;",
        "let w = q;",
        "let u = max(u_raw, core);",
        "let core_fade = smoothstep(core * WARP_CORE_FADE_LO, core * WARP_CORE_FADE_HI, u_raw);",
        "let travel = select(g.warp_travel, -g.warp_travel, reversed);",
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
