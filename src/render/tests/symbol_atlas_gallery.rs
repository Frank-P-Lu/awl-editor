//! SURVEY AID — not a shipped law, not wired into any production path.
//!
//! A shopping catalog for future marks (fold glyphs, ornaments, bullets,
//! world flourishes) built from what awl ALREADY bundles — nothing here
//! carries a new-font cost. Reads every `assets/fonts/*.ttf` file's own
//! `cmap` table directly via `ttf_parser` — never a Unicode chart range —
//! because a chart cannot see what a SPECIFIC font actually maps, and
//! `Junicode-Ornaments.ttf`'s best material lives almost entirely in the
//! Private Use Area, which no chart names at all.
//!
//! Renders through the BROWSER, not the real awl pipeline: each face's raw
//! bytes are embedded as a base64 `@font-face` `data:` URI in the generated
//! HTML. That is acceptable for a browse-only inventory page and is NOT a
//! substitute for the real rendering pipeline — any glyph shortlisted here
//! gets re-rendered through the fold-mark candidate gallery
//! (`captures/item-475-glyph-survey`, the actual `FontSystem`/`SwashCache`/GPU
//! path) before any taste call.
//!
//! Runs ONLY when `AWL_SYMBOL_ATLAS_OUT` names an output directory — a total
//! no-op otherwise, so no gate, filtered or unfiltered, ever writes gallery
//! files. `#[ignore]` is a second, independent gate on top of that.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt::Write as _;
use std::path::Path;

use ttf_parser::Face;

const FONTS_DIR: &str = "assets/fonts";

/// Every `.ttf` filename physically present in `assets/fonts/`, sorted — the
/// DIRECTORY, not a hand-kept roster (matches `font_licence.rs`'s own
/// `bundled_ttf_files`, duplicated rather than shared across two otherwise
/// independent survey/law files).
fn bundled_ttf_files() -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(FONTS_DIR)
        .unwrap_or_else(|e| panic!("{FONTS_DIR} must be readable from the repo root: {e}"))
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| name.ends_with(".ttf"))
        .collect();
    names.sort();
    names
}

/// Unicode BLOCKS considered in scope for a fold/ornament/bullet/flourish
/// shopping catalog, used only to LABEL and GROUP codepoints a face's own
/// cmap already reported — never to decide whether a face "should" cover a
/// codepoint. Deliberately excludes Basic Latin/Latin-1 (plain ASCII
/// punctuation), Superscripts/Subscripts, Currency Symbols, Number Forms,
/// pure Mathematical Operators, Box Drawing/Block Elements, Braille, and the
/// bulk CJK Unified Ideographs/Kana/Hangul blocks — all real but not
/// "ornament" material, and the ideograph blocks alone would drown the
/// catalog in tens of thousands of rows per CJK face. Also excludes
/// Halfwidth and Fullwidth Forms (U+FF00-FFEF): measured directly against
/// the bundled CJK faces, that block is 85-95% fullwidth duplicates of plain
/// ASCII letters/digits (Ａ-Ｚ, ０-９) rather than ornament material — it
/// alone would have inflated e.g. Noto Sans JP from 65 to 157 in-scope
/// codepoints, more than double, for zero fold-mark-relevant content.
const SYMBOL_BLOCKS: &[(u32, u32, &str)] = &[
    (0x2000, 0x206F, "General Punctuation"),
    (0x2100, 0x214F, "Letterlike Symbols"),
    (0x2190, 0x21FF, "Arrows"),
    (0x2300, 0x23FF, "Miscellaneous Technical"),
    (0x2400, 0x243F, "Control Pictures"),
    (0x2440, 0x245F, "Optical Character Recognition"),
    (0x2460, 0x24FF, "Enclosed Alphanumerics"),
    (0x25A0, 0x25FF, "Geometric Shapes"),
    (0x2600, 0x26FF, "Miscellaneous Symbols"),
    (0x2700, 0x27BF, "Dingbats"),
    (0x27F0, 0x27FF, "Supplemental Arrows-A"),
    (0x2900, 0x297F, "Supplemental Arrows-B"),
    (0x2B00, 0x2BFF, "Miscellaneous Symbols and Arrows"),
    (0x3000, 0x303F, "CJK Symbols and Punctuation"),
    (0x3200, 0x32FF, "Enclosed CJK Letters and Months"),
    (0x3300, 0x33FF, "CJK Compatibility"),
    (0xFE10, 0xFE1F, "Vertical Forms"),
    (0xFE30, 0xFE4F, "CJK Compatibility Forms"),
    (0xFE50, 0xFE6F, "Small Form Variants"),
    (0xE000, 0xF8FF, "Private Use Area"),
];

