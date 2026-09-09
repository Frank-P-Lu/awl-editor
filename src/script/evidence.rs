//! DOCUMENT-scoped Han-ambiguity EVIDENCE — the widened [`super::dominant_cjk`]
//! contract: for a document whose CJK is ambiguous Han (no kana, no hangul —
//! the case [`super::dominant_cjk`] can only call `Han`), scan the text for a
//! STRONGER signal than "Han is used by four languages" before falling back
//! to the `cjk_priority` setting. In priority order (an earlier rule wins over
//! a later one when a document somehow carries both signals):
//!
//!  1. any kana anywhere -> `Ja` (Japanese prose always mixes kana with kanji,
//!     so kana is decisive for the WHOLE document, not just its own runs);
//!  2. any character encodable in the Simplified-Chinese national charset
//!     (GB 2312) but in neither the Japanese one (JIS X 0208) nor the
//!     Traditional one (Big5) -> `ZhHans`;
//!  3. any character encodable in Big5 but in neither GB 2312 nor JIS X 0208
//!     -> `ZhHant` (the mirror of 2);
//!  4. any hangul -> `Ko`;
//!  5. none of the above (every Han character present is shared across
//!     scripts, or there is no CJK at all) -> `None`, meaning "consult the
//!     `cjk_priority` setting" ([`super::effective_cjk_priority`]).
//!
//! DOCUMENT-scoped and DECISIVE ON PRESENCE, not majority: one stray kana
//! character (or one simplified-only/traditional-only/hangul character)
//! settles the WHOLE document, on the same reasoning `dominant_cjk` already
//! uses for kana/hangul over a merely-present Han run — a genuinely mixed
//! Japanese-and-Chinese document is not resolved by this tier (rule 1 makes
//! it read as Japanese as a whole); that per-paragraph case is a deliberately
//! separate, un-scoped follow-up, not a defect in this rule set.
//!
//! Steps 2/3's character tables are GENERATED — never hand-curated, never
//! derived from which font happens to be bundled — by
//! `scripts/regenerate-han-evidence.py`, which uses the THREE LEGACY NATIONAL
//! CHARSET CODECS Python's standard library ships with the interpreter itself
//! (`gb2312`, `big5`, `shift_jis`): zero network, deterministic, reproducible
//! with nothing but a stock Python 3. See that script's own doc comment for
//! why this stands in for a live Unicode Unihan fetch (not vendored in this
//! repository, and this project never phones home to get it).

use super::Script;
use crate::frontmatter::Lang;

/// The bitset span: `Script::Han`'s own three ranges (CJK Unified Ideographs,
/// Extension A, and the Compatibility Ideographs) all fall within
/// `0x3400..=0xFAFF`, so one contiguous bitset over that span covers all of
/// them (the two small gaps inside it are never Han and read as unset bits,
/// never consulted since only a `Script::Han`-classified `char` reaches
/// [`is_simplified_only`]/[`is_traditional_only`]).
const SPAN_LO: u32 = 0x3400;
const SPAN_HI: u32 = 0xFAFF;

static SIMPLIFIED_ONLY: &[u8] = include_bytes!("../../assets/i18n/han-simplified-only.bin");
static TRADITIONAL_ONLY: &[u8] = include_bytes!("../../assets/i18n/han-traditional-only.bin");

fn bit_set(table: &[u8], c: char) -> bool {
    let cp = c as u32;
    if !(SPAN_LO..=SPAN_HI).contains(&cp) {
        return false;
    }
    let offset = (cp - SPAN_LO) as usize;
    table
        .get(offset / 8)
        .is_some_and(|byte| byte & (1 << (offset % 8)) != 0)
}

/// GB 2312, and in neither JIS X 0208 nor Big5 — see the module doc's rule 2.
pub fn is_simplified_only(c: char) -> bool {
    bit_set(SIMPLIFIED_ONLY, c)
}

/// Big5, and in neither GB 2312 nor JIS X 0208 — the mirror of
/// [`is_simplified_only`] (module doc rule 3).
pub fn is_traditional_only(c: char) -> bool {
    bit_set(TRADITIONAL_ONLY, c)
}

/// The document-scoped Han-ambiguity evidence scan (module doc's 5 rules).
/// `None` means "no decisive evidence" — either the document has no CJK at
/// all, or every CJK character present is ambiguous/shared; either way the
/// caller falls back to the `cjk_priority` setting
/// ([`super::effective_cjk_priority`]). A single pass: kana short-circuits
/// immediately (rule 1 is strictly highest priority), everything else is
/// gathered once and resolved in priority order after the scan.
pub fn cjk_evidence(text: &str) -> Option<Lang> {
    let mut has_simplified_only = false;
    let mut has_traditional_only = false;
    let mut has_hangul = false;
    for c in text.chars() {
        match super::classify_char(c) {
            Some(Script::Kana) => return Some(Lang::Ja),
            Some(Script::Han) => {
                if is_simplified_only(c) {
                    has_simplified_only = true;
                } else if is_traditional_only(c) {
                    has_traditional_only = true;
                }
            }
            Some(Script::Hangul) => has_hangul = true,
            Some(Script::Bopomofo) | None => {}
        }
    }
    resolve_evidence(false, has_simplified_only, has_traditional_only, has_hangul)
}

