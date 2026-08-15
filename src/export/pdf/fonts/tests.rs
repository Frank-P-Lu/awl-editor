use super::*;

#[derive(Default)]
struct SegmentCount(usize);

impl ttf_parser::OutlineBuilder for SegmentCount {
    fn move_to(&mut self, _: f32, _: f32) {
        self.0 += 1;
    }
    fn line_to(&mut self, _: f32, _: f32) {
        self.0 += 1;
    }
    fn quad_to(&mut self, _: f32, _: f32, _: f32, _: f32) {
        self.0 += 1;
    }
    fn curve_to(&mut self, _: f32, _: f32, _: f32, _: f32, _: f32, _: f32) {
        self.0 += 1;
    }
    fn close(&mut self) {
        self.0 += 1;
    }
}

#[test]
fn pdf_faces_are_true_type_installable_ofl_inventory_faces() {
    for face in &ASSETS {
        let parsed = Face::parse(face.bytes, 0).expect(face.pdf_name);
        assert_eq!(
            parsed.permissions(),
            Some(Permissions::Installable),
            "{} fsType",
            face.pdf_name
        );
        assert!(
            embedding_is_permitted(face),
            "{} outline embedding",
            face.pdf_name
        );
        // Subsetting is both technically supported by these `glyf` faces and
        // permitted by their installable-embedding license bits.
        assert!(
            parsed.is_subsetting_allowed(),
            "{} subsetting permitted by license",
            face.pdf_name
        );
        assert!(parsed.tables().cmap.is_some(), "{} cmap", face.pdf_name);
        assert!(parsed.tables().glyf.is_some(), "{} glyf", face.pdf_name);
        assert!(parsed.tables().hmtx.is_some(), "{} hmtx", face.pdf_name);
    }
    let inventory = crate::embedded_docs::FONT_LICENSES_MD;
    let ofl = crate::embedded_docs::FONT_OFL_TXT;
    for file in [
        "Bitter-Regular.ttf",
        "Bitter-Bold.ttf",
        "IBMPlexMono-Light.ttf",
        "IBMPlexMono-Bold.ttf",
        "NotoSerifJP-Regular.ttf",
        "NotoSansJP-Regular.ttf",
    ] {
        assert!(
            inventory.contains(file),
            "missing inventory record for {file}"
        );
    }
    assert!(ofl.contains("SIL OPEN FONT LICENSE Version 1.1"));
}

#[test]
fn cached_coverage_lookup_matches_every_bundled_face() {
    for asset in &ASSETS {
        let parsed = Face::parse(asset.bytes, 0).expect(asset.pdf_name);
        for ch in "Awl café — []{}() 😀 🦉\n".chars() {
            assert_eq!(
                has_glyph(asset.role, ch),
                parsed.glyph_index(ch).is_some(),
                "{} coverage for {ch:?}",
                asset.pdf_name
            );
        }
    }
}

#[test]
fn subsets_preserve_composite_outlines_and_sfnt_checksum() {
    for asset in &ASSETS {
        let source = Face::parse(asset.bytes, 0).unwrap();
        let probe = if matches!(asset.role, FontRole::JapaneseSerif | FontRole::JapaneseSans) {
            '日'
        } else {
            'é'
        };
        let id = source
            .glyph_index(probe)
            .unwrap_or_else(|| panic!("{} contains {probe:?}", asset.pdf_name));
        let bytes = subset(asset.role, &BTreeSet::from([id.0]));
        assert_eq!(checksum(&bytes), 0xB1B0_AFBA, "{} checksum", asset.pdf_name);
        let subset = Face::parse(&bytes, 0).unwrap();
        let mut source_segments = SegmentCount::default();
        let mut subset_segments = SegmentCount::default();
        assert_eq!(
            source.outline_glyph(id, &mut source_segments),
            subset.outline_glyph(id, &mut subset_segments),
            "{} outline bounds",
            asset.pdf_name
        );
        assert_eq!(
            source_segments.0, subset_segments.0,
            "{} composite components",
            asset.pdf_name
        );
    }
}