/// The block name a codepoint falls in, or `None` when it sits outside every
/// block above (only reachable via `AwlMarks.ttf`'s scope exemption below).
fn block_for(cp: u32) -> Option<&'static str> {
    SYMBOL_BLOCKS
        .iter()
        .find(|(lo, hi, _)| cp >= *lo && cp <= *hi)
        .map(|(_, _, name)| *name)
}

/// `AwlMarks.ttf` is awl's OWN composed symbol face — every codepoint it maps
/// is already, by construction, a symbol/ornament/keycap glyph, so its scope
/// is its full cmap rather than the block filter above.
fn is_exempt_face(filename: &str) -> bool {
    filename == "AwlMarks.ttf"
}

/// This face's own `name`-table family (nameID 16 Typographic Family first,
/// then nameID 1 Family), the same table `fontdb`/the app's real `FontSystem`
/// read — never a filename guess. Falls back to the filename stem only when
/// a face carries neither record.
fn family_name(face: &Face, filename: &str) -> String {
    for want in [
        ttf_parser::name_id::TYPOGRAPHIC_FAMILY,
        ttf_parser::name_id::FAMILY,
    ] {
        for name in face.names() {
            if name.name_id == want
                && let Some(s) = name.to_string()
                && !s.trim().is_empty()
            {
                return s;
            }
        }
    }
    filename.trim_end_matches(".ttf").to_string()
}

/// Every codepoint this face's `cmap` maps, read directly from the table
/// (unioned across every subtable the face carries) — a fontTools-style
/// read, not a chart lookup.
fn cmap_codepoints(face: &Face) -> HashSet<u32> {
    let mut cps = HashSet::new();
    if let Some(cmap) = face.tables().cmap {
        for subtable in cmap.subtables {
            subtable.codepoints(|cp| {
                cps.insert(cp);
            });
        }
    }
    cps
}

/// One glyph entry in the catalog: its own name (from the face's `post`/CFF
/// glyph-name table where present — real ground truth, never invented) and
/// which Unicode block it groups under.
struct GlyphEntry {
    codepoint: u32,
    glyph_name: Option<String>,
    block: &'static str,
}

/// One bundled face's in-scope glyph roster, plus the raw bytes needed to
/// embed it as a browser `@font-face`.
struct FaceCatalog {
    filename: String,
    display_name: String,
    css_family: String,
    bytes: Vec<u8>,
    glyphs: Vec<GlyphEntry>,
}

