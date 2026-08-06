use super::super::*;

/// The JetBrains-Mono world (Mangrove) reports that font — the second bundled
/// mono face, distinct from Tawny/Potoroo's IBM Plex Mono.
#[test]
fn mangrove_is_jetbrains_mono() {
    let m = THEMES
        .iter()
        .find(|t| t.name == "Mangrove")
        .expect("Mangrove world present");
    assert_eq!(m.font, "JetBrains Mono");
    assert!(m.dark);
    // Galah is the Figtree world.
    let g = THEMES.iter().find(|t| t.name == "Galah").unwrap();
    assert_eq!(g.font, "Figtree");
}

/// PER-WORLD CODE MONO: every world names a `mono` companion that is ONE of the
/// bundled monospace families (IBM Plex Mono / JetBrains Mono / Monaspace Xenon /
/// Iosevka). A world whose DISPLAY face is already one of those monos REUSES its own
/// face (`mono == font`); every other world borrows a bundled mono (`mono != font`).
#[test]
fn every_world_has_a_bundled_mono() {
    const BUNDLED_MONOS: [&str; 4] = [
        "IBM Plex Mono",
        "JetBrains Mono",
        "Monaspace Xenon",
        "Iosevka",
    ];
    // The worlds whose DISPLAY face is itself a bundled mono (so they reuse it).
    // Wagtail was the FIFTH (sharing Mangrove's JetBrains Mono); Firetail is the
    // SIXTH — it derives from Potoroo's warm den and shares its Monaspace Xenon
    // slab-mono display (a logged, honest consequence of adding worlds faster than
    // bundled display faces; see `worlds.rs::FIRETAIL`'s own doc comment).
    // Cassowary (the NERV terminal) is the SEVENTH — it shares Currawong's
    // Iosevka as the terminal-readout face for both display and code.
    const MONO_DISPLAY: [&str; 7] = [
        "Tawny",
        "Currawong",
        "Potoroo",
        "Mangrove",
        "Wagtail",
        "Firetail",
        "Cassowary",
    ];
    for t in THEMES.iter() {
        assert!(
            BUNDLED_MONOS.contains(&t.mono),
            "{}'s mono {:?} is not a bundled monospace family",
            t.name,
            t.mono
        );
        if MONO_DISPLAY.contains(&t.name) {
            assert_eq!(
                t.mono, t.font,
                "{} has a mono display face → must reuse it",
                t.name
            );
        } else {
            assert_ne!(
                t.mono, t.font,
                "{} is a serif/sans world → its code mono must differ from its display face",
                t.name
            );
        }
    }
    // Sanity: the exact reuse assignments (confirmed from theme.rs).
    assert_eq!(TAWNY.mono, "IBM Plex Mono");
    assert_eq!(CURRAWONG.mono, "Iosevka");
    assert_eq!(POTOROO.mono, "Monaspace Xenon");
    assert_eq!(MANGROVE.mono, "JetBrains Mono");
    assert_eq!(WAGTAIL.mono, "JetBrains Mono"); // shares Mangrove's exact display font (logged)
    // And a couple of the borrowed assignments.
    assert_eq!(SALTPAN.mono, "Monaspace Xenon"); // Fraunces serif → slab-serif mono
    assert_eq!(BOWERBIRD.mono, "JetBrains Mono"); // cool technical navy → crisp mono
    assert_eq!(GALAH.mono, "IBM Plex Mono"); // warm humanist sans → warm humanist mono
}

