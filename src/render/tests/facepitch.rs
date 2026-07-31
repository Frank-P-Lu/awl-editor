//! ITEM 97 — THE FACE-PITCH ROSTER LAWS. The membership question the caret's
//! mono/proportional fork asks ("is this family monospaced?") used to be answered
//! by a literal three-name match in `caret::font_is_mono`, and a literal list is
//! unsweepable by construction: it had already lost Monaspace Xenon and JetBrains
//! Mono once, and it never had **Iosevka**, so Currawong and Cassowary — two
//! genuinely fixed-pitch worlds — drew the PROPORTIONAL ink-hugging caret instead
//! of the uniform grid (measured at zoom 1 before the fix: Currawong's caret top
//! sat at y18 on `l` and y23 on `o`/`g`, a 5px letter-to-letter wobble, against a
//! fixed y20 on Tawny/Mangrove/Potoroo/Firetail).
//!
//! The replacement is a MEASUREMENT (`render::facepitch` reads each bundled
//! face's own advance widths) plus a DECLARATION (`FONT_THEME_FACES` pairs every
//! `include_bytes!` with a `Pitch`, so a new face cannot compile without one).
//! These laws are the join between the two, and the sweep over the shipped
//! roster:
//!
//!   * every bundled display face MEASURES the pitch it DECLARES — a wrong call
//!     fails here rather than changing the caret quietly;
//!   * the declared roster and the shipped roster are the SAME SET, both ways —
//!     bundling a face without declaring it, or declaring one that is not
//!     bundled, fails;
//!   * every `Theme::font` / `Theme::mono` in the whole world roster is a member
//!     of that set, and the FULL world → face → mono? table is pinned name by
//!     name, so a new world (or a re-faced one) fails until its row is written;
//!   * the bold companion of every mono family is itself mono, so `**bold**`
//!     keeps the grid;
//!   * `caret::font_is_mono` agrees with the measurement for every member —
//!     including the two the retired list missed.
//!
//! GPU-FREE: everything here reads font BYTES and consts, so it runs on any box.

use super::super::*;

use crate::render::facepitch::{self, Pitch};

/// THE SHIPPED FACE ROSTER, DECLARED — every family a bundled display face
/// registers under, with the pitch its own metrics must measure. This is the
/// no-wildcard sweep: [`bundled_face_roster_is_exactly_this_declared_set`]
/// compares it against the real roster in BOTH directions, so there is no
/// catch-all arm for a new face to fall into. Bundle a face (or retire one)
/// without editing this list and the suite fails.
///
/// Family names are the fontdb-REGISTERED ones, which is why two of them read
/// oddly ("Newsreader 16pt 16pt", "Fraunces 9pt" — the optical-size masters).
const DECLARED_ROSTER: &[(&str, Pitch)] = &[
    ("Bitter", Pitch::Proportional),
    ("EB Garamond", Pitch::Proportional),
    ("Figtree", Pitch::Proportional),
    ("Fira Sans", Pitch::Proportional),
    ("Fraunces 9pt", Pitch::Proportional),
    ("IBM Plex Mono", Pitch::Mono),
    ("IBM Plex Sans", Pitch::Proportional),
    ("Iosevka", Pitch::Mono),
    ("JetBrains Mono", Pitch::Mono),
    ("Literata", Pitch::Proportional),
    ("Monaspace Xenon", Pitch::Mono),
    ("Newsreader 16pt 16pt", Pitch::Proportional),
    ("Sour Gummy", Pitch::Proportional),
    ("Zilla Slab", Pitch::Proportional),
    // A DUOSPACE, not a mono: bundled and bold-paired, currently unassigned to
    // any world. Its presence here is the negative control that keeps the
    // measurement honest — a predicate that answered "mono" for anything
    // near-gridded would fail on this row.
    ("iA Writer Quattro S", Pitch::Proportional),
];

