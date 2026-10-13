//! Source newlines and blank paragraphs remain observable through real PDF layout.
use super::super::layout::Op;
use super::fixture::NoImages;
use crate::export::{PdfAppearance, PdfDocument, PdfOptions, PdfPaper};

fn document(text: &str, preserve: bool, paper: PdfPaper) -> PdfDocument {
    PdfDocument::build(
        text,
        &NoImages,
        PdfOptions {
            preserve_line_breaks: preserve,
            paper,
            ..PdfOptions::default()
        },
    )
    .unwrap()
}
fn baselines(doc: &PdfDocument) -> Vec<f32> {
    let mut ys = Vec::new();
    for op in &doc.layout.pages[0].ops {
        if let Op::Glyph(g) = op
            && !ys.contains(&g.y)
        {
            ys.push(g.y);
        }
    }
    ys
}
#[test]
fn source_newlines_toggle_but_explicit_breaks_and_blank_rows_survive_both_modes() {
    let _serial = crate::testlock::serial();
    for paper in [PdfPaper::A4, PdfPaper::Letter] {
        let on = document("Alpha\nBravo\nCharlie  \nDelta\\\nEcho", true, paper);
        let off = document("Alpha\nBravo\nCharlie  \nDelta\\\nEcho", false, paper);
        assert_eq!(baselines(&on).len(), 5);
        assert_eq!(baselines(&off).len(), 3);
        for preserve in [false, true] {
            let one = document("Alpha\n\nBravo", preserve, paper);
            let three = document("Alpha\n\n\n\nBravo", preserve, paper);
            assert!((baselines(&three)[1] - baselines(&one)[1] - 32.0).abs() < 0.001);
            let hard = document("Alpha  \nBravo", preserve, paper);
            assert_eq!(baselines(&hard).len(), 2);
            let long = document(
                &"A long line must wrap within the printable paper width. ".repeat(50),
                preserve,
                paper,
            );
            assert!(baselines(&long).len() > 10);
            for page in &long.layout.pages {
                for op in &page.ops {
                    if let Op::Glyph(g) = op {
                        assert!(g.x >= long.layout.geometry.margin_x - 0.1);
                        assert!(g.x < long.layout.geometry.width - long.layout.geometry.margin_x);
                    }
                }
            }
        }
    }
}
#[test]
fn source_spacing_does_not_duplicate_code_blank_lines_or_drop_nested_prose() {
    let _serial = crate::testlock::serial();
    use crate::export::model::{Block, parse_pdf};
    let doc = parse_pdf(
        concat!(
            "First\n\n\nSecond\n\n```\na\n\nb\n```\n\n",
            "- item\n  continuation\n\n  paragraph\n\n> quoted\n>\n> more\n"
        ),
        true,
    );
    assert!(
        matches!(doc.blocks[1], Block::BlankLines(2)),
        "{:?}",
        doc.blocks
    );
    let code = doc
        .blocks
        .iter()
        .find_map(|b| {
            if let Block::CodeBlock { code, .. } = b {
                Some(code)
            } else {
                None
            }
        })
        .unwrap();
    assert_eq!(code, "a\n\nb");
    assert!(doc.blocks.iter().any(|b| matches!(b, Block::List(_))));
    assert!(doc.blocks.iter().any(|b| matches!(b, Block::BlockQuote(_))));
}
#[test]
fn every_theme_has_clean_top_margins_and_keeps_document_ornaments() {
    let _serial = crate::testlock::serial();
    for theme in 0..crate::theme::THEMES.len() {
        for preserve in [false, true] {
            let doc = PdfDocument::build(
                &"# Heading\n\nText\nwith a line break.\n\n***\n\n".repeat(20),
                &NoImages,
                PdfOptions {
                    appearance: PdfAppearance::CurrentTheme(theme),
                    preserve_line_breaks: preserve,
                    ..PdfOptions::default()
                },
            )
            .unwrap();
            for page in &doc.layout.pages {
                let top = doc.layout.geometry.margin_y;
                assert!(
                    !page
                        .ops
                        .iter()
                        .any(|op| { matches!(op, Op::ColorLine { y1, .. } if *y1 < top) })
                );
            }
            assert!(doc.layout.pages.iter().flat_map(|p| &p.ops).any(
                |op| matches!(op, Op::Glyph(g) if g.role == super::super::fonts::FontRole::Ornament)
            ));
        }
    }
}

#[test]
fn spaced_nested_quote_prefixes_preserve_empty_source_rows() {
    let _serial = crate::testlock::serial();
    use crate::export::model::{Block, parse_pdf};
    let doc = parse_pdf("> > Alpha\n> >\n> >\n> > Bravo", true);
    let Block::BlockQuote(outer) = &doc.blocks[0] else {
        panic!("{:?}", doc.blocks)
    };
    let Block::BlockQuote(inner) = &outer[0] else {
        panic!("{outer:?}")
    };
    assert!(matches!(inner[1], Block::BlankLines(2)), "{inner:?}");
}

#[test]
fn blank_rows_after_list_items_stay_in_source_order() {
    let _serial = crate::testlock::serial();
    use crate::export::model::{Block, parse_pdf};
    for text in [
        "- one\n\n\n- two",
        "- one\n\n\nAfter",
        "- outer\n  - one\n\n\n  - two",
    ] {
        for preserve in [false, true] {
            let doc = parse_pdf(text, preserve);
            let Block::List(list) = &doc.blocks[0] else {
                panic!("{:?}", doc.blocks)
            };
            let first = &list.items[0].blocks;
            let blocks = first
                .iter()
                .find_map(|b| {
                    if let Block::List(inner) = b {
                        Some(&inner.items[0].blocks)
                    } else {
                        None
                    }
                })
                .unwrap_or(first);
            assert!(
                matches!(blocks[0], Block::Paragraph(_)),
                "{text}: {blocks:?}"
            );
            assert!(
                matches!(blocks.last(), Some(Block::BlankLines(2))),
                "{text}: {blocks:?}"
            );
        }
    }
}
