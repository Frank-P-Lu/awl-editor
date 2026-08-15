use std::collections::BTreeMap;

use crate::export::model::{Block, Document, Inline};

use super::super::fonts::{ROLES, role_index};
use super::parser::{Pdf, decode_utf16_hex, hex_value_after};

pub(super) fn recover_page_text(pdf: &Pdf<'_>) -> Vec<String> {
    let cmaps = ROLES
        .iter()
        .flat_map(|role| {
            let font = role_index(*role) + 1;
            pdf.objects
                .values()
                .find(|object| {
                    let text = object.text();
                    text.contains("/CMapName /")
                        && text.contains(super::super::fonts::asset(*role).pdf_name)
                })
                .map(|object| parse_cmap(object.stream().unwrap()))
                .unwrap_or_default()
                .into_iter()
                .map(move |(glyph, text)| ((font, glyph), text))
                .collect::<Vec<_>>()
        })
        .collect::<BTreeMap<_, _>>();
    pdf.page_streams()
        .into_iter()
        .map(|stream| {
            let content = std::str::from_utf8(stream).unwrap();
            let mut recovered = String::new();
            let mut actual = None;
            for line in content.lines() {
                if line.starts_with("/Span << /ActualText <") {
                    actual = Some(decode_utf16_hex(hex_value_after(line, "/ActualText <")));
                } else if line.contains("BT /F") {
                    let words = line.split_whitespace().collect::<Vec<_>>();
                    let font = words
                        .iter()
                        .find(|word| word.starts_with("/F"))
                        .unwrap()
                        .trim_start_matches("/F")
                        .parse::<usize>()
                        .unwrap();
                    let glyph = words
                        .iter()
                        .find(|word| word.starts_with('<') && word.ends_with('>'))
                        .map(|word| u16::from_str_radix(&word[1..word.len() - 1], 16).unwrap())
                        .unwrap();
                    let mapped = cmaps
                        .get(&(font, glyph))
                        .unwrap_or_else(|| panic!("F{font} glyph {glyph} lacks ToUnicode"));
                    recovered.push_str(actual.take().as_deref().unwrap_or(mapped));
                }
            }
            recovered
        })
        .collect()
}

fn parse_cmap(bytes: &[u8]) -> BTreeMap<u16, String> {
    let text = std::str::from_utf8(bytes).unwrap();
    let mut map = BTreeMap::new();
    for line in text.lines() {
        let words = line.split_whitespace().collect::<Vec<_>>();
        if words.len() == 2 && words[0].len() == 6 && words[1].starts_with('<') {
            let id = u16::from_str_radix(&words[0][1..5], 16).unwrap();
            map.insert(id, decode_utf16_hex(&words[1][1..words[1].len() - 1]));
        }
    }
    map
}

pub(super) fn expected_kinds(doc: &Document) -> Vec<&'static str> {
    fn inlines(nodes: &[Inline], out: &mut Vec<&'static str>) {
        for inline in nodes {
            out.push(inline_kind(inline));
            match inline {
                Inline::Strong(children)
                | Inline::Emphasis(children)
                | Inline::Strikethrough(children)
                | Inline::Highlight(children)
                | Inline::Link { children, .. } => inlines(children, out),
                Inline::Text(_)
                | Inline::Code(_)
                | Inline::Image { .. }
                | Inline::SoftBreak
                | Inline::HardBreak => {}
            }
        }
    }
    fn blocks(nodes: &[Block], out: &mut Vec<&'static str>) {
        for block in nodes {
            match block {
                Block::Heading {
                    inlines: children, ..
                } => {
                    out.push("heading");
                    inlines(children, out);
                }
                Block::Paragraph(children) => {
                    out.push("paragraph");
                    inlines(children, out);
                }
                Block::BlockQuote(children) => {
                    out.push("blockquote");
                    blocks(children, out);
                }
                Block::CodeBlock { .. } => {
                    out.push("codeblock");
                    out.push("text");
                }
                Block::List(list) => {
                    out.push("list");
                    for item in &list.items {
                        out.push("item");
                        blocks(&item.blocks, out);
                    }
                }
                Block::Rule => out.push("rule"),
                Block::Table(table) => {
                    out.push("table");
                    out.push("table-head");
                    row(&table.head, out);
                    out.push("table-body");
                    for cells in &table.rows {
                        row(cells, out);
                    }
                }
            }
        }
    }
    fn row(cells: &[Vec<Inline>], out: &mut Vec<&'static str>) {
        out.push("table-row");
        for cell in cells {
            out.push("table-cell");
            inlines(cell, out);
        }
    }
    let mut out = Vec::new();
    blocks(&doc.blocks, &mut out);
    out
}

pub(super) fn manifest_elements(metadata: &str) -> Vec<(String, String)> {
    metadata
        .match_indices("<awl:element id=\"")
        .map(|(start, _)| {
            let tag = &metadata[start..metadata[start..].find('>').unwrap() + start];
            (attribute(tag, "id"), attribute(tag, "kind"))
        })
        .collect()
}

pub(super) fn manifest_has(metadata: &str, kind: &str, attrs: &[(&str, &str)]) -> bool {
    metadata.match_indices("<awl:element ").any(|(start, _)| {
        let tag = &metadata[start..metadata[start..].find('>').unwrap() + start];
        attribute(tag, "kind") == kind
            && attrs
                .iter()
                .all(|(name, value)| attribute_opt(tag, name) == Some(*value))
    })
}

