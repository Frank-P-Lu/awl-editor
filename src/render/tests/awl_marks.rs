//! The roster-driven law for the one derived Awl Marks face.

use std::collections::BTreeSet;

use super::super::*;
use ttf_parser::Face;

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

#[test]
fn adopted_mark_roster_is_complete_named_and_role_enrolled() {
    let roster = marks::roster();
    assert_eq!(
        roster.len(),
        94,
        "the decided, deduplicated union is 94 glyphs"
    );
    assert_eq!(
        role_codepoints("chrome").len(),
        34,
        "phase one's chrome roster"
    );
    assert_eq!(
        role_codepoints("ornament-536").len(),
        64,
        "item 536's final union"
    );
    assert_eq!(
        role_codepoints("reference-537"),
        BTreeSet::from([0x002A, 0x00A7, 0x00B6, 0x2016, 0x2020, 0x2021]),
        "the traditional * † ‡ § ‖ ¶ reference ladder is enrolled exactly"
    );
    assert_eq!(
        role_codepoints("symbol-span").len(),
        16,
        "the current chrome symbol-span consumer set remains explicit"
    );
    for mark in roster {
        assert!(
            !mark.name.trim().is_empty() && !mark.source_range.trim().is_empty(),
            "U+{:04X} must retain a name and source range",
            mark.codepoint
        );
        assert!(
            mark.roles.iter().all(|role| matches!(
                *role,
                "chrome" | "symbol-span" | "ornament-536" | "reference-537"
            )),
            "U+{:04X} has an unknown role: {:?}",
            mark.codepoint,
            mark.roles
        );
    }
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
        "AwlMarks.ttf cmap drift: unrostered={} missing_from_face={} — regenerate after every roster edit",
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
    assert_eq!(
        actual.len(),
        94,
        "the cmap law must not pass over an empty roster"
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
    for ch in "⌘⇧⌥⌃↵⇥⌫❧❦☙❡❥⁂§†‡".chars() {
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
    let mut checked = 0usize;
    for world in theme::THEMES
        .iter()
        .filter(|world| world.ornament_face == theme::ORNAMENT_MARKS)
    {
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
            checked += 1;
        }
    }
    assert!(
        checked >= 30,
        "the consumer sweep enrolled too few live marks: {checked}"
    );
}
