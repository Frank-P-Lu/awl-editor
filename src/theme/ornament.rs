//! src/theme/ornament.rs — the per-world SECTION-BREAK ornament trio + the
//! per-world LIST-BULLET pair (the ornament trio, one level down): the shared
//! [`Ornaments`] type, the ornament FACE constants, the three ornament
//! SCALE tiers, and the two bullet-scale tiers. See [`crate::theme::worlds`]
//! for how each world picks from this data.

// --- The PER-SYNTAX thematic-break ornament set -----------------------------

/// The PER-SYNTAX thematic-break ornament set — one glyph for each of markdown's
/// three `<hr>` spellings, so a break's ORNAMENT tracks what the author typed:
/// `---` (dash), `***` (star), `___` (underscore). Each renders CENTERED in the
/// writing column from the world's ornament face and is REVEALED back to its raw
/// characters when the caret lands on the line (reveal-on-cursor). A field is a
/// whole shaped run rather than one scalar: the Nishiki cabinet includes joining
/// pieces whose intended drawing only exists when they are shaped together.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Ornaments {
    /// The mark for a `---` dash rule.
    pub dash: &'static str,
    /// The mark for a `***` star rule.
    pub star: &'static str,
    /// The mark for a `___` underscore rule.
    pub underscore: &'static str,
}

impl Ornaments {
    /// The trio in break-syntax order — `---` / `***` / `___`.
    pub const fn of(dash: &'static str, star: &'static str, underscore: &'static str) -> Ornaments {
        Ornaments {
            dash,
            star,
            underscore,
        }
    }

    /// The ornament this world draws for a given break syntax.
    pub const fn pick(&self, kind: crate::markdown::BreakKind) -> &'static str {
        match kind {
            crate::markdown::BreakKind::Dash => self.dash,
            crate::markdown::BreakKind::Star => self.star,
            crate::markdown::BreakKind::Underscore => self.underscore,
        }
    }
}

/// The shared DEFAULT ornament set: `---` → ❧ fleuron, `***` → ⁂ asterism (three
/// stars for three asterisks), `___` → ❦ floral heart. All three are bundled in
/// the derived `AwlMarks.ttf` (the [`ORNAMENT_MARKS`] face), so they render in
/// every world that keeps that face.
pub const ORNAMENTS_DEFAULT: Ornaments = Ornaments {
    dash: "❧",
    star: "⁂",
    underscore: "❦",
};

// --- The section-break ORNAMENT FACE (and About end-mark face) ----------------
//
// The section-break/About ornament cabinet is Nishiki-derived in every live
// world. Keycaps (⌘⌥⇧) and the plain typographic marks (§ † ‡ …) stay on the
// derived marks face (`render::SYMBOL_FAMILY`). List bullets deliberately retain
// their previous per-world face until their own fitting-room decision is made.
//
// The legacy faces remain bundled and registered because the bullet transition
// still uses them. Section-break glyph coverage is derived from the live Nishiki
// assignments and pinned against the font manifest by render laws.

/// The EB Garamond ornament face — Renaissance fleurons for the literary serif
/// worlds, registered from `EBGaramond-Regular.ttf`. Covers ❧ ❦ ☙ and NOTHING
/// else: no ⁂, ❡ or ❥, so a Garamond world's trio can only be those three
/// permuted. The never-tofu coverage test holds every world to its own face.
pub const ORNAMENT_GARAMOND: &str = "EB Garamond";

/// The Junicode ornament face — antique Caslon flowers for the expressive/slab
/// worlds, registered from `Junicode-Ornaments.ttf`. Covers ❧ ❦ ☙ ⁂ ⁑ plus a
/// deep pool of PUA botanical/damask/tile clusters, but NOT ❡/❥.
pub const ORNAMENT_JUNICODE: &str = "Junicode";

/// The Nishiki-derived marks face (== `render::SYMBOL_FAMILY`, `AwlMarks.ttf`).
/// Covers the default ornaments (❧ ❦ ☙ ❡ ❥ ⁂) plus the star/floret/geometric
/// pool (✦ ✧ ✴ ✶ ✷ ✽ ✿ ❀ ❁ ❂ ❖ ◆ ◈ ⬥ ⭑).
///
/// Naming the constant HERE keeps `theme.rs` free of a `crate::render`
/// dependency in the `const` world literals; `theme::tests::ornament` pins the
/// two spellings equal.
pub const ORNAMENT_MARKS: &str = "Awl Marks";

