//! Minimal binary-safe PDF 1.7 object writer. Object IDs are planned up front;
//! every offset and stream length is derived from the bytes actually emitted.

use std::fmt::Write as _;

use super::fonts::{
    BASE_ROLES, FontAsset, FontRole, ROLES, descriptor_for, glyph_widths_for, role_index,
    subset_for,
};
use super::layout::{GlyphOp, Layout, Op, PdfColor};

const FONT_BASE: u32 = 3;
const FONT_OBJECTS: u32 = 5;

pub(super) fn emit(layout: &Layout, metadata: &[u8]) -> Result<Vec<u8>, String> {
    let (active_roles, mut next) = active_role_plan(layout);
    let mut image_ids = Vec::new();
    for image in &layout.images {
        let main = next;
        next += 1;
        let alpha = image.alpha.as_ref().map(|_| {
            let id = next;
            next += 1;
            id
        });
        image_ids.push((main, alpha));
    }
    let metadata_id = next;
    next += 1;
    struct PageIds {
        content: u32,
        annots: Vec<u32>,
        page: u32,
    }
    let mut page_ids = Vec::new();
    for page in &layout.pages {
        let content = next;
        next += 1;
        let annots = page
            .links
            .iter()
            .map(|_| {
                let id = next;
                next += 1;
                id
            })
            .collect();
        let page = next;
        next += 1;
        page_ids.push(PageIds {
            content,
            annots,
            page,
        });
    }

    let mut objects: Vec<Option<Vec<u8>>> = vec![None; next as usize];
    objects[1] =
        Some(format!("<< /Type /Catalog /Pages 2 0 R /Metadata {metadata_id} 0 R >>").into_bytes());
    let kids = page_ids
        .iter()
        .map(|p| format!("{} 0 R", p.page))
        .collect::<Vec<_>>()
        .join(" ");
    objects[2] = Some(
        format!(
            "<< /Type /Pages /Count {} /Kids [{kids}] >>",
            page_ids.len()
        )
        .into_bytes(),
    );

    for (index, role) in active_roles.iter().copied().enumerate() {
        write_layout_font(&mut objects, index, role, layout);
    }
    for (index, image) in layout.images.iter().enumerate() {
        let (main, alpha) = image_ids[index];
        if let (Some(id), Some(data)) = (alpha, image.alpha.as_ref()) {
            objects[id as usize] = Some(stream(
                &format!(
                    "/Type /XObject /Subtype /Image /Width {} /Height {} /ColorSpace /DeviceGray /BitsPerComponent 8",
                    image.width, image.height
                ),
                data,
            ));
        }
        let filter = if image.jpeg {
            " /Filter /DCTDecode"
        } else {
            ""
        };
        let smask = alpha.map_or(String::new(), |id| format!(" /SMask {id} 0 R"));
        objects[main as usize] = Some(stream(
            &format!(
                "/Type /XObject /Subtype /Image /Width {} /Height {} /ColorSpace {} /BitsPerComponent 8{filter}{smask}",
                image.width, image.height, image.color_space
            ),
            &image.data,
        ));
    }
    objects[metadata_id as usize] = Some(stream("/Type /Metadata /Subtype /XML", metadata));

    let font_resources = active_roles
        .iter()
        .enumerate()
        .map(|(i, role)| format!("/F{} {} 0 R", role_index(*role) + 1, font_type0_id(i)))
        .collect::<Vec<_>>()
        .join(" ");
    let image_resources = image_ids
        .iter()
        .enumerate()
        .map(|(i, (id, _))| format!("/Im{} {id} 0 R", i + 1))
        .collect::<Vec<_>>()
        .join(" ");
    for (pi, page) in layout.pages.iter().enumerate() {
        let ids = &page_ids[pi];
        let content = page_content(page, layout);
        objects[ids.content as usize] = Some(stream("", &content));
        for (link, id) in page.links.iter().zip(&ids.annots) {
            let bottom = layout.geometry.height - link.y - link.h;
            objects[*id as usize] = Some(format!(
                "<< /Type /Annot /Subtype /Link /Rect [{:.3} {:.3} {:.3} {:.3}] /Border [0 0 0] /A << /S /URI /URI <{}> >> >>",
                link.x, bottom, link.x+link.w, bottom+link.h, hex(link.url.as_bytes())).into_bytes());
        }
        let annots = if ids.annots.is_empty() {
            String::new()
        } else {
            format!(
                " /Annots [{}]",
                ids.annots
                    .iter()
                    .map(|id| format!("{id} 0 R"))
                    .collect::<Vec<_>>()
                    .join(" ")
            )
        };
        objects[ids.page as usize] = Some(format!(
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {:.3} {:.3}] /Resources << /Font << {font_resources} >> /XObject << {image_resources} >> >> /Contents {} 0 R{annots} >>",
            layout.geometry.width, layout.geometry.height, ids.content).into_bytes());
    }
    finish(objects)
}