/// Every world declares a per-theme CJK (Japanese) fallback list whose
/// CHARACTER matches the world. After the Phase 2 "JP face variety" round
/// there are FIVE possible ladders (up from two), each still ordered
/// BUNDLED-first then mac-primary (Hiragino) then linux-fallback (Noto CJK):
/// the neutral MINCHO/GOTHIC pair for the worlds this round left alone, plus
/// three per-world overrides — SHIPPORI (bookish serif) for the warm
/// book-serif worlds, ZENMARU (rounded sans) for the two dedicated sans
/// worlds, and KLEE (kaisho/brush) for the two Klee worlds (so their JA
/// matches their ZH's WenKai). Mirrors the shape of
/// `zh_hans_ladder_matches_world_character_with_klee_override`.
#[test]
fn cjk_fallback_matches_world_character() {
    let shippori = ["Gumtree", "Bilby", "Bombora", "Paperbark"];
    let zenmaru = ["Galah", "Bowerbird"];
    let klee = ["Mopoke", "Quokka"];
    let mincho = ["Saltpan", "Mulga", "Magpie"]; // neutral serif (Noto Serif JP)
    let gothic = [
        "Tawny",
        "Potoroo",
        "Mangrove",
        "Currawong",
        "Wagtail",
        "Firetail",
        "Brolga",
        "Cassowary",
        "Kite",
    ]; // neutral sans/mono (Noto Sans JP)
    for t in THEMES.iter() {
        assert!(!t.cjk.is_empty(), "{} has no CJK fallback list", t.name);
        if shippori.contains(&t.name) {
            assert_eq!(
                t.cjk, CJK_JA_SHIPPORI,
                "{} is a book-serif world -> Shippori JA",
                t.name
            );
        } else if zenmaru.contains(&t.name) {
            assert_eq!(
                t.cjk, CJK_JA_ZENMARU,
                "{} is a sans world -> Zen Maru JA",
                t.name
            );
        } else if klee.contains(&t.name) {
            assert_eq!(
                t.cjk, CJK_JA_KLEE,
                "{} is a Klee world -> Klee One JA",
                t.name
            );
        } else if mincho.contains(&t.name) {
            assert_eq!(
                t.cjk, CJK_MINCHO,
                "{} is a neutral serif world -> mincho JA",
                t.name
            );
        } else if gothic.contains(&t.name) {
            assert_eq!(
                t.cjk, CJK_GOTHIC,
                "{} is a neutral sans/mono world -> gothic JA",
                t.name
            );
        } else {
            panic!("{} not classified for CJK fallback", t.name);
        }
    }
    // Priority order: bundled face first, macOS Hiragino, Linux Noto CJK. The
    // three variety ladders keep the NEUTRAL Noto face as their bundled floor
    // (so `AWL_CJK_FORCE=floor` drops cleanly to it; never-tofu unchanged).
    assert_eq!(
        CJK_MINCHO,
        &["Noto Serif JP", "Hiragino Mincho ProN", "Noto Serif CJK JP"]
    );
    assert_eq!(
        CJK_GOTHIC,
        &[
            "Noto Sans JP",
            "Hiragino Kaku Gothic ProN",
            "Noto Sans CJK JP"
        ]
    );
    assert_eq!(
        CJK_JA_SHIPPORI,
        &[
            "Shippori Mincho",
            "Noto Serif JP",
            "Hiragino Mincho ProN",
            "Noto Serif CJK JP"
        ]
    );
    assert_eq!(
        CJK_JA_ZENMARU,
        &[
            "Zen Maru Gothic",
            "Noto Sans JP",
            "Hiragino Kaku Gothic ProN",
            "Noto Sans CJK JP"
        ]
    );
    assert_eq!(
        CJK_JA_KLEE,
        &[
            "Klee One",
            "Noto Sans JP",
            "Hiragino Kaku Gothic ProN",
            "Noto Sans CJK JP"
        ]
    );
}

/// THE NEVER-TOFU LAW (structural half — the environment-independent part
/// of it): every [`FontId`] has a NON-EMPTY candidate ladder on EVERY
/// world. This is the actual regression the law guards against — a world
/// accidentally shipping an empty ladder for a script would guarantee
/// tofu with no possible resolution, regardless of what's installed on
/// the machine running awl. (The COMPLEMENTARY half — that `Latin`/`Ja`
/// always resolve to a concretely-registered face via the real font DB —
/// is `render::tests::cjk::latin_and_ja_always_resolve_to_an_embedded_face`,
/// since it needs a built `FontSystem` to check against.)
#[test]
fn every_font_id_has_a_nonempty_candidate_ladder_on_every_world() {
    for t in THEMES.iter() {
        for id in ALL_FONT_IDS {
            assert!(
                !t.candidates(id).is_empty(),
                "{} has an EMPTY candidate ladder for {:?} — guaranteed tofu",
                t.name,
                id
            );
        }
    }
}

