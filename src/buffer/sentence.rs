//! THE ONE OWNER OF THE SENTENCE-MOTION BOUNDARY (M-e / M-a / M-k,
//! docs/config.md): where `Action::ForwardSentence`/`BackwardSentence`/
//! `DeleteSentenceForward`/`DeleteSentenceBackward` land.
//!
//! **The rule, and why it isn't hand-rolled.** A terminator heuristic (scan
//! for `. ! ?` followed by whitespace) breaks on exactly the case prose is
//! full of: "e.g. the second case" is not a sentence end at its period, and
//! neither a hand rule nor a short blocklist of abbreviations generalizes —
//! the SAME period needs to behave differently depending on what follows it.
//! [UAX #29 sentence
//! segmentation](https://unicode.org/reports/tr29/#Sentence_Boundaries) is
//! the Unicode-maintained answer: table-driven, already in the grapheme path
//! (`crate::grapheme`) via the `unicode-segmentation` crate, called here
//! through `split_sentence_bound_indices` rather than re-derived by hand. It
//! correctly keeps together an abbreviation followed by a LOWERCASE
//! continuation ("e.g. the second case" — SB8: don't break before a run that
//! resolves to `Lower` before any of `Upper`/`OLetter`/a terminator), an
//! adjacent-initials run like "U.S." (SB7: `Upper ATerm × Upper`, no
//! intervening whitespace, so "U" and "S" don't fragment into two false
//! one-letter sentences), a closing quote/paren riding along with its
//! terminator ("Sentence." she said. — SB9/SB9a: `Close*` after
//! `STerm`/`ATerm` doesn't itself break), and a numeric decimal point not
//! counting as a terminator at all (SB6).
//!
//! **The honest limit, measured rather than assumed.** Bare UAX #29 carries
//! no abbreviation DICTIONARY — SB7's adjacency rule only reaches
//! single-letter-initial runs like "U.S.", and SB8's lowercase-lookahead only
//! reaches an abbreviation followed by an ordinary lowercase word. Neither
//! covers a title abbreviation followed by a CAPITALIZED proper noun: "Dr.
//! Smith left the building." breaks right after "Dr. " — measured, not
//! guessed, and pinned as a known case in
//! `buffer/tests/sentence_boundary.rs`'s
//! `documented_gap_title_abbreviation_before_a_capitalized_name`. Nothing in
//! the default algorithm can distinguish that from a genuine sentence
//! boundary before a capitalized name without a locale-specific abbreviation
//! list, which `unicode-segmentation` doesn't carry and which this module
//! deliberately does not bolt on — that would be exactly the hand-rolled,
//! non-generalizing heuristic this rule was chosen to avoid, just moved one
//! layer down. UAX #29 still strictly dominates a naive terminator scan (it
//! wins on ellipsis, quotes/parens, decimals, CJK, and the lowercase-
//! continuation class of abbreviation) — it just doesn't win on every axis.
//!
//! **What "boundary" means here.** A UAX #29 sentence chunk bundles its
//! OWN trailing whitespace (the crate's own doctest: `"Mr. Fox jumped. "`
//! is one chunk, trailing space included) — so the boundary BETWEEN two
//! chunks is the first character of the FOLLOWING sentence, past the
//! separating space(s). Forward sentence motion therefore lands on the next
//! sentence's first character (mirroring word motion's own "past the
//! separator, onto the next token" landing); backward sentence motion lands
//! on the current sentence's first character, or — mirroring
//! [`super::word_backward_boundary`]'s "already at a boundary" case — the
//! PREVIOUS sentence's first character when the cursor already sits at one.
//!
//! **Windowed, not whole-buffer.** The segmenter takes a `&str`, and the
//! document may be megabytes; re-segmenting the whole buffer on every M-e
//! would be O(doc) per keystroke. Its internal state is a 4-slot,
//! COLLAPSING record of recent categories (`fwd::SentenceBreaksState` in the
//! crate) — a run of closing quotes or spaces collapses to one slot
//! regardless of length — so a bounded window of real context primes the
//! state machine correctly without reading the whole rope, the same
//! trade [`crate::grapheme::next_cluster_boundary`]/`prev_cluster_boundary`
//! already make for cluster breaks. One rule doesn't shrink to a fixed
//! window, though: SB8's abbreviation lookahead (`match_sb8` in the crate)
//! scans forward past a terminator for the first
//! upper/lower/separator/terminator character, skipping everything else
//! (digits, symbols) — unboundedly, by the crate's own admission ("TODO
//! cache this, it is currently quadratic"). A boundary found strictly
//! INSIDE the window is trusted immediately; one found only at the window's
//! own forced edge is untrusted (it may be an artifact of the truncation,
//! not a real break) and the window widens and re-segments — exactly
//! [`crate::grapheme::next_cluster_boundary`]'s widen-on-ambiguity shape.
//! For ordinary prose this converges on the first pass; a pathological run
//! with no real letter for thousands of characters pays for it, same as a
//! pathological combining-mark run already does for grapheme stepping.

use unicode_segmentation::UnicodeSegmentation;