/// The module's priority resolution (rules 1-4) over four PRESENCE booleans,
/// shared by [`cjk_evidence`]'s whole-document scan (which always passes
/// `false` for `has_kana` here — a `true` kana presence already returned
/// early in its own loop) and a retained per-line aggregate's "does at least
/// one CURRENT line carry this signal" question, so the two can never drift
/// on which signal wins when a document carries more than one.
fn resolve_evidence(
    has_kana: bool,
    has_simplified_only: bool,
    has_traditional_only: bool,
    has_hangul: bool,
) -> Option<Lang> {
    if has_kana {
        Some(Lang::Ja)
    } else if has_simplified_only {
        Some(Lang::ZhHans)
    } else if has_traditional_only {
        Some(Lang::ZhHant)
    } else if has_hangul {
        Some(Lang::Ko)
    } else {
        None
    }
}

/// One line's contribution to the document-scoped evidence scan (the module
/// doc's four decisive signals). Unlike [`cjk_evidence`], this does NOT
/// short-circuit on a kana hit: a retained per-line projection needs every
/// flag a line carries so it can correctly RETRACT that line's contribution
/// when its text changes — a line that loses its only kana character must
/// stop counting toward a "document has kana" tally, not just stop being
/// rescanned.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct LineEvidence {
    pub(crate) kana: bool,
    pub(crate) simplified_only: bool,
    pub(crate) traditional_only: bool,
    pub(crate) hangul: bool,
}

/// Scan exactly one line's text for [`LineEvidence`] — the per-line unit a
/// retained projection re-tokenizes for only the lines a reshape's own
/// changed band touches, instead of [`cjk_evidence`]'s whole-document scan
/// repeated on every edit.
pub(crate) fn line_evidence(text: &str) -> LineEvidence {
    let mut ev = LineEvidence::default();
    for c in text.chars() {
        match super::classify_char(c) {
            Some(Script::Kana) => ev.kana = true,
            Some(Script::Han) => {
                if is_simplified_only(c) {
                    ev.simplified_only = true;
                } else if is_traditional_only(c) {
                    ev.traditional_only = true;
                }
            }
            Some(Script::Hangul) => ev.hangul = true,
            Some(Script::Bopomofo) | None => {}
        }
    }
    ev
}

/// The retained mirror of [`cjk_evidence`]: four running counts of how many
/// CURRENTLY-retained lines carry each signal, resolved through the same
/// [`resolve_evidence`] priority order. A document-scope cache (not
/// per-line), because the resolution itself is document-scope — module doc's
/// rule 5 is answered once per document, not once per line.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct EvidenceCounts {
    pub(crate) kana_lines: u32,
    pub(crate) simplified_lines: u32,
    pub(crate) traditional_lines: u32,
    pub(crate) hangul_lines: u32,
}

impl EvidenceCounts {
    pub(crate) fn add(&mut self, ev: LineEvidence) {
        self.kana_lines += u32::from(ev.kana);
        self.simplified_lines += u32::from(ev.simplified_only);
        self.traditional_lines += u32::from(ev.traditional_only);
        self.hangul_lines += u32::from(ev.hangul);
    }

    pub(crate) fn remove(&mut self, ev: LineEvidence) {
        self.kana_lines -= u32::from(ev.kana);
        self.simplified_lines -= u32::from(ev.simplified_only);
        self.traditional_lines -= u32::from(ev.traditional_only);
        self.hangul_lines -= u32::from(ev.hangul);
    }

    /// The exact same answer a fresh [`cjk_evidence`] scan of the whole
    /// document would give, derived from presence counts instead of a scan.
    pub(crate) fn resolve(self) -> Option<Lang> {
        resolve_evidence(
            self.kana_lines > 0,
            self.simplified_lines > 0,
            self.traditional_lines > 0,
            self.hangul_lines > 0,
        )
    }
}

/// Fold the document's evidence ([`cjk_evidence`]'s result) into `cjk_priority`
/// for the render ladder: when evidence is decisive, promote it to the FRONT
/// (the only slot [`super::doc_lang_for`]'s Han branch ever reads) so it wins
/// regardless of what the user's ladder says; when evidence is `None`, the
/// ladder passes through unchanged and the setting decides — module doc's
/// rule 5, and the ONLY point at which the `cjk_priority` setting is
/// consulted for an ambiguous Han run. This is the caller-side seam
/// [`super::resolve_font_id`]/[`super::doc_lang_for`] never had to change
/// shape for: they still just read `cjk_priority.first()`.
pub fn effective_cjk_priority(evidence: Option<Lang>, cjk_priority: &[Lang]) -> Vec<Lang> {
    match evidence {
        Some(ev) => {
            let mut out = Vec::with_capacity(cjk_priority.len().max(1));
            out.push(ev);
            out.extend(cjk_priority.iter().copied().filter(|&l| l != ev));
            out
        }
        None => cjk_priority.to_vec(),
    }
}

#[cfg(test)]
mod tests;