/// The Nishiki-derived cabinet used by every section break. The family name
/// intentionally stays private (`Awl Marks`); Nishiki-teki is the source and
/// register identity, not a shipped family name.
pub const ORNAMENT_NISHIKI: &str = ORNAMENT_MARKS;

// --- The per-world FOLD-MARK glyph (a THIRD thing this same register drives) --
//
// The fold chevron (`render::layers::fold_chevron`) reads the SAME
// [`OrnamentRegister`] the section-break fleuron does, never a second per-world
// list — so a new world inherits a fold mark the moment it sets
// `ornament_face`, with no second field to remember.

enum_with_all! {
    /// The ornament-face FLAVOUR TIERS a world's `ornament_face`
    /// resolves to — see [`ornament_register`]. Exhaustively matched by
    /// [`fold_mark_for`], so a fourth tier (a new ornament face constant) is a
    /// compile error here until it is given a fold mark, not a silent
    /// fallthrough to one of the other three.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum OrnamentRegister {
        /// The true literary serifs — [`ORNAMENT_GARAMOND`]'s own register.
        Garamond,
        /// The antique/expressive worlds — [`ORNAMENT_JUNICODE`]'s register.
        Junicode,
        /// The retired geometric grouping, retained in the exhaustive fold
        /// vocabulary while live worlds use [`Nishiki`](Self::Nishiki).
        Marks,
        /// The Nishiki-teki-derived cabinet in [`ORNAMENT_NISHIKI`].
        Nishiki,
    }
}

/// Classify a world's own `ornament_face` into its [`OrnamentRegister`].
///
/// PANICS on an unrecognised face rather than defaulting. Reaching the `else`
/// arm means a FOURTH ornament face was registered without a matching register
/// variant, and a default there would leave that face's worlds with a silently
/// wrong fold mark instead of a loud failure.
pub fn ornament_register(face: &'static str) -> OrnamentRegister {
    if face == ORNAMENT_GARAMOND {
        OrnamentRegister::Garamond
    } else if face == ORNAMENT_JUNICODE {
        OrnamentRegister::Junicode
    } else if face == ORNAMENT_NISHIKI {
        OrnamentRegister::Nishiki
    } else {
        panic!(
            "ornament_register: {face:?} is not a registered ornament face — a world's \
             ornament_face must resolve to a real register so its fold mark is never \
             silently unset"
        );
    }
}

/// One register's fold-mark glyph spec: which char, in which already-bundled
/// face, at what fraction of the heading's own font size.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FoldMark {
    pub ch: char,
    pub face: &'static str,
    /// Fraction of the heading's own font size the mark composes at. Every
    /// register but one draws a glyph already comfortably narrower than its
    /// leading-pad box at full size (see `render::layers::fold_chevron`'s own
    /// box-fit numbers); the manicule alone needs a real fraction below 1.0,
    /// because a pointing hand's ink footprint is wider — at ANY font size —
    /// than the angle marks the box was originally sized around.
    pub size_frac: f32,
}

/// The fold mark this register draws. The glyphs are the USER'S OWN PICKS from
/// a rendered survey (`captures/`), each already covered by a bundled OFL face
/// (`assets/fonts/LICENSES.md`) so none of them costs new font bytes. An
/// EXHAUSTIVE match, no wildcard arm: a new [`OrnamentRegister`] variant fails
/// to compile here until it is given a mark.
pub fn fold_mark_for(register: OrnamentRegister) -> FoldMark {
    match register {
        // EB Garamond's own angle-quote — the pre-quad original, from the
        // true literary serifs' own display face.
        OrnamentRegister::Garamond => FoldMark {
            ch: '\u{203A}',
            face: ORNAMENT_GARAMOND,
            size_frac: 1.0,
        },
        // The manuscript-margin manicule — from EB Garamond (Junicode itself
        // carries no such glyph), the antique/expressive register's wilder
        // pick.
        OrnamentRegister::Junicode => FoldMark {
            ch: '\u{261E}',
            face: ORNAMENT_GARAMOND,
            size_frac: 0.7,
        },
        // Iosevka's own disclosure triangle — the modern/technical
        // register's pick, the classic Finder/macOS convention.
        OrnamentRegister::Marks => FoldMark {
            ch: '\u{25B8}',
            face: "Iosevka",
            size_frac: 1.0,
        },
        // The derived cabinet keeps the same quiet disclosure triangle the
        // geometric register established. Nishiki supplies the section-break
        // drawing; the fold mark remains Iosevka's purpose-built UI triangle.
        OrnamentRegister::Nishiki => FoldMark {
            ch: '\u{25B8}',
            face: "Iosevka",
            size_frac: 1.0,
        },
    }
}

