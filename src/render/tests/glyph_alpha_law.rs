//! GLYPH INK REACHES THE RENDERER WITH ITS OWN ALPHA — and the decision that
//! it should is recorded here, because it could have gone the other way.
//!
//! [`theme::Srgb`] accepts an alpha byte and [`theme::Srgb::to_glyphon`] used to
//! throw it away, so a caller that authored a translucent text colour got an
//! OPAQUE glyph with nothing anywhere to say so. Two facts settle which of the
//! two repairs is right. Alpha is meaningful in glyphon's own mask path — the
//! fragment stage multiplies a glyph's coverage by `color.a` and the pipeline
//! blends with `ALPHA_BLENDING` — and awl already SPENDS that: the outline
//! pane's edge fade scales a glyph colour's alpha and the rows really do fade.
//! So the type was not accepting a value the renderer cannot use; the converter
//! was dropping one the renderer honours.
//!
//! Three laws, and the third is the other half of the decision:
//!
//! 1. every alpha byte survives the conversion,
//! 2. two inks differing ONLY in alpha draw different frames — through a real
//!    glyphon renderer on a real device, because a conversion that carries a
//!    byte nothing reads would satisfy law 1 alone,
//! 3. NOTHING SHIPPED MOVED: every ink a `to_glyphon` call site can hand the
//!    converter is opaque on every world, so the repair changed no pixel of the
//!    product. That is what makes this a latent trap rather than a live defect,
//!    and it is asserted per world rather than argued.
//!
//! Law 2's pair is a presence floor beside a difference floor, for the reason
//! this repo keeps relearning: "the faded frame is fainter" gets HAPPIER as the
//! ink vanishes, and a frame with no glyphs at all would be its best result.

use super::super::*;
use super::dither::{FMT, offscreen, read_pixels};

/// The probe canvas. Small: the claim is about one glyph run's ink, and the
/// readback is the expensive part.
const W: u32 = 220;
const H: u32 = 64;

/// A light ground and a dark ink, chosen only so the two are far apart in
/// value — no world's tokens are involved, because this law is about the
/// CONVERTER, not about any theme.
const GROUND: theme::Srgb = theme::Srgb::rgb(0xF4, 0xF2, 0xEC);
const INK_RGB: (u8, u8, u8) = (0x10, 0x12, 0x18);

/// The faded arm's alpha. Any value clear of both ends works; this one is a
/// quarter, so the expected travel ratio (~0.12 after the sRGB encode) sits far
/// from the 0.5 the law demands.
const FADED_ALPHA: u8 = 0x40;

/// Rec. 709 luma of a read-back pixel, in bytes. The frames are compared to
/// EACH OTHER and to their own corner pixel, never to an authored constant, so
/// a backend that rounds a shade differently moves every term together.
fn luma(px: [u8; 4]) -> f32 {
    0.2126 * px[0] as f32 + 0.7152 * px[1] as f32 + 0.0722 * px[2] as f32
}

/// Draw `"Alpha"` in `ink` over [`GROUND`] through a real glyphon renderer and
/// read the frame back. `None` only where the machine has no adapter.
///
/// Deliberately its own minimal glyphon stack rather than a `TextPipeline`: the
/// subject is one `TextArea`'s colour, and a whole pipeline would route it
/// through a world's theme tokens — which are exactly what law 3 proves cannot
/// carry an alpha to test with.
fn glyph_frame(ink: theme::Srgb) -> Option<Vec<[u8; 4]>> {
    let (device, queue) = crate::test_gpu::shared_device_queue()?;
    let mut font_system = build_font_system();
    let mut swash_cache = SwashCache::new();
    let cache = Cache::new(&device);
    let mut viewport = Viewport::new(&device, &cache);
    let mut atlas = TextAtlas::new(&device, &queue, &cache, FMT);
    let mut renderer =
        TextRenderer::new(&mut atlas, &device, wgpu::MultisampleState::default(), None);

    let mut buffer = GlyphBuffer::new(&mut font_system, GlyphMetrics::new(32.0, 40.0));
    buffer.set_size(&mut font_system, Some(W as f32), Some(H as f32));
    buffer.set_wrap(&mut font_system, Wrap::None);
    let attrs = Attrs::new()
        .family(Family::Monospace)
        .color(ink.to_glyphon());
    buffer.set_text(&mut font_system, "Alpha", &attrs, Shaping::Advanced, None);
    buffer.shape_until_scroll(&mut font_system, false);

    viewport.update(
        &queue,
        Resolution {
            width: W,
            height: H,
        },
    );
    renderer
        .prepare(
            &device,
            &queue,
            &mut font_system,
            &mut atlas,
            &viewport,
            [TextArea {
                buffer: &buffer,
                left: 4.0,
                top: 4.0,
                scale: 1.0,
                bounds: TextBounds {
                    left: 0,
                    top: 0,
                    right: W as i32,
                    bottom: H as i32,
                },
                default_color: ink.to_glyphon(),
                custom_glyphs: &[],
            }],
            &mut swash_cache,
        )
        .expect("glyphon prepare failed in the glyph-alpha probe");

    let (texture, tview) = offscreen(&device, W, H);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("awl glyph-alpha probe encoder"),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("awl glyph-alpha probe pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &tview,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(GROUND.to_wgpu_clear()),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        renderer
            .render(&atlas, &viewport, &mut pass)
            .expect("glyphon render failed in the glyph-alpha probe");
    }
    queue.submit(Some(encoder.finish()));
    Some(read_pixels(&device, &queue, &texture, W, H))
}

