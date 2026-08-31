//! src/theme/ornament.rs — the per-world SECTION-BREAK ornament trio + the
//! per-world LIST-BULLET pair (the ornament trio, one level down): the shared
//! [`Ornaments`] type, the three ornament FACE constants, the three ornament
//! SCALE tiers, and the two bullet-scale tiers. See [`crate::theme::worlds`]
//! for how each world picks from this data.

// --- The PER-SYNTAX thematic-break ornament set -----------------------------

/// The PER-SYNTAX thematic-break ornament set — one glyph for each of markdown's
/// three `<hr>` spellings, so a break's ORNAMENT tracks what the author typed:
/// `---` (dash), `***` (star), `___` (underscore). Each renders CENTERED in the
/// writing column from the bundled `SYMBOL_FAMILY` face (see
/// [`crate::render::spans::is_symbol`]), and is REVEALED back to its raw characters
/// when the caret lands on the line (reveal-on-cursor). The three defaults live in
/// [`ORNAMENTS_DEFAULT`]; a world may override for its own face's flavour.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Ornaments {
    /// The mark for a `---` dash rule.
    pub dash: char,
    /// The mark for a `***` star rule.
    pub star: char,
    /// The mark for a `___` underscore rule.
    pub underscore: char,
}

impl Ornaments {
    /// The trio in break-syntax order — `---` / `***` / `___`.
    pub const fn of(dash: char, star: char, underscore: char) -> Ornaments {
        Ornaments {
            dash,
            star,
            underscore,
        }
    }

    /// The ornament this world draws for a given break syntax.
    pub const fn pick(&self, kind: crate::markdown::BreakKind) -> char {
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
    dash: '❧',
    star: '⁂',
    underscore: '❦',
};

// --- The per-world ORNAMENT FACE (the fleuron / About end-mark face) ----------
//
// ONLY the section-break/About ornament changes face per world. Keycaps (⌘⌥⇧)
// and the plain typographic marks (§ † ‡ • ◦ ▪ …) stay on the derived marks face
// (`render::SYMBOL_FAMILY`) whatever a world's ornament face is.
//
// Three faces, all bundled and OFL, one per flavour register — and each one's
// GLYPH COVERAGE is the constraint that decides which trios a world may pick,
// so it is stated on the constant itself. Which world takes which face is
// `theme::worlds` data, not restated here.

/// The EB Garamond ornament face — Renaissance fleurons for the literary serif
/// worlds, registered from `EBGaramond-Regular.ttf`. Covers ❧ ❦ ☙ and NOTHING
/// else: no ⁂, ❡ or ❥, so a Garamond world's trio can only be those three
/// permuted. The never-tofu coverage test holds every world to its own face.
pub const ORNAMENT_GARAMOND: &str = "EB Garamond";

/// The Junicode ornament face — antique Caslon flowers for the expressive/slab
/// worlds, registered from `Junicode-Ornaments.ttf`. Covers ❧ ❦ ☙ ⁂ ⁑ plus a
/// deep pool of PUA botanical/damask/tile clusters, but NOT ❡/❥.
pub const ORNAMENT_JUNICODE: &str = "Junicode";

/// The Nishiki-derived marks face (== `render::SYMBOL_FAMILY`, `AwlMarks.ttf`) —
/// the geometric/technical worlds' ornament face.
/// Covers the default ornaments (❧ ❦ ☙ ❡ ❥ ⁂) plus the star/floret/geometric
/// pool (✦ ✧ ✴ ✶ ✷ ✽ ✿ ❀ ❁ ❂ ❖ ◆ ◈ ⬥ ⭑).
///
/// Naming the constant HERE keeps `theme.rs` free of a `crate::render`
/// dependency in the `const` world literals; `theme::tests::ornament` pins the
/// two spellings equal.
pub const ORNAMENT_MARKS: &str = "Awl Marks";

// --- The per-world FOLD-MARK glyph (a THIRD thing this same register drives) --
//
// The fold chevron (`render::layers::fold_chevron`) reads the SAME
// [`OrnamentRegister`] the section-break fleuron does, never a second per-world
// list — so a new world inherits a fold mark the moment it sets
// `ornament_face`, with no second field to remember.

enum_with_all! {
    /// The three ornament-face FLAVOUR TIERS every world's `ornament_face`
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
        /// The modern/technical/geometric worlds — [`ORNAMENT_MARKS`]'s
        /// register.
        Marks,
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
    } else if face == ORNAMENT_MARKS {
        OrnamentRegister::Marks
    } else {
        panic!(
            "ornament_register: {face:?} is not one of the three registered ornament \
             faces (ORNAMENT_GARAMOND/ORNAMENT_JUNICODE/ORNAMENT_MARKS) — a world's \
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
    }
}

// --- The per-world ORNAMENT SCALE (how big the section-break fleuron reads) ----
//
// A thematic-break line grows its whole ROW by
// [`crate::theme::Theme::ornament_scale`], in three tiers keyed to the ornament's
// CHARACTER rather than to the world — detailed flowers reward size, clean
// geometric marks do not.
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
// own [`crate::theme::Theme::ornament_face`] — the same face discipline as the
// section-break trio, so a bullet can only use glyphs that face actually ships.
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