/// One set retained from the fitting room but not assigned to a shipping world.
/// The glyph details remain in the fitting-room artifact; this roster records
/// the durable product decision that keeps a set available (or benched) and why.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReserveOrnamentSet {
    pub name: &'static str,
    pub reason: &'static str,
}

/// The complete unworn shelf after the twenty unique world assignments.
pub const RESERVE_ORNAMENT_SETS: &[ReserveOrnamentSet] = &[
    ReserveOrnamentSet {
        name: "Lunar",
        reason: "benched: moon circles did not fit; Moonfaces is the playful replacement",
    },
    ReserveOrnamentSet {
        name: "Florets",
        reason: "liked, but no world story beat the twenty selected sets",
    },
    ReserveOrnamentSet {
        name: "Geometrics",
        reason: "fine but not exceptional beside the selected cabinet",
    },
    ReserveOrnamentSet {
        name: "Tally",
        reason: "benched: its counting rationale reads as culturally alien to Western eyes",
    },
    ReserveOrnamentSet {
        name: "Manicules",
        reason: "swapped out during fitting; retained as printerly margin-mark reserve",
    },
    ReserveOrnamentSet {
        name: "Caslon",
        reason: "lovely chain, cartouche, and rosette; too ornate for the stark worlds",
    },
    ReserveOrnamentSet {
        name: "Genjiko II",
        reason: "benched as redundant with the selected Genjiko set",
    },
    ReserveOrnamentSet {
        name: "Keizuko",
        reason: "benched as redundant with the selected Genjiko set",
    },
    ReserveOrnamentSet {
        name: "Palms",
        reason: "liked, but no world story beat the twenty selected sets",
    },
    ReserveOrnamentSet {
        name: "Acorns",
        reason: "benched by the three-distinct-drawings rule",
    },
    ReserveOrnamentSet {
        name: "Hearts",
        reason: "unworn after the story-led assignment",
    },
    ReserveOrnamentSet {
        name: "Snow",
        reason: "benched by the three-distinct-drawings rule",
    },
    ReserveOrnamentSet {
        name: "Tallybars",
        reason: "unworn after the story-led assignment",
    },
    ReserveOrnamentSet {
        name: "Heraldry",
        reason: "unworn after the story-led assignment",
    },
    ReserveOrnamentSet {
        name: "Harbour",
        reason: "benched: anchor, sailboat, and helm mix illustration and diagram registers",
    },
    ReserveOrnamentSet {
        name: "Reference Marks",
        reason: "reserved for the traditional footnote-reference ladder",
    },
    ReserveOrnamentSet {
        name: "Rubrication",
        reason: "typographic heritage reserve; no world assignment won",
    },
    ReserveOrnamentSet {
        name: "Curiosities",
        reason: "shelved; Currawong is its natural wearer if revisited",
    },
];

// --- The per-world ORNAMENT SCALE (how big the section-break fleuron reads) ----
//
// A thematic-break line grows its whole ROW by
// [`crate::theme::Theme::ornament_scale`], in three tiers keyed to the ornament's
// existing per-world tuning. The cabinet repick deliberately keeps these dials:
// the new glyph identity does not flatten the row rhythm chosen for each world.
//
// The field is read by BOTH `render::spans::md_line_scale` (the break ROW height)
// and `render::layers::prepare_ornaments` (the glyph LINE-BOX). Both must read
// this one dial or the tall row stops centering the glyph. TASTE DEFAULTS, one
// per tier.

/// ORNATE ornament scale — the Junicode Caslon-flower worlds. The most detailed
/// ornaments carry the most size.
pub const ORNAMENT_SCALE_ORNATE: f32 = 2.2;

/// FLEURON ornament scale — the EB Garamond literary-serif worlds. A generous but
/// slightly quieter flourish than the ornate flowers.
pub const ORNAMENT_SCALE_FLEURON: f32 = 1.8;

/// GEOMETRIC ornament scale — the Awl Marks stars/florets/diamonds. The clean
/// geometric marks read best kept modest, so they sit lowest on the tier ladder.
pub const ORNAMENT_SCALE_GEOMETRIC: f32 = 1.5;

