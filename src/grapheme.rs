//! THE ONE OWNER OF THE CHARACTER STEP — extended grapheme cluster boundaries.
//!
//! A caret step is a step over what the reader sees as one character, which is
//! an extended grapheme cluster (UAX #29), not a Unicode scalar. `e` + U+0301
//! COMBINING ACUTE is two scalars and one visual `é`: the mark renders ATOP its
//! base, so the position between them does not exist on screen and no caret may
//! occupy it. Precomposed U+00E9 is one scalar and one cluster, and behaves
//! identically through these functions — the two spellings of `é` must feel the
//! same, which is the whole point.
//!
//! Indices in and out are CHAR indices, matching the rope's and
//! [`crate::textbox::TextBox`]'s own coordinate system: this is a boundary
//! rule, not a storage change. The document's bytes are never normalized —
//! awl is a plain-text editor, and NFC-ing a loaded file would rewrite it.
//!
//! Storage is abstracted behind a `char_at` closure exactly as the word-boundary
//! owners in [`crate::buffer`] are, so the rope-backed [`crate::buffer::Buffer`]
//! and the `String`-backed [`crate::textbox::TextBox`] share this rule by
//! construction rather than by convention — `textbox.rs`'s parity table proves
//! the two never diverge.
//!
//! This owns the CHARACTER step only. Word motion, word delete and vertical
//! motion keep their own rules (see `textbox.rs`'s "two word rules" doc).

use unicode_segmentation::UnicodeSegmentation;

/// How many chars of context to segment per step. Keeping this bounded is what
/// makes a caret step O(cluster) instead of O(document) — the rope may hold
/// megabytes and this runs on every arrow key. A cluster is short in practice
/// (the longest emoji ZWJ and tag-flag sequences run under a dozen chars;
/// Unicode's Stream-Safe format caps combining runs at 30), so the window is
/// ample — and neither direction TRUNCATES: forward widens until the cluster
/// ends inside the window, backward walks left to a break that no amount of
/// further context could undo.
const WINDOW: usize = 64;

/// The next cluster boundary at or after `cursor` — where one Right / C-f
/// lands, and the end of what one forward-delete removes. `len` is the char
/// length of the text; `char_at(i)` yields the char at `i` (`i < len`).
///
/// Steps at least one char whenever `cursor < len`, so a caret left mid-cluster
/// by some other path (a click, a clamp after an edit) still makes progress and
/// lands ON a boundary rather than sticking.
pub(crate) fn next_cluster_boundary(
    cursor: usize,
    len: usize,
    char_at: impl Fn(usize) -> char,
) -> usize {
    if cursor >= len {
        return len;
    }
    let mut span = WINDOW;
    loop {
        let end = (cursor + span).min(len);
        let window: String = (cursor..end).map(&char_at).collect();
        let step = window
            .graphemes(true)
            .next()
            .map(|g| g.chars().count())
            .unwrap_or(1)
            .max(1);
        // A cluster that ran to the window's edge may continue past it; widen
        // and re-segment rather than reporting the edge as a boundary.
        if cursor + step < end || end == len {
            return (cursor + step).min(len);
        }
        span *= 2;
    }
}

/// The previous cluster boundary before `cursor` — where one Left / C-b lands,
/// and the start of what one Backspace removes. `char_at(i)` yields the char at
/// `i` (`i < cursor`). Steps at least one char whenever `cursor > 0`.
pub(crate) fn prev_cluster_boundary(cursor: usize, char_at: impl Fn(usize) -> char) -> usize {
    if cursor == 0 {
        return 0;
    }
    // Segmenting BACKWARD needs left context, and how much is NOT bounded by
    // the answer's own length: regional indicators pair off from their run's
    // true start, and an indic linker reaches back to the consonant before it.
    // So the window's left edge slides further left until it sits on a break no
    // amount of additional context could undo, and the segmentation from there
    // is exact. In ordinary text that first candidate already qualifies.
    let mut start = cursor.saturating_sub(WINDOW);
    while start > 0 && !break_is_context_free(char_at(start - 1), char_at(start)) {
        start -= 1;
    }
    let window: String = (start..cursor).map(&char_at).collect();
    let step = window
        .graphemes(true)
        .next_back()
        .map(|g| g.chars().count())
        .unwrap_or(1)
        .max(1)
        .min(cursor - start);
    cursor - step
}

