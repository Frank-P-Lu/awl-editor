use super::super::*;

/// Every world's [`Theme::ornament_face`] AND [`Theme::bullet_face`] are the
/// one Nishiki-derived cabinet, so dividers, bullets, and task markers resolve
/// through one verified font despite remaining distinct vocabularies. (The font-DB
/// half — that each face actually COVERS its glyphs — is `render::tests::cjk::
/// ornament_glyphs_resolve_in_each_worlds_assigned_face` and `render::tests::
/// markdown::bullet_glyphs_resolve_in_each_worlds_assigned_face`, both of
/// which need a built `FontSystem`.) Also pins `ORNAMENT_MARKS ==
/// render::SYMBOL_FAMILY`, the one coupling `theme.rs` states as data rather
/// than importing.
#[test]
fn every_world_ornament_face_is_a_registered_ornament_face() {
    assert_eq!(
        ORNAMENT_MARKS,
        crate::render::SYMBOL_FAMILY,
        "the Nishiki ornament face IS the derived marks face"
    );
    for t in THEMES.iter() {
        assert_eq!(
            t.ornament_face, ORNAMENT_NISHIKI,
            "{} must wear the Nishiki ornament register",
            t.name
        );
        assert_eq!(
            t.bullet_face, ORNAMENT_NISHIKI,
            "{} must draw its bullet triple from the Nishiki register too — no world's \
             bullet may keep a retired transitional face",
            t.name
        );
        // The design-table contract: THREE DISTINCT symbols per world (dash /
        // star / underscore), so a break's ornament tracks the syntax the author
        // typed instead of collapsing to one shared mark. (The font-DB half —
        // that each glyph actually resolves in `ornament_face` — is the render
        // test `ornament_glyphs_resolve_in_each_worlds_assigned_face`.)
        let (d, s, u) = (t.ornaments.dash, t.ornaments.star, t.ornaments.underscore);
        assert!(
            d != s && s != u && d != u,
            "{} ornament trio is not three distinct glyphs: dash={:?} star={:?} underscore={:?}",
            t.name,
            d,
            s,
            u
        );
    }
}

/// NEVER-DRIFT law: every world ships a finite, positive [`Theme::ornament_scale`]
/// that never regresses below the smallest of the three historical tiers. Also
/// pins the three tier VALUES (still real: they are the historical taste
/// defaults, and [`crate::theme::worlds::SALTPAN`] — the roster's own
/// equalization TARGET — still wears [`ORNAMENT_SCALE_ORNATE`] literally,
/// untouched).
///
/// A shared numeric tier is blind to the axis a glyph's own SHAPE hides: two
/// worlds on the SAME tier can carry very different ink-to-em ratios (a chess
/// knight and a run of solid bars fill their em-box differently), so this
/// field moved from "exactly one of three named tier constants" to a per-world
/// literal EQUALIZED against the roster's own live ink-height target — a per-
/// world literal is no longer drift here, it is the mechanism. The real
/// equalization claim — every world's rendered ornament INK lands within a
/// roster-derived tolerance band, measured by real differential pixel
/// arithmetic — is `render::tests::ornament_scale`'s job, which is a strictly
/// stronger and pixel-grounded oracle than a second copy of specific floats
/// could ever be; pinning fresh literals here would only go stale the next
/// time a live taste pass retunes one world.
#[test]
fn every_world_has_an_ornament_scale() {
    // The three tiers are the settled historical taste defaults, still real:
    // the roster target (Saltpan) wears ORNATE literally, and every OTHER
    // world's new per-world literal is measured against these as its own
    // starting floor below.
    assert_eq!(ORNAMENT_SCALE_ORNATE, 2.2, "ornate tier is 2.2");
    assert_eq!(ORNAMENT_SCALE_FLEURON, 1.8, "fleuron tier is 1.8");
    assert_eq!(ORNAMENT_SCALE_GEOMETRIC, 1.5, "geometric tier is 1.5");
    assert!(
        std::hint::black_box(ORNAMENT_SCALE_ORNATE) > ORNAMENT_SCALE_FLEURON
            && ORNAMENT_SCALE_FLEURON > ORNAMENT_SCALE_GEOMETRIC,
        "the tiers descend ornate > fleuron > geometric"
    );

    // Every world's scale is finite, positive, and never below the SMALLEST
    // historical tier — "equalize upward" means no world may end up smaller
    // than where the whole historical tier ladder started.
    for t in THEMES.iter() {
        assert!(
            t.ornament_scale.is_finite() && t.ornament_scale >= ORNAMENT_SCALE_GEOMETRIC,
            "{} has an ornament_scale {} below the geometric floor {} — equalizing \
             upward must never shrink a world",
            t.name,
            t.ornament_scale,
            ORNAMENT_SCALE_GEOMETRIC
        );
    }

    // The roster TARGET is untouched (multiplier 1.000 — it IS the ceiling
    // the rest of the roster was raised to meet), and two worlds the item
    // names directly (the user's own chess-vs-bars comparison) both actually
    // grew past their old shared tier rather than staying pinned to it.
    let by = |name: &str| set_active_by_name(name).unwrap().ornament_scale;
    let _t = crate::testlock::serial();
    assert_eq!(
        by("Saltpan"),
        ORNAMENT_SCALE_ORNATE,
        "Saltpan is the roster's own equalization target and stays on the shared tier"
    );
    assert!(
        by("Currawong") > ORNAMENT_SCALE_GEOMETRIC,
        "Currawong (the user's chess-piece set) must have grown past its old geometric tier"
    );
    assert!(
        by("Mulga") > ORNAMENT_SCALE_ORNATE,
        "Mulga (the user's bar-glyph set) must have grown past its old ornate tier"
    );
    set_active(DEFAULT_THEME);
}