fn active_role_plan(layout: &Layout) -> (Vec<FontRole>, u32) {
    let roles = ROLES
        .iter()
        .copied()
        .filter(|role| BASE_ROLES.contains(role) || !layout.unicode[role_index(*role)].is_empty())
        .collect::<Vec<_>>();
    let next = FONT_BASE + FONT_OBJECTS * roles.len() as u32;
    (roles, next)
}

fn write_layout_font(
    objects: &mut [Option<Vec<u8>>],
    object_index: usize,
    role: FontRole,
    layout: &Layout,
) {
    write_font_objects(
        objects,
        object_index,
        role,
        layout.fonts.asset(role),
        &layout.unicode[role_index(role)],
    );
}

fn font_type0_id(index: usize) -> u32 {
    FONT_BASE + index as u32 * FONT_OBJECTS
}

fn write_font_objects(
    objects: &mut [Option<Vec<u8>>],
    object_index: usize,
    role: FontRole,
    a: &FontAsset,
    unicode: &std::collections::BTreeMap<u16, String>,
) {
    let base = font_type0_id(object_index);
    let cid = base + 1;
    let desc_id = base + 2;
    let file = base + 3;
    let cmap = base + 4;
    objects[base as usize] = Some(format!("<< /Type /Font /Subtype /Type0 /BaseFont /{} /Encoding /Identity-H /DescendantFonts [{cid} 0 R] /ToUnicode {cmap} 0 R >>", a.pdf_name).into_bytes());
    let glyphs = unicode.keys().copied().chain([0]).collect();
    let widths = glyph_widths_for(a, &glyphs)
        .iter()
        .map(|(id, width)| format!("{id} [{width}]"))
        .collect::<Vec<_>>()
        .join(" ");
    objects[cid as usize] = Some(format!("<< /Type /Font /Subtype /CIDFontType2 /BaseFont /{} /CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> /FontDescriptor {desc_id} 0 R /CIDToGIDMap /Identity /DW 1000 /W [{widths}] >>", a.pdf_name).into_bytes());
    let d = descriptor_for(a);
    let fixed = matches!(role, FontRole::Mono | FontRole::MonoBold);
    let flags = if fixed { 33 } else { 34 };
    let stem = if matches!(role, FontRole::SerifBold | FontRole::MonoBold) {
        140
    } else {
        80
    };
    objects[desc_id as usize] = Some(format!("<< /Type /FontDescriptor /FontName /{} /Flags {flags} /FontBBox [{} {} {} {}] /ItalicAngle 0 /Ascent {} /Descent {} /CapHeight {} /StemV {stem} /FontFile2 {file} 0 R >>",a.pdf_name,d.bbox[0],d.bbox[1],d.bbox[2],d.bbox[3],d.ascent,d.descent,d.cap_height).into_bytes());
    let font = subset_for(a, &glyphs);
    objects[file as usize] = Some(stream(&format!("/Length1 {}", font.len()), &font));
    objects[cmap as usize] = Some(stream("", to_unicode(a.pdf_name, unicode).as_bytes()));
}