/// `Theme::candidates` for `Latin` is always exactly the world's own
/// [`Theme::font`] — a single-element floor, never a fallback list.
#[test]
fn latin_candidates_is_the_worlds_own_display_face() {
    for t in THEMES.iter() {
        assert_eq!(t.candidates(FontId::Latin), vec![t.font], "{}", t.name);
    }
}

/// THE CHINESE ROUND: zh-Hans now mirrors `cjk_fallback_matches_world_character`
/// exactly — SERIF worlds get [`CJK_ZH_HANS_SERIF`] (bundled Noto Serif SC),
/// SANS/MONO worlds get [`CJK_ZH_HANS_SANS`] (bundled Noto Sans SC), EXCEPT the
/// two Klee-derived worlds (Mopoke, Quokka) which get the CHARACTERFUL
/// [`CJK_ZH_HANS_KLEE`] override (bundled LXGW WenKai first). zh-Hant/ko remain
/// v1-uniform (zh-Hant: still no bundled asset at all; ko: one bundled face,
/// no serif/sans split yet — both documented taste calls, logged above).
#[test]
fn zh_hans_ladder_matches_world_character_with_klee_override() {
    let mincho = [
        "Gumtree",
        "Saltpan",
        "Bilby",
        "Bombora",
        "Mulga",
        "Magpie",
        "Paperbark",
    ];
    let klee = ["Mopoke", "Quokka"];
    let gothic = [
        "Tawny",
        "Potoroo",
        "Mangrove",
        "Galah",
        "Bowerbird",
        "Currawong",
        "Wagtail",
        "Firetail",
        "Brolga",
        "Cassowary",
        "Kite",
    ];
    for t in THEMES.iter() {
        assert!(
            !t.zh_hans.is_empty(),
            "{} has no zh-Hans candidate list",
            t.name
        );
        if klee.contains(&t.name) {
            assert_eq!(
                t.zh_hans, CJK_ZH_HANS_KLEE,
                "{} is a Klee world -> WenKai zh-Hans",
                t.name
            );
        } else if mincho.contains(&t.name) {
            assert_eq!(
                t.zh_hans, CJK_ZH_HANS_SERIF,
                "{} is a serif world -> Serif SC zh-Hans",
                t.name
            );
        } else if gothic.contains(&t.name) {
            assert_eq!(
                t.zh_hans, CJK_ZH_HANS_SANS,
                "{} is a sans/mono world -> Sans SC zh-Hans",
                t.name
            );
        } else {
            panic!("{} not classified for zh-Hans fallback", t.name);
        }
    }
    assert_eq!(
        CJK_ZH_HANS_SERIF,
        &["Noto Serif SC", "PingFang SC", "Noto Sans CJK SC"]
    );
    assert_eq!(
        CJK_ZH_HANS_SANS,
        &["Noto Sans SC", "PingFang SC", "Noto Sans CJK SC"]
    );
    assert_eq!(
        CJK_ZH_HANS_KLEE,
        &[
            "LXGW WenKai",
            "Noto Sans SC",
            "PingFang SC",
            "Noto Sans CJK SC"
        ]
    );
}