fn assert_bullet_pair_law(t: &Theme) {
    assert_ne!(
        t.bullets.0, t.bullets.1,
        "{}: levels 1/2 must be distinct glyphs, got {:?}",
        t.name, t.bullets
    );
    assert_ne!(
        t.bullets.1, t.bullets.2,
        "{}: levels 2/3 must be distinct glyphs, got {:?}",
        t.name, t.bullets
    );
    assert_ne!(
        t.bullets.0, t.bullets.2,
        "{}: levels 1/3 must be distinct glyphs, got {:?}",
        t.name, t.bullets
    );
    assert!(
        [
            BULLET_SCALE_GARAMOND,
            BULLET_SCALE_PAPER_TOOL,
            BULLET_SCALE_ORNAMENT,
            BULLET_SCALE_PLAIN
        ]
        .contains(&t.bullet_scale),
        "{}: bullet scale {} is outside the measured fitting tiers",
        t.name,
        t.bullet_scale
    );
    let is_paper_tool_set = t.bullets == ('\u{270E}', '\u{2701}', '\u{2709}');
    assert_eq!(
        t.bullet_scale == BULLET_SCALE_PAPER_TOOL,
        is_paper_tool_set,
        "{}: the paper-tool fitting tier belongs exactly to its approved glyph set",
        t.name
    );

    let divider_set: std::collections::BTreeSet<char> = t
        .ornaments
        .dash
        .chars()
        .chain(t.ornaments.star.chars())
        .chain(t.ornaments.underscore.chars())
        .collect();
    let bullet_set = std::collections::BTreeSet::from([t.bullets.0, t.bullets.1, t.bullets.2]);
    let task_set: std::collections::BTreeSet<char> = TASK_MARKERS.iter().copied().collect();
    assert!(
        bullet_set.is_disjoint(&divider_set),
        "{}: bullets and every divider component must be disjoint: \
         bullets={bullet_set:?} dividers={divider_set:?}",
        t.name
    );
    assert!(
        bullet_set.is_disjoint(&task_set),
        "{}: bullets and task state must be disjoint: bullets={bullet_set:?} tasks={task_set:?}",
        t.name
    );
    assert!(
        divider_set.is_disjoint(&task_set),
        "{}: dividers and task state must be disjoint: dividers={divider_set:?} tasks={task_set:?}",
        t.name
    );
    assert_ne!(task_marker(false), task_marker(true));
    for ch in [t.bullets.0, t.bullets.1, t.bullets.2] {
        let want = if ch == '\u{2638}' {
            t.bullet_scale * BULLET_WHEEL_OPTICAL_SCALE
        } else {
            t.bullet_scale
        };
        assert_eq!(
            t.bullet_scale_for(ch),
            want,
            "{}: only the nautical wheel receives the optical enlargement",
            t.name
        );
        let want_ink = if BULLET_FULL_INK_MARKS.contains(&ch) {
            t.base_content
        } else {
            t.muted
        };
        assert_eq!(
            t.bullet_ink_for(ch),
            want_ink,
            "{}: only the thin paper-tool drawings receive full document ink",
            t.name
        );
    }
}

