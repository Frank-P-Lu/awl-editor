//! Nearest-neighbour ownership for language-neutral CJK punctuation.

use super::{Script, classify_char};

/// Resolve the CJK script role at `byte` with the nearest textual context.
/// Strong script characters return their own class. A CJK-common punctuation
/// run first inherits the strong script immediately outside that run on the
/// left, then on the right; Latin/whitespace at an edge blocks inheritance on
/// that side. An isolated punctuation run remains [`Script::Common`] so the
/// caller can use document tag/evidence, or leave it in the base display face
/// when a Latin-only document supplies neither.
///
/// This is the one contextual owner shared by shaping and the caret. `byte`
/// must be a scalar boundary in `text`; a non-boundary or non-CJK byte returns
/// `None`.
pub fn contextual_script_at(text: &str, byte: usize) -> Option<Script> {
    let ch = text.get(byte..)?.chars().next()?;
    let classified = classify_char(ch)?;
    if classified != Script::Common {
        return Some(classified);
    }

    let left = text[..byte]
        .chars()
        .rev()
        .find_map(|candidate| match classify_char(candidate) {
            Some(Script::Common) => None,
            Some(script) => Some(Some(script)),
            None => Some(None),
        })
        .flatten();
    if left.is_some() {
        return left;
    }

    let after = byte + ch.len_utf8();
    let right = text[after..]
        .chars()
        .find_map(|candidate| match classify_char(candidate) {
            Some(Script::Common) => None,
            Some(script) => Some(Some(script)),
            None => Some(None),
        })
        .flatten();
    right.or(Some(Script::Common))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn common_punctuation_inherits_only_immediate_cjk_context() {
        let _guard = crate::testlock::serial();
        let cases = [
            ("の「", 3, Script::Kana),
            ("「の", 0, Script::Kana),
            ("这「", 3, Script::Han),
            ("「國", 0, Script::Han),
            ("한「", 3, Script::Hangul),
            ("「한", 0, Script::Hangul),
            ("漢「」", 3, Script::Han),
            ("「」漢", 0, Script::Han),
        ];
        for (text, byte, want) in cases {
            assert_eq!(contextual_script_at(text, byte), Some(want), "{text:?}");
        }
        assert_eq!(contextual_script_at("latin「only", 5), Some(Script::Common));
        assert_eq!(contextual_script_at("「", 0), Some(Script::Common));
    }
}
