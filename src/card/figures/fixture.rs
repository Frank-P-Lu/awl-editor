//! THE fixture the drawn<>announced figure laws share: one document, one
//! caret, and the figures HAND-DERIVED from them — never by calling the code
//! those laws test. A sibling file of `mod.rs`, declared `pub(crate) mod
//! fixture;` there, so `crate::card::figures::fixture` names it exactly as
//! it would if the module were still inline.
//!
//! It lives beside the owner so the pure `fold` law, the GPU pipeline law and
//! the capture-level History-preview law cannot drift apart on what the
//! document is or what it should read. Every number below is written out
//! with its arithmetic, because an oracle that recomputes the figure through
//! `DocFigures::of` would agree with any bug the derivation has.

use super::CountUnit;

/// A markdown document with a frontmatter language and two sibling
/// sections. Nine logical lines (the trailing newline makes an empty ninth),
/// 77 characters.
pub const DOC: &str =
    "---\nlang: ja\n---\n# Alpha\nalpha one two\nalpha three four\n# Beta\nbeta five six\n";

/// The logical line of `# Alpha`, whose section is lines 4–5.
pub const FOLD_HEADING: usize = 3;

/// [`DOC`] with `# Alpha` collapsed: lines 4 and 5 are gone, so the caret's
/// line 7 sits at filtered line 5. 46 characters.
pub const FOLDED: &str = "---\nlang: ja\n---\n# Alpha\n# Beta\nbeta five six\n";

/// The caret, in `DOC`'s own line/column space — the start of `beta five
/// six`, 63 characters in: 4 + 9 + 4 + 8 + 14 + 17 + 7 for the seven lines
/// before it, each counted with its own newline.
pub const CARET: (usize, usize) = (7, 0);

/// The caret's line once `# Alpha` is folded: 7 minus the 2 hidden lines.
pub const FOLDED_CARET_LINE: usize = 5;

/// The DOCUMENT's readout. The frontmatter block is metadata, so the
/// manuscript is `# Alpha / alpha one two / alpha three four / # Beta /
/// beta five six` — 2 + 3 + 3 + 2 + 3 = 13 whitespace-separated tokens (the
/// `#`s are tokens; that is what awl's tokenizer counts), and 13 words at
/// 200 wpm rounds up to 1 minute.
pub const WORDS: &str = "13 words · 1 min";
/// [`WORDS`] as the `(words, reading_minutes, unit)` triple the sidecar
/// reports. The body is plain, space-separated Latin prose (`# Alpha` /
/// `alpha one two` / …) — zero ideographic characters — so it stays
/// `Words` even though the frontmatter above tags `lang: ja`: the unit is
/// decided by what the manuscript actually says, never by a declared tag
/// (see [`dominant_unit`]'s doc comment for why).
pub const WORDS_PAIR: (usize, usize, CountUnit) = (13, 1, CountUnit::Words);

/// The DOCUMENT's through-doc percent: 63 characters into 77 is 81.81%,
/// which rounds to 82.
pub const PERCENT: u32 = 82;

/// The VISIBLE-only readout, if the figures were derived from the folded
/// text: the manuscript falls to `# Alpha / # Beta / beta five six`, which
/// is 2 plus 2 plus 3 = 7 tokens. Asserted so the laws prove the two
/// readings really differ; a fixture where they agreed would go green over
/// the bug.
pub const FOLDED_WORDS: &str = "7 words · 1 min";

/// The VISIBLE-only percent: 32 characters into 46 is 69.56%, rounding to
/// 70.
pub const FOLDED_PERCENT: u32 = 70;

/// A History preview's diff transcript — what the renderer is asked to shape
/// while the picker previews an older version. It is not the user's
/// document: no frontmatter, and its own six tokens have nothing to do with
/// the manuscript's thirteen.
pub const TRANSCRIPT: &str = "# beta five six\n\n~~alpha one~~ ==alpha two==\n";
