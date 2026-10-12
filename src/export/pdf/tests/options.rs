use ttf_parser::{Face, Permissions};

use super::super::fonts::{FontRole, FontRoster, ROLES, embedding_is_permitted};
use super::super::layout::Op;
use super::fixture::{Images, NoImages};
use super::parser::{Pdf, reference};
use crate::export::{PdfAppearance, PdfDocument, PdfOptions, PdfPaper};

#[test]
fn options_cover_both_papers_and_reject_unknown_theme_indices() {
    let _serial = crate::testlock::serial();
    let a4 = PdfDocument::build("# A4\n\nBody.", &NoImages, PdfOptions::default()).unwrap();
    assert_eq!(a4.page_size(), (595.276, 841.890));
    assert_eq!(a4.page_count(), 1);

    let letter = PdfDocument::build(
        "# Letter\n\nBody.",
        &NoImages,
        PdfOptions {
            paper: PdfPaper::Letter,
            appearance: PdfAppearance::Plain,
        },
    )
    .unwrap();
    assert_eq!(letter.page_size(), (612.0, 792.0));
    let bytes = letter.bytes().unwrap();
    let pdf = Pdf::parse(&bytes);
    for page in pdf.page_ids() {
        assert!(
            pdf.object(page)
                .text()
                .contains("/MediaBox [0 0 612.000 792.000]")
        );
    }

    let error = PdfDocument::build(
        "Body.",
        &NoImages,
        PdfOptions {
            paper: PdfPaper::A4,
            appearance: PdfAppearance::CurrentTheme(crate::theme::THEMES.len()),
        },
    )
    .err()
    .expect("an out-of-range theme must not silently substitute a face");
    assert!(error.contains("outside"));
}

#[test]
fn every_theme_has_embeddable_body_mono_and_japanese_faces_and_legible_ink() {
    let _serial = crate::testlock::serial();
    for theme in &crate::theme::THEMES {
        let roster = FontRoster::for_theme(theme).unwrap_or_else(|error| {
            panic!(
                "{} cannot build a closed PDF font roster: {error}",
                theme.name
            )
        });
        assert_eq!(roster.asset(FontRole::Serif).family, theme.font);
        assert_eq!(roster.asset(FontRole::SerifBold).family, theme.font);
        assert_eq!(roster.asset(FontRole::Mono).family, theme.mono);
        assert_eq!(roster.asset(FontRole::MonoBold).family, theme.mono);
        for role in ROLES {
            let asset = roster.asset(role);
            let face = Face::parse(asset.bytes, 0)
                .unwrap_or_else(|_| panic!("{} {} must parse", theme.name, asset.family));
            let families = face
                .names()
                .into_iter()
                .filter(|name| {
                    matches!(
                        name.name_id,
                        ttf_parser::name_id::FAMILY | ttf_parser::name_id::TYPOGRAPHIC_FAMILY
                    )
                })
                .filter_map(|name| name.to_string())
                .collect::<std::collections::BTreeSet<_>>();
            assert!(
                families.contains(asset.family),
                "{} {:?} maps {} onto the wrong bundled bytes: {families:?}",
                theme.name,
                role,
                asset.family
            );
            let expected_permission =
                if matches!(asset.family, "Monaspace Xenon" | "iA Writer Quattro S") {
                    Permissions::Editable
                } else {
                    Permissions::Installable
                };
            assert_eq!(
                face.permissions(),
                Some(expected_permission),
                "{} {} fsType must match the reviewed bundled metadata",
                theme.name,
                asset.family
            );
            assert_eq!(
                asset.weight,
                face.weight().to_number(),
                "{} {:?} must request the bundled face's authentic weight",
                theme.name,
                role
            );
            assert!(
                face.is_subsetting_allowed() && face.is_outline_embedding_allowed(),
                "{} {} must permit subset outline embedding",
                theme.name,
                asset.family
            );
            assert!(
                embedding_is_permitted(asset),
                "{} {} must permit PDF outline embedding",
                theme.name,
                asset.family
            );
            assert!(
                face.tables().glyf.is_some(),
                "{} {} glyf",
                theme.name,
                asset.family
            );
            assert!(
                face.tables().hmtx.is_some(),
                "{} {} hmtx",
                theme.name,
                asset.family
            );
        }
        for (surface, label) in [
            (theme.base_100, "paper"),
            (theme.base_200, "code surface"),
            (theme.base_300, "strong surface"),
        ] {
            let ratio = contrast(theme.base_content, surface);
            assert!(
                ratio >= 4.5,
                "{} body ink on {label} is only {ratio:.2}:1",
                theme.name
            );
        }
    }
}

