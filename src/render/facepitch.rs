//! Face PITCH — is a bundled display family MONOSPACED? — MEASURED from the
//! face's own metrics, never recognised by name (queue item 97).
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
//! ITEM 105 widened the same per-face measurement pass with a second fact:
//! [`typical_letter_ratio`], each face's own typical-letter / ascent ratio (the
//! mean of its x-height and cap-height). The caret's
//! GLYPHLESS fallback (`caret::TextPipeline::caret_cell_vertical`'s line-cell
//! arm — a space, end-of-line, an empty line) needs a "how tall is a typical
//! letter here" quantity to size a SYNTHETIC ink box in the same baseline-
//! relative convention the real ink-box arm uses, and this is that quantity,
//! read off the same bytes `measure_pitch` already parses rather than a
//! hand-tuned constant.

use std::collections::BTreeMap;
use std::sync::{Arc, OnceLock};

use glyphon::cosmic_text::fontdb;
use glyphon::cosmic_text::skrifa::{
    FontRef, MetadataProvider,
    instance::{LocationRef, Size},
};

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

/// The TYPICAL-LETTER RATIO used when no bundled/measured face is known for a
/// family (a system fallback face, an `AWL_FONT` override, or a bundled face
/// whose own `OS/2`/`post` tables carry neither x-height nor cap-height). Not
/// load-bearing precision — this is a FALLBACK for the case the real
/// measurement below cannot answer, and the value only ever feeds
/// [`super::caret::TextPipeline::caret_cell_vertical`]'s GLYPHLESS synthetic
/// box (item 105), never a real glyph's own ink.
pub(crate) const DEFAULT_TYPICAL_LETTER_RATIO: f32 = 0.62;

/// The x-height/ascent ratio used by the caret's vertical insertion band when
/// a real glyph is shorter than an ordinary letter.  Unlike the typical-letter
/// ratio below, this is deliberately the bare x-height: it is the minimum
/// vertical presence of a character about to be typed, not an approximation
/// for a glyphless column beside an arbitrary neighbour.
pub(crate) const DEFAULT_X_HEIGHT_RATIO: f32 = 0.48;

/// MEASURE one font file's own TYPICAL-LETTER-TO-ASCENT ratio (item 105): how
/// tall a "generic" letter's ink sits relative to the font's own ascent, read
/// straight from the face's `OS/2`/`hhea` tables through the SAME skrifa
/// `metrics()` call every other per-face fact here uses.
///
/// Deliberately NOT the bare x-height. A glyphless anchor (space / end-of-line
/// / an empty line / a ligature) has no letter of its own, so ANY single fixed
/// reference is an approximation — but the two obvious single choices both
/// under-serve one of the two glyph classes the caret's own ink-box arm treats
/// as routinely different heights:
///   * x-height alone reproduces item 91's ORIGINAL bug in miniature at the
///     seam for an ASCENDER neighbour (`l`/`h`/`b`/`d`) — x-height sits well
///     below a real ascender's ink top, so the fallback would visibly SHRINK
///     leaving a tall letter for end-of-line;
///   * cap-height alone reintroduces the bug's ORIGINAL direction for an
///     ORDINARY x-height letter (`a`/`m`/`e`) — the literal case the user's
///     `aaa` fixture reports — hanging empty accent space above it again.
///     The MEAN of the two is the balance point: still strictly font-measured (no
///     hand-tuned per-world offset), and it halves the worst-case residual against
///     EITHER class rather than zeroing one at the other's expense. `Size::unscaled()`
///     keeps every quantity in font design units, so the ratio is a pure per-font
///     constant independent of the font size a row happens to be shaped at — the
///     caller multiplies it by that ROW's own (already size/zoom/DPI-scaled)
///     `max_ascent` to get a real pixel height.
///
/// Falls back to [`DEFAULT_TYPICAL_LETTER_RATIO`] when the file won't parse or
/// the face declares NEITHER metric (some symbol/geometric faces don't); a
/// face with only one of the two uses that one alone rather than discarding a
/// real measurement.
fn measure_typical_letter_ratio(bytes: &[u8]) -> f32 {
    let Ok(font) = FontRef::new(bytes) else {
        return DEFAULT_TYPICAL_LETTER_RATIO;
    };
    let m = font.metrics(Size::unscaled(), LocationRef::default());
    if m.ascent <= 0.0 {
        return DEFAULT_TYPICAL_LETTER_RATIO;
    }
    let xh = m.x_height.filter(|v| *v > 0.0);
    let ch = m.cap_height.filter(|v| *v > 0.0);
    let px = match (xh, ch) {
        (Some(xh), Some(ch)) => (xh + ch) * 0.5,
        (Some(v), None) | (None, Some(v)) => v,
        (None, None) => return DEFAULT_TYPICAL_LETTER_RATIO,
    };
    (px / m.ascent).clamp(0.2, 0.95)
}

fn measure_x_height_ratio(bytes: &[u8]) -> f32 {
    let Ok(font) = FontRef::new(bytes) else {
        return DEFAULT_X_HEIGHT_RATIO;
    };
    let m = font.metrics(Size::unscaled(), LocationRef::default());
    let Some(xh) = m.x_height.filter(|v| *v > 0.0) else {
        return DEFAULT_X_HEIGHT_RATIO;
    };
    if m.ascent <= 0.0 {
        return DEFAULT_X_HEIGHT_RATIO;
    }
    (xh / m.ascent).clamp(0.2, 0.95)
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
    /// This face's own typical-letter / ascent ratio ([`measure_typical_letter_ratio`]) —
    /// read alongside the pitch measurement so a face's bytes are parsed once
    /// for both facts, not twice.
    pub typical_letter_ratio: f32,
    /// This face's own x-height / ascent ratio.  The caret uses this only to
    /// keep punctuation's vertical body in the row's ordinary-letter band.
    pub x_height_ratio: f32,
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
                typical_letter_ratio: measure_typical_letter_ratio(bytes),
                x_height_ratio: measure_x_height_ratio(bytes),
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

/// THE RATIO the caret's proportional-fallback SYNTHETIC ink box (item 105)
/// rides: `family`'s own measured x-height/ascent, or
/// [`DEFAULT_TYPICAL_LETTER_RATIO`] for a family this roster does not know (a system
/// fallback face, an `AWL_FONT` override — never a bundled display face, exactly
/// the same unknown-family shape [`family_is_mono`] answers `false` to).
pub fn typical_letter_ratio(family: &str) -> f32 {
    roster()
        .get(family)
        .map(|f| f.typical_letter_ratio)
        .unwrap_or(DEFAULT_TYPICAL_LETTER_RATIO)
}

/// The measured x-height/ascent ratio for the row's actual face, or the
/// conservative fallback for an unknown override face.
pub fn x_height_ratio(family: &str) -> f32 {
    roster()
        .get(family)
        .map(|f| f.x_height_ratio)
        .unwrap_or(DEFAULT_X_HEIGHT_RATIO)
}
