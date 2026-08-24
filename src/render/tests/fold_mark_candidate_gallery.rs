//! SURVEY AID — not a shipped law, not wired into any production path.
//!
//! Renders candidate fold-mark glyphs through the REAL `rotated_label`
//! mechanism (the same `LabelMask` + `RotatedLabelPipeline` production code
//! will draw through once a candidate is picked) at both settled turns, at
//! the H1/H3 heading-ladder sizes, over two worlds, at 1x/2x DPI — composited
//! into gallery sheets a human picks a fold mark from. Every candidate comes
//! from an already-bundled face; nothing here composes or writes a font.
//!
//! Runs ONLY when `AWL_FOLD_MARK_GALLERY_OUT` names an output directory — a
//! total no-op otherwise (mirrors `AWL_CJK_FORCE`'s "unset changes nothing"
//! contract), so no gate, filtered or unfiltered, ever writes gallery files.
//! `#[ignore]` is a second, independent gate on top of that.

use super::super::*;
use super::headless_dqp;
use crate::rotated_label::RotatedLabelPipeline;
use crate::rotated_label::geometry::label_axis_deg;
use crate::rotated_label::mask::LabelMask;

struct Candidate {
    /// Exact fontdb family name, matching a bundled face's own `name` table —
    /// see `assets/fonts/LICENSES.md`.
    family: &'static str,
    ch: char,
    /// Filename-safe identity, used only in the sheet's own notes.
    slug: &'static str,
}

/// Every candidate has REAL glyph coverage in an already-bundled face,
/// confirmed by a direct `ttf_parser` read over `assets/fonts/*.ttf` before
/// this file was written (see the round's report — not re-asserted here as a
/// permanent law, since the roster is a one-time survey input, not a product
/// invariant). Leads with NO bundled coverage (the vertical presentation
/// forms U+FE3F/FE40/FE41/FE42) and leads with coverage but the wrong SHAPE
/// for a directional wedge (the corner brackets U+300C/300D) are deliberately
/// absent — see the report.
const CANDIDATES: &[Candidate] = &[
    // The pre-quad original: SINGLE RIGHT-POINTING ANGLE QUOTATION MARK, from
    // the warm prose serif already bundled as a `Theme::font`.
    Candidate {
        family: "EB Garamond",
        ch: '\u{203A}',
        slug: "ebgaramond-angle-quote",
    },
    // The same codepoint, from the mono/code face — a thinner, more
    // geometric stroke than the serif reading, answering the "reads fat"
    // complaint from the opposite direction (a different face, not a
    // thinner quad).
    Candidate {
        family: "Iosevka",
        ch: '\u{203A}',
        slug: "iosevka-angle-quote",
    },
    // The item's named "angle bracket family" lead: CJK-flavored, full-width
    // punctuation, real coverage in the SAME already-bundled EB Garamond —
    // no CJK face or AwlMarks composition required to show it.
    Candidate {
        family: "EB Garamond",
        ch: '\u{3009}',
        slug: "ebgaramond-cjk-angle-bracket",
    },
    // Found by inspecting coverage rather than named by the brief: the
    // classic disclosure triangle (Finder/macOS convention), real coverage
    // in the mono/code face.
    Candidate {
        family: "Iosevka",
        ch: '\u{25B8}',
        slug: "iosevka-disclosure-triangle",
    },
    // A heavier weight in the SAME angle-quote family, from a different
    // already-bundled mono face — the opposite end of the "reads fat" taste
    // question from the two thin picks above.
    Candidate {
        family: "JetBrains Mono",
        ch: '\u{276F}',
        slug: "jetbrainsmono-heavy-angle-quote",
    },
    // The wildcard, at the user's request for one wilder option: the
    // MANICULE — the pointing hand scribes drew in manuscript margins to
    // flag a passage, which is a fold mark's exact job. Real coverage in
    // the same warm serif as the original angle quote, and it points right
    // at rest like every other member, so the quarter-turn grammar and the
    // direction-at-rest law hold unchanged (fingertip tapers, cuff is the
    // open end).
    Candidate {
        family: "EB Garamond",
        ch: '\u{261E}',
        slug: "ebgaramond-manicule",
    },
];

