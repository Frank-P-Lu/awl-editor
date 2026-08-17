//! Face PITCH — is a bundled display family MONOSPACED? — MEASURED from the
//! face's own metrics, never recognised by name.
//!
//! WHAT WENT WRONG. `caret::font_is_mono` used to be a three-name string match
//! (`"IBM Plex Mono" | "JetBrains Mono" | "Monaspace Xenon"`). The predicate had
//! one owner, but its MEMBERSHIP was a literal list no sweep could check, so
//! every new mono face had to be remembered into it by hand. Iosevka — a
//! genuinely fixed-pitch face, and the display face of BOTH Currawong and
//! Cassowary — was never added, so those two worlds silently fell through to the
//! PROPORTIONAL caret arm: their block hugged each glyph's own ink box instead of
//! holding the uniform cell grid every other mono world keeps (measured before
//! the fix at zoom 1: Currawong's caret top sat at y18 on `l`, y23 on `o`/`g` —
//! a 5px wobble letter to letter — while Tawny/Mangrove/Potoroo/Firetail held a
//! fixed top on all three). The same list had already lost Monaspace Xenon and
//! JetBrains Mono once before, which is the tell that a hand-kept list is the
//! wrong mechanism rather than a one-off oversight.
//!
//! THE MECHANISM. Pitch is now DERIVED from the shipped font file:
//!
//! * [`measure_pitch`] asks the face for the advance width of every probe glyph
//!   in [`PITCH_PROBE`] through **skrifa**, the very font stack cosmic-text
//!   shapes and rasterises with (`cosmic_text::skrifa` — the same crate, the
//!   same tables). A face whose probe advances are all one number is
//!   [`Pitch::Mono`]; any spread makes it [`Pitch::Proportional`]. That is the
//!   product definition of monospace, read off the font itself, so a newly
//!   bundled mono face is classified correctly the moment it ships — nothing to
//!   remember.
//! * The family a face registers under comes from **fontdb**, through the exact
//!   call `render::build_font_system` registers with, so the key of this table
//!   is byte-for-byte the string a world's `Theme::font` / `Theme::mono` names
//!   (including the awkward real ones — `"Newsreader 16pt 16pt"`,
//!   `"Fraunces 9pt"`).
//!
//! WHY THE WEIGHT-300 TRAP CANNOT REACH IT (CLAUDE.md's tripwire; docs/fonts.md).
//! IBM Plex Mono ships as Light/300 and a default-400 REQUEST silently drops it
//! during cosmic-text's `weight_diff == 0` fallback filtering — which is a
//! property of *selecting* a face out of a populated DB. Nothing here selects:
//! each file is parsed on its own, and its own glyphs are measured. There is no
//! weight request to mismatch, so `mono_safe_weight`'s compensation is neither
//! needed nor bypassed.
//!
//! WHY MEMBERSHIP IS SWEEPABLE NOW. Derivation alone would be silent — a face
//! could change class with no one noticing — so the roster is also DECLARED at
//! the one place a face enters the build: [`crate::render::FONT_THEME_FACES`]
//! pairs every `include_bytes!` with its [`Pitch`], and the tuple type means a
//! new face **cannot compile** without one. `tests::facepitch` then pins
//! declaration against measurement family by family, and pins that every
//! `Theme::font` / `Theme::mono` in `theme::THEMES` is a member of that roster.
//! So: a new mono face is handled correctly by construction, a mis-declared one
//! fails the suite, and a world pointed at an unregistered family fails the
//! suite — the caret's look can no longer change by omission.
//!
//! The same per-face measurement pass carries a second fact:
//! [`typical_letter_ratio`], each face's own typical-letter / ascent ratio (the
//! mean of its x-height and cap-height). The caret's
//! GLYPHLESS fallback (`caret::TextPipeline::caret_cell_vertical`'s line-cell
//! arm — a space, end-of-line, an empty line) needs a "how tall is a typical
//! letter here" quantity to size a SYNTHETIC ink box in the same baseline-
//! relative convention the real ink-box arm uses, and this is that quantity,
//! read off the same bytes `measure_pitch` already parses rather than a
//! hand-tuned constant.
//!
//! And a third: [`vertical_em_metrics`], each face's own ascent/descent per em.
//! An EMPTY row carries no glyph metrics at all — cosmic-text hands it a
//! `LayoutLine` with `max_ascent`/`max_descent` of ZERO — so the caret's row
//! lookup has nothing to size or centre against there; these two fractions,
//! times that row's own font size, rebuild the pair the row would have carried
//! with one letter in it.
//!
//! And a fourth: [`ink_envelope_em`], each face's own real ascender-to-descender
//! ink extremes over a representative roster (`INK_ENVELOPE_PROBE`), read from
//! the face's own glyph OUTLINE bounds rather than its declared `hhea`/`OS2`
//! metrics. The literal Block caret's proportional-face vertical envelope
//! rides this, not `typical_letter_ratio` (tuned to the mean letter, not the
//! extremes an ascender/descender can reach) and not `vertical_em_metrics`
//! (a line-spacing figure generous enough, on some bundled faces, to exceed
//! the app's own row height on its own).