#[test]
fn ibm_plex_mono_light_and_bold_shape_into_their_owned_pdf_roles() {
    let _serial = crate::testlock::serial();
    let theme = crate::theme::THEMES
        .iter()
        .position(|theme| theme.name == "Quokka")
        .unwrap();
    let document = PdfDocument::build(
        "`zzzz` and **`qqqq`**",
        &NoImages,
        PdfOptions {
            paper: PdfPaper::A4,
            appearance: PdfAppearance::CurrentTheme(theme),
        },
    )
    .unwrap();
    let glyphs = document.layout.pages[0]
        .ops
        .iter()
        .filter_map(|op| match op {
            Op::Glyph(glyph) => Some(glyph),
            _ => None,
        });
    let roles = glyphs
        .filter(|glyph| glyph.actual == "z" || glyph.actual == "q")
        .map(|glyph| (glyph.actual.as_str(), glyph.role))
        .collect::<Vec<_>>();
    assert!(roles.iter().any(|role| *role == ("z", FontRole::Mono)));
    assert!(roles.iter().any(|role| *role == ("q", FontRole::MonoBold)));
    assert!(roles.iter().all(|(text, role)| {
        *role
            == if *text == "z" {
                FontRole::Mono
            } else {
                FontRole::MonoBold
            }
    }));
    let regular = document.layout.fonts.asset(FontRole::Mono);
    let bold = document.layout.fonts.asset(FontRole::MonoBold);
    assert_eq!((regular.family, regular.weight), ("IBM Plex Mono", 300));
    assert_eq!((bold.family, bold.weight), ("IBM Plex Mono", 700));
}

#[test]
fn themed_emboldened_japanese_sets_fill_and_stroke_to_the_same_legible_ink() {
    let _serial = crate::testlock::serial();
    let theme_index = crate::theme::THEMES
        .iter()
        .position(|theme| theme.name == "Cassowary")
        .unwrap();
    let theme = &crate::theme::THEMES[theme_index];
    let document = PdfDocument::build(
        "**日**",
        &NoImages,
        PdfOptions {
            paper: PdfPaper::A4,
            appearance: PdfAppearance::CurrentTheme(theme_index),
        },
    )
    .unwrap();
    let bytes = document.bytes().unwrap();
    let pdf = Pdf::parse(&bytes);
    let content = std::str::from_utf8(pdf.page_streams()[0]).unwrap();
    let lines = content.lines().collect::<Vec<_>>();
    let actual = lines
        .iter()
        .position(|line| *line == "/Span << /ActualText <FEFF65E5> >> BDC")
        .expect("Japanese glyph keeps ActualText");
    let channel = |value: u8| value as f32 / 255.0;
    let (r, green, b) = (
        channel(theme.base_content.r),
        channel(theme.base_content.g),
        channel(theme.base_content.b),
    );
    assert_eq!(
        lines[actual - 1],
        format!("q {r:.3} {green:.3} {b:.3} rg {r:.3} {green:.3} {b:.3} RG")
    );
    assert!(lines[actual + 1].contains(" 2 Tr "));
}

#[test]
fn metadata_identifies_paper_dimensions_and_the_snapshot_appearance() {
    let _serial = crate::testlock::serial();
    let plain = PdfDocument::build("Body.", &NoImages, PdfOptions::default()).unwrap();
    let meta = metadata(&plain.bytes().unwrap());
    assert!(meta.contains("paper=\"A4\""));
    assert!(meta.contains("width-pt=\"595.276\""));
    assert!(meta.contains("height-pt=\"841.890\""));
    assert!(meta.contains("appearance=\"plain\""));
    assert!(!meta.contains(" theme=\""));

    let index = crate::theme::THEMES
        .iter()
        .position(|theme| theme.name == "Cassowary")
        .unwrap();
    let themed = PdfDocument::build(
        "Body.",
        &NoImages,
        PdfOptions {
            paper: PdfPaper::Letter,
            appearance: PdfAppearance::CurrentTheme(index),
        },
    )
    .unwrap();
    let meta = metadata(&themed.bytes().unwrap());
    assert!(meta.contains("paper=\"Letter\""));
    assert!(meta.contains("appearance=\"theme\""));
    assert!(meta.contains("theme=\"Cassowary\""));
}