/// NEVER-DRIFT law (per-world LIST BULLETS): every world ships a three-glyph
/// [`Theme::bullets`] triple. The approved roster is exact; bullets, divider
/// components, and the shared task glyphs are disjoint within
/// every world. The font-DB half — that each glyph resolves — is
/// `render::tests::markdown::bullet_glyphs_resolve_in_each_worlds_assigned_face`.
#[test]
fn every_world_has_a_bullet_pair() {
    assert_eq!(
        BULLET_FULL_INK_MARKS,
        ['\u{270E}', '\u{2701}', '\u{2709}'],
        "full-ink enrollment is exactly Paperbark's pencil/scissors/envelope triple"
    );
    assert_eq!(
        BULLETS_PLAIN,
        ('•', '◦', '▪'),
        "the reusable plain baseline remains • / ◦ / ▪"
    );
    assert_eq!(BULLET_SCALE_PLAIN, 1.0, "the plain scale keeps body size");
    assert!(
        std::hint::black_box(BULLET_SCALE_ORNAMENT) > 0.0
            && BULLET_SCALE_ORNAMENT < BULLET_SCALE_PLAIN,
        "ornament bullets shape smaller than the plain body-size bullets"
    );
    let expected = [
        ("Tawny", ('\u{1F330}', '\u{1F331}', '\u{1F98B}')),
        ("Mopoke", ('\u{2606}', '\u{2601}', '\u{2604}')),
        ("Currawong", ('\u{2657}', '\u{2654}', '\u{2656}')),
        ("Potoroo", ('\u{1F330}', '\u{1F331}', '\u{1F98B}')),
        ("Gumtree", ('\u{1F426}', '\u{1F98B}', '\u{1F343}')),
        ("Bilby", ('\u{2606}', '\u{2601}', '\u{2604}')),
        ("Saltpan", ('\u{25B3}', '\u{25C7}', '\u{25CB}')),
        ("Quokka", ('\u{1F377}', '\u{2615}', '\u{2694}')),
        ("Bombora", ('\u{2693}', '\u{26F5}', '\u{2638}')),
        ("Bowerbird", ('\u{2606}', '\u{2601}', '\u{2604}')),
        ("Mulga", ('\u{2160}', '\u{2161}', '\u{2162}')),
        ("Mangrove", ('\u{2693}', '\u{26F5}', '\u{2638}')),
        ("Galah", ('\u{2680}', '\u{2681}', '\u{2682}')),
        ("Magpie", ('\u{203B}', '\u{2301}', '\u{2234}')),
        ("Brolga", ('\u{273E}', '\u{2742}', '\u{273A}')),
        ("Wagtail", ('\u{266D}', '\u{266E}', '\u{266F}')),
        ("Firetail", ('\u{2604}', '\u{2607}', '\u{2739}')),
        ("Cassowary", ('\u{2607}', '\u{2301}', '\u{2733}')),
        ("Paperbark", ('\u{270E}', '\u{2701}', '\u{2709}')),
        ("Kite", ('\u{2606}', '\u{2601}', '\u{2604}')),
    ];
    assert_eq!(THEMES.len(), expected.len(), "every live world is enrolled");
    assert_eq!(TASK_MARKERS, ['\u{2610}', '\u{1F5F9}']);
    for (t, (name, bullets)) in THEMES.iter().zip(expected) {
        assert_eq!(t.name, name, "world roster order drifted");
        assert_eq!(t.bullets, bullets, "{name}: approved bullet triple drifted");
        assert_bullet_pair_law(t);
    }
    assert_eq!(MULGA.bullet_for_depth(0), '\u{2160}');
    assert_eq!(MULGA.bullet_for_depth(1), '\u{2161}');
    assert_eq!(MULGA.bullet_for_depth(2), '\u{2162}');
    assert_eq!(MULGA.bullet_for_depth(3), '\u{2160}');

    let gumtree_divider: String = GUMTREE.ornaments.dash.chars().collect();
    assert_eq!(gumtree_divider, "\u{F591}\u{F592}\u{F592}\u{F593}");
    assert!(
        ![GUMTREE.bullets.0, GUMTREE.bullets.1, GUMTREE.bullets.2].contains(&'\u{F591}'),
        "a detached SNAKE-4 tail is not a list bullet"
    );
}