/// Backward priming context for [`sentence_forward_boundary`]: chars behind
/// the cursor fed into the same window so the segmenter's state isn't
/// artificially reset to "start of text" right where we're asking about —
/// a fixed size is safe (never widened) because the state it seeds is the
/// 4-slot collapsing record described above, not the raw char count.
const SENT_BACK_CTX: usize = 64;

/// Forward priming context for [`sentence_backward_boundary`], the mirror of
/// [`SENT_BACK_CTX`]: characters AFTER the cursor, so a terminator sitting
/// just behind the cursor is classified with the same lookahead the crate
/// would give it reading the document straight through.
const SENT_FWD_CTX: usize = 64;

/// Starting window span for the side being searched; doubles on ambiguity
/// (see the module doc). Generous enough that ordinary prose — including a
/// multi-clause sentence or a soft-wrapped paragraph, which is exactly as
/// "far" from the cursor in this LOGICAL-char window as an unwrapped one —
/// never widens.
const SENT_WINDOW_INIT: usize = 256;

/// Byte length of the first `n` chars of `s` (`s.len()` if `s` has `<= n`
/// chars) — converts a window-relative CHAR offset (what callers pass) to
/// the byte offset `split_sentence_bound_indices` speaks.
fn char_prefix_bytes(s: &str, n: usize) -> usize {
    s.char_indices().nth(n).map(|(b, _)| b).unwrap_or(s.len())
}

/// The ONE owner of the SENTENCE-MOTION forward boundary (M-e): from char
/// index `cursor`, the start of the FOLLOWING sentence (see the module doc
/// for why that's past the terminator's own trailing whitespace, not just
/// after the terminator). `len` is the document's char length; `char_at(i)`
/// yields the char at `i` (`cursor <= i < len`, and up to [`SENT_BACK_CTX`]
/// chars before `cursor` for priming).
///
/// Steps at least one char whenever `cursor < len` (matching word/char
/// motion's "always make progress" contract) and saturates at `len` —
/// UAX #29's own SB2 rule ("break at end of text") makes end-of-buffer a
/// sentence boundary in its own right, so a caret already in the buffer's
/// last sentence lands on `len`, exactly like [`super::word_forward_boundary`]
/// at the last word.
pub(crate) fn sentence_forward_boundary(
    cursor: usize,
    len: usize,
    char_at: impl Fn(usize) -> char,
) -> usize {
    if cursor >= len {
        return len;
    }
    let start = cursor.saturating_sub(SENT_BACK_CTX);
    let mut span = SENT_WINDOW_INIT;
    loop {
        let end = (cursor + span).min(len);
        let window: String = (start..end).map(&char_at).collect();
        let cursor_byte = char_prefix_bytes(&window, cursor - start);
        let found = window
            .split_sentence_bound_indices()
            .map(|(b, _)| b)
            .find(|&b| b > cursor_byte);
        match found {
            // Strictly inside the window, or the window already reaches the
            // real document end: trustworthy. Otherwise `b == window.len()`
            // is SB2's forced end-of-text break on a TRUNCATED string, not a
            // real one — widen and re-segment with more forward context.
            Some(b) if b < window.len() || end == len => {
                let char_off = window[..b].chars().count();
                return (start + char_off).min(len);
            }
            _ if end == len => return len,
            _ => span *= 2,
        }
    }
}

/// The exact mirror of [`sentence_forward_boundary`] — the start of the
/// CURRENT sentence (or the PREVIOUS one, if `cursor` already sits at a
/// sentence start), for M-a and the start side of a sentence-spanning
/// backward kill. `char_at(i)` yields the char at `i` (`0 <= i < len`, and up
/// to [`SENT_FWD_CTX`] chars at/after `cursor` for lookahead priming).
///
/// Steps at least one char whenever `cursor > 0` and saturates at `0`.
pub(crate) fn sentence_backward_boundary(
    cursor: usize,
    len: usize,
    char_at: impl Fn(usize) -> char,
) -> usize {
    if cursor == 0 {
        return 0;
    }
    let fwd_end = (cursor + SENT_FWD_CTX).min(len);
    let mut span = SENT_WINDOW_INIT;
    loop {
        let start = cursor.saturating_sub(span);
        let window: String = (start..fwd_end).map(&char_at).collect();
        let cursor_byte = char_prefix_bytes(&window, cursor - start);
        let found = window
            .split_sentence_bound_indices()
            .map(|(b, _)| b)
            .filter(|&b| b < cursor_byte)
            .max();
        match found {
            // `b == 0` is only trustworthy when the window's own left edge
            // IS the document start — SB1 ("break at start of text") always
            // reports byte 0 as a boundary of whatever string it's handed,
            // real document start or not, so an untruncated window (`start
            // == 0`) confirms it while a truncated one (more real text sits
            // further left) doesn't.
            Some(b) if b > 0 || start == 0 => {
                let char_off = window[..b].chars().count();
                return start + char_off;
            }
            _ if start == 0 => return 0,
            _ => span *= 2,
        }
    }
}
