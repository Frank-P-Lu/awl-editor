//! The roster-driven law for the one derived Awl Marks face.

use std::collections::BTreeSet;

use super::super::*;
use ttf_parser::{Face, GlyphId, OutlineBuilder};

fn roster_codepoints() -> BTreeSet<u32> {
    marks::roster().iter().map(|mark| mark.codepoint).collect()
}

fn role_codepoints(role: &str) -> BTreeSet<u32> {
    marks::roster()
        .iter()
        .filter(|mark| mark.roles.contains(&role))
        .map(|mark| mark.codepoint)
        .collect()
}

fn cmap_codepoints(face: &Face) -> BTreeSet<u32> {
    let mut codepoints = BTreeSet::new();
    let cmap = face.tables().cmap.expect("AwlMarks.ttf has a cmap");
    for subtable in cmap.subtables {
        if subtable.is_unicode() {
            subtable.codepoints(|codepoint| {
                codepoints.insert(codepoint);
            });
        }
    }
    codepoints
}

fn names(face: &Face, name_id: u16) -> BTreeSet<String> {
    face.names()
        .into_iter()
        .filter(|name| name.name_id == name_id)
        .filter_map(|name| name.to_string())
        .collect()
}

#[derive(Default)]
struct OutlineInk {
    moves: usize,
    segments: usize,
    closes: usize,
}

impl OutlineBuilder for OutlineInk {
    fn move_to(&mut self, _x: f32, _y: f32) {
        self.moves += 1;
    }

    fn line_to(&mut self, _x: f32, _y: f32) {
        self.segments += 1;
    }

    fn quad_to(&mut self, _x1: f32, _y1: f32, _x: f32, _y: f32) {
        self.segments += 1;
    }

    fn curve_to(&mut self, _x1: f32, _y1: f32, _x2: f32, _y2: f32, _x: f32, _y: f32) {
        self.segments += 1;
    }

    fn close(&mut self) {
        self.closes += 1;
    }
}

#[test]
fn adopted_mark_roster_is_complete_named_and_role_enrolled() {
    let roster = marks::roster();
    assert!(
        !roster.is_empty(),
        "the adoption roster must enroll real marks"
    );
    for role in ["chrome", "symbol-span", "ornament-536", "reference-537"] {
        assert!(
            !role_codepoints(role).is_empty(),
            "the declared {role:?} purpose has no enrolled mark"
        );
    }
    assert_eq!(
        role_codepoints("reference-537"),
        BTreeSet::from([0x002A, 0x00A7, 0x00B6, 0x2016, 0x2020, 0x2021]),
        "the traditional * † ‡ § ‖ ¶ reference ladder is enrolled exactly"
    );
    for mark in roster {
        assert!(
            !mark.name.trim().is_empty() && !mark.source_range.trim().is_empty(),
            "U+{:04X} must retain a name and source range",
            mark.codepoint
        );
        assert!(
            !mark.roles.is_empty()
                && mark.roles.iter().all(|role| matches!(
                    *role,
                    "chrome" | "symbol-span" | "ornament-536" | "reference-537"
                )),
            "U+{:04X} has an unknown role: {:?}",
            mark.codepoint,
            mark.roles
        );
    }
}

/// CMAP PRESENCE IS NOT GLYPH PRESENCE. A cmap may legally map a codepoint to
/// glyph zero (`.notdef`), or to an empty/zero-sized glyph. Either shape keeps
/// the key in the cmap while drawing tofu or nothing. Ask the derived face's
/// own outline tables at the pure parser seam: every roster row must map to a
/// nonzero glyph id with positive advance, a non-degenerate bounding box, and
/// at least one closed outline carrying real segments.
#[test]
fn every_rostered_mark_maps_to_a_nonzero_outlined_glyph() {
    let face = Face::parse(FONT_SYMBOLS, 0).expect("AwlMarks.ttf parses");
    let roster = marks::roster();
    assert!(
        !roster.is_empty(),
        "the outline sweep must enroll real marks"
    );
    let mut checked = BTreeSet::new();
    for mark in roster {
        let ch = char::from_u32(mark.codepoint).expect("roster contains Unicode scalars");
        let glyph = face
            .glyph_index(ch)
            .unwrap_or_else(|| panic!("U+{:04X} has no glyph mapping", mark.codepoint));
        assert_ne!(
            glyph,
            GlyphId(0),
            "U+{:04X} maps to .notdef/tofu glyph zero",
            mark.codepoint
        );
        assert!(
            face.glyph_hor_advance(glyph)
                .is_some_and(|advance| advance > 0),
            "U+{:04X} has no positive horizontal advance",
            mark.codepoint
        );
        let bounds = face
            .glyph_bounding_box(glyph)
            .unwrap_or_else(|| panic!("U+{:04X} has no outline bounding box", mark.codepoint));
        assert!(
            bounds.width() > 0 && bounds.height() > 0,
            "U+{:04X} has a degenerate outline box {bounds:?}",
            mark.codepoint
        );
        let mut ink = OutlineInk::default();
        face.outline_glyph(glyph, &mut ink)
            .unwrap_or_else(|| panic!("U+{:04X} has no drawable outline", mark.codepoint));
        assert!(
            ink.moves > 0 && ink.segments > 1 && ink.closes > 0,
            "U+{:04X} has no closed, nontrivial outline: moves={} segments={} closes={}",
            mark.codepoint,
            ink.moves,
            ink.segments,
            ink.closes
        );
        checked.insert(mark.codepoint);
    }
    assert_eq!(
        checked,
        roster_codepoints(),
        "the outline/no-tofu law must sweep the roster itself"
    );
}