struct World {
    theme: &'static crate::theme::Theme,
    slug: &'static str,
}

const WORLDS: &[World] = &[
    World {
        theme: &crate::theme::GALAH,
        slug: "galah-light",
    },
    World {
        theme: &crate::theme::BOWERBIRD,
        slug: "bowerbird-dark",
    },
];

/// H1 and H3 — the two rungs the brief asks for, read from the SAME
/// `heading_scale` production headings size against (`render/spans/layout.rs`
/// `md_line_scale`), not a re-picked pair of numbers.
const LEVELS: [u8; 2] = [1, 3];

/// Base document font size at zoom=1, 1x DPI — the same constant a real
/// heading's pixel size scales from (`render::FONT_SIZE`).
const BASE_PX: f32 = super::super::FONT_SIZE;

const HEADING_WORD: &str = "Heading";

/// Collapsed (`›`, pointing right) is the glyph's own upright orientation —
/// `deg=0.0`. Expanded (`⌄`, pointing down) needs the run's ADVANCE axis
/// pointing straight down the screen: `label_axis_deg`'s own doc says `270°`
/// reads "top to bottom" (`axis=[0,1]`, screen +y = down) — so a glyph whose
/// ink sits in the +advance direction when upright ends up pointing down at
/// `270°`. `90°` is the OTHER quarter turn (`axis=[0,-1]`, up) and would be
/// the wrong sign for this family of "points right at rest" glyphs. Verified,
/// not just derived: `fold_mark_candidates_settle_in_opposite_directions`
/// below grades this on rendered pixels for every candidate.
fn turn_deg(collapsed: bool) -> f32 {
    if collapsed { 0.0 } else { 270.0 }
}

fn shape_one_char(
    font_system: &mut FontSystem,
    family: &'static str,
    ch: char,
    px: f32,
) -> GlyphBuffer {
    let mut buf = GlyphBuffer::new(font_system, GlyphMetrics::new(px, px * 1.3));
    buf.set_size(font_system, Some(px * 4.0), Some(px * 2.0));
    buf.set_wrap(font_system, Wrap::None);
    let mut s = String::new();
    s.push(ch);
    buf.set_text(
        font_system,
        &s,
        &Attrs::new().family(Family::Name(family)),
        Shaping::Advanced,
        None,
    );
    buf.shape_until_scroll(font_system, false);
    buf
}

fn shape_word(
    font_system: &mut FontSystem,
    family: &'static str,
    bold: bool,
    text: &str,
    px: f32,
) -> GlyphBuffer {
    let mut buf = GlyphBuffer::new(font_system, GlyphMetrics::new(px, px * 1.3));
    buf.set_size(font_system, Some(px * 8.0), Some(px * 2.0));
    buf.set_wrap(font_system, Wrap::None);
    let mut attrs = Attrs::new().family(Family::Name(family));
    if bold {
        attrs = attrs.weight(glyphon::Weight::BOLD);
    }
    buf.set_text(font_system, text, &attrs, Shaping::Advanced, None);
    buf.shape_until_scroll(font_system, false);
    buf
}

/// One rendered tile: the candidate mark (at `turn_deg`) beside an upright
/// heading specimen in the world's own display face — the same pairing the
/// brief asks the taste call to be made against ("reads fat and
/// world-generic… beside a heading"), not an isolated glyph.
struct Tile {
    rgba: Vec<[u8; 4]>,
    w: u32,
    h: u32,
    /// The mark's own screen bounds within the tile, `[x, y, w, h]` — the
    /// crop the direction law grades.
    mark_bounds: [f32; 4],
}