#[test]
fn preview_and_pdf_use_the_same_stored_glyph_positions_without_reflow() {
    let _serial = crate::testlock::serial();
    let theme = crate::theme::THEMES
        .iter()
        .position(|theme| theme.name == "Saltpan")
        .unwrap();
    let document = PdfDocument::build(
        "# Shared layout\n\nSelectable body text.",
        &NoImages,
        PdfOptions {
            paper: PdfPaper::A4,
            appearance: PdfAppearance::CurrentTheme(theme),
        },
    )
    .unwrap();
    let first = document.layout.pages[0]
        .ops
        .iter()
        .find_map(|op| match op {
            Op::Glyph(glyph) => Some((glyph.x, glyph.y, glyph.size, glyph.glyph_id)),
            _ => None,
        })
        .expect("fixture has a glyph");
    let before = document.bytes().unwrap();
    let preview = document.preview_page(0, 1.5).unwrap();
    let after = document.bytes().unwrap();
    assert_eq!(
        before, after,
        "preview cannot mutate or rebuild the PDF display list"
    );
    assert_eq!(preview.dimensions(), (893, 1263));

    let content = Pdf::parse(&before).page_streams()[0];
    let content = std::str::from_utf8(content).unwrap();
    assert!(content.contains(&format!(
        "{:.3} {:.3} Tm <{:04X}>",
        first.0,
        document.layout.geometry.height - first.1,
        first.3
    )));
    let paper = document.layout.appearance.palette.unwrap().paper.0;
    let x0 = ((first.0 - 2.0) * 1.5).max(0.0) as u32;
    let x1 = ((first.0 + first.2 + 4.0) * 1.5).ceil() as u32;
    let y0 = ((first.1 - first.2 - 3.0) * 1.5).max(0.0) as u32;
    let y1 = ((first.1 + 3.0) * 1.5).ceil() as u32;
    assert!(
        (y0..y1.min(preview.height())).any(|y| {
            (x0..x1.min(preview.width())).any(|x| preview.get_pixel(x, y).0[..3] != paper)
        }),
        "stored first-glyph box must contain preview ink"
    );
    assert!(document.preview_page(1, 1.0).is_err());
    assert!(document.preview_page(0, 0.0).is_err());
}

#[test]
fn jpeg_preview_plate_is_labelled_and_does_not_change_geometry_or_pdf_bytes() {
    let _serial = crate::testlock::serial();
    let document = PdfDocument::build(
        "Before.\n\n![jpeg photo](photo.jpg)\n\nAfter.",
        &Images::new(),
        PdfOptions::default(),
    )
    .unwrap();
    let (page, rect) = document
        .layout
        .pages
        .iter()
        .enumerate()
        .find_map(|(page, page_layout)| {
            page_layout.ops.iter().find_map(|op| match op {
                Op::Image { index, x, y, w, h } if document.layout.images[*index].jpeg => {
                    Some((page, (*x, *y, *w, *h)))
                }
                _ => None,
            })
        })
        .expect("fixture JPEG has stored geometry");
    let before_positions = glyph_positions(&document);
    let before = document.bytes().unwrap();
    let preview = document.preview_page(page, 2.0).unwrap();
    assert_eq!(glyph_positions(&document), before_positions);
    assert_eq!(document.bytes().unwrap(), before);

    let (x, y, w, h) = rect;
    let mut colors = std::collections::BTreeSet::new();
    for py in (y * 2.0) as u32..((y + h) * 2.0).ceil() as u32 {
        for px in (x * 2.0) as u32..((x + w) * 2.0).ceil() as u32 {
            colors.insert(preview.get_pixel(px, py).0);
        }
    }
    assert!(
        colors.len() >= 4,
        "JPEG preview plate needs border plus a visible text label: {colors:?}"
    );
    let parsed = Pdf::parse(&before);
    assert!(
        parsed
            .objects
            .values()
            .any(|object| object.text().contains("/Filter /DCTDecode")),
        "final PDF still embeds the original JPEG stream"
    );
}

fn metadata(bytes: &[u8]) -> String {
    let pdf = Pdf::parse(bytes);
    let id = reference(pdf.object(1).text(), "/Metadata ");
    std::str::from_utf8(pdf.object(id).stream().unwrap())
        .unwrap()
        .to_string()
}

fn glyph_positions(document: &PdfDocument) -> Vec<(usize, u16, u32, u32)> {
    document
        .layout
        .pages
        .iter()
        .enumerate()
        .flat_map(|(page, layout)| {
            layout.ops.iter().filter_map(move |op| match op {
                Op::Glyph(glyph) => {
                    Some((page, glyph.glyph_id, glyph.x.to_bits(), glyph.y.to_bits()))
                }
                _ => None,
            })
        })
        .collect()
}

fn contrast(a: crate::theme::Srgb, b: crate::theme::Srgb) -> f32 {
    fn luminance(color: crate::theme::Srgb) -> f32 {
        let linear = |channel: u8| {
            let value = channel as f32 / 255.0;
            if value <= 0.04045 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * linear(color.r) + 0.7152 * linear(color.g) + 0.0722 * linear(color.b)
    }
    let (a, b) = (luminance(a), luminance(b));
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}
