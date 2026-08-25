use super::super::*;

/// Every world's [`Theme::ornament_face`] is exactly one of the THREE bundled
/// ornament faces — no world ships an unregistered / typo'd family that would
/// tofu the section-break fleuron. (The font-DB half — that each face actually
/// COVERS its world's glyphs — is `render::tests::cjk::
/// ornament_glyphs_resolve_in_each_worlds_assigned_face`, which needs a built
/// `FontSystem`.) Also pins `ORNAMENT_MARKS == render::SYMBOL_FAMILY`, the one
/// coupling `theme.rs` states as data rather than importing.
#[test]
fn every_world_ornament_face_is_a_registered_ornament_face() {
    assert_eq!(
        ORNAMENT_MARKS,
        crate::render::SYMBOL_FAMILY,
        "the geometric worlds' ornament face IS the merged marks face"
    );
    for t in THEMES.iter() {
        assert!(
            matches!(
                t.ornament_face,
                ORNAMENT_GARAMOND | ORNAMENT_JUNICODE | ORNAMENT_MARKS
            ),
            "{} has an unrecognized ornament_face {:?}",
            t.name,
            t.ornament_face
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

/// NEVER-DRIFT law: every world ships an [`Theme::ornament_scale`], and it is
/// exactly one of the three named tier constants — a world can't silently drift to
/// a bare literal that neither reader (`md_line_scale` / `prepare_ornaments`) would
/// then keep in lockstep. Also pins the three tier VALUES (the taste defaults) and
/// a sample world per tier, keyed to the ornament's CHARACTER.
#[test]
fn every_world_has_an_ornament_scale() {
    // The three tiers are the settled taste defaults.
    assert_eq!(ORNAMENT_SCALE_ORNATE, 2.2, "ornate tier is 2.2");
    assert_eq!(ORNAMENT_SCALE_FLEURON, 1.8, "fleuron tier is 1.8");
    assert_eq!(ORNAMENT_SCALE_GEOMETRIC, 1.5, "geometric tier is 1.5");
    assert!(
        std::hint::black_box(ORNAMENT_SCALE_ORNATE) > ORNAMENT_SCALE_FLEURON
            && ORNAMENT_SCALE_FLEURON > ORNAMENT_SCALE_GEOMETRIC,
        "the tiers descend ornate > fleuron > geometric"
    );

    // Every world's scale IS one of the three tiers — no stray literal.
    for t in THEMES.iter() {
        assert!(
            matches!(
                t.ornament_scale,
                ORNAMENT_SCALE_ORNATE | ORNAMENT_SCALE_FLEURON | ORNAMENT_SCALE_GEOMETRIC
            ),
            "{} has an off-tier ornament_scale {}",
            t.name,
            t.ornament_scale
        );
    }

    // One sample per tier (the spec's pinned assignments).
    let by = |name: &str| set_active_by_name(name).unwrap().ornament_scale;
    let _t = crate::testlock::serial();
    assert_eq!(by("Mopoke"), 2.2, "Mopoke (Junicode flowers) is ornate 2.2");
    assert_eq!(
        by("Bombora"),
        1.8,
        "Bombora (Garamond fleurons) is fleuron 1.8"
    );
    assert_eq!(
        by("Currawong"),
        1.5,
        "Currawong (geometric marks) is geometric 1.5"
    );
    set_active(DEFAULT_THEME);
}

/// One world's own bullet-pair law: the per-level pairwise distinctness, the
/// `bullet_scale` tier membership (including the one named, face-derived
/// off-tier exception), and the plain-pair/plain-scale/geometric lockstep —
/// pulled out of [`every_world_has_a_bullet_pair`]'s roster loop as its own
/// named unit, so "the law for one world" and "sweep every world" are two
/// separate, independently readable concerns.
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
    // OFF-TIER EXCEPTION (EXACTLY one, pinned by NAME and VALUE — never a
    // loose "any float passes" escape hatch): the shared
    // [`BULLET_SCALE_ORNAMENT`] tier is a byproduct of two unrelated font
    // metrics (see that constant's own doc) that pair badly on a manicule
    // (too wide, touching the following text). Every other world stays on
    // a shared tier.
    // The exception is FACE-DERIVED, not a world list. The shared tier is
    // scaled against the concealed `"- "` prefix's advance in the world's
    // OWN BODY font, so the world that needs a tighter dial is decided by
    // that face: EB Garamond's narrow punctuation advance crowds a
    // half-body fleuron into the following text, on every world that wears
    // it.
    let off_tier_exception = (t.font == ORNAMENT_GARAMOND).then_some(BULLET_SCALE_GARAMOND);
    assert!(
        matches!(t.bullet_scale, BULLET_SCALE_PLAIN | BULLET_SCALE_ORNAMENT)
            || off_tier_exception == Some(t.bullet_scale),
        "{}: off-tier bullet_scale {} (not a logged theme-QA padding exception)",
        t.name,
        t.bullet_scale
    );
    // The geometric/technical worlds keep the plain pair AND body size, in
    // lockstep — a characterful pair at body size (or plain at half) would be
    // a taste drift; a geometric world is byte-identical to before this round.
    // The two off-tier exceptions are excluded from this lockstep check (their
    // whole POINT is a bullet_scale that differs from the shared ORNAMENT tier
    // while keeping a characterful, non-plain pair).
    let geometric = t.ornament_face == ORNAMENT_MARKS;
    if off_tier_exception.is_none() {
        assert_eq!(
            t.bullets == BULLETS_PLAIN,
            t.bullet_scale == BULLET_SCALE_PLAIN,
            "{}: plain-pair and plain-scale must agree (geometric restraint)",
            t.name
        );
    }
    if geometric {
        assert_eq!(
            t.bullets, BULLETS_PLAIN,
            "{}: an Awl-Marks world keeps the plain • / ◦ (restraint)",
            t.name
        );
    } else {
        assert_ne!(
            t.bullets, BULLETS_PLAIN,
            "{}: an antique/literary serif world draws a characterful bullet",
            t.name
        );
    }
}

/// NEVER-DRIFT law (per-world LIST BULLETS): every world ships a three-glyph
/// [`Theme::bullets`] triple (the per-level rotation) whose three levels
/// are PAIRWISE DISTINCT, and a [`Theme::bullet_scale`] that is exactly one of
/// the two named tier constants (no stray literal). The font-DB half — that
/// each glyph actually resolves in the world's [`Theme::ornament_face`] — is
/// `render::tests::markdown::bullet_glyphs_resolve_in_each_worlds_assigned_face`.
/// Also pins the geometric worlds to the plain byte-identical
/// [`BULLETS_PLAIN`]/[`BULLET_SCALE_PLAIN`] (restraint) and the manicule
/// showpiece (Bombora's level-1 ☞, exclusive to that one level).
#[test]
fn every_world_has_a_bullet_pair() {
    assert_eq!(
        BULLETS_PLAIN,
        ('•', '◦', '▪'),
        "the plain bullet triple is • / ◦ / ▪"
    );
    assert_eq!(BULLET_SCALE_PLAIN, 1.0, "plain bullets keep body size");
    assert!(
        std::hint::black_box(BULLET_SCALE_ORNAMENT) > 0.0
            && BULLET_SCALE_ORNAMENT < BULLET_SCALE_PLAIN,
        "ornament bullets shape smaller than the plain body-size bullets"
    );
    for t in THEMES.iter() {
        assert_bullet_pair_law(t);
    }
    // The TRIPLE CYCLES every THREE levels — depth 2 is the third rung, and
    // depth 3 wraps back to level 1.
    assert_eq!(TAWNY.bullet_for_depth(0), '•');
    assert_eq!(TAWNY.bullet_for_depth(1), '◦');
    assert_eq!(TAWNY.bullet_for_depth(2), '▪');
    assert_eq!(TAWNY.bullet_for_depth(3), '•');
    assert_eq!(TAWNY.bullet_for_depth(4), '◦');
    assert_eq!(TAWNY.bullet_for_depth(5), '▪');
    assert_eq!(BOMBORA.bullet_for_depth(0), '☞');
    assert_eq!(BOMBORA.bullet_for_depth(1), '❧');
    assert_eq!(BOMBORA.bullet_for_depth(2), '❦');
    assert_eq!(BOMBORA.bullet_for_depth(3), '☞');
    // The manicule showpiece: Bombora alone rides the antique pointing hand,
    // at its top level (level 1) — NEVER at level 3 either (the rotation
    // composes with, never dilutes, the "one world, one level" pick).
    assert_eq!(
        BOMBORA.bullets.0, '☞',
        "Bombora's level-1 bullet is the manicule"
    );
    assert!(
        THEMES
            .iter()
            .filter(|t| t.bullets.0 == '☞' || t.bullets.1 == '☞' || t.bullets.2 == '☞')
            .count()
            == 1,
        "exactly one world uses the manicule bullet, at exactly one level (a hand everywhere is loud)"
    );
}

/// NEVER-DRIFT law (per-world FOLD MARK): every world's fold-chevron glyph
/// (`Theme::fold_mark`) is DERIVED from its `ornament_face`, never a
/// per-world literal — proved by construction rather than by inspection:
/// [`OrnamentRegister::ALL`] is the exhaustive roster `fold_mark_for` matches
/// with no wildcard arm (a fourth register fails to compile there until it is
/// given a mark), so this test only has to prove EVERY world's own
/// `ornament_face` actually resolves to one of the three (no silent
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

    // Non-vacuous grouping check: the three registers are represented (every
    // world lands in one of them), and worlds sharing a register share a
    // mark while worlds in different registers draw different marks — the
    // property a hand-per-world list could violate silently.
    for a in THEMES.iter() {
        for b in THEMES.iter() {
            let same_register =
                ornament_register(a.ornament_face) == ornament_register(b.ornament_face);
            assert_eq!(
                a.fold_mark() == b.fold_mark(),
                same_register,
                "{} vs {}: fold marks must agree iff their ornament registers agree",
                a.name,
                b.name
            );
        }
    }
}

/// NEVER-DRIFT law (per-world LIST-ITEM INDENT): every world's
/// [`Theme::list_indent_scale`] is exactly one of the two named tier constants
/// (no stray literal, mirroring [`every_world_has_a_bullet_pair`]'s
/// `bullet_scale` sweep) and — since the shared tier IS the shared bullet-scale
/// tier's own roster — agrees with the world's own bullet PAIR: a plain `•`/
/// `◦`/`▪` world stays at the byte-identical [`LIST_INDENT_SCALE_PLAIN`], an
/// antique/literary-serif world (hedera/fleuron/manicule) steps up to
/// [`LIST_INDENT_SCALE_WIDE`]. `>= 1.0` on every world: the scale only ever
/// WIDENS the typed indent, never narrows it below what the raw spaces alone
/// already give.
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
        let plain_pair = t.bullets == BULLETS_PLAIN;
        assert_eq!(
            t.list_indent_scale == LIST_INDENT_SCALE_PLAIN,
            plain_pair,
            "{}: plain-pair and plain-indent-scale must agree",
            t.name
        );
    }
}
