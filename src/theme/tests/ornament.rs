use super::super::*;

/// Every world's [`Theme::ornament_face`] AND [`Theme::bullet_face`] are the
/// one Nishiki-derived cabinet — the bullet transition (item that derives
/// each world's bullet triple from its own worn ornament set) retired the
/// transitional Garamond/Junicode bullet faces, so a bullet is drawn from
/// exactly the same font as that world's section-break trio. (The font-DB
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

/// Brolga is the ONE named, evidenced exception to "every world's bullet is
/// drawn from its own worn set": real-pixel measurement (`--exact` runs of
/// `render::tests::awl_marks_pixels::
/// every_rule_ornament_and_existing_bullet_is_legible_at_its_real_size` and
/// the two `bullet_glyph_never_touches_the_following_text*` laws, swept over
/// `bullet_scale` from 0.55 to 0.95) found NO value where all three doves of
/// Brolga's own Dovecote set both clear the legibility floor and stay clear
/// of the following text: below the crowding threshold (~0.68) at least one
/// dove's peak contrast sits under the floor, and at the scale where every
/// dove's contrast finally clears it (~0.75) the widest dove already fills
/// the bullet's fixed-width reserved box edge to edge — a genuine per-glyph
/// incompatibility with the bullet role, not a skipped measurement. Brolga
/// stays on the pre-existing plain triple pending a curator's pick of a
/// different single glyph for its bullet, or a mechanism change (a wider
/// bullet box) — both outside what this derivation pass owns.
const BULLET_PAIR_EXCEPTION: &str = "Brolga";

/// One world's own bullet-pair law: the per-level pairwise distinctness, the
/// `bullet_scale` tier membership (including the named off-tier exceptions),
/// and that every bullet glyph is drawn from the SAME adopted glyph union as
/// that world's own ornament trio, never an outside pick — except
/// [`BULLET_PAIR_EXCEPTION`], whose own doc records the real-pixel evidence.
/// Pulled out of [`every_world_has_a_bullet_pair`]'s roster loop as its own
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
    if t.name == BULLET_PAIR_EXCEPTION {
        assert_eq!(
            t.bullets, BULLETS_PLAIN,
            "{}: the named exception stays on the plain triple, not a half-migrated pick",
            t.name
        );
        assert_eq!(
            t.bullet_scale, BULLET_SCALE_PLAIN,
            "{}: the named exception stays on the plain scale",
            t.name
        );
        return;
    }
    // OFF-TIER EXCEPTIONS (each pinned by NAME and VALUE — never a loose "any
    // float passes" escape hatch). Two, for two unrelated reasons:
    // (a) CROWDING — a FACE rule, not a world list: the shared
    // [`BULLET_SCALE_ORNAMENT`] tier is scaled against the concealed `"- "`
    // prefix's advance in the world's OWN BODY font, and EB Garamond's own
    // narrow punctuation advance crowds a half-body glyph into the following
    // text on every world that wears it, so the world that needs the tighter
    // dial is decided by `t.font` alone.
    // (b) LEGIBILITY — a named single-world exception: Bilby's own worn
    // glyph measured below the real-pixel contrast floor at the shared tier
    // (`render::tests::awl_marks_pixels::
    // every_rule_ornament_and_existing_bullet_is_legible_at_its_real_size`),
    // so it alone steps up to its own named scale rather than leaving its own
    // worn set for a different glyph.
    let off_tier_exception = if t.font == ORNAMENT_GARAMOND {
        Some(BULLET_SCALE_GARAMOND)
    } else if t.name == "Bilby" {
        Some(BULLET_SCALE_HANAMI)
    } else {
        None
    };
    assert!(
        t.bullet_scale == BULLET_SCALE_ORNAMENT || off_tier_exception == Some(t.bullet_scale),
        "{}: off-tier bullet_scale {} (not a logged theme-QA padding/legibility exception)",
        t.name,
        t.bullet_scale
    );
    // THE DERIVATION LAW: a bullet is never invented vocabulary. Every glyph
    // in `t.bullets` must be a member of the exact codepoint set this world's
    // OWN ornament trio already draws from (its adopted set) — enrolled from
    // the roster itself, never a named member, so a world that ships a bullet
    // from outside its own worn set (the retired pre-Nishiki vocabulary
    // included) fails by construction rather than by a hand-list of banned
    // glyphs.
    let own_set: std::collections::BTreeSet<char> = t
        .ornaments
        .dash
        .chars()
        .chain(t.ornaments.star.chars())
        .chain(t.ornaments.underscore.chars())
        .collect();
    for (level, ch) in [
        ("level-1", t.bullets.0),
        ("level-2", t.bullets.1),
        ("level-3", t.bullets.2),
    ] {
        assert!(
            own_set.contains(&ch),
            "{}: {} bullet {:?} (U+{:04X}) is not a member of this world's own worn \
             ornament set {:?} — a bullet may only reach into the set its own trio \
             already wears",
            t.name,
            level,
            ch,
            ch as u32,
            own_set,
        );
    }
}