/// THE FULL WORLD → DISPLAY FACE → CODE FACE TABLE, pinned name by name, in
/// `theme::THEMES` cycle order. Every world appears exactly once;
/// [`every_world_face_is_a_roster_member_and_the_table_holds`] asserts the world
/// NAMES here are exactly `theme::THEMES`'s, so a world added, retired or
/// re-faced fails this table instead of silently acquiring (or losing) the mono
/// caret grid.
const WORLD_FACES: &[(&str, &str, &str)] = &[
    ("Tawny", "IBM Plex Mono", "IBM Plex Mono"),
    ("Mopoke", "Bitter", "IBM Plex Mono"),
    ("Currawong", "Iosevka", "Iosevka"),
    ("Potoroo", "Monaspace Xenon", "Monaspace Xenon"),
    ("Gumtree", "Literata", "Monaspace Xenon"),
    ("Bilby", "Newsreader 16pt 16pt", "Monaspace Xenon"),
    ("Saltpan", "Fraunces 9pt", "Monaspace Xenon"),
    ("Quokka", "Sour Gummy", "IBM Plex Mono"),
    ("Bombora", "EB Garamond", "Monaspace Xenon"),
    ("Bowerbird", "IBM Plex Sans", "JetBrains Mono"),
    ("Mulga", "Zilla Slab", "Monaspace Xenon"),
    ("Mangrove", "JetBrains Mono", "JetBrains Mono"),
    ("Galah", "Figtree", "IBM Plex Mono"),
    ("Magpie", "Bitter", "Monaspace Xenon"),
    ("Brolga", "IBM Plex Sans", "IBM Plex Mono"),
    ("Wagtail", "JetBrains Mono", "JetBrains Mono"),
    ("Firetail", "Monaspace Xenon", "Monaspace Xenon"),
    ("Cassowary", "Iosevka", "Iosevka"),
    ("Paperbark", "EB Garamond", "Monaspace Xenon"),
    ("Kite", "Fira Sans", "JetBrains Mono"),
];

/// The seven MONO-DISPLAY worlds — the ones whose caret must hold the uniform
/// grid. DERIVED from [`WORLD_FACES`] + the measurement, never hand-listed, so it
/// cannot drift; `tests/caret_mono_grid_pixels.rs` re-derives the same set from
/// the same source and asserts the grid in real pixels.
pub(crate) fn mono_display_worlds() -> Vec<&'static str> {
    WORLD_FACES
        .iter()
        .filter(|(_, font, _)| facepitch::family_is_mono(font))
        .map(|(name, _, _)| *name)
        .collect()
}

/// EVERY bundled display face measures the pitch it declares. The declaration is
/// what a human wrote next to the `include_bytes!`; the measurement is what the
/// file's own advance widths say. They must agree, and a face whose probe
/// coverage is incomplete (`None`) is a failure, never a silent demotion to
/// proportional.
#[test]
fn bundled_display_faces_measure_the_pitch_they_declare() {
    let _t = crate::testlock::serial();
    for (bytes, declared) in crate::render::bundled_display_faces() {
        let family = facepitch::registered_family(bytes)
            .expect("every bundled display face registers a family name through fontdb");
        let measured = facepitch::measure_pitch(bytes).unwrap_or_else(|| {
            panic!(
                "{family}: could not measure a pitch — the face must cover every \
                 probe glyph in facepitch::PITCH_PROBE ({:?})",
                facepitch::PITCH_PROBE
            )
        });
        assert_eq!(
            measured, declared,
            "{family}: declared {declared:?} in FONT_THEME_FACES but its own advance \
             widths measure {measured:?}"
        );
    }
}

/// THE ROSTER SWEEP, both directions: the bundled faces and
/// [`DECLARED_ROSTER`] are the same set of families, with the same pitches. A new
/// `include_bytes!` in `FONT_THEME_FACES` fails here until its row is written;
/// a stale row for a retired face fails here too.
#[test]
fn bundled_face_roster_is_exactly_this_declared_set() {
    let _t = crate::testlock::serial();
    let mut shipped: Vec<(String, Pitch)> = facepitch::roster()
        .iter()
        .map(|(fam, facts)| {
            (
                fam.clone(),
                facts.measured.expect("every bundled face measures a pitch"),
            )
        })
        .collect();
    shipped.sort();
    let mut declared: Vec<(String, Pitch)> = DECLARED_ROSTER
        .iter()
        .map(|(f, p)| ((*f).to_string(), *p))
        .collect();
    declared.sort();
    assert_eq!(
        shipped, declared,
        "the shipped bundled-display-face roster and DECLARED_ROSTER have drifted — \
         a face was bundled, retired or re-pitched without updating this law"
    );
    // Non-vacuity: the set really does contain both classes, so an empty or
    // all-one-class roster could never pass by accident.
    assert!(declared.iter().any(|(_, p)| *p == Pitch::Mono));
    assert!(declared.iter().any(|(_, p)| *p == Pitch::Proportional));
}