#[allow(clippy::too_many_arguments)]
fn render_tile(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    font_system: &mut FontSystem,
    swash_cache: &mut SwashCache,
    candidate: &Candidate,
    world: &crate::theme::Theme,
    level: u8,
    collapsed: bool,
    dpi: f32,
) -> Option<Tile> {
    let mark_px = BASE_PX * crate::markdown::heading_scale(level) * dpi;
    let heading_px = mark_px;
    let canvas_w = (420.0 * dpi) as u32;
    let canvas_h = (130.0 * dpi) as u32;
    let left_margin = 40.0 * dpi;
    let reserved_mark_col = 70.0 * dpi;

    let mark_buf = shape_one_char(font_system, candidate.family, candidate.ch, mark_px);
    let mark_mask = LabelMask::compose(device, queue, font_system, swash_cache, &mark_buf)?;
    let axis_mark = label_axis_deg(turn_deg(collapsed));
    let raw_mark_bounds =
        crate::rotated_label::geometry::label_bounds([0.0, 0.0], axis_mark, mark_mask.ink());
    let mark_origin = [
        left_margin - (raw_mark_bounds[0] + raw_mark_bounds[2] * 0.5),
        canvas_h as f32 * 0.5 - (raw_mark_bounds[1] + raw_mark_bounds[3] * 0.5),
    ];
    let mark_bounds =
        crate::rotated_label::geometry::label_bounds(mark_origin, axis_mark, mark_mask.ink());

    let heading_buf = shape_word(
        font_system,
        world.font,
        world.heading_bold,
        HEADING_WORD,
        heading_px,
    );
    let heading_mask = LabelMask::compose(device, queue, font_system, swash_cache, &heading_buf)?;
    let axis_heading = label_axis_deg(0.0);
    let raw_heading_bounds =
        crate::rotated_label::geometry::label_bounds([0.0, 0.0], axis_heading, heading_mask.ink());
    let heading_x = left_margin + reserved_mark_col;
    let heading_origin = [
        heading_x - raw_heading_bounds[0],
        canvas_h as f32 * 0.5 - (raw_heading_bounds[1] + raw_heading_bounds[3] * 0.5),
    ];

    let ink = [
        crate::theme::srgb_channel_to_linear_f32(world.base_content.r),
        crate::theme::srgb_channel_to_linear_f32(world.base_content.g),
        crate::theme::srgb_channel_to_linear_f32(world.base_content.b),
    ];

    let mut mark_pipe = RotatedLabelPipeline::new(device, super::dither::FMT);
    mark_pipe.prepare(
        device,
        queue,
        canvas_w,
        canvas_h,
        &mark_mask,
        mark_origin,
        axis_mark,
        ink,
        ink,
        1.0,
    );
    let mut heading_pipe = RotatedLabelPipeline::new(device, super::dither::FMT);
    heading_pipe.prepare(
        device,
        queue,
        canvas_w,
        canvas_h,
        &heading_mask,
        heading_origin,
        axis_heading,
        ink,
        ink,
        1.0,
    );

    let (texture, view) = super::dither::offscreen(device, canvas_w, canvas_h);
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("fold-mark gallery tile encoder"),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("fold-mark gallery tile pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(world.base_100.to_wgpu_clear()),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        mark_pipe.draw(&mut pass);
        heading_pipe.draw(&mut pass);
    }
    queue.submit(Some(encoder.finish()));
    let rgba = super::dither::read_pixels(device, queue, &texture, canvas_w, canvas_h);
    Some(Tile {
        rgba,
        w: canvas_w,
        h: canvas_h,
        mark_bounds,
    })
}

/// Whether a pixel differs from the tile's own background by enough to count
/// as ink.
fn is_ink(p: [u8; 4], bg: [u8; 3]) -> bool {
    let d = (p[0] as i32 - bg[0] as i32).unsigned_abs()
        + (p[1] as i32 - bg[1] as i32).unsigned_abs()
        + (p[2] as i32 - bg[2] as i32).unsigned_abs();
    d > 24
}

