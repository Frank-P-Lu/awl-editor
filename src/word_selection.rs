//! Pointer WORD selection, kept deliberately separate from word motion/delete.
//!
//! Code buffers retain the editor's identifier-shaped word class in
//! [`crate::buffer`]. Prose uses the platform's linguistic segmenter on macOS;
//! the portable fallback keeps the existing English class and only narrows an
//! unspaced CJK run to the extended grapheme under the pointer. The adapter
//! sets no language, consults no dictionary, and performs no network work.

/// The unspaced scripts for which the portable editor-style alphanumeric run is
/// a particularly bad selection unit. Hangul is deliberately absent: Korean
/// prose normally carries spaces, matching the same distinction used by the
/// document word-count owner.
fn is_unspaced_cjk(c: char) -> bool {
    matches!(
        crate::script::classify_char(c),
        Some(
            crate::script::Script::Kana
                | crate::script::Script::Bopomofo
                | crate::script::Script::Han
        )
    )
}

/// One extended grapheme when `idx` points into an unspaced CJK cluster;
/// otherwise `None`, so the caller can preserve awl's existing English and
/// punctuation rule. Indices are rope CHAR indices.
pub(crate) fn portable_cjk_grapheme_bounds(
    idx: usize,
    len: usize,
    char_at: impl Fn(usize) -> char,
) -> Option<(usize, usize)> {
    if len == 0 {
        return None;
    }
    let hit = if idx < len { idx } else { len - 1 };
    let start = crate::grapheme::snap_backward(hit, len, &char_at);
    let end = crate::grapheme::next_cluster_boundary(start, len, &char_at);
    (start..end)
        .any(|i| is_unspaced_cjk(char_at(i)))
        .then_some((start, end))
}

/// The macOS NaturalLanguage adapter. This module alone translates among
/// NSString UTF-16 offsets, the line-local Rust string, and document-wide rope
/// CHAR indices. Its answer is snapped outward through awl's UAX #29 owner
/// before it crosses back into the editor core.
#[cfg(target_os = "macos")]
mod macos {
    use objc2::rc::{Allocated, Retained};
    use objc2::runtime::AnyObject;
    use objc2::{class, msg_send};
    use objc2_foundation::{NSRange, NSString};

    #[link(name = "NaturalLanguage", kind = "framework")]
    unsafe extern "C" {}

    const NL_TOKEN_UNIT_WORD: isize = 0;
    const NS_NOT_FOUND: usize = usize::MAX;

    fn char_to_utf16(text: &str, char_idx: usize) -> usize {
        text.chars().take(char_idx).map(char::len_utf16).sum()
    }

    enum Round {
        Backward,
        Forward,
    }

    /// Convert an NSString offset to a Rust char index. NaturalLanguage emits
    /// scalar boundaries, but rounding makes the adapter total even if a future
    /// tokenizer reports an offset inside a surrogate pair.
    fn utf16_to_char(text: &str, offset: usize, round: Round) -> usize {
        let mut utf16 = 0usize;
        for (char_idx, c) in text.chars().enumerate() {
            if offset <= utf16 {
                return char_idx;
            }
            let next = utf16 + c.len_utf16();
            if offset < next {
                return match round {
                    Round::Backward => char_idx,
                    Round::Forward => char_idx + 1,
                };
            }
            utf16 = next;
        }
        text.chars().count()
    }