/// How far this frame's ink travelled from THIS frame's own ground, and how
/// many pixels moved at all — both read out of the same frame, so nothing here
/// is compared against an authored byte.
///
/// The ground reading is the top-left corner, which the run's `left`/`top`
/// inset keeps clear of every glyph.
fn travel(frame: &[[u8; 4]]) -> (f32, usize) {
    let ground = luma(frame[0]);
    let mut max = 0.0f32;
    let mut moved = 0usize;
    for px in frame {
        let d = (luma(*px) - ground).abs();
        if d > INK_EPSILON {
            moved += 1;
        }
        max = max.max(d);
    }
    (max, moved)
}

/// A luma delta below this is rasterizer noise rather than ink.
const INK_EPSILON: f32 = 8.0;

/// The faded arm must travel less than this share of the opaque arm's travel.
/// Calibrated from three readings, as an appearance floor must be: the shipped
/// ratio at [`FADED_ALPHA`] is ~0.12, the ratio a converter that drops alpha
/// produces is exactly 1.0, and this floor sits between them with room for a
/// backend that rounds the blend differently.
const FADED_TRAVEL_CEILING: f32 = 0.5;

/// How many one-off glyph inks `glyph_inks_of_the_active_world` lists by hand —
/// the arity of its own array literal, which the compiler holds it to.
const SINGLETON_GLYPH_INKS: usize = 9;

#[test]
fn every_alpha_byte_survives_the_conversion_to_glyphon() {
    let mut graded = 0usize;
    for a in 0..=u8::MAX {
        let c = theme::Srgb::rgba(0x2A, 0x5C, 0x91, a).to_glyphon();
        assert_eq!(
            (c.r(), c.g(), c.b(), c.a()),
            (0x2A, 0x5C, 0x91, a),
            "Srgb::to_glyphon must hand glyphon the colour it was given, alpha included"
        );
        graded += 1;
    }
    assert_eq!(
        graded,
        usize::from(u8::MAX) + 1,
        "every alpha byte is graded, not a sampled few"
    );
}

#[test]
fn two_inks_differing_only_in_alpha_draw_different_frames() {
    let _g = crate::testlock::serial();
    if !crate::test_gpu::adapter_present() {
        eprintln!(
            "skipping two_inks_differing_only_in_alpha_draw_different_frames: no wgpu adapter"
        );
        return;
    }
    let (r, g, b) = INK_RGB;
    let Some(opaque) = glyph_frame(theme::Srgb::rgba(r, g, b, 0xFF)) else {
        return;
    };
    let Some(faded) = glyph_frame(theme::Srgb::rgba(r, g, b, FADED_ALPHA)) else {
        return;
    };

    let (opaque_travel, opaque_moved) = travel(&opaque);
    let (faded_travel, faded_moved) = travel(&faded);

    // PRESENCE, both arms — without this the law's difference floor is happiest
    // when the faded arm draws nothing at all, and an empty frame is the one
    // outcome that must never read as a pass.
    assert!(
        opaque_moved >= 40 && opaque_travel > 4.0 * INK_EPSILON,
        "the opaque arm drew no real ink to compare against: {opaque_moved} pixels moved, \
         furthest {opaque_travel:.1} from its own ground"
    );
    assert!(
        faded_moved >= 40 && faded_travel > INK_EPSILON,
        "the faded arm drew no visible ink at all, so 'fainter' would be vacuous: \
         {faded_moved} pixels moved, furthest {faded_travel:.1} from its own ground"
    );

    // THE SUBJECT: the same glyphs, the same rgb, one alpha byte apart, must
    // reach the framebuffer differently. A converter that drops alpha makes
    // these two frames identical and this ratio exactly 1.0.
    let differing = opaque
        .iter()
        .zip(faded.iter())
        .filter(|(a, b)| a != b)
        .count();
    assert!(
        differing > 0,
        "two text colours differing only in alpha drew a byte-identical frame \
         ({} pixels compared, none differ) — the alpha never reached the renderer",
        opaque.len()
    );
    assert!(
        faded_travel < FADED_TRAVEL_CEILING * opaque_travel,
        "alpha {FADED_ALPHA:#04x} ink travelled {faded_travel:.1} from its ground where the \
         opaque ink travelled {opaque_travel:.1} (ratio {:.3}, ceiling {FADED_TRAVEL_CEILING}) \
         — the alpha is reaching the renderer weakened or not at all",
        faded_travel / opaque_travel
    );
}