/// Every world's DISPLAY face and CODE companion is a member of the bundled
/// roster, and the whole world → face table holds name by name. A world pointed
/// at an unbundled family would render in a fallback face AND answer `false` to
/// `font_is_mono` — this is the law that makes that a build failure.
#[test]
fn every_world_face_is_a_roster_member_and_the_table_holds() {
    let _t = crate::testlock::serial();
    let mut declared_names: Vec<&str> = WORLD_FACES.iter().map(|(n, _, _)| *n).collect();
    let mut shipped_names: Vec<&str> = theme::THEMES.iter().map(|t| t.name).collect();
    declared_names.sort();
    shipped_names.sort();
    assert_eq!(
        declared_names, shipped_names,
        "WORLD_FACES and theme::THEMES have drifted — a world was added or retired \
         without declaring its display/code faces here"
    );
    for t in theme::THEMES.iter() {
        let (_, font, mono) = WORLD_FACES
            .iter()
            .find(|(n, _, _)| *n == t.name)
            .expect("checked above that the name sets match");
        assert_eq!(t.font, *font, "{}: display face", t.name);
        assert_eq!(t.mono, *mono, "{}: code companion face", t.name);
        for family in [t.font, t.mono] {
            assert!(
                facepitch::roster().contains_key(family),
                "{}: names family {family:?}, which is not a bundled display face — \
                 it would shape in a system fallback and answer false to font_is_mono",
                t.name
            );
        }
    }
}

/// `caret::font_is_mono` — the predicate the caret's grid/ink fork actually
/// calls — answers exactly the measurement, for every roster member. Includes the
/// two the retired hardcoded list missed (Iosevka, and the whole reason for this
/// round), and pins the unknown-family answer the old list also gave.
#[test]
fn font_is_mono_answers_the_measurement_for_every_roster_member() {
    let _t = crate::testlock::serial();
    for (family, declared) in DECLARED_ROSTER {
        assert_eq!(
            crate::caret::font_is_mono(family),
            declared.is_mono(),
            "font_is_mono({family:?}) disagrees with the face's declared+measured pitch"
        );
    }
    // THE REGRESSION THIS ROUND EXISTS FOR: the retired predicate was
    // `matches!(family, "IBM Plex Mono" | "JetBrains Mono" | "Monaspace Xenon")`,
    // so Iosevka answered false and Currawong/Cassowary lost the grid.
    assert!(
        crate::caret::font_is_mono("Iosevka"),
        "Iosevka is a fixed-pitch face"
    );
    // A family that is not a bundled display face (a system fallback, an
    // `AWL_FONT` override) is not claimed either way — false, as before.
    assert!(!crate::caret::font_is_mono("Helvetica"));
    assert!(!crate::caret::font_is_mono(""));
}

/// The BOLD companion of every MONO family is itself mono. `**bold**` requests
/// `Weight::BOLD` and lands on the 700 FILE (`FONT_THEME_BOLD_FACES`), so a bold
/// that lost the fixed advance would break the grid on exactly the emphasised
/// run, where the caret still draws its uniform cell. Sweeps the bold list by
/// FAMILY, so it covers every mono display face without a second name list.
#[test]
fn bold_companions_of_mono_families_hold_the_same_grid() {
    let _t = crate::testlock::serial();
    let mut checked = 0usize;
    for &bold in crate::render::FONT_THEME_BOLD_FACES {
        let Some(family) = facepitch::registered_family(bold) else {
            continue;
        };
        let Some(regular) = facepitch::roster().get(&family).and_then(|f| f.measured) else {
            panic!("bold face {family:?} registers a family with no bundled Regular")
        };
        let measured = facepitch::measure_pitch(bold)
            .unwrap_or_else(|| panic!("{family} Bold: incomplete probe coverage"));
        assert_eq!(
            measured, regular,
            "{family} Bold measures {measured:?} but its Regular measures {regular:?} — \
             a bold that changes pitch breaks the caret grid on emphasised runs"
        );
        checked += 1;
    }
    assert!(
        checked >= DECLARED_ROSTER.len(),
        "every bundled family ships a bold (got {checked})"
    );
}