use std::collections::BTreeMap;
use std::sync::{Arc, OnceLock};

use glyphon::cosmic_text::fontdb;
use glyphon::cosmic_text::skrifa::{
    FontRef, MetadataProvider,
    instance::{LocationRef, Size},
};

mod measure;
pub(crate) use measure::{DEFAULT_ASCENT_EM, DEFAULT_DESCENT_EM, DEFAULT_TYPICAL_LETTER_RATIO};

/// Whether a face's glyphs all share one advance width.
///
/// DELIBERATELY two-valued, not three: "duospaced" faces (iA Writer Quattro S,
/// bundled and currently unassigned) sit near a grid but do not hold one, and
/// the caret's question — "may I draw a uniform cell here?" — has no third
/// answer. A quattro measures [`Pitch::Proportional`], which is the look it has
/// always had.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Pitch {
    /// Every probe glyph has the SAME advance: a real fixed grid.
    Mono,
    /// The advances differ: the caret must ride each glyph's own ink.
    Proportional,
}

impl Pitch {
    /// `true` for [`Pitch::Mono`] — the spelling every caller wants.
    pub fn is_mono(self) -> bool {
        matches!(self, Pitch::Mono)
    }
}

/// The glyphs whose advances decide the verdict: two narrow letters, two wide
/// ones, an x-height letter, an ascender, two descenders, punctuation and
/// digits. A proportional face separates `i`/`l` from `m`/`W` by a wide margin;
/// a duospaced face separates them by a small one; a real mono separates them by
/// nothing at all. All are plain ASCII, so every bundled Latin display face
/// (several are subset) is required to cover the whole set — a face that does
/// not is a [`None`] verdict here and a FAILING law over there, never a silent
/// demotion to proportional.
pub const PITCH_PROBE: &str = "iIlLmMwWxXgjy.,0189";

/// MEASURE one font file's pitch from its own advance widths.
///
/// `None` when the file will not parse or does not cover the whole
/// [`PITCH_PROBE`] — an "I cannot tell", never a guess. `Size::unscaled()` keeps
/// the advances in font units (integers), so the equality below is exact and
/// carries no float tolerance to argue about.
pub fn measure_pitch(bytes: &[u8]) -> Option<Pitch> {
    let font = FontRef::new(bytes).ok()?;
    let charmap = font.charmap();
    let metrics = font.glyph_metrics(Size::unscaled(), LocationRef::default());
    let mut first: Option<f32> = None;
    for ch in PITCH_PROBE.chars() {
        let gid = charmap.map(ch)?;
        let adv = metrics.advance_width(gid)?;
        if adv <= 0.0 {
            return None;
        }
        match first {
            None => first = Some(adv),
            Some(w) if w == adv => {}
            Some(_) => return Some(Pitch::Proportional),
        }
    }
    first.map(|_| Pitch::Mono)
}