/// NEVER-DRIFT law (per-world LIST BULLETS): every world ships a three-glyph
/// [`Theme::bullets`] triple (the per-level rotation) whose three levels are
/// PAIRWISE DISTINCT, drawn entirely from that world's own worn ornament set
/// (no retired vocabulary survives, and no world invents a pick outside its
/// set), and a [`Theme::bullet_scale`] that is exactly the shared tier or a
/// named exception (no stray literal) — save [`BULLET_PAIR_EXCEPTION`], whose
/// own doc records why it stays unmigrated. The font-DB half — that each
/// glyph actually resolves in the world's [`Theme::bullet_face`] — is
/// `render::tests::markdown::bullet_glyphs_resolve_in_each_worlds_assigned_face`.
#[test]
fn every_world_has_a_bullet_pair() {
    assert_eq!(
        BULLETS_PLAIN,
        ('•', '◦', '▪'),
        "the plain bullet triple is • / ◦ / ▪ (retained as a documented shape, worn by every \
         live world except one named, evidenced exception)"
    );
    assert_eq!(
        BULLET_SCALE_PLAIN, 1.0,
        "the plain (and named-exception) scale keeps body size"
    );
    assert!(
        std::hint::black_box(BULLET_SCALE_ORNAMENT) > 0.0
            && BULLET_SCALE_ORNAMENT < BULLET_SCALE_PLAIN,
        "ornament bullets shape smaller than the plain body-size bullets"
    );
    for t in THEMES.iter() {
        assert_bullet_pair_law(t);
        if t.name == BULLET_PAIR_EXCEPTION {
            continue;
        }
        // No OTHER live world's bullet keeps the pre-Nishiki plain triple or
        // any other world's set — the derivation law above already proves
        // membership in THIS world's own set, which is disjoint from
        // BULLETS_PLAIN on the rest of the roster (no other world's ornament
        // trio contains `•`, `◦`, or `▪`), so this is a cheap explicit
        // restatement of the "no retired vocabulary" clause for a reader
        // scanning this test alone.
        assert_ne!(
            t.bullets, BULLETS_PLAIN,
            "{}: the plain triple is retired pre-Nishiki vocabulary, worn by no other world",
            t.name
        );
    }
    // The TRIPLE CYCLES every THREE levels — depth 2 is the third rung, and
    // depth 3 wraps back to level 1. Mulga now wears Genjikō (its own worn
    // ornament set) rather than the old shared fleuron triple.
    assert_eq!(MULGA.bullet_for_depth(0), '\u{F501}');
    assert_eq!(MULGA.bullet_for_depth(1), '\u{F500}');
    assert_eq!(MULGA.bullet_for_depth(2), '\u{F51B}');
    assert_eq!(MULGA.bullet_for_depth(3), '\u{F501}');
    assert_eq!(MULGA.bullet_for_depth(4), '\u{F500}');
    assert_eq!(MULGA.bullet_for_depth(5), '\u{F51B}');
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
/// This dial no longer correlates with [`Theme::bullets`]: since every world's
/// bullet triple now derives from that world's own worn ornament set (none
/// keep the old plain `•`/`◦`/`▪`, see [`every_world_has_a_bullet_pair`]), the
/// old "plain pair ⟺ plain indent" lockstep would force every world onto the
/// wide tier, which is a real per-world taste dial this item never touched —
/// left exactly as authored.
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
