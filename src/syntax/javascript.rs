//! JavaScript — the shared definition walk ([`crate::syntax::scanner`]) under
//! JS's constants. Everything outside the four Alabaster roles (keywords,
//! operators, identifiers, punctuation) stays the default ink:
//!
//! - `Comment`    — `// line` and `/* block */` comments (JS does NOT nest block
//!   comments — the first `*/` closes).
//! - `Str`        — `"..."` / `'...'` strings and `` `...` `` template literals
//!   (a template — interpolations and all — is ONE `Str` span).
//! - `Constant`   — numeric literals (incl. `0x`/`0o`/`0b`, floats, exponents,
//!   `_` separators, BigInt `n` suffix) and `true` / `false` / `null` /
//!   `undefined` / `NaN` / `Infinity`.
//! - `Definition` — the identifier right after a `function` / `class` / `const` /
//!   `let` / `var` introducer.
//!
//! JS's regex-literal-vs-division ambiguity is deliberately sidestepped: a lone
//! `/` is left as a plain operator (regex literals are not one of the four roles),
//! so no division is ever mis-read.

use super::scanner::{BlockComment, LangSpec, LineComment, Number, WordRule};

pub(super) const SPEC: LangSpec = LangSpec {
    line: LineComment::Slashes,
    block: BlockComment::Flat,
    string_at,
    number: Number::Shared(super::NumOpts {
        radix: b"xXoObB",
        radix_extra: b"",
        dot_dot_stops: true,
    }),
    ident_start: super::is_ident_start_dollar,
    ident_continue: super::is_ident_continue_dollar,
    def_kws: &["function", "class", "const", "let", "var"],
    const_words: &["true", "false", "null", "undefined", "NaN", "Infinity"],
    words: WordRule::Standard,
    // A generator's `*` sits between `function` and the name.
    def_survives: b"*",
    receiver_kw: None,
};

/// A `` ` `` template (spans newlines, `${…}` rides inside the one span) or a
/// `"`/`'` string (which does not cross a raw newline).
fn string_at(b: &[u8], i: usize) -> Option<usize> {
    match b[i] {
        b'`' => Some(super::scan_quoted(b, i, b'`', false)),
        b'"' | b'\'' => Some(super::scan_quoted(b, i, b[i], true)),
        _ => None,
    }
}

#[cfg(test)]
fn spans(text: &str) -> Vec<(std::ops::Range<usize>, super::SynKind)> {
    super::scanner::scan(&SPEC, text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::SynKind;
    use crate::syntax::testutil::{at, has};

    #[test]
    fn line_comment() {
        let t = "let x = 1; // hi there\n";
        let s = spans(t);
        assert_eq!(at(t, &s, SynKind::Comment), vec!["// hi there"], "{s:?}");
    }

    #[test]
    fn block_comment_not_nested() {
        // JS block comments do NOT nest: the first `*/` closes.
        let t = "/* a /* b */ c */ x";
        let s = spans(t);
        assert!(has(&s, 0, 12, SynKind::Comment), "{s:?}");
    }

    #[test]
    fn string_with_escaped_quote() {
        let t = r#"let s = "a\"b";"#;
        let s = spans(t);
        assert_eq!(at(t, &s, SynKind::Str), vec![r#""a\"b""#], "{s:?}");
    }

    #[test]
    fn single_quoted_string() {
        let t = "let s = 'hi';";
        let s = spans(t);
        assert_eq!(at(t, &s, SynKind::Str), vec!["'hi'"], "{s:?}");
    }

    #[test]
    fn template_literal_with_interpolation() {
        let t = "let s = `hi ${name} and\nmore`;";
        let s = spans(t);
        // The whole template (interpolation + newline) is ONE Str span.
        assert_eq!(
            at(t, &s, SynKind::Str),
            vec!["`hi ${name} and\nmore`"],
            "{s:?}"
        );
    }

    #[test]
    fn numbers_and_constants() {
        let t =
            "let a = 42; let b = 0xFF; let c = 3.14; let d = 1_000n; let ok = true; let z = null;";
        let s = spans(t);
        let cs = at(t, &s, SynKind::Constant);
        for want in ["42", "0xFF", "3.14", "1_000n", "true", "null"] {
            assert!(cs.contains(&want), "missing {want}: {cs:?}");
        }
    }

    #[test]
    fn undefined_and_nan_are_constants() {
        let t = "let a = undefined; let b = NaN; let c = Infinity;";
        let s = spans(t);
        let cs = at(t, &s, SynKind::Constant);
        for want in ["undefined", "NaN", "Infinity"] {
            assert!(cs.contains(&want), "missing {want}: {cs:?}");
        }
    }

    #[test]
    fn definition_after_function_class_and_binding() {
        let t = "function frobnicate(x) {}\nclass Widget {}\nconst MAX = 100;\nlet count = 0;";
        let s = spans(t);
        let ds = at(t, &s, SynKind::Definition);
        assert!(ds.contains(&"frobnicate"), "{ds:?}");
        assert!(ds.contains(&"Widget"), "{ds:?}");
        assert!(ds.contains(&"MAX"), "{ds:?}");
        assert!(ds.contains(&"count"), "{ds:?}");
    }

    #[test]
    fn generator_name_after_function_star() {
        // `function* gen` — the `*` must not clear the definition expectation.
        let t = "function* gen() {}";
        let s = spans(t);
        assert!(at(t, &s, SynKind::Definition).contains(&"gen"), "{s:?}");
    }

    #[test]
    fn keyword_itself_is_not_styled() {
        // `function` keyword stays default ink; only the NAME is a Definition.
        let t = "function main() {}";
        let s = spans(t);
        assert!(
            !has(&s, 0, 8, SynKind::Definition),
            "the `function` keyword must stay plain: {s:?}"
        );
        assert!(
            has(&s, 9, 13, SynKind::Definition),
            "`main` is the definition: {s:?}"
        );
    }

    #[test]
    fn division_is_not_a_comment_or_string() {
        // A lone `/` is plain — not a comment, not a regex Str.
        let t = "return a / b;";
        assert!(spans(t).is_empty(), "{:?}", spans(t));
    }

    #[test]
    fn plain_code_has_no_spans() {
        let t = "return compute(a, b) + offset;";
        assert!(spans(t).is_empty(), "{:?}", spans(t));
    }

    #[test]
    fn reference_snippet() {
        // A compact end-to-end snippet asserting all four roles at once.
        let t = "// sum\nfunction add(a, b) {\n    const total = a + b; // ok\n    return total;\n}\nconst MAX = 100;\n";
        let s = spans(t);
        assert_eq!(
            at(t, &s, SynKind::Comment),
            vec!["// sum", "// ok"],
            "{s:?}"
        );
        let ds = at(t, &s, SynKind::Definition);
        assert!(
            ds.contains(&"add") && ds.contains(&"MAX") && ds.contains(&"total"),
            "{ds:?}"
        );
        assert!(at(t, &s, SynKind::Constant).contains(&"100"), "{s:?}");
    }
}