/// zh-Hant stays v1-uniform across every world — it still has NO bundled
/// asset (Big5 subsetting is banked, not attempted). ko, HOWEVER, now
/// carries a serif/sans split after the "CJK companions" round: the SERIF
/// worlds (same six that get [`CJK_ZH_HANS_SERIF`]) get [`CJK_KO_SERIF`]
/// (bundled Gowun Batang first), the SANS/MONO worlds keep the plain
/// [`CJK_KO`] (Noto Sans KR) floor — mirroring the ja/zh-Hans serif/sans
/// split's shape (`cjk_fallback_matches_world_character`,
/// `zh_hans_ladder_matches_world_character_with_klee_override`).
#[test]
fn zh_hant_uniform_ko_splits_serif_from_sans() {
    // The SERIF worlds — exactly the ones whose zh_hans is CJK_ZH_HANS_SERIF
    // (Theme::cjk is a mincho-family ja ladder). Kept as an explicit roster so
    // a world silently switching character fails HERE, not as a tofu box.
    let serif = [
        "Gumtree",
        "Bilby",
        "Bombora",
        "Saltpan",
        "Mulga",
        "Magpie",
        "Paperbark",
    ];
    for t in THEMES.iter() {
        assert_eq!(t.zh_hant, CJK_ZH_HANT, "{}: zh-Hant stays uniform", t.name);
        if serif.contains(&t.name) {
            assert_eq!(
                t.ko, CJK_KO_SERIF,
                "{} is a serif world -> Gowun Batang ko",
                t.name
            );
            // A serif world's ko is a mincho-family ja ladder, never gothic.
            assert!(
                t.zh_hans == CJK_ZH_HANS_SERIF,
                "{} classified serif for ko but not for zh-Hans — the two must agree",
                t.name
            );
        } else {
            assert_eq!(
                t.ko, CJK_KO,
                "{} is a sans/mono world -> Noto Sans KR ko",
                t.name
            );
        }
    }
    assert_eq!(CJK_ZH_HANT, &["PingFang TC", "Noto Sans CJK TC"]);
    assert_eq!(
        CJK_KO,
        &["Noto Sans KR", "Apple SD Gothic Neo", "Noto Sans CJK KR"]
    );
    // Gowun Batang FIRST (the bundled characterful serif Korean), then the
    // SAME Noto Sans KR bundled floor CJK_KO uses (the AWL_CJK_FORCE=floor
    // target), then serif-first system trailing candidates.
    assert_eq!(
        CJK_KO_SERIF,
        &[
            "Gowun Batang",
            "Noto Sans KR",
            "AppleMyungjo",
            "Noto Serif CJK KR",
            "Apple SD Gothic Neo",
            "Noto Sans CJK KR",
        ]
    );
    // The floor CJK_KO_SERIF drops to under AWL_CJK_FORCE=floor is exactly
    // CJK_KO's bundled floor — so the ko-worlds gallery's "floor" side is the
    // plain Noto Sans KR, machine-independent.
    assert_eq!(
        CJK_KO_SERIF[1], CJK_KO[0],
        "ko-serif floor == the bundled Noto Sans KR floor"
    );
}