    fn token_utf16_range(text: &str, hit_char: usize) -> Option<NSRange> {
        let string = NSString::from_str(text);
        let hit_utf16 = char_to_utf16(text, hit_char.min(text.chars().count()));

        // SAFETY: NLTokenizer is present on every supported macOS (10.14+).
        // `initWithUnit:` is its designated initializer; `setString:` retains
        // the NSString for the lifetime of the query. The index is clamped to
        // the NSString's UTF-16 length, which the API explicitly accepts.
        let tokenizer: Retained<AnyObject> = unsafe {
            let allocated: Allocated<AnyObject> = msg_send![class!(NLTokenizer), alloc];
            msg_send![allocated, initWithUnit: NL_TOKEN_UNIT_WORD]
        };
        unsafe {
            let _: () = msg_send![&*tokenizer, setString: &*string];
        }
        let range: NSRange = unsafe { msg_send![&*tokenizer, tokenRangeAtIndex: hit_utf16] };
        let utf16_len = string.length();
        (range.location != NS_NOT_FOUND
            && range.length > 0
            && range.location <= utf16_len
            && range.length <= utf16_len - range.location)
            .then_some(range)
    }

    pub(crate) fn linguistic_word_bounds(
        line: &str,
        hit_in_line: usize,
        line_start: usize,
        document_len: usize,
        char_at: impl Fn(usize) -> char,
    ) -> Option<(usize, usize)> {
        let range = token_utf16_range(line, hit_in_line)?;
        let local_start = utf16_to_char(line, range.location, Round::Backward);
        let local_end = utf16_to_char(line, range.location + range.length, Round::Forward);
        let start = (line_start + local_start).min(document_len);
        let end = (line_start + local_end).min(document_len);
        Some((
            crate::grapheme::snap_backward(start, document_len, &char_at),
            crate::grapheme::snap_forward(end, document_len, &char_at),
        ))
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn utf16_conversion_rounds_surrogate_interiors_outward() {
            let text = "a💻b";
            assert_eq!(char_to_utf16(text, 2), 3);
            assert_eq!(utf16_to_char(text, 2, Round::Backward), 1);
            assert_eq!(utf16_to_char(text, 2, Round::Forward), 2);
        }

        #[test]
        fn tokenizer_pins_the_reported_japanese_compound() {
            let text = "大幅に構成が変わっており";
            assert_eq!(
                linguistic_word_bounds(text, 3, 0, text.chars().count(), |i| {
                    text.chars().nth(i).unwrap()
                }),
                Some((3, 5)),
                "NLTokenizer(.word) selects 構成"
            );
        }

        #[test]
        fn tokenizer_utf16_answer_returns_document_char_indices() {
            let text = "👩🏽‍💻 構成";
            assert_eq!(
                linguistic_word_bounds(text, 5, 0, text.chars().count(), |i| {
                    text.chars().nth(i).unwrap()
                }),
                Some((5, 7)),
                "a seven-unit emoji prefix must not shift the rope range"
            );
        }
    }
}

#[cfg(target_os = "macos")]
pub(crate) use macos::linguistic_word_bounds;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grapheme::boundaries_of;

    fn bounds(text: &str, idx: usize) -> Option<(usize, usize)> {
        portable_cjk_grapheme_bounds(idx, text.chars().count(), |i| text.chars().nth(i).unwrap())
    }

    #[test]
    fn portable_fallback_selects_one_unspaced_cjk_grapheme() {
        assert_eq!(bounds("前半。後半", 0), Some((0, 1)));
        assert_eq!(bounds("前半。後半", 3), Some((3, 4)));
        assert_eq!(
            bounds("한글", 0),
            None,
            "spaced-language Hangul is unchanged"
        );
    }

    #[test]
    fn portable_fallback_never_splits_a_cluster() {
        let text = "葛\u{e0100}文";
        assert_eq!(bounds(text, 0), Some((0, 2)));
        let all = boundaries_of(text);
        for idx in 0..=text.chars().count() {
            if let Some((start, end)) = bounds(text, idx) {
                assert!(all.contains(&start) && all.contains(&end));
            }
        }
    }

    #[test]
    fn portable_fallback_declines_english_punctuation_and_emoji() {
        for text in ["hello", "snake_case", "...", "👩🏽‍💻"] {
            assert_eq!(bounds(text, 0), None, "{text:?} keeps the existing rule");
        }
    }
}
