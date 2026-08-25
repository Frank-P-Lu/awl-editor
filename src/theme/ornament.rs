//! src/theme/ornament.rs — the per-world SECTION-BREAK ornament trio + the
//! per-world LIST-BULLET pair (the ornament trio, one level down): the shared
//! [`Ornaments`] type, the three ornament FACE constants, the three ornament
//! SCALE tiers, and the two bullet-scale tiers. See [`crate::theme::worlds`]
//! for how each of the sixteen worlds picks from this data.

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
    /// `---` (a dash rule) → the fleuron. Default ❧ (U+2767).
    pub dash: char,
    /// `***` (a star rule) → the asterism — three stars for three asterisks, the
    /// natural match. Default ⁂ (U+2042).
    pub star: char,
    /// `___` (an underscore rule) → the floral heart. Default ❦ (U+2766).
    pub underscore: char,
}

impl Ornaments {
    /// The trio in break-syntax order — `---` / `***` / `___`. A const
    /// constructor so a world literal states its three marks on one line
    /// instead of restating three field names it cannot reorder anyway.
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
/// the merged `AwlMarks.ttf` (the [`ORNAMENT_MARKS`] face), so they render in
/// every world that keeps that face.
pub const ORNAMENTS_DEFAULT: Ornaments = Ornaments {
    dash: '❧',
    star: '⁂',
    underscore: '❦',
};

// --- The per-world ORNAMENT FACE (the fleuron / About end-mark face) ----------
//
// Each world draws its markdown SECTION-BREAK ornament (the `---`/`***`/`___`
// fleuron) AND its About-card closing end-mark in its OWN assigned face, instead
// of the shared merged marks face. Keycaps (⌘⌥⇧) and the plain typographic marks
// (§ † ‡ • ◦ ▪ …) stay on the merged marks face (`render::SYMBOL_FAMILY`) — ONLY
// the section-break/About ornament changes face. The three faces (all bundled,
// all OFL) map to the three flavour registers:
//
//   * [`ORNAMENT_GARAMOND`] — EB Garamond's Renaissance fleurons (❧ ❦ ☙), for the
//     TRUE literary serifs (Bilby, Bombora). NOTE: EB Garamond ships NO ⁂ asterism
//     (nor ❡/❥) and only those THREE fleurons, so a Garamond world's trio is exactly
//     {❧, ☙, ❦} permuted — never ⁂ — see the NEVER-TOFU coverage test.
//   * [`ORNAMENT_JUNICODE`] — Junicode's antique Caslon flowers (❧ ❦ ☙ + the ⁂/⁑
//     asterisms + a deep pool of PUA botanical/damask/tile ornaments), for the
//     antique/expressive/slab worlds AND the warm/pale literary serifs whose display
//     face carries no fleurons of its own (Gumtree, Saltpan, Magpie, Mopoke, Mulga):
//     each gets a distinct in-character trio (a botanical sprig / running vine /
//     quatrefoil-tile / damask-flourish / typographic-asterism family, respectively).
//   * [`ORNAMENT_MARKS`] — the merged marks face itself (`render::SYMBOL_FAMILY`),
//     for the modern/technical/GEOMETRIC worlds: it carries the Noto Sans Symbols
//     2 geometric marks (its ❡ ❥ come from NS2; ❧ ❦ ☙ from EB Garamond; ⁂ from
//     Junicode). There is no STANDALONE "Noto Sans Symbols 2" registered face —
//     its glyphs live in this merged face, which is exactly the clean geometric
//     look the technical worlds want, so they simply keep it (their ornament is
//     byte-identical to before this round).

/// The EB Garamond ornament face — refined Renaissance fleurons for the literary
/// serif worlds. Registered from `EBGaramond-Regular.ttf` (also Bombora's own
/// display face). Covers ❧ ❦ ☙ but NOT ⁂/❡/❥.
pub const ORNAMENT_GARAMOND: &str = "EB Garamond";

/// The Junicode ornament face — antique Caslon flowers for the expressive/slab
/// worlds. Registered from `Junicode-Ornaments.ttf`. Covers ❧ ❦ ☙ ⁂ ⁑ + PUA
/// fleuron clusters (NOT ❡/❥).
pub const ORNAMENT_JUNICODE: &str = "Junicode";

/// The merged marks face (== `render::SYMBOL_FAMILY`, `AwlMarks.ttf`) — the
/// geometric/technical worlds' ornament face. Carries the Noto Sans Symbols 2
/// geometric marks; covers the default ornaments (❧ ❦ ☙ ❡ ❥ ⁂) PLUS the expanded
/// star/floret/geometric pool this round draws its per-world trios from (✦ ✧ ✴ ✶
/// ✷ ✽ ✿ ❀ ❁ ❂ ❖ ◆ ◈ ⬥ ⭑). Naming the constant here keeps `theme.rs` free of a
/// `crate::render` dependency in the `const` world literals; the two are asserted
/// equal by a test.
pub const ORNAMENT_MARKS: &str = "Awl Marks";

// --- The per-world FOLD-MARK glyph (a THIRD thing this same register drives) --
//
// The fold chevron (`render::layers::fold_chevron`) draws a real font glyph —
// `›`/`☞`/`▸` depending on flavour — rotated a quarter turn between its
// collapsed and expanded states through the `rotated_label` mask capability.
// WHICH glyph a world draws is keyed off the SAME [`OrnamentRegister`] its
// section-break fleuron already reads (`Theme::ornament_face`), never a second
// per-world list: a world's own `ornament_face` classifies it once, and both
// the fleuron AND the fold mark read that one classification. A new world
// inherits a fold mark the moment it sets `ornament_face` — there is no
// second field to remember.

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
/// PANICS on an unrecognised face rather than defaulting: `ornament_face` only
/// ever holds one of the three registered constants (every world literal in
/// `theme::worlds` sets it to one of them), so reaching the `else` arm means a
/// FOURTH ornament face was registered without a matching register variant —
/// exactly the silent-fallthrough this round's brief forbids, so it fails
/// loud instead of guessing.
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

/// The fold mark this register draws — user-picked from the item-475 glyph
/// survey (`captures/item-475-glyph-survey/README.md`), each already covered
/// by an already-bundled OFL face (`assets/fonts/LICENSES.md`; no new font
/// bytes). An EXHAUSTIVE match, no wildcard arm: a new [`OrnamentRegister`]
/// variant fails to compile here until it is given a mark.
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
            size_frac: 0.65,
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
// A thematic-break line (`---`/`***`/`___`) grows its whole ROW by a scale factor
// so the centered ornament reads as a generous flourish (the size counterpart of
// the leading-`#` heading scan). That scale is now PER-WORLD ([`crate::theme::Theme::ornament_scale`]),
// keyed to the ornament's CHARACTER — the detailed flowers reward size, the clean
// geometric marks don't — in three tiers that line up with the three ornament faces:
//
//   * ORNATE   — the [`ORNAMENT_JUNICODE`] Caslon flowers (antique/expressive worlds).
//   * FLEURON  — the [`ORNAMENT_GARAMOND`] Renaissance fleurons (literary serifs).
//   * GEOMETRIC — the [`ORNAMENT_MARKS`] stars/florets/diamonds (modern/technical).
//
// The field is read by BOTH `render::spans::md_line_scale` (the break ROW height)
// and `render::layers::prepare_ornaments` (the glyph LINE-BOX), so the two never
// drift — the tall row always centers the glyph. These are TASTE DEFAULTS: one
// dial per tier, tuned from the gallery.

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
// The unordered-list bullet ([`crate::theme::Theme::bullets`], drawn over a concealed `-`/`*`/`+`
// the caret is off) is PER-WORLD DATA in the world's own [`crate::theme::Theme::ornament_face`] —
// the same face + `no-new-machinery` discipline as the section-break fleuron trio,
// scoped by flavour:
//
//   * The MODERN / TECHNICAL / geometric worlds (the [`ORNAMENT_MARKS`] worlds) keep
//     the plain [`BULLETS_PLAIN`] `•`/`◦`/`▪` at [`BULLET_SCALE_PLAIN`] (byte-identical
//     to before this round, level 1/2) — restraint IS their character; a bullet is not
//     the place to decorate them for symmetry.
//   * The ANTIQUE / LITERARY serifs (the [`ORNAMENT_JUNICODE`] + [`ORNAMENT_GARAMOND`]
//     worlds) draw a small hedera / fleuron — and Bombora the antique MANICULE ☞ —
//     at [`BULLET_SCALE_ORNAMENT`] (~half body), so the mark reads as quiet list
//     RHYTHM, never a loud flourish. Garamond ships the manicule (☞ U+261E) and the
//     three fleurons ❧ ❦ ☙; Junicode ships ❧ ❦ ☙ ⁑ (no plain `•`), so those worlds'
//     triples come from that pool — every pick verified present in its own face by
//     `render::tests::markdown::bullet_glyphs_resolve_in_each_worlds_assigned_face`.
//
// PER-LEVEL ROTATION: [`crate::theme::Theme::bullets`]
// widened from a two-glyph PAIR to a three-glyph TRIPLE — [`crate::theme::Theme::bullet_for_depth`]
// now cycles `.0`/`.1`/`.2` every THREE levels (was every two), composing the new
// LEVEL axis with the existing per-WORLD axis: `.0` (depth 0) and `.1` (depth 1)
// are UNCHANGED on every world (Bombora's manicule, Mopoke's rosette, and every
// geometric world's `•`/`◦` all still land exactly where they did), and `.2`
// (depth 2, cycling back to `.0` at depth 3) is the new third rung —
// for [`BULLETS_PLAIN`] the literal filled/hollow/small-square trio the round's brief
// asked for, for a Junicode/Garamond world the one hedera of its own three-fleuron
// family `{❧, ❦, ☙}` its pair didn't already use (Bombora's manicule stays exclusive to
// depth 0 — "a pointing hand on every bullet is loud" — so its `.2` picks from the
// fleuron pool alongside `.1`, never the hand again).

/// The plain geometric bullet triple — level-1 `•` (U+2022, filled) / level-2 `◦`
/// (U+25E6, hollow) / level-3 `▪` (U+25AA, small square), all three in the merged
/// [`ORNAMENT_MARKS`] face — the modern/technical worlds' bullets (levels 1/2
/// byte-identical to the former `•`/`◦` pair; level 3 is the new rung).
pub const BULLETS_PLAIN: (char, char, char) = ('•', '◦', '▪');

/// PLAIN bullet scale — the geometric `•`/`◦` worlds keep body size (1.0), so their
/// bullets render exactly as before this round.
pub const BULLET_SCALE_PLAIN: f32 = 1.0;

/// ORNAMENT bullet scale — a hedera / fleuron / manicule shaped at ~half body so it
/// reads as a quiet bullet-sized marker, not a section-break flourish. A TASTE
/// DEFAULT (one dial for every characterful world), tuned from the veto gallery.
///
/// **Two logged exceptions (theme-QA round, padding audit):** the shared tier
/// is a reasonable default, but it is a byproduct of two UNRELATED font
/// metrics — the concealed `"<marker> "` prefix's advance in the world's own
/// BODY font, and the ornament glyph's own ink width in its ORNAMENT face —
/// that happen to pair badly on two worlds: Bombora's antique manicule ☞
/// against EB Garamond's narrow punctuation advance (the glyph's ink reached,
/// and on some rows touched, the following text) and Mopoke's rosette against
/// iA Writer Quattro S's wide duospaced advance (too small to fill the gap).
/// Mopoke's retired with its face change; Garamond's became
/// [`BULLET_SCALE_GARAMOND`] below. Every other characterful world stays on
/// this shared tier, byte-identical. See `render::tests::markdown::
/// bullet_glyph_never_touches_the_following_text_in_any_world`.
pub const BULLET_SCALE_ORNAMENT: f32 = 0.55;

/// The EB-GARAMOND-BODY bullet scale. This was a one-world literal on Bombora
/// until a SECOND world gained EB Garamond as its BODY face and
/// the padding law failed on it the same way, for the same reason — the
/// shared tier is scaled against the concealed `"- "` prefix's advance in the
/// world's own body font, and EB Garamond's punctuation advance is narrow
/// enough that a half-body fleuron crowds out the text that follows. So it is
/// a FACE rule, not a taste exception on one world: a world whose body face is
/// EB Garamond carries this tier, and `theme::tests::every_world_has_a_bullet_pair`
/// derives the allowance from the face rather than a world-name list.
pub const BULLET_SCALE_GARAMOND: f32 = 0.35;

// --- The per-world LIST-ITEM INDENT scale (the other half of bullet-
// level readability) ---------------------------------------------------------
//
// A nested list item's visual indent was, before this round, ENTIRELY a
// byproduct of its literal typed leading spaces (`markdown::LIST_INDENT` = 2
// per level) rendered at plain body-font advance — never a deliberate render
// choice, just whatever a world's own space-glyph happened to measure.
// [`crate::theme::Theme::list_indent_scale`] makes the per-level STEP a real,
// PER-WORLD DATA dial: `render::spans::add_list_indent_span` widens a list
// line's leading-space RUN (bytes `0..indent`) by this factor before layout,
// so each nesting level reads with a touch more breathing room than the bare
// two typed spaces alone give it — pure advance, no visible glyph (spaces
// carry no ink), and depth 0 (`indent == 0`) is an empty range, so a top-level
// item is byte-identical on every world regardless of this dial.
//
// Two tiers, mirroring [`BULLET_SCALE_PLAIN`]/[`BULLET_SCALE_ORNAMENT`]'s own
// shape: the modern/technical/geometric worlds stay at the byte-identical
// [`LIST_INDENT_SCALE_PLAIN`] (restraint, no change), while the antique/
// literary-serif worlds (the same roster [`BULLET_SCALE_ORNAMENT`] already
// singles out) step up to [`LIST_INDENT_SCALE_WIDE`] — their more decorative
// register earns the roomier rail. SENSIBLE DEFAULTS, not final taste: the
// exact multipliers are judged in the pre-tag pass; the MECHANISM
// (a pure per-world dial, read fresh off the active theme, composing for free
// with every existing nesting depth) is what this round commits to.

/// PLAIN list-indent scale — the geometric/technical worlds' nested list items
/// render at exactly their literal typed indent (byte-identical to before this
/// round).
pub const LIST_INDENT_SCALE_PLAIN: f32 = 1.0;

/// WIDE list-indent scale — the antique/literary-serif worlds' nested list
/// items get a touch more breathing room per level: each leading-space run
/// renders 50% wider than its literal typed width, so a depth-1 item (2 typed
/// spaces) gains one extra space-width of rail, a depth-2 item (4 typed
/// spaces) gains two, and so on — linear in depth for free, since the typed
/// indent itself already is.
pub const LIST_INDENT_SCALE_WIDE: f32 = 1.5;