/// Is the break between `a` and `b` guaranteed, whatever precedes `a`? Two
/// probes answer it without a private copy of Unicode's property tables:
///
/// 1. `a` must not attach to a preceding char (Extend / SpacingMark / ZWJ) —
///    those are exactly the classes whose rules can reach back PAST `a`
///    (GB9c's indic linker, GB11's emoji ZWJ sequence).
/// 2. `a` and `b` must break when segmented on their own — which rules out
///    `a` being Prepend, a CR before an LF, and a regional indicator that
///    would pair with `b`.
///
/// Conservative by construction: a false answer only costs another step left.
fn break_is_context_free(a: char, b: char) -> bool {
    let mut probe = String::with_capacity(12);
    probe.push('x');
    probe.push(a);
    if probe.graphemes(true).count() != 2 {
        return false;
    }
    probe.clear();
    probe.push(a);
    probe.push(b);
    probe.graphemes(true).count() == 2
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Walk `text` end to end with [`next_cluster_boundary`], then back with
    /// [`prev_cluster_boundary`], and return the two boundary sequences.
    fn walk(text: &str) -> (Vec<usize>, Vec<usize>) {
        let chars: Vec<char> = text.chars().collect();
        let len = chars.len();
        let mut forward = vec![0];
        let mut i = 0;
        while i < len {
            let next = next_cluster_boundary(i, len, |k| chars[k]);
            assert!(next > i, "next_cluster_boundary stalled at {i} in {text:?}");
            i = next;
            forward.push(i);
        }
        let mut backward = vec![len];
        let mut j = len;
        while j > 0 {
            let prev = prev_cluster_boundary(j, |k| chars[k]);
            assert!(prev < j, "prev_cluster_boundary stalled at {j} in {text:?}");
            j = prev;
            backward.push(j);
        }
        backward.reverse();
        (forward, backward)
    }

    /// THE CLUSTER LAW: stepping forward and stepping backward visit the SAME
    /// boundaries, and those boundaries are exactly the ones UAX #29 names —
    /// swept over the classes a scalar step gets wrong (decomposed marks,
    /// stacked marks, emoji ZWJ, regional-indicator flags, variation
    /// selectors, skin-tone modifiers, Hangul jamo) alongside the classes it
    /// gets right (ASCII, CJK, precomposed forms, lone BMP emoji), which must
    /// stay byte-identical in behavior.
    #[test]
    fn cluster_boundaries_match_uax29_in_both_directions() {
        let corpus = [
            ("ascii", "hello"),
            ("cjk", "日本語"),
            ("decomposed", "e\u{0301}X"),
            ("precomposed", "\u{00e9}X"),
            ("stacked marks", "a\u{0301}\u{0308}\u{0327}b"),
            ("emoji zwj family", "a👨‍👩‍👧‍👦b"),
            ("flag", "a🇯🇵b"),
            ("two flags", "🇯🇵🇺🇸"),
            ("odd flag run", "🇦🇧🇨"),
            ("skin tone", "a👍🏽b"),
            ("variation selector", "a❤️b"),
            ("hangul jamo", "\u{1100}\u{1161}\u{11a8}z"),
            ("keycap", "a1\u{fe0f}\u{20e3}b"),
            ("indic conjunct", "a\u{0915}\u{094d}\u{0915}b"),
            ("newline", "a\nb"),
            ("empty", ""),
        ];
        for (label, text) in corpus {
            let expected: Vec<usize> = std::iter::once(0)
                .chain(text.graphemes(true).scan(0usize, |acc, g| {
                    *acc += g.chars().count();
                    Some(*acc)
                }))
                .collect();
            let (forward, backward) = walk(text);
            assert_eq!(forward, expected, "{label}: forward boundaries");
            assert_eq!(backward, expected, "{label}: backward boundaries");
        }
    }

    /// The decomposed pair is ONE step in each direction — the defect this
    /// owner exists to fix, stated as an arithmetic assertion rather than a
    /// walk, so it names its own bug.
    #[test]
    fn decomposed_pair_is_one_step() {
        let chars: Vec<char> = "e\u{0301}X".chars().collect();
        assert_eq!(next_cluster_boundary(0, chars.len(), |i| chars[i]), 2);
        assert_eq!(prev_cluster_boundary(2, |i| chars[i]), 0);
    }

    /// A caret sitting INSIDE a cluster (a clamp or a click could put it there)
    /// makes progress and lands on a real boundary in both directions.
    #[test]
    fn a_caret_inside_a_cluster_escapes_to_a_boundary() {
        let chars: Vec<char> = "e\u{0301}X".chars().collect();
        assert_eq!(next_cluster_boundary(1, chars.len(), |i| chars[i]), 2);
        assert_eq!(prev_cluster_boundary(1, |i| chars[i]), 0);
    }

    /// The window WIDENS: a cluster longer than [`WINDOW`] is still crossed
    /// whole, and a regional-indicator run longer than the window still pairs
    /// from its true start.
    #[test]
    fn clusters_longer_than_the_window_still_step_whole() {
        let mut long = String::from("a");
        long.extend(std::iter::repeat_n('\u{0301}', WINDOW * 3));
        long.push('b');
        let chars: Vec<char> = long.chars().collect();
        let len = chars.len();
        assert_eq!(
            next_cluster_boundary(0, len, |i| chars[i]),
            WINDOW * 3 + 1,
            "one step crosses the whole stack"
        );
        assert_eq!(
            prev_cluster_boundary(WINDOW * 3 + 1, |i| chars[i]),
            0,
            "one step back crosses the whole stack"
        );

        // An ODD-length regional-indicator run: the final flag pairs only if
        // the run is counted from its true start, so a truncated window would
        // report the wrong boundary here.
        let flags: String = std::iter::repeat_n('\u{1f1ef}', WINDOW * 2 + 1).collect();
        let chars: Vec<char> = flags.chars().collect();
        let len = chars.len();
        assert_eq!(
            prev_cluster_boundary(len, |i| chars[i]),
            len - 1,
            "the odd trailing regional indicator stands alone"
        );
    }
}