#[test]
fn bundled_face_cmap_is_exactly_the_roster_in_both_directions() {
    let face = Face::parse(FONT_SYMBOLS, 0).expect("AwlMarks.ttf parses");
    let expected = roster_codepoints();
    let actual = cmap_codepoints(&face);
    let unrostered: Vec<_> = actual.difference(&expected).copied().collect();
    let missing_from_face: Vec<_> = expected.difference(&actual).copied().collect();
    assert!(
        unrostered.is_empty() && missing_from_face.is_empty(),
        concat!(
            "AwlMarks.ttf cmap drift: unrostered={} missing_from_face={} — ",
            "regenerate after every roster edit"
        ),
        unrostered
            .iter()
            .map(|cp| format!("U+{cp:04X}"))
            .collect::<Vec<_>>()
            .join(","),
        missing_from_face
            .iter()
            .map(|cp| format!("U+{cp:04X}"))
            .collect::<Vec<_>>()
            .join(",")
    );
    assert!(
        !expected.is_empty(),
        "the cmap law must not pass over an empty roster"
    );
    assert_eq!(
        actual.len(),
        expected.len(),
        "the exact-set comparison is exhaustive"
    );
}

#[test]
fn derived_face_identity_weight_and_ofl_metadata_are_preserved() {
    let face = Face::parse(FONT_SYMBOLS, 0).expect("AwlMarks.ttf parses");
    assert!(names(&face, ttf_parser::name_id::FAMILY).contains(SYMBOL_FAMILY));
    assert!(names(&face, ttf_parser::name_id::FULL_NAME).contains(SYMBOL_FAMILY));
    assert_eq!(
        face.weight().to_number(),
        400,
        "the weight-500 trap is normalised"
    );
    assert!(
        names(&face, ttf_parser::name_id::COPYRIGHT_NOTICE)
            .iter()
            .any(|value| value.contains("Umihotaru")),
        "the upstream copyright notice stays embedded"
    );
    assert!(
        names(&face, 13)
            .iter()
            .any(|value| value.contains("SIL OPEN FONT LICENSE Version 1.1")),
        "the full upstream OFL metadata stays embedded"
    );
    assert!(
        names(&face, 14)
            .iter()
            .any(|value| value.contains("scripts.sil.org/OFL")),
        "the upstream OFL URL stays embedded"
    );
    let sha = "ca8782436f7dd82fc9fd93d28c9ec38c0c4ac0044f601a51451f0d648ac52809";
    assert!(
        marks::RAW.contains(sha) && crate::embedded_docs::FONT_LICENSES_MD.contains(sha),
        "the regeneration owner and shipped licence ledger must record the same upstream sha256"
    );
}

#[test]
fn symbol_spans_and_existing_awl_marks_consumers_derive_from_the_roster() {
    // The shipped chrome strings that already consume this role. This is a
    // consumer census, not the routing owner: routing itself is the roster role
    // above. Removing a roster row while a real consumer remains must fail by
    // glyph, rather than quietly making that consumer fall back to tofu.
    for ch in "⌘⇧⌥⌃↵⇥⌫❧❦☙❡❥⁂§".chars() {
        assert!(
            marks::roster()
                .iter()
                .any(|mark| mark.codepoint == ch as u32),
            "live chrome still consumes {ch:?} (U+{:04X}) but the roster removed it",
            ch as u32
        );
        assert!(
            is_symbol(ch),
            "live chrome mark {ch:?} lost explicit family routing"
        );
    }
    for mark in marks::roster() {
        let ch = char::from_u32(mark.codepoint).expect("roster contains only Unicode scalars");
        assert_eq!(
            is_symbol(ch),
            mark.roles.contains(&"symbol-span"),
            "U+{:04X}: symbol routing must derive from the roster role",
            mark.codepoint
        );
    }

    let adopted = roster_codepoints();
    let mut enrolled_worlds = BTreeSet::new();
    let mut checked = Vec::new();
    for world in theme::THEMES
        .iter()
        .filter(|world| world.ornament_face == theme::ORNAMENT_MARKS)
    {
        enrolled_worlds.insert(world.name);
        for ch in [
            world.ornaments.dash,
            world.ornaments.star,
            world.ornaments.underscore,
            world.bullets.0,
            world.bullets.1,
            world.bullets.2,
        ] {
            assert!(
                adopted.contains(&(ch as u32)),
                "{} consumes {:?} (U+{:04X}) from Awl Marks but the roster cannot see it",
                world.name,
                ch,
                ch as u32
            );
            checked.push((world.name, ch));
        }
    }
    assert!(
        !enrolled_worlds.is_empty() && !checked.is_empty(),
        "no live Awl Marks world/consumer enrolled in the roster sweep"
    );
}
