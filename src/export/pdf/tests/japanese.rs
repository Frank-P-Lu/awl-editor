use ttf_parser::Face;

use super::super::fonts::{FontRole, asset, fallback_char, is_japanese_scalar, role_index};
use super::fixture::{self, NoImages};
use super::parser::{Pdf, decode_utf16_hex, has_tokens, hex_value_after};
use super::semantic::recover_page_text;
use super::{glyph_operator, golden};
use crate::export::{model, pdf::emit};

#[test]
fn japanese_fallback_paints_real_subset_glyphs_and_preserves_semantics() {
    let markdown = fixture::japanese_markdown();
    let bytes = emit(&model::parse(&markdown), &NoImages);
    let pdf = Pdf::parse(&bytes);
    let pages = recover_page_text(&pdf);
    let recovered = pages.join("");
    assert!(
        pages.len() >= 2,
        "Japanese fixture must exercise page breaks"
    );
    for text in [
        "日本語 PDF",
        "通常文かな",
        "強調字",
        "符号列",
        "太字コード",
        "表のセル",
        "コードブロックが読める",
        "最終ページの印",
    ] {
        assert!(recovered.contains(text), "extracted PDF dropped {text:?}");
    }
    assert!(
        !recovered.contains('□'),
        "Japanese must not extract as tofu"
    );

    let annotations = pdf
        .objects
        .values()
        .filter(|object| has_tokens(&object.text(), "/Subtype /Link"))
        .count();
    assert_eq!(annotations, 1, "mixed-script link keeps its annotation");

    assert_japanese_faces_and_subsets(&pdf);
    golden("japanese.pdf", &bytes);
}

fn assert_japanese_faces_and_subsets(pdf: &Pdf<'_>) {
    let content = pdf
        .page_streams()
        .into_iter()
        .flat_map(|stream| std::str::from_utf8(stream).unwrap().lines())
        .collect::<Vec<_>>();
    let normal = glyph_operator(&content, '通');
    assert!(has_tokens(normal, "BT /F5 11.000 Tf") && !has_tokens(normal, "2 Tr"));
    let bold = glyph_operator(&content, '強');
    assert!(has_tokens(bold, "BT /F5 11.000 Tf 2 Tr"), "{bold}");
    let mono = glyph_operator(&content, '符');
    assert!(has_tokens(mono, "BT /F6 11.000 Tf") && !has_tokens(mono, "2 Tr"));
    let bold_mono = glyph_operator(&content, '太');
    assert!(
        has_tokens(bold_mono, "BT /F6 11.000 Tf 2 Tr"),
        "{bold_mono}"
    );

    for (index, line) in content.iter().enumerate() {
        if !line.starts_with("/Span << /ActualText <") {
            continue;
        }
        let actual = decode_utf16_hex(hex_value_after(line, "/ActualText <"));
        if actual.chars().any(is_japanese_scalar) {
            let operator = content[index + 1];
            assert!(
                has_tokens(operator, "/F5") || has_tokens(operator, "/F6"),
                "Japanese cluster {actual:?} escaped to a Latin/tofu face: {operator}"
            );
        }
    }

    for (role, object_base, used, unused) in [
        (FontRole::JapaneseSerif, 23, 'が', '森'),
        (FontRole::JapaneseSans, 28, '符', '森'),
    ] {
        let source = Face::parse(asset(role).bytes, 0).unwrap();
        let embedded = pdf.object(object_base + 3).stream().unwrap();
        let subset = Face::parse(embedded, 0).unwrap();
        let used = source.glyph_index(used).unwrap();
        let unused = source.glyph_index(unused).unwrap();
        assert_eq!(
            source.glyph_bounding_box(used),
            subset.glyph_bounding_box(used),
            "{} used outline",
            asset(role).pdf_name
        );
        assert!(
            subset.glyph_bounding_box(unused).is_none(),
            "{} unused outline must be absent from subset",
            asset(role).pdf_name
        );
        assert!(
            embedded.len() < asset(role).bytes.len() / 8,
            "{} subset {} vs source {}",
            asset(role).pdf_name,
            embedded.len(),
            asset(role).bytes.len()
        );
    }

    let square = fallback_char(FontRole::Serif);
    let square_id = Face::parse(asset(FontRole::Serif).bytes, 0)
        .unwrap()
        .glyph_index(square)
        .unwrap()
        .0;
    assert!(
        content.iter().all(|line| {
            !has_tokens(line, "/F1") || !line.contains(&format!("<{square_id:04X}> Tj"))
        }),
        "the synthetic Japanese fixture must not paint the Latin fallback square"
    );
    assert_eq!(role_index(FontRole::JapaneseSerif), 4);
    assert_eq!(role_index(FontRole::JapaneseSans), 5);
}