/// The set of MONO-DISPLAY worlds, pinned. Not a second source of truth — it is
/// derived from `WORLD_FACES` + the measurement — but naming the seven here makes
/// the round's product claim explicit and makes a world silently JOINING or
/// LEAVING the uniform-grid caret a failure rather than a look change nobody
/// asked for.
#[test]
fn the_mono_display_worlds_are_these_seven() {
    let _t = crate::testlock::serial();
    assert_eq!(
        mono_display_worlds(),
        vec![
            "Tawny",     // IBM Plex Mono
            "Currawong", // Iosevka  — regained the grid this round
            "Potoroo",   // Monaspace Xenon
            "Mangrove",  // JetBrains Mono
            "Wagtail",   // JetBrains Mono
            "Firetail",  // Monaspace Xenon
            "Cassowary", // Iosevka  — regained the grid this round
        ]
    );
    // And the complement really is proportional — no world sits in neither camp.
    let mono = mono_display_worlds();
    for (name, font, _) in WORLD_FACES {
        assert_eq!(
            mono.contains(name),
            facepitch::family_is_mono(font),
            "{name} ({font}) falls between the two camps"
        );
    }
}

/// ITEM 105 — every bundled display face's `typical_letter_ratio` is a REAL,
/// SANE measurement, not the fallback default sneaking in for a face that
/// really does declare `x_height`/`cap_height`: within `measure_typical_letter_ratio`'s
/// own clamp range, and for the roster's proportional faces specifically —
/// where the caret's synthetic ink box actually matters, since a mono world
/// never reads this ratio at all — the CLAMP alone is too weak a check (a
/// clamp bound is trivially satisfiable by the fallback constant too), so this
/// also asserts a real SPREAD across the roster: different faces' own
/// x-height/cap-height proportions differ, so a roster of all-fallback values
/// (every face silently failing to measure) would collapse to one repeated
/// number and fail the spread check.
#[test]
fn every_proportional_face_measures_a_sane_typical_letter_ratio() {
    let _t = crate::testlock::serial();
    let mono = mono_display_worlds();
    let mut ratios: Vec<f32> = Vec::new();
    let mut checked = 0usize;
    for (name, font, _) in WORLD_FACES {
        if mono.contains(name) {
            continue; // the caret never reads this ratio on a mono world
        }
        let ratio = facepitch::typical_letter_ratio(font);
        assert!(
            (0.2..=0.95).contains(&ratio),
            "{name} ({font}): typical_letter_ratio out of the measured clamp range: {ratio}"
        );
        ratios.push(ratio);
        checked += 1;
    }
    assert!(
        checked >= 11,
        "every proportional-display world is swept (got {checked})"
    );

    let (min, max) = (
        ratios.iter().cloned().fold(f32::MAX, f32::min),
        ratios.iter().cloned().fold(f32::MIN, f32::max),
    );
    assert!(
        max - min > 0.02,
        "the roster's own faces must measure genuinely DIFFERENT ratios, not \
         one repeated fallback value: min={min} max={max}"
    );
}

/// An UNKNOWN family (never bundled — a system fallback face, an `AWL_FONT`
/// override) answers the documented fallback constant, exactly the same
/// "unknown family" shape `family_is_mono` already has.
#[test]
fn unknown_family_falls_back_to_the_documented_typical_letter_ratio() {
    let ratio = facepitch::typical_letter_ratio("Not A Real Bundled Family");
    assert!(
        (ratio - facepitch::DEFAULT_TYPICAL_LETTER_RATIO).abs() < 1e-6,
        "an unknown family must answer the documented fallback constant, got {ratio}"
    );
}