fn to_unicode(name: &str, map: &std::collections::BTreeMap<u16, String>) -> String {
    let mut out = format!(
        "/CIDInit /ProcSet findresource begin\n12 dict begin\nbegincmap\n/CIDSystemInfo << /Registry (Adobe) /Ordering (UCS) /Supplement 0 >> def\n/CMapName /{name}-UCS def\n/CMapType 2 def\n1 begincodespacerange\n<0000> <FFFF>\nendcodespacerange\n"
    );
    let entries = map
        .iter()
        .filter(|(_, s)| !s.is_empty())
        .collect::<Vec<_>>();
    for chunk in entries.chunks(100) {
        writeln!(out, "{} beginbfchar", chunk.len()).unwrap();
        for (id, text) in chunk {
            writeln!(out, "<{id:04X}> <{}>", utf16_hex(text, false)).unwrap();
        }
        out.push_str("endbfchar\n");
    }
    out.push_str("endcmap\nCMapName currentdict /CMap defineresource pop\nend\nend\n");
    out
}

fn page_content(page: &super::layout::Page, layout: &Layout) -> Vec<u8> {
    let mut out = String::new();
    for op in &page.ops {
        match op {
            Op::Rect { x, y, w, h, gray } => {
                if let Some(color) = themed_fill(layout, *gray) {
                    write_color_rect(&mut out, color, *x, *y, *w, *h, layout.geometry.height);
                } else {
                    writeln!(
                        out,
                        "q {:.3} g {:.3} {:.3} {:.3} {:.3} re f Q",
                        gray,
                        x,
                        layout.geometry.height - y - h,
                        w,
                        h
                    )
                    .unwrap();
                }
            }
            Op::Line {
                x1,
                y1,
                x2,
                y2,
                width,
                gray,
            } => {
                if let Some(color) = themed_stroke(layout, *gray) {
                    write_color_line(
                        &mut out,
                        color,
                        (*x1, *y1),
                        (*x2, *y2),
                        *width,
                        layout.geometry.height,
                    );
                } else {
                    writeln!(
                        out,
                        "q {:.3} G {:.3} w {:.3} {:.3} m {:.3} {:.3} l S Q",
                        gray,
                        width,
                        x1,
                        layout.geometry.height - y1,
                        x2,
                        layout.geometry.height - y2
                    )
                    .unwrap();
                }
            }
            Op::Glyph(g) => write_glyph(&mut out, g, layout),
            Op::Image { index, x, y, w, h } => {
                let alt = &layout.images[*index].alt;
                writeln!(out, "/Figure << /Alt <{}> >> BDC", utf16_hex(alt, true)).unwrap();
                writeln!(
                    out,
                    "q {:.3} 0 0 {:.3} {:.3} {:.3} cm /Im{} Do Q",
                    w,
                    h,
                    x,
                    layout.geometry.height - y - h,
                    index + 1
                )
                .unwrap();
                out.push_str("EMC\n");
            }
            Op::ColorRect { x, y, w, h, color } => {
                write_color_rect(&mut out, *color, *x, *y, *w, *h, layout.geometry.height)
            }
            Op::ColorLine {
                x1,
                y1,
                x2,
                y2,
                width,
                color,
            } => write_color_line(
                &mut out,
                *color,
                (*x1, *y1),
                (*x2, *y2),
                *width,
                layout.geometry.height,
            ),
        }
    }
    out.into_bytes()
}

fn write_glyph(out: &mut String, g: &GlyphOp, layout: &Layout) {
    let shear = if g.italic { 0.212_556 } else { 0.0 };
    let font = role_index(g.role) + 1;
    if let Some(palette) = layout.appearance.palette {
        let [r, green, b] = rgb_floats(palette.ink);
        writeln!(
            out,
            "q {r:.3} {green:.3} {b:.3} rg {r:.3} {green:.3} {b:.3} RG"
        )
        .unwrap();
    }
    writeln!(
        out,
        "/Span << /ActualText <{}> >> BDC",
        utf16_hex(&g.actual, true)
    )
    .unwrap();
    if g.embolden {
        writeln!(
            out,
            "q 0.280 w BT /F{font} {:.3} Tf 2 Tr 1 0 {:.6} 1 {:.3} {:.3} Tm <{:04X}> Tj ET Q EMC",
            g.size,
            shear,
            g.x,
            layout.geometry.height - g.y,
            g.glyph_id
        )
        .unwrap();
    } else {
        writeln!(
            out,
            "BT /F{font} {:.3} Tf 1 0 {:.6} 1 {:.3} {:.3} Tm <{:04X}> Tj ET EMC",
            g.size,
            shear,
            g.x,
            layout.geometry.height - g.y,
            g.glyph_id
        )
        .unwrap();
    }
    if layout.appearance.palette.is_some() {
        out.push_str("Q\n");
    }
}

