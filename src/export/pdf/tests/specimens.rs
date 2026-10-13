//! Synthetic letter pages exercise theme continuation and provide review artifacts.
use super::fixture::NoImages;
use super::parser::Pdf;
use crate::export::{PdfAppearance, PdfDocument, PdfOptions, PdfPaper};

fn letter() -> String {
    let mut text = String::from(concat!(
        "# A letter across pages\n\nDear reader,\nWelcome to this synthetic letter.\n",
        "These authored lines stay separate when preservation is on.\n\n\n\n",
        "This is a **synthetic sample** for reviewing pagination. ",
        "It contains no personal or work information. ",
        "*Emphasis* and `inline code` remain text.\n\n",
        "- An ordinary first bullet\n- A second bullet with **strong words**\n",
        "  - A nested observation\n\n***\n\n## A quiet beginning\n\n",
    ));
    for section in 1..=12 {
        text.push_str(["\n---\n", "\n***\n", "\n___\n"][(section - 1) % 3]);
        text.push_str(&format!("\n## Section {section:02}\n\n"));
        for paragraph in 1..=3 {
            text.push_str(&format!(
                concat!(
                    "Letter paragraph {:02}.{}: a page should carry its type, color and margins ",
                    "into the next sheet. The reader can follow the thought through a natural ",
                    "break without finding clipped lines or an isolated heading. ",
                    "We keep the source as plain text and let the paper set the measure.\n",
                    "A short authored continuation belongs to the same paragraph.  \n",
                    "This explicit hard break remains in both settings.\n\n",
                ),
                section, paragraph
            ));
        }
        text.push_str(
            "- Read the continuation in order.\n- Keep the page comfortable to read.\n\n",
        );
    }
    text.push_str(concat!(
        "## Closing\n\n日本語の文字。A final **clear sentence** closes the letter.\n\n",
        "Kind regards,\nThe synthetic writer\nEND OF SYNTHETIC LETTER\n",
    ));
    text
}

#[test]
fn representative_theme_letters_keep_both_papers_and_continuation_pages() {
    let _serial = crate::testlock::serial();
    let markdown = letter();
    let output = std::env::var_os("AWL_PDF_SPECIMEN_DIR").map(std::path::PathBuf::from);
    if let Some(dir) = &output {
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(dir.join("synthetic-letter.md"), &markdown).unwrap();
    }
    let mut manifest = Vec::new();
    for name in ["Quokka", "Paperbark", "Cassowary", "Kite"] {
        let theme = crate::theme::THEMES
            .iter()
            .position(|theme| theme.name == name)
            .unwrap();
        for (paper, preserve) in [
            (PdfPaper::A4, true),
            (PdfPaper::A4, false),
            (PdfPaper::Letter, true),
            (PdfPaper::Letter, false),
        ] {
            let document = PdfDocument::build(
                &markdown,
                &NoImages,
                PdfOptions {
                    preserve_line_breaks: preserve,
                    page_numbers: true,
                    paper,
                    appearance: PdfAppearance::CurrentTheme(theme),
                },
            )
            .unwrap();
            assert!(
                document.page_count() >= 3,
                "{name} must cover continuation sheets"
            );
            assert_eq!(document.page_size(), paper.size());
            let bytes = document.bytes().unwrap();
            let pdf = Pdf::parse(&bytes);
            assert_eq!(pdf.page_ids().len(), document.page_count());
            let stem = format!(
                "{name}-{}-{}",
                paper.name(),
                if preserve { "preserved" } else { "reflowed" }
            );
            if let Some(dir) = &output {
                std::fs::write(dir.join(format!("{stem}.pdf")), &bytes).unwrap();
            }
            for page in [0, document.page_count() / 2, document.page_count() - 1] {
                let image = document.preview_page(page, 1.5).unwrap();
                assert!(
                    image.width() > 800 && image.height() > 1100,
                    "normal-size review page"
                );
                let margin_x = (document.layout.geometry.margin_x * 1.5) as u32;
                let margin_y = (document.layout.geometry.margin_y * 1.5) as u32;
                let background = image.get_pixel(0, 0);
                let body_ink = (margin_y..image.height() - margin_y).any(|y| {
                    (margin_x..image.width() - margin_x)
                        .any(|x| image.get_pixel(x, y) != background)
                });
                assert!(
                    body_ink,
                    "{name} page {} has body ink, excluding page rules",
                    page + 1
                );
                let actual: String = document.layout.pages[page]
                    .ops
                    .iter()
                    .filter_map(|op| match op {
                        super::super::layout::Op::Glyph(glyph) => Some(glyph.actual.as_str()),
                        _ => None,
                    })
                    .collect();
                assert!(actual.chars().filter(|c| c.is_alphabetic()).count() > 20);
                if page + 1 == document.page_count() {
                    assert!(
                        actual.contains("END OF SYNTHETIC LETTER"),
                        "final text survived pagination"
                    );
                }
                if let Some(dir) = &output {
                    image
                        .save(dir.join(format!("{stem}-page-{:02}.png", page + 1)))
                        .unwrap();
                }
            }
            manifest.push(serde_json::json!({
                "theme": name,
                "preserve_line_breaks": preserve,
                "paper": paper.name(),
                "pages": document.page_count(),
                "pdf": format!("{stem}.pdf"),
                "preview_source": "stored PDF layout; CPU raster",
                "preview_scale": 1.5,
            }));
        }
    }
    if let Some(dir) = &output {
        std::fs::write(
            dir.join("specimens.json"),
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
    }
}
