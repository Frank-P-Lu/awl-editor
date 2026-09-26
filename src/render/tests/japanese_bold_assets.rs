//! Structural laws for the five genuine Japanese heavy companions.

use std::collections::BTreeSet;

use super::super::*;

fn cmap(bytes: &[u8]) -> BTreeSet<u32> {
    let face = ttf_parser::Face::parse(bytes, 0).expect("valid bundled font");
    let mut out = BTreeSet::new();
    for subtable in face.tables().cmap.expect("bundled font has cmap").subtables {
        if subtable.is_unicode() {
            subtable.codepoints(|cp| {
                out.insert(cp);
            });
        }
    }
    out
}

fn family_names(bytes: &[u8]) -> BTreeSet<String> {
    let face = ttf_parser::Face::parse(bytes, 0).expect("valid bundled font");
    face.names()
        .into_iter()
        .filter(|name| name.name_id == ttf_parser::name_id::FAMILY)
        .filter_map(|name| name.to_string())
        .collect()
}

type FontPair = (
    &'static str,
    &'static [u8],
    &'static [u8],
    &'static str,
    u16,
);

#[test]
fn every_heavy_face_preserves_its_regular_cmap_and_authentic_metadata() {
    let _g = crate::testlock::serial();
    let pairs: &[FontPair] = &[
        (
            "Noto Sans JP",
            include_bytes!("../../../assets/fonts/NotoSansJP-Regular.ttf"),
            include_bytes!("../../../assets/fonts/NotoSansJP-Bold.ttf"),
            "Noto Sans JP",
            700,
        ),
        (
            "Noto Serif JP",
            include_bytes!("../../../assets/fonts/NotoSerifJP-Regular.ttf"),
            include_bytes!("../../../assets/fonts/NotoSerifJP-Bold.ttf"),
            "Noto Serif JP",
            700,
        ),
        (
            "Shippori Mincho",
            include_bytes!("../../../assets/fonts/ShipporiMincho-Regular.ttf"),
            include_bytes!("../../../assets/fonts/ShipporiMincho-Bold.ttf"),
            "Shippori Mincho",
            700,
        ),
        (
            "Zen Maru Gothic",
            include_bytes!("../../../assets/fonts/ZenMaruGothic-Regular.ttf"),
            include_bytes!("../../../assets/fonts/ZenMaruGothic-Bold.ttf"),
            "Zen Maru Gothic",
            700,
        ),
        (
            "Klee One",
            include_bytes!("../../../assets/fonts/KleeOne-Regular.ttf"),
            include_bytes!("../../../assets/fonts/KleeOne-SemiBold.ttf"),
            "Klee One SemiBold",
            600,
        ),
    ];

    for &(label, regular, heavy, heavy_family, heavy_weight) in pairs {
        assert_eq!(cmap(heavy), cmap(regular), "{label}: exact Regular cmap");
        let face = ttf_parser::Face::parse(heavy, 0).unwrap();
        assert_eq!(face.weight().to_number(), heavy_weight, "{label}: weight");
        assert!(
            family_names(heavy).contains(heavy_family),
            "{label}: authentic family metadata {:?}",
            family_names(heavy)
        );
    }
}

#[test]
fn production_font_database_resolves_every_japanese_heavy_companion() {
    let _g = crate::testlock::serial();
    let font_system = build_font_system();
    for &(regular, heavy) in JA_BOLD_COMPANION_FAMILIES {
        let resolved = TextPipeline::resolve_ja_bold_in_db(
            font_system.db(),
            Some((regular, glyphon::Weight(400))),
        );
        let (family, weight) = resolved.unwrap_or_else(|| {
            panic!("{regular}: registered production DB has no heavy companion {heavy}")
        });
        assert_eq!(family, heavy, "{regular}: explicit companion family");
        let expected = if regular == "Klee One" { 600 } else { 700 };
        assert_eq!(weight.0, expected, "{regular}: authentic registered weight");
    }
}