/// Every ink an `Srgb::to_glyphon()` call site can produce on the ACTIVE world,
/// each under the label its failure is reported by. The world axis is swept by
/// the caller; this is the per-world surface.
///
/// The three enum axes come from their own rosters (`ALL`), so a new elevation,
/// placard rung or syntax role enrols itself here instead of quietly widening
/// the product past this law.
fn glyph_inks_of_the_active_world() -> Vec<(String, theme::Srgb)> {
    let th = theme::active();
    // The one-off accessors, hand-listed because there is no roster to ask.
    // The array's DECLARED length is `SINGLETON_GLYPH_INKS`, so the cell
    // arithmetic below cannot drift from the list: adding an entry here without
    // moving that const fails to compile.
    let singletons: [(&str, theme::Srgb); SINGLETON_GLYPH_INKS] = [
        ("base_content", theme::base_content()),
        ("muted", theme::muted()),
        ("faint", theme::faint()),
        ("error", theme::error()),
        ("fold_afford_tail_ink", theme::fold_afford_tail_ink()),
        (
            "selected_row_secondary_ink(surface_selected)",
            theme::selected_row_secondary_ink(theme::surface_selected()),
        ),
        ("strike_ink", strike_ink(&th)),
        (
            "lerp(base_content, muted)",
            lerp_srgb(th.base_content, th.muted, 0.28),
        ),
        // The open menu title's own glyph ink, whichever branch this world takes.
        (
            "highlight_treatment ink",
            match th.highlight_treatment(theme::selection_document()) {
                theme::HighlightTreatment::ValueBand(_) => theme::muted(),
                theme::HighlightTreatment::InverseFill { ink, .. } => ink,
            },
        ),
    ];
    let mut out: Vec<(String, theme::Srgb)> = singletons
        .iter()
        .map(|(name, ink)| ((*name).to_string(), *ink))
        .collect();
    for e in theme::Elevation::ALL {
        out.push((format!("pane_surface({e:?})"), theme::pane_surface(e)));
    }
    for i in theme::PlacardInk::ALL {
        out.push((format!("placard_ink({i:?})"), theme::placard_ink(i)));
    }
    for k in crate::syntax::SynKind::ALL {
        out.push((
            format!("role_style_for({k:?}).fg"),
            role_style_for(&th, k).fg,
        ));
    }
    out
}

#[test]
fn no_ink_a_glyphon_call_site_can_produce_is_translucent_on_any_world() {
    let _g = crate::testlock::serial();
    let restore = theme::active_index();
    // Ask the roster for both axes: how many worlds, and how many inks each
    // world answers with. Neither number is typed anywhere.
    let per_world = SINGLETON_GLYPH_INKS
        + theme::Elevation::ALL.len()
        + theme::PlacardInk::ALL.len()
        + crate::syntax::SynKind::ALL.len();
    let expected = theme::THEMES.len() * per_world;

    let mut graded = 0usize;
    let mut enrolled: Vec<&'static str> = Vec::new();
    for (i, t) in theme::THEMES.iter().enumerate() {
        theme::set_active(i);
        enrolled.push(t.name);
        for (what, ink) in glyph_inks_of_the_active_world() {
            assert_eq!(
                ink.a, 0xFF,
                "{}: {what} is translucent (alpha {:#04x}). Carrying alpha through \
                 Srgb::to_glyphon therefore MOVES a shipped pixel on this world, which the \
                 repair was established not to do — decide that deliberately rather than \
                 inheriting it",
                t.name, ink.a
            );
            graded += 1;
        }
    }
    theme::set_active(restore);

    assert_eq!(
        graded, expected,
        "the glyph-ink sweep graded {graded} cells, not {expected} — worlds enrolled: {enrolled:?}"
    );
}