/// The horizontal SPAN (max_x - min_x, in pixels) of inked pixels within a
/// thin horizontal strip `[y0, y1)` of `rect`'s own x-range. `0.0` when the
/// strip carries no ink.
///
/// This is a SPAN, not a density: a right-pointing wedge (`›`, `〉`, `▸`)
/// tapers to a single point at its vertex and is naturally WIDEST at its
/// open end, so total ink MASS is higher at the open end regardless of which
/// way the wedge points — an ink-density check would grade the physics of a
/// taper, not its direction. The span the ink occupies at each end is what
/// actually distinguishes "converges here" (vertex, near-zero span) from
/// "spreads here" (open end, near the full mark width).
fn ink_x_span(tile: &Tile, bg: [u8; 3], rect: [f32; 4], y0: f32, y1: f32) -> f32 {
    let x0 = rect[0].max(0.0) as u32;
    let x1 = ((rect[0] + rect[2]).min(tile.w as f32)) as u32;
    let py0 = y0.max(0.0) as u32;
    let py1 = (y1.min(tile.h as f32)) as u32;
    if x1 <= x0 || py1 <= py0 {
        return 0.0;
    }
    let mut min_x: Option<u32> = None;
    let mut max_x: Option<u32> = None;
    for y in py0..py1 {
        for x in x0..x1 {
            if is_ink(tile.rgba[(y * tile.w + x) as usize], bg) {
                min_x = Some(min_x.map_or(x, |m| m.min(x)));
                max_x = Some(max_x.map_or(x, |m| m.max(x)));
            }
        }
    }
    match (min_x, max_x) {
        (Some(a), Some(b)) => (b - a) as f32,
        _ => 0.0,
    }
}

/// THE DIRECTION-AT-REST LAW: every candidate is a RIGHT-pointing glyph
/// upright (its vertex sits at its own advance-direction maximum, its open
/// end at the minimum — true by definition for anything named
/// "right-pointing"). Turned to `expanded`, that advance direction maps to
/// straight down the screen (`turn_deg`'s own derivation), so the vertex
/// must land at the BOTTOM of the expanded mark and the open end at the TOP
/// — measured as horizontal ink SPAN narrowing toward the bottom, never the
/// top, with zero animation frames. This is what makes `turn_deg`'s sign a
/// proven fact rather than a recalled one.
#[test]
fn fold_mark_candidates_settle_in_opposite_directions() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(64.0, 64.0) else {
        eprintln!("skipping fold_mark_candidates_settle_in_opposite_directions: no wgpu adapter");
        return;
    };
    let world = &crate::theme::GALAH;
    for c in CANDIDATES {
        for &level in &LEVELS {
            let collapsed = render_tile(
                &device,
                &queue,
                &mut p.font_system,
                &mut p.swash_cache,
                c,
                world,
                level,
                true,
                2.0,
            )
            .expect("collapsed tile composes");
            let expanded = render_tile(
                &device,
                &queue,
                &mut p.font_system,
                &mut p.swash_cache,
                c,
                world,
                level,
                false,
                2.0,
            )
            .expect("expanded tile composes");

            let bg = [world.base_100.r, world.base_100.g, world.base_100.b];
            // Thin strips at the very top and very bottom edge of the
            // expanded mark's own box — narrow enough to sample near each
            // end without also sampling the shape's own middle taper.
            let strip_h = (expanded.mark_bounds[3] * 0.15).max(1.0);
            let top_y0 = expanded.mark_bounds[1];
            let bottom_y1 = expanded.mark_bounds[1] + expanded.mark_bounds[3];
            let top_span = ink_x_span(
                &expanded,
                bg,
                expanded.mark_bounds,
                top_y0,
                top_y0 + strip_h,
            );
            let bottom_span = ink_x_span(
                &expanded,
                bg,
                expanded.mark_bounds,
                bottom_y1 - strip_h,
                bottom_y1,
            );
            assert!(
                bottom_span < top_span,
                "{} L{level}: expanded mark must TAPER toward its BOTTOM edge \
                 (vertex pointing down, narrower span there than at the open top) — \
                 top_span={top_span:.1}px bottom_span={bottom_span:.1}px",
                c.slug
            );

            // Non-vacuous, and glyph-shape-agnostic (unlike asserting which
            // way a PARTICULAR candidate's ink box happens to lean): a exact
            // quarter turn transposes a bounding box exactly — width and
            // height swap — regardless of whether the glyph itself reads
            // wide or tall upright. This is the direct geometric proof the
            // turn actually rotated the mask rather than leaving it inert.
            const EPS: f32 = 0.5;
            assert!(
                (collapsed.mark_bounds[2] - expanded.mark_bounds[3]).abs() < EPS
                    && (collapsed.mark_bounds[3] - expanded.mark_bounds[2]).abs() < EPS,
                "{} L{level}: a quarter turn must transpose the ink box's width/height \
                 exactly — collapsed {:?} vs expanded {:?}",
                c.slug,
                collapsed.mark_bounds,
                expanded.mark_bounds
            );
        }
    }
}