/// NEVER-DRIFT law (per-world FOLD MARK): every world's fold-chevron glyph
/// (`Theme::fold_mark`) is DERIVED from its `ornament_face`, never a
/// per-world literal — proved by construction rather than by inspection:
/// [`OrnamentRegister::ALL`] is the exhaustive roster `fold_mark_for` matches
/// with no wildcard arm (a new register fails to compile there until it is
/// given a mark), so this test only has to prove EVERY world's own
/// `ornament_face` actually resolves to a register (no silent
/// fallthrough), and that the resolved mark for a given register is the SAME
/// mark regardless of which world asked — the only way `fold_mark` could stay
/// per-world data by accident (two Junicode-register worlds getting two
/// different marks) rather than a real derivation.
#[test]
fn every_world_has_a_fold_mark_derived_from_its_ornament_register() {
    // The user-picked spec (item-475 glyph survey), pinned by VALUE: a
    // regression here is a taste regression, not a mechanism one.
    assert_eq!(
        fold_mark_for(OrnamentRegister::Garamond),
        FoldMark {
            ch: '\u{203A}',
            face: ORNAMENT_GARAMOND,
            size_frac: 1.0,
        },
        "Garamond register: EB Garamond's angle-quote"
    );
    assert_eq!(
        fold_mark_for(OrnamentRegister::Junicode),
        FoldMark {
            ch: '\u{261E}',
            face: ORNAMENT_GARAMOND,
            size_frac: 0.7,
        },
        "Junicode register: the EB-Garamond manicule"
    );
    assert_eq!(
        fold_mark_for(OrnamentRegister::Marks),
        FoldMark {
            ch: '\u{25B8}',
            face: "Iosevka",
            size_frac: 1.0,
        },
        "Marks register: Iosevka's disclosure triangle"
    );
    assert_eq!(
        fold_mark_for(OrnamentRegister::Nishiki),
        FoldMark {
            ch: '\u{25B8}',
            face: "Iosevka",
            size_frac: 1.0,
        },
        "Nishiki register consciously keeps Iosevka's disclosure triangle"
    );

    // Every register in the roster gets a mark with REAL ink dimensions (a
    // sentinel default/zero spec would pass every other assertion here).
    for register in OrnamentRegister::ALL {
        let mark = fold_mark_for(register);
        assert!(
            !mark.face.is_empty() && mark.size_frac > 0.0,
            "{register:?}: fold mark must be a real (face, size) spec, got {mark:?}"
        );
    }

    // Every world's ornament_face resolves (no panic — `ornament_register`
    // panics on an unregistered face) to the SAME mark every other world
    // sharing that register gets: the derivation is a pure function of the
    // register, not a second per-world table that happens to agree today.
    for t in THEMES.iter() {
        let register = ornament_register(t.ornament_face);
        assert_eq!(
            t.fold_mark(),
            fold_mark_for(register),
            "{}: Theme::fold_mark must equal fold_mark_for(its own register) exactly",
            t.name
        );
    }

    // All live worlds now share the Nishiki register and therefore one mark.
    for a in THEMES.iter() {
        for b in THEMES.iter() {
            assert_eq!(a.fold_mark(), b.fold_mark(), "{} vs {}", a.name, b.name);
        }
    }
}

#[test]
fn reserve_ornament_shelf_is_complete_named_and_reasoned() {
    assert_eq!(
        RESERVE_ORNAMENT_SETS.len(),
        18,
        "twenty worn sets leave eighteen reserves"
    );
    let mut names = std::collections::BTreeSet::new();
    for set in RESERVE_ORNAMENT_SETS {
        assert!(names.insert(set.name), "duplicate reserve set {}", set.name);
        assert!(
            !set.reason.trim().is_empty(),
            "{} has no reserve reason",
            set.name
        );
    }
}

/// NEVER-DRIFT law (per-world LIST-ITEM INDENT): every world's
/// [`Theme::list_indent_scale`] is exactly one of the two named tier constants
/// (no stray literal) and `>= 1.0`: the scale only ever WIDENS the typed
/// indent, never narrows it below what the raw spaces alone already give.
///
/// This dial is independent of [`Theme::bullets`]; indentation remains an
/// authored per-world geometry choice rather than a property of marker shape.
#[test]
fn every_world_has_a_list_indent_scale() {
    assert_eq!(
        LIST_INDENT_SCALE_PLAIN, 1.0,
        "the plain tier is byte-identical"
    );
    assert!(
        std::hint::black_box(LIST_INDENT_SCALE_WIDE) > LIST_INDENT_SCALE_PLAIN,
        "the wide tier must actually widen the indent"
    );
    for t in THEMES.iter() {
        assert!(
            t.list_indent_scale == LIST_INDENT_SCALE_PLAIN
                || t.list_indent_scale == LIST_INDENT_SCALE_WIDE,
            "{}: off-tier list_indent_scale {}",
            t.name,
            t.list_indent_scale
        );
        assert!(
            t.list_indent_scale >= 1.0,
            "{}: indent scale must never shrink the typed indent",
            t.name
        );
    }
}