/// Minimal base64 (standard alphabet, `=` padding) — no new crate dependency
/// for a single-use `data:` URI encoder in a `#[ignore]`d survey tool.
fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(CHARS[((n >> 18) & 0x3F) as usize] as char);
        out.push(CHARS[((n >> 12) & 0x3F) as usize] as char);
        out.push(if chunk.len() > 1 {
            CHARS[((n >> 6) & 0x3F) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            CHARS[(n & 0x3F) as usize] as char
        } else {
            '='
        });
    }
    out
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[test]
#[ignore]
fn symbol_atlas_gallery() {
    let Ok(out_dir) = std::env::var("AWL_SYMBOL_ATLAS_OUT") else {
        eprintln!(
            "skipping symbol_atlas_gallery: set AWL_SYMBOL_ATLAS_OUT=<dir> to run \
             (see gallery/item-486-symbol-atlas/shoot.sh)"
        );
        return;
    };
    std::fs::create_dir_all(&out_dir).expect("gallery out dir creatable");

    let files = bundled_ttf_files();
    assert!(
        files.len() >= 40,
        "{FONTS_DIR}/*.ttf looks empty or the directory read failed — found {} files, \
         expected the ~45-face roster (non-vacuity floor)",
        files.len()
    );

    let byte_bufs: Vec<Vec<u8>> = files
        .iter()
        .map(|f| {
            std::fs::read(Path::new(FONTS_DIR).join(f))
                .unwrap_or_else(|e| panic!("{f}: could not read: {e}"))
        })
        .collect();
    let faces: Vec<Face> = files
        .iter()
        .zip(&byte_bufs)
        .map(|(f, b)| {
            Face::parse(b, 0).unwrap_or_else(|e| panic!("{f}: ttf_parser could not parse: {e:?}"))
        })
        .collect();

    // Global roster: which faces carry a given codepoint at all — checked
    // directly against every face's own `glyph_index`, not derived from the
    // per-face scoped list below, so a face outside this item's block filter
    // (e.g. only reachable via AwlMarks's exemption) still shows correctly
    // in every OTHER face's "also carried by" roster.
    let mut all_in_scope_cps: BTreeSet<u32> = BTreeSet::new();
    let mut per_face_cps: Vec<HashSet<u32>> = Vec::with_capacity(faces.len());
    for (filename, face) in files.iter().zip(&faces) {
        let raw = cmap_codepoints(face);
        let exempt = is_exempt_face(filename);
        let scoped: HashSet<u32> = raw
            .into_iter()
            .filter(|cp| exempt || block_for(*cp).is_some())
            .collect();
        all_in_scope_cps.extend(scoped.iter().copied());
        per_face_cps.push(scoped);
    }

    assert!(
        !all_in_scope_cps.is_empty(),
        "zero symbol-range codepoints found across all {} bundled faces — \
         the cmap read or the block filter is broken (non-vacuity floor)",
        files.len()
    );

    let mut roster: BTreeMap<u32, Vec<String>> = BTreeMap::new();
    for &cp in &all_in_scope_cps {
        let Some(ch) = char::from_u32(cp) else {
            continue;
        };
        let mut faces_with_it = Vec::new();
        for (filename, face) in files.iter().zip(&faces) {
            if face.glyph_index(ch).is_some() {
                faces_with_it.push(filename.clone());
            }
        }
        roster.insert(cp, faces_with_it);
    }

    let mut catalogs: Vec<FaceCatalog> = Vec::new();
    for (i, (filename, face)) in files.iter().zip(&faces).enumerate() {
        let scoped = &per_face_cps[i];
        if scoped.is_empty() {
            continue;
        }
        let mut glyphs: Vec<GlyphEntry> = Vec::new();
        for &cp in scoped {
            let Some(ch) = char::from_u32(cp) else {
                continue;
            };
            let Some(gid) = face.glyph_index(ch) else {
                continue;
            };
            let glyph_name = face.glyph_name(gid).map(|s| s.to_string());
            let block = block_for(cp).unwrap_or("Other (AwlMarks-exempt, outside listed blocks)");
            glyphs.push(GlyphEntry {
                codepoint: cp,
                glyph_name,
                block,
            });
        }
        glyphs.sort_by_key(|g| g.codepoint);
        catalogs.push(FaceCatalog {
            filename: filename.clone(),
            display_name: family_name(face, filename),
            css_family: format!("symatlas-face-{i}"),
            bytes: byte_bufs[i].clone(),
            glyphs,
        });
    }
    catalogs.sort_by(|a, b| a.filename.cmp(&b.filename));

    eprintln!(
        "symbol_atlas_gallery: {} faces carry symbol-range material out of {} bundled",
        catalogs.len(),
        files.len()
    );
    for c in &catalogs {
        eprintln!(
            "  {:<28} {:>4} in-scope codepoints",
            c.filename,
            c.glyphs.len()
        );
    }
    eprintln!(
        "  {} distinct in-scope codepoints across the whole roster",
        all_in_scope_cps.len()
    );

    let html = render_html(&catalogs, &roster);
    let path = format!("{out_dir}/symbol-atlas.html");
    std::fs::write(&path, &html).unwrap_or_else(|e| panic!("could not write {path}: {e}"));
    eprintln!("wrote {path} ({} bytes)", html.len());
}

