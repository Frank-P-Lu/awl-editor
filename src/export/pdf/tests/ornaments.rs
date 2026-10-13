use super::super::{fonts::FontRole, layout::Op};
use super::fixture::NoImages;
use crate::export::{PdfAppearance, PdfDocument, PdfOptions, PdfPaper};

#[test]
fn themed_dividers_keep_all_three_editor_marks_in_both_papers() {
    let _serial = crate::testlock::serial();
    for (index, theme) in crate::theme::THEMES.iter().enumerate() {
        for paper in [PdfPaper::A4, PdfPaper::Letter] {
            let document = PdfDocument::build(
                "Before.\n\n---\n\nMiddle.\n\n* * *\n\nAfter.\n\n___\n",
                &NoImages,
                PdfOptions {
                    paper,
                    appearance: PdfAppearance::CurrentTheme(index),
                    page_numbers: false,
                },
            )
            .unwrap();
            let glyphs: Vec<_> = document
                .layout
                .pages
                .iter()
                .flat_map(|p| &p.ops)
                .filter_map(|op| match op {
                    Op::Glyph(g) if g.role == FontRole::Ornament => Some(g),
                    _ => None,
                })
                .collect();
            let actual: String = glyphs.iter().map(|g| g.actual.as_str()).collect();
            assert_eq!(
                actual,
                format!(
                    "{}{}{}",
                    theme.ornaments.dash, theme.ornaments.star, theme.ornaments.underscore
                ),
                "{}",
                theme.name
            );
            assert_eq!(
                document.layout.fonts.asset(FontRole::Ornament).family,
                theme.ornament_face
            );
            assert!(glyphs.iter().all(|g| g.glyph_id != 0
                && g.x > 0.0
                && g.x < paper.size().0
                && g.y > 0.0
                && g.y < document.layout.geometry.bottom()));
            assert!(glyphs.iter().all(|g| g.color == Some(theme.muted.into())));
            let bytes = document.bytes().unwrap();
            assert!(String::from_utf8_lossy(&bytes).contains("AWLMarks"));
            let recovered =
                super::semantic::recover_page_text(&super::parser::Pdf::parse(&bytes)).join("");
            for mark in [
                theme.ornaments.dash,
                theme.ornaments.star,
                theme.ornaments.underscore,
            ] {
                assert!(
                    recovered.contains(mark),
                    "{}: emitted PDF must recover {mark}",
                    theme.name
                );
            }
            document.preview_page(0, 1.0).unwrap();
        }
    }
}

#[test]
fn plain_dividers_preserve_neutral_rules_and_code_stays_literal() {
    let _serial = crate::testlock::serial();
    let plain =
        PdfDocument::build("Before.\n\n***\n\nAfter.", &NoImages, PdfOptions::default()).unwrap();
    assert!(
        plain.layout.pages[0]
            .ops
            .iter()
            .any(|op| matches!(op, Op::Line { .. }))
    );
    assert!(
        !plain.layout.pages[0]
            .ops
            .iter()
            .any(|op| matches!(op, Op::Glyph(g) if g.role == FontRole::Ornament))
    );
    let code = PdfDocument::build(
        "```\n***\n```",
        &NoImages,
        PdfOptions {
            appearance: PdfAppearance::CurrentTheme(0),
            ..PdfOptions::default()
        },
    )
    .unwrap();
    assert!(
        !code.layout.pages[0]
            .ops
            .iter()
            .any(|op| matches!(op, Op::Glyph(g) if g.role == FontRole::Ornament))
    );
}

#[test]
fn nested_and_spaced_dividers_after_frontmatter_keep_their_syntax() {
    let _serial = crate::testlock::serial();
    use crate::export::model::Block;
    use crate::markdown::BreakKind;
    let doc = crate::export::model::parse(
        "---\ntitle: Example\n---\n\n  * * *\n\n> ___\n\n- Item\n\n  - - -\n",
    );
    assert!(matches!(doc.blocks[0], Block::Rule(BreakKind::Star)));
    let Block::BlockQuote(quote) = &doc.blocks[1] else {
        panic!("expected quote")
    };
    assert!(matches!(quote[0], Block::Rule(BreakKind::Underscore)));
    let Block::List(list) = &doc.blocks[2] else {
        panic!("expected list")
    };
    assert!(
        list.items[0]
            .blocks
            .iter()
            .any(|b| matches!(b, Block::Rule(BreakKind::Dash)))
    );
}

#[test]
fn a_divider_near_the_bottom_moves_as_one_group() {
    let _serial = crate::testlock::serial();
    let (index, theme) = crate::theme::THEMES
        .iter()
        .enumerate()
        .max_by_key(|(_, t)| t.ornaments.star.chars().count())
        .unwrap();
    for paper in [PdfPaper::A4, PdfPaper::Letter] {
        let options = PdfOptions {
            paper,
            appearance: PdfAppearance::CurrentTheme(index),
            page_numbers: false,
        };
        let mut moved = false;
        for count in 20..45 {
            let prefix = "A short paragraph.\n\n".repeat(count);
            let before = PdfDocument::build(&prefix, &NoImages, options).unwrap();
            let after = PdfDocument::build(&format!("{prefix}***\n"), &NoImages, options).unwrap();
            let pages: Vec<_> = after
                .layout
                .pages
                .iter()
                .enumerate()
                .filter_map(|(index, p)| {
                    let marks: Vec<_> = p
                        .ops
                        .iter()
                        .filter_map(|op| match op {
                            Op::Glyph(g) if g.role == FontRole::Ornament => Some(g),
                            _ => None,
                        })
                        .collect();
                    if marks.is_empty() {
                        return None;
                    }
                    assert_eq!(
                        marks.iter().map(|g| g.actual.as_str()).collect::<String>(),
                        theme.ornaments.star
                    );
                    assert!(marks.iter().all(|g| g.y >= after.layout.geometry.margin_y
                        && g.y < after.layout.geometry.bottom()));
                    Some(index)
                })
                .collect();
            assert_eq!(pages.len(), 1);
            moved |= pages[0] == before.page_count();
        }
        assert!(moved, "fixture must force an ornament onto the next page");
    }
}