fn themed_fill(layout: &Layout, gray: f32) -> Option<PdfColor> {
    let palette = layout.appearance.palette?;
    Some(if gray <= 0.91 {
        palette.surface_strong
    } else if gray <= 0.96 {
        palette.surface
    } else {
        palette.paper
    })
}

fn themed_stroke(layout: &Layout, gray: f32) -> Option<PdfColor> {
    let palette = layout.appearance.palette?;
    Some(if gray <= 0.30 {
        palette.accent
    } else if gray <= 0.68 {
        palette.muted
    } else {
        palette.faint
    })
}

fn rgb_floats(color: PdfColor) -> [f32; 3] {
    color.0.map(|channel| channel as f32 / 255.0)
}

fn write_color_rect(
    out: &mut String,
    color: PdfColor,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    page_h: f32,
) {
    let [r, green, b] = rgb_floats(color);
    writeln!(
        out,
        "q {r:.3} {green:.3} {b:.3} rg {x:.3} {:.3} {w:.3} {h:.3} re f Q",
        page_h - y - h
    )
    .unwrap();
}

fn write_color_line(
    out: &mut String,
    color: PdfColor,
    from: (f32, f32),
    to: (f32, f32),
    width: f32,
    page_h: f32,
) {
    let ((x1, y1), (x2, y2)) = (from, to);
    let [r, green, b] = rgb_floats(color);
    writeln!(
        out,
        "q {r:.3} {green:.3} {b:.3} RG {width:.3} w {x1:.3} {:.3} m {x2:.3} {:.3} l S Q",
        page_h - y1,
        page_h - y2
    )
    .unwrap();
}

fn stream(dict: &str, data: &[u8]) -> Vec<u8> {
    let mut out = format!("<< {dict} /Length {} >>\nstream\n", data.len()).into_bytes();
    out.extend_from_slice(data);
    out.extend_from_slice(b"\nendstream");
    out
}
// Every object id `finish` walks was reserved by exactly one allocation site
// in `emit` above, paired with exactly one write over the SAME collection
// (the per-role font quintuple, the per-image main+conditional-alpha pair,
// the per-page content+annots+page triple, the two fixed catalog/pages ids,
// and metadata) — so a hole here means an id was reserved without a matching
// write, not that any document shape can trigger one today. `Err` rather
// than a panic is defense against that pairing silently drifting apart in a
// future change, with nothing but this function to catch it.
fn finish(objects: Vec<Option<Vec<u8>>>) -> Result<Vec<u8>, String> {
    let mut out = b"%PDF-1.7\n%\xE2\xE3\xCF\xD3\n".to_vec();
    let mut offsets = vec![0usize; objects.len()];
    for id in 1..objects.len() {
        offsets[id] = out.len();
        out.extend_from_slice(format!("{id} 0 obj\n").as_bytes());
        let body = objects[id]
            .as_ref()
            .ok_or_else(|| format!("PDF object {id} reserved but never written"))?;
        out.extend_from_slice(body);
        out.extend_from_slice(b"\nendobj\n");
    }
    let xref = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", objects.len()).as_bytes());
    for offset in offsets.iter().skip(1) {
        out.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            objects.len()
        )
        .as_bytes(),
    );
    Ok(out)
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02X}")).collect()
}
fn utf16_hex(text: &str, bom: bool) -> String {
    let mut out = String::new();
    if bom {
        out.push_str("FEFF");
    }
    for unit in text.encode_utf16() {
        write!(out, "{unit:04X}").unwrap();
    }
    out
}

#[cfg(test)]
mod tests;