/// Paste `tile` into `sheet`'s raw RGBA buffer at `(x0, y0)`.
fn blit(sheet: &mut [u8], sheet_w: u32, x0: u32, y0: u32, tile: &Tile) {
    for y in 0..tile.h {
        for x in 0..tile.w {
            let sx = x0 + x;
            let sy = y0 + y;
            let si = ((sy * sheet_w + sx) * 4) as usize;
            let ti = (y * tile.w + x) as usize;
            let p = tile.rgba[ti];
            sheet[si] = p[0];
            sheet[si + 1] = p[1];
            sheet[si + 2] = p[2];
            sheet[si + 3] = 255;
        }
    }
}

/// Generates the gallery sheets: one PNG per (world, dpi), each a
/// candidates-by-(level x state) grid. Total no-op unless
/// `AWL_FOLD_MARK_GALLERY_OUT` is set.
#[test]
#[ignore]
fn fold_mark_candidate_gallery() {
    let Ok(out_dir) = std::env::var("AWL_FOLD_MARK_GALLERY_OUT") else {
        eprintln!(
            "skipping fold_mark_candidate_gallery: set AWL_FOLD_MARK_GALLERY_OUT=<dir> to run \
             (see captures/item-475-glyph-survey/shoot.sh)"
        );
        return;
    };
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(64.0, 64.0) else {
        eprintln!("skipping fold_mark_candidate_gallery: no wgpu adapter");
        return;
    };
    std::fs::create_dir_all(&out_dir).expect("gallery out dir creatable");

    // Columns: (level, collapsed) pairs, in reading order H1-collapsed,
    // H1-expanded, H3-collapsed, H3-expanded.
    let columns: Vec<(u8, bool)> = LEVELS
        .iter()
        .flat_map(|&level| [(level, true), (level, false)])
        .collect();

    for world in WORLDS {
        for &dpi in &[1.0f32, 2.0] {
            let tiles: Vec<Vec<Tile>> = CANDIDATES
                .iter()
                .map(|c| {
                    columns
                        .iter()
                        .map(|&(level, collapsed)| {
                            render_tile(
                                &device,
                                &queue,
                                &mut p.font_system,
                                &mut p.swash_cache,
                                c,
                                world.theme,
                                level,
                                collapsed,
                                dpi,
                            )
                            .expect("gallery tile composes")
                        })
                        .collect()
                })
                .collect();

            let tile_w = tiles[0][0].w;
            let tile_h = tiles[0][0].h;
            let sheet_w = tile_w * columns.len() as u32;
            let sheet_h = tile_h * CANDIDATES.len() as u32;
            let mut sheet = vec![0u8; (sheet_w * sheet_h * 4) as usize];
            for (row, row_tiles) in tiles.iter().enumerate() {
                for (col, tile) in row_tiles.iter().enumerate() {
                    blit(
                        &mut sheet,
                        sheet_w,
                        col as u32 * tile_w,
                        row as u32 * tile_h,
                        tile,
                    );
                }
            }
            let img = image::RgbaImage::from_raw(sheet_w, sheet_h, sheet)
                .expect("sheet buffer matches its own declared dimensions");
            let path = format!("{out_dir}/{}-{}x.png", world.slug, dpi as u32);
            img.save(&path).expect("gallery sheet saves");
            eprintln!("wrote {path} ({sheet_w}x{sheet_h})");
        }
    }
}