// --- The per-world LIST BULLET triple + scale (the ornament trio, one level down) --
//
// The unordered-list bullet ([`crate::theme::Theme::bullets`], drawn over a
// concealed `-`/`*`/`+` the caret is off) is PER-WORLD DATA drawn in the world's
// own [`crate::theme::Theme::bullet_face`]. That transitional face preserves
// the already-approved pairs until their separate fitting round; a bullet can
// only use glyphs that face actually ships.
// `render::tests::markdown::bullet_glyphs_resolve_in_each_worlds_assigned_face`
// holds every pick to that.
//
// [`crate::theme::Theme::bullet_for_depth`] cycles `.0`/`.1`/`.2` every THREE
// nesting levels, composing the LEVEL axis with the per-WORLD one. Bombora's
// manicule is deliberately exclusive to depth 0 — a pointing hand on every
// bullet is loud — so that world's `.2` comes from the fleuron pool instead.

/// The plain geometric bullet triple — `•` filled / `◦` hollow / `▪` small
/// square, all three in the merged [`ORNAMENT_MARKS`] face. Restraint IS the
/// modern/technical worlds' character; a bullet is not the place to decorate
/// them for symmetry with the ornate worlds.
pub const BULLETS_PLAIN: (char, char, char) = ('•', '◦', '▪');

/// PLAIN bullet scale — the geometric worlds' bullets sit at body size.
pub const BULLET_SCALE_PLAIN: f32 = 1.0;

/// ORNAMENT bullet scale — a hedera / fleuron / manicule shaped at ~half body so
/// it reads as a quiet bullet-sized marker, not a section-break flourish. A
/// TASTE DEFAULT, one dial for every characterful world.
///
/// ⚠️ This tier is a byproduct of two UNRELATED font metrics: the concealed
/// `"<marker> "` prefix's advance in the world's own BODY font, and the ornament
/// glyph's own ink width in its ORNAMENT face. Nothing makes them agree, so a
/// world can land here with its bullet crowding the text that follows —
/// [`BULLET_SCALE_GARAMOND`] is the one such rule that has been needed so far,
/// and `render::tests::markdown::bullet_glyph_never_touches_the_following_text_in_any_world`
/// is what catches the next one.
pub const BULLET_SCALE_ORNAMENT: f32 = 0.55;

/// The EB-GARAMOND-BODY bullet scale — a FACE rule, never a per-world taste
/// exception. EB Garamond's punctuation advance is narrow enough that a
/// half-body fleuron crowds out the text that follows, so EVERY world whose
/// BODY face is EB Garamond carries this tier.
/// `theme::tests::every_world_has_a_bullet_pair` derives the allowance from the
/// face rather than from a world-name list, which is what keeps the next world
/// to adopt that face from failing the padding law.
pub const BULLET_SCALE_GARAMOND: f32 = 0.35;

// --- The per-world LIST-ITEM INDENT scale (the other half of bullet-
// level readability) ---------------------------------------------------------
//
// [`crate::theme::Theme::list_indent_scale`] makes a nested list item's
// per-level STEP a real dial rather than whatever a world's own space glyph
// happens to measure: `render::spans::add_list_indent_span` widens the leading
// -space RUN (bytes `0..indent`) by this factor before layout. Pure advance, no
// visible glyph — spaces carry no ink — and depth 0 is an empty range, so a
// top-level item is untouched on every world whatever the dial says.
//
// Two tiers, mirroring [`BULLET_SCALE_PLAIN`]/[`BULLET_SCALE_ORNAMENT`] over the
// same roster split. TASTE DEFAULTS: the multipliers are judged from the
// gallery, and the mechanism is what is committed to.

/// PLAIN list-indent scale — the geometric/technical worlds' nested list items
/// render at exactly their literal typed indent.
pub const LIST_INDENT_SCALE_PLAIN: f32 = 1.0;

/// WIDE list-indent scale — the antique/literary-serif worlds' nested list
/// items get a touch more breathing room per level: each leading-space run
/// renders 50% wider than its literal typed width, so a depth-1 item (2 typed
/// spaces) gains one extra space-width of rail, a depth-2 item (4 typed
/// spaces) gains two, and so on — linear in depth for free, since the typed
/// indent itself already is.
pub const LIST_INDENT_SCALE_WIDE: f32 = 1.5;
