use std::collections::BTreeMap;

use super::super::layout::Op;
use super::fixture::NoImages;
use super::parser::Pdf;
use super::semantic::recover_page_text;
use crate::export::{PdfAppearance, PdfDocument, PdfOptions, PdfPaper};

#[derive(Debug)]
struct StoredLine {
    page: usize,
    text: String,
}

#[test]
fn stored_lines_keep_headings_with_paragraphs_and_avoid_widows_on_every_appearance() {
    let _serial = crate::testlock::serial();
    let markdown = boundary_markdown();
    let appearances = std::iter::once(PdfAppearance::Plain)
        .chain((0..crate::theme::THEMES.len()).map(PdfAppearance::CurrentTheme));

    for paper in [PdfPaper::A4, PdfPaper::Letter] {
        for appearance in appearances.clone() {
            let document =
                PdfDocument::build(&markdown, &NoImages, PdfOptions { paper, appearance }).unwrap();
            let lines = stored_lines(&document);
            assert!(
                document.page_count() >= 2,
                "{paper:?} {appearance:?} fixture"
            );

            for section in 0..18 {
                let heading = format!("H{section:02}");
                let prefix = format!("P{section:02}w");
                let heading = lines
                    .iter()
                    .find(|line| line.text.contains(&heading))
                    .unwrap_or_else(|| panic!("{paper:?} {appearance:?} omitted {heading}"));
                let paragraph = lines
                    .iter()
                    .filter(|line| line.text.contains(&prefix))
                    .collect::<Vec<_>>();
                assert!(
                    paragraph.len() >= 2,
                    "{paper:?} {appearance:?} {prefix} shaped into {:?}",
                    paragraph
                );
                assert_eq!(
                    heading.page, paragraph[0].page,
                    "{paper:?} {appearance:?} left {heading:?} behind"
                );
                assert_eq!(
                    paragraph[0].page, paragraph[1].page,
                    "{paper:?} {appearance:?} kept {heading:?} with only one paragraph line"
                );

                let mut fragments = BTreeMap::<usize, usize>::new();
                for line in paragraph {
                    *fragments.entry(line.page).or_default() += 1;
                }
                assert!(
                    fragments.values().all(|count| *count >= 2),
                    "{paper:?} {appearance:?} {prefix} has an orphan/widow: {fragments:?}"
                );
            }
        }
    }
}

#[test]
fn default_plain_pdf_keeps_every_boundary_token_selectable() {
    let _serial = crate::testlock::serial();
    let document =
        PdfDocument::build(&boundary_markdown(), &NoImages, PdfOptions::default()).unwrap();
    let bytes = document.bytes().unwrap();
    let extracted = recover_page_text(&Pdf::parse(&bytes)).concat();
    for section in 0..18 {
        for token in [
            format!("H{section:02}"),
            format!("P{section:02}w00"),
            format!("P{section:02}w27"),
        ] {
            assert!(
                extracted.contains(&token),
                "default Plain PDF omitted {token}"
            );
        }
    }
}

fn boundary_markdown() -> String {
    let mut markdown = String::from("# Pagination boundaries\n\n");
    for section in 0..18 {
        markdown.push_str(&format!("## H{section:02}\n\n"));
        for word in 0..28 {
            markdown.push_str(&format!("P{section:02}w{word:02} "));
        }
        markdown.push_str("\n\n");
    }
    markdown
}

fn stored_lines(document: &PdfDocument) -> Vec<StoredLine> {
    let mut lines = Vec::new();
    for (page, layout) in document.layout.pages.iter().enumerate() {
        let mut current_y = None;
        let mut text = String::new();
        for glyph in layout.ops.iter().filter_map(|op| match op {
            Op::Glyph(glyph) => Some(glyph),
            _ => None,
        }) {
            if current_y.is_some_and(|y| y != glyph.y.to_bits()) {
                lines.push(StoredLine { page, text });
                text = String::new();
            }
            current_y = Some(glyph.y.to_bits());
            text.push_str(&glyph.actual);
        }
        if current_y.is_some() {
            lines.push(StoredLine { page, text });
        }
    }
    lines
}