fn render_html(catalogs: &[FaceCatalog], roster: &BTreeMap<u32, Vec<String>>) -> String {
    let mut out = String::new();
    out.push_str(
        "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n\
         <title>awl symbol atlas — bundled-face shopping catalog</title>\n<style>\n",
    );
    for c in catalogs {
        let b64 = base64_encode(&c.bytes);
        writeln!(
            out,
            "@font-face {{ font-family: '{}'; \
             src: url(data:font/ttf;base64,{}) format('truetype'); \
             font-display: swap; }}",
            c.css_family, b64
        )
        .unwrap();
    }
    out.push_str(
        "\nbody { font-family: -apple-system, sans-serif; margin: 2rem; background: #f7f4ee; \
         color: #1b1b1b; }\n\
         h1 { font-size: 1.4rem; }\n\
         .meta { color: #555; font-size: 0.85rem; margin-bottom: 2rem; }\n\
         .toc { columns: 3; margin-bottom: 2rem; font-size: 0.85rem; }\n\
         section.face { border-top: 3px solid #222; margin-top: 2.5rem; padding-top: 0.5rem; }\n\
         h2 { font-size: 1.1rem; margin-bottom: 0; }\n\
         h2 .file { color: #777; font-weight: normal; font-size: 0.8rem; }\n\
         h3.block { font-size: 0.85rem; text-transform: uppercase; letter-spacing: 0.04em; \
         color: #555; margin: 1.2rem 0 0.4rem; }\n\
         .grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(150px, 1fr)); \
         gap: 0.5rem; }\n\
         .cell { border: 1px solid #ddd; border-radius: 6px; padding: 0.5rem; background: #fff; }\n\
         .glyph { font-size: 2.4rem; line-height: 1.1; height: 2.6rem; display: flex; \
         align-items: center; }\n\
         .cp { font-family: ui-monospace, monospace; font-size: 0.72rem; color: #333; }\n\
         .name { font-size: 0.7rem; color: #666; overflow-wrap: anywhere; }\n\
         .roster { font-size: 0.65rem; color: #888; margin-top: 0.2rem; }\n\
         </style>\n</head>\n<body>\n",
    );
    writeln!(
        out,
        "<h1>awl symbol atlas — bundled-face shopping catalog</h1>"
    )
    .unwrap();
    writeln!(
        out,
        "<p class=\"meta\">Grouped by face, then by Unicode block. Glyphs rendered via \
         browser @font-face from the SAME already-bundled .ttf bytes awl embeds — a browse \
         inventory, not the real rendering pipeline. {} faces carry symbol-range material; \
         {} distinct codepoints total. Regenerate: \
         <code>sh gallery/item-486-symbol-atlas/shoot.sh</code>.</p>",
        catalogs.len(),
        roster.len()
    )
    .unwrap();

    out.push_str("<div class=\"toc\"><strong>Faces:</strong><ul>\n");
    for c in catalogs {
        writeln!(
            out,
            "<li><a href=\"#face-{}\">{}</a> ({})</li>",
            c.css_family,
            escape_html(&c.display_name),
            c.glyphs.len()
        )
        .unwrap();
    }
    out.push_str("</ul></div>\n");

    for c in catalogs {
        writeln!(
            out,
            "<section class=\"face\" id=\"face-{}\">\n<h2>{} <span class=\"file\">{}</span></h2>",
            c.css_family,
            escape_html(&c.display_name),
            escape_html(&c.filename)
        )
        .unwrap();

        let mut by_block: BTreeMap<&'static str, Vec<&GlyphEntry>> = BTreeMap::new();
        for g in &c.glyphs {
            by_block.entry(g.block).or_default().push(g);
        }
        // Stable, human order: SYMBOL_BLOCKS' own declared order, then any
        // exemption-only bucket last.
        let mut block_order: Vec<&'static str> =
            SYMBOL_BLOCKS.iter().map(|(_, _, name)| *name).collect();
        block_order.push("Other (AwlMarks-exempt, outside listed blocks)");

        for block in block_order {
            let Some(entries) = by_block.get(block) else {
                continue;
            };
            writeln!(
                out,
                "<h3 class=\"block\">{} ({})</h3>",
                block,
                entries.len()
            )
            .unwrap();
            out.push_str("<div class=\"grid\">\n");
            for g in entries {
                let name = g.glyph_name.as_deref().unwrap_or("(unnamed)");
                let also: Vec<&str> = roster
                    .get(&g.codepoint)
                    .map(|v| {
                        v.iter()
                            .filter(|f| f.as_str() != c.filename)
                            .map(|s| s.as_str())
                            .collect()
                    })
                    .unwrap_or_default();
                let roster_line = if also.is_empty() {
                    "only this face".to_string()
                } else {
                    format!("also: {}", also.join(", "))
                };
                writeln!(
                    out,
                    "<div class=\"cell\"><div class=\"glyph\" style=\"font-family:'{}';\">\
                     &#x{:X};</div>\
                     <div class=\"cp\">U+{:04X}</div><div class=\"name\">{}</div>\
                     <div class=\"roster\">{}</div></div>",
                    c.css_family,
                    g.codepoint,
                    g.codepoint,
                    escape_html(name),
                    escape_html(&roster_line)
                )
                .unwrap();
            }
            out.push_str("</div>\n");
        }
        out.push_str("</section>\n");
    }

    out.push_str("</body>\n</html>\n");
    out
}