/// AXIS COVERAGE RULER (the reason [`Lens`] + [`ThemeTags`] exist at all, with
/// no runtime picker reading them): every declared axis SECTION
/// stays covered by a curated band of worlds, so the axes remain a meaningful
/// build-time description of the roster. A world may OPT OUT (`None`) of an axis, but
/// any `Some(tag)` must be one of that axis's declared sections (no world under a
/// header that doesn't exist); the name-keyed accessor [`tag_for`] agrees with the
/// inline field; every world HEADLINES at least one axis; and `All` groups nothing.
/// THIS is the coverage check meant by "the axes become a build-time ruler"
/// (the decision is recorded in THEMES.md).
#[test]
fn axis_coverage_ruler() {
    for lens in [Lens::Time, Lens::Register, Lens::Voice, Lens::Temperature] {
        let sections = lens.sections();
        for t in THEMES.iter() {
            if let Some(tag) = t.tags.section(lens) {
                assert!(
                    sections.contains(&tag),
                    "{} has invalid {:?} tag {:?} (not in {:?})",
                    t.name,
                    lens,
                    tag,
                    sections
                );
            }
            // The name-keyed accessor agrees with the inline field.
            assert_eq!(
                tag_for(t.name, lens),
                t.tags.section(lens),
                "{} tag_for disagrees",
                t.name
            );
        }
        // Every declared header shows a CURATED band of worlds: never an empty
        // faint header, never the pre-curation crowd (Time=Night once held 6). The
        // upper bound widened 3→4 when the roster grew to sixteen — the sixteenth
        // world (Firetail, the warm lava statement world) headlines Temperature=Warm,
        // which every section was already at its 3-cap when it arrived; 4 is still
        // curated, nowhere near the old crowd.
        for sect in sections {
            let n = THEMES
                .iter()
                .filter(|t| t.tags.section(lens) == Some(*sect))
                .count();
            assert!(
                (2..=4).contains(&n),
                "{:?} section {sect:?} shows {n} worlds (curation wants 2–4)",
                lens
            );
        }
    }
    // Every world headlines at least ONE axis (present under some section), so no
    // world is invisible to the coverage ruler.
    for t in THEMES.iter() {
        let shown = [Lens::Time, Lens::Register, Lens::Voice, Lens::Temperature]
            .iter()
            .any(|&l| t.tags.section(l).is_some());
        assert!(shown, "{} headlines no axis", t.name);
    }
    // The degenerate All axis groups nothing.
    assert!(Lens::All.sections().is_empty());
    assert_eq!(THEMES[0].tags.section(Lens::All), None);
    // The ruler's STRIP shape: All parked FIRST, five axes total.
    assert_eq!(*Lens::STRIP.first().unwrap(), Lens::All);
    assert_eq!(Lens::STRIP.len(), 5);
}

/// The sixteen worlds map onto at least SIX CLEARLY-distinct display faces
/// (IBM Plex Mono / JetBrains Mono / Literata / Newsreader / IBM Plex Sans /
/// Figtree / Zilla Slab), so cycling worlds visibly reskins the glyph shapes,
/// not just the palette. The two newly-registered faces (JetBrains Mono,
/// Figtree) are both present.
#[test]
fn at_least_six_distinct_faces() {
    let mut faces: Vec<&str> = THEMES.iter().map(|t| t.font).collect();
    faces.sort_unstable();
    faces.dedup();
    assert!(
        faces.len() >= 6,
        "expected >=6 distinct display faces, got {faces:?}"
    );
    assert!(faces.contains(&"JetBrains Mono"), "JetBrains Mono missing");
    assert!(faces.contains(&"Figtree"), "Figtree missing");
    // Home (Tawny) renders in the bundled mono so it looks exactly like home.
    assert_eq!(TAWNY.font, "IBM Plex Mono");
}

/// Mopoke's body face is the warm slab Bitter
/// (shared with Magpie — precedented face-sharing, no new asset) and its
/// nested-bullet triple is a one-register, weight-descends-with-depth ornament
/// set (a solid damask rosette → its open four-fold sibling → a small foliate
/// sprig), all three in Mopoke's Junicode ornament face. This pins the DATA off
/// any GPU; the render laws
/// `render::tests::markdown::bullet_glyphs_resolve_in_each_worlds_assigned_face`
/// (they resolve) and `..::bullet_glyph_never_touches_the_following_text_in_any_world`
/// (they never touch the text) cover the appearance half.
#[test]
fn mopoke_body_face_is_bitter_with_the_item_30_bullet_triple() {
    assert_eq!(
        MOPOKE.font, "Bitter",
        "Mopoke's body face is the warm slab Bitter"
    );
    assert_eq!(
        MOPOKE.mono, "IBM Plex Mono",
        "Mopoke keeps IBM Plex Mono for code"
    );
    assert_eq!(
        MOPOKE.bullets,
        ('\u{E670}', '\u{EF92}', '\u{E67D}'),
        "Mopoke's bullet triple descends in weight within one ornament register"
    );
    // Face-sharing is precedented, never a new asset: Magpie draws in Bitter too.
    assert_eq!(
        MAGPIE.font, "Bitter",
        "Bitter is bundled + shared (Magpie's masthead face)"
    );
}