/// A [`Pitch::Mono`] face's OWN cell pitch, as a fraction of its em square: the
/// advance width of any [`PITCH_PROBE`] glyph (they are all equal on a real
/// mono face — that is the definition [`measure_pitch`] checks) divided by the
/// face's own `units_per_em`. `None` for a face that does not measure
/// [`Pitch::Mono`] (there is no single cell width to report) or whose bytes
/// [`measure_pitch`] could not read.
///
/// This is the ONE place the caret's mono/proportional block-width law reads a
/// per-face pitch VALUE rather than the two-valued [`Pitch`] classification —
/// the oracle the law compares the live shaped advance against, derived
/// straight from the shipped `hmtx`/`head` tables rather than from anything
/// `render::caret` itself computes. `cfg(test)`-only: no production call site
/// reads a bare em-pitch value — the caret rides [`family_is_mono`], the
/// two-valued predicate — so this exists solely as the law's independent
/// oracle.
#[cfg(test)]
pub fn mono_pitch_em(bytes: &[u8]) -> Option<f32> {
    if measure_pitch(bytes) != Some(Pitch::Mono) {
        return None;
    }
    let font = FontRef::new(bytes).ok()?;
    let charmap = font.charmap();
    let glyph_metrics = font.glyph_metrics(Size::unscaled(), LocationRef::default());
    // Any probe glyph: `measure_pitch` already proved they share one advance.
    let gid = charmap.map(
        PITCH_PROBE
            .chars()
            .next()
            .expect("PITCH_PROBE is non-empty"),
    )?;
    let adv = glyph_metrics.advance_width(gid)?;
    let upem = font
        .metrics(Size::unscaled(), LocationRef::default())
        .units_per_em;
    if upem == 0 {
        return None;
    }
    Some(adv / upem as f32)
}

/// The family name a font file REGISTERS UNDER, resolved through the same
/// `fontdb` path `render::build_font_system` uses — so this is the string a
/// `Theme::font` must name, not an approximation of it. A private, single-face
/// `Database` (no system scan, no fallback) keeps it cheap and hermetic.
pub fn registered_family(bytes: &[u8]) -> Option<String> {
    let mut db = fontdb::Database::new();
    let ids = db.load_font_source(fontdb::Source::Binary(Arc::new(bytes.to_vec())));
    let id = *ids.first()?;
    db.face(id)?.families.first().map(|(name, _)| name.clone())
}

/// Every bundled face the per-script resolution ladder can select.
pub fn bundled_cjk_faces() -> impl Iterator<Item = &'static [u8]> {
    crate::render::FONT_CJK_FACES
        .iter()
        .chain(crate::render::FONT_JA_VARIETY_FACES)
        .chain(crate::render::FONT_ZH_KO_FACES)
        .chain(crate::render::FONT_CJK_COMPANION_FACES)
        .copied()
}

/// One bundled display face, as the roster knows it.
#[derive(Clone, Copy, Debug)]
pub struct FaceFacts {
    /// What the build DECLARES it to be (the tuple in `FONT_THEME_FACES`).
    /// Deliberately NOT read by the renderer — the caret rides `measured` — so
    /// its one consumer is the law that joins the two
    /// (`render::tests::facepitch`).
    #[allow(dead_code)]
    pub declared: Pitch,
    /// What its own advance widths SAY it is — `None` if it could not be read.
    pub measured: Option<Pitch>,
    /// This face's own typical-letter / ascent ratio ([`measure::typical_letter_ratio`]) —
    /// read alongside the pitch measurement so a face's bytes are parsed once
    /// for both facts, not twice.
    pub typical_letter_ratio: f32,
    /// This face's own ascent and descent as em fractions
    /// ([`measure::vertical_em_metrics`]) — the pair a row with NO GLYPHS has to
    /// reconstruct, since cosmic-text gives an empty row zeros for both.
    pub vertical_em: (f32, f32),
    /// This face's own real ink extremes over its representative roster
    /// ([`measure::ink_envelope_em`]) — the ASCENDER-to-DESCENDER envelope the
    /// proportional Block caret's own vertical policy rides, distinct from
    /// both `typical_letter_ratio` (the mean, not the extremes) and
    /// `vertical_em` (the generous line-spacing metric, not real ink).
    pub ink_envelope_em: (f32, f32),
}