pub(super) fn manifest_text_nodes(metadata: &str) -> Vec<&str> {
    metadata
        .split('<')
        .filter_map(|part| part.split_once('>').map(|(_, text)| text))
        .filter(|text| !text.is_empty())
        .collect()
}

fn attribute(tag: &str, name: &str) -> String {
    attribute_opt(tag, name).unwrap().to_string()
}

fn attribute_opt<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let marker = format!("{name}=\"");
    let start = tag.find(&marker)? + marker.len();
    let end = tag[start..].find('"').unwrap() + start;
    Some(&tag[start..end])
}

pub(super) fn text_fragments(doc: &Document) -> Vec<String> {
    fn inlines(nodes: &[Inline], out: &mut Vec<String>) {
        for inline in nodes {
            match inline {
                Inline::Text(text) | Inline::Code(text) => out.push(text.clone()),
                Inline::Strong(children)
                | Inline::Emphasis(children)
                | Inline::Strikethrough(children)
                | Inline::Highlight(children)
                | Inline::Link { children, .. } => inlines(children, out),
                Inline::Image { .. } | Inline::SoftBreak | Inline::HardBreak => {}
            }
        }
    }
    fn blocks(nodes: &[Block], out: &mut Vec<String>) {
        for block in nodes {
            match block {
                Block::Heading {
                    inlines: children, ..
                }
                | Block::Paragraph(children) => inlines(children, out),
                Block::BlockQuote(children) => blocks(children, out),
                Block::CodeBlock { code, .. } => out.extend(code.lines().map(str::to_string)),
                Block::List(list) => {
                    for item in &list.items {
                        blocks(&item.blocks, out);
                    }
                }
                Block::Rule => {}
                Block::Table(table) => {
                    for cell in &table.head {
                        inlines(cell, out);
                    }
                    for row in &table.rows {
                        for cell in row {
                            inlines(cell, out);
                        }
                    }
                }
            }
        }
    }
    let mut out = Vec::new();
    blocks(&doc.blocks, &mut out);
    out
}

pub(super) fn all_inlines(doc: &Document) -> Vec<&Inline> {
    fn visit_inlines<'a>(inlines: &'a [Inline], out: &mut Vec<&'a Inline>) {
        for inline in inlines {
            out.push(inline);
            match inline {
                Inline::Strong(children)
                | Inline::Emphasis(children)
                | Inline::Strikethrough(children)
                | Inline::Highlight(children)
                | Inline::Link { children, .. } => visit_inlines(children, out),
                Inline::Text(_)
                | Inline::Code(_)
                | Inline::Image { .. }
                | Inline::SoftBreak
                | Inline::HardBreak => {}
            }
        }
    }
    fn visit_blocks<'a>(blocks: &'a [Block], out: &mut Vec<&'a Inline>) {
        for block in blocks {
            match block {
                Block::Heading { inlines, .. } | Block::Paragraph(inlines) => {
                    visit_inlines(inlines, out)
                }
                Block::BlockQuote(children) => visit_blocks(children, out),
                Block::CodeBlock { .. } | Block::Rule => {}
                Block::List(list) => {
                    for item in &list.items {
                        visit_blocks(&item.blocks, out);
                    }
                }
                Block::Table(table) => {
                    for cell in &table.head {
                        visit_inlines(cell, out);
                    }
                    for row in &table.rows {
                        for cell in row {
                            visit_inlines(cell, out);
                        }
                    }
                }
            }
        }
    }
    let mut out = Vec::new();
    visit_blocks(&doc.blocks, &mut out);
    out
}

pub(super) fn inline_kind(inline: &Inline) -> &'static str {
    match inline {
        Inline::Text(_) => "text",
        Inline::Strong(_) => "strong",
        Inline::Emphasis(_) => "emphasis",
        Inline::Strikethrough(_) => "strikethrough",
        Inline::Highlight(_) => "highlight",
        Inline::Code(_) => "code",
        Inline::Link { .. } => "link",
        Inline::Image { .. } => "image",
        Inline::SoftBreak => "softbreak",
        Inline::HardBreak => "hardbreak",
    }
}

pub(super) fn block_roster(block: &Block) -> Vec<&'static str> {
    let mut out = vec![match block {
        Block::Heading { .. } => "heading",
        Block::Paragraph(_) => "paragraph",
        Block::BlockQuote(_) => "blockquote",
        Block::CodeBlock { .. } => "codeblock",
        Block::List(_) => "list",
        Block::Rule => "rule",
        Block::Table(_) => "table",
    }];
    match block {
        Block::BlockQuote(children) => out.extend(children.iter().flat_map(block_roster)),
        Block::List(list) => out.extend(
            list.items
                .iter()
                .flat_map(|item| item.blocks.iter().flat_map(block_roster)),
        ),
        Block::Heading { .. }
        | Block::Paragraph(_)
        | Block::CodeBlock { .. }
        | Block::Rule
        | Block::Table(_) => {}
    }
    out
}

pub(super) fn page_of(pages: &[String], needle: &str) -> usize {
    pages
        .iter()
        .position(|page| page.contains(needle))
        .unwrap_or_else(|| panic!("missing {needle:?}"))
}

pub(super) fn pad_paragraphs(count: usize) -> String {
    (0..count).map(|i| format!("pad {i:02}\n\n")).collect()
}