/// THE ROSTER: every bundled display family → its declared pitch, measured
/// pitch, and measured x-height ratio.
///
/// Built once (a few TTF header parses; no system font enumeration) and then
/// read per frame by [`family_is_mono`] / [`typical_letter_ratio`]. Keyed by the
/// REGISTERED family name, so a lookup with a `Theme::font` string hits
/// directly.
pub fn roster() -> &'static BTreeMap<String, FaceFacts> {
    static ROSTER: OnceLock<BTreeMap<String, FaceFacts>> = OnceLock::new();
    ROSTER.get_or_init(|| {
        let mut out = BTreeMap::new();
        for (bytes, declared) in crate::render::bundled_display_faces() {
            let Some(family) = registered_family(bytes) else {
                continue;
            };
            out.entry(family).or_insert(FaceFacts {
                declared,
                measured: measure_pitch(bytes),
                typical_letter_ratio: measure::typical_letter_ratio(bytes),
                vertical_em: measure::vertical_em_metrics(bytes),
                ink_envelope_em: measure::ink_envelope_em(bytes),
            });
        }
        out
    })
}

/// THE PREDICATE the caret rides: is `family` a bundled face with a real fixed
/// grid?
///
/// Answers from the MEASUREMENT, not the declaration — so the day a mono face is
/// bundled it behaves like one, whatever anybody remembered to write down (the
/// declaration's job is to make the omission fail a test, not to drive the
/// pixels). An unknown family — a system fallback face, an `AWL_FONT` override —
/// is not a bundled display face and answers `false`, exactly as the old name
/// list did.
pub fn family_is_mono(family: &str) -> bool {
    roster()
        .get(family)
        .and_then(|f| f.measured)
        .map(Pitch::is_mono)
        .unwrap_or(false)
}

/// THE RATIO the proportional caret's ONE vertical box rides
/// (`TextPipeline::caret_synthetic_ink_box`): `family`'s own measured mean of
/// x-height and cap-height over its ascent, or
/// [`DEFAULT_TYPICAL_LETTER_RATIO`] for a family this roster does not know (a system
/// fallback face, an `AWL_FONT` override — never a bundled display face, exactly
/// the same unknown-family shape [`family_is_mono`] answers `false` to).
pub fn typical_letter_ratio(family: &str) -> f32 {
    roster()
        .get(family)
        .map(|f| f.typical_letter_ratio)
        .unwrap_or(DEFAULT_TYPICAL_LETTER_RATIO)
}

/// `family`'s own measured `(ascent, descent)` em fractions
/// ([`measure::vertical_em_metrics`]), or the
/// [`DEFAULT_ASCENT_EM`]/[`DEFAULT_DESCENT_EM`] pair for a family this roster
/// does not know — the same unknown-family shape [`family_is_mono`] answers
/// `false` to. Returned as a PAIR because its one consumer (the caret's
/// glyphless-row reconstruction) needs both to place a baseline, and reading
/// one without the other is how an ascent gets centred against a different
/// font's descent.
pub fn vertical_em_metrics(family: &str) -> (f32, f32) {
    roster()
        .get(family)
        .map(|f| f.vertical_em)
        .unwrap_or((DEFAULT_ASCENT_EM, DEFAULT_DESCENT_EM))
}

/// `family`'s own real ink extremes ([`measure::ink_envelope_em`]) — an
/// `(ascender_top, descender_bottom)` em-fraction pair, each strictly a real
/// probed glyph's outline and never `vertical_em_metrics`'s generous
/// line-spacing figure. The unknown-family fallback is the same shape every
/// other per-face fact here falls back to.
pub fn ink_envelope_em(family: &str) -> (f32, f32) {
    roster()
        .get(family)
        .map(|f| f.ink_envelope_em)
        .unwrap_or((measure::DEFAULT_INK_ASCENT_EM, measure::DEFAULT_INK_DESCENT_EM))
}

/// The resolved bundled CJK face's stable one-em ideographic cell, expressed
/// as `(ascent, descent)` fractions around the baseline. `None` is honest for a
/// system-only face whose bytes are not part of the product; the caller then
/// retains the existing row/Latin fallback rather than inventing its metrics.
pub fn ideographic_cell_em(family: &str) -> Option<(f32, f32)> {
    static CJK: OnceLock<BTreeMap<String, (f32, f32)>> = OnceLock::new();
    CJK.get_or_init(|| {
        let mut out = BTreeMap::new();
        for bytes in bundled_cjk_faces() {
            let Some(family) = registered_family(bytes) else {
                continue;
            };
            let Some(cell) = measure::ideographic_cell_em(bytes) else {
                continue;
            };
            out.entry(family).or_insert(cell);
        }
        out
    })
    .get(family)
    .copied()
}
