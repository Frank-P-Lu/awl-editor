//! Kotlin — the shared definition walk ([`crate::syntax::scanner`]) under
//! Kotlin's constants. It emits only the four Alabaster roles and leaves
//! everything else (keywords, operators, identifiers, punctuation) default ink:
//!
//! - `Comment`    — `// line` and `/* block */` comments. Kotlin block comments
//!   NEST, so the whole nested run is one span.
//! - `Str`        — `"strings"` (with `\` escapes), `'c'` char literals, and
//!   raw/multiline triple-quoted `"""..."""` (no escapes). A `$name` / `${expr}`
//!   interpolation rides INSIDE the one `Str` span.
//! - `Constant`   — numeric literals (`0x`/`0b` radixes, `_` separators,
//!   `L`/`u`/`f` suffixes, floats) and `true` / `false` / `null`.
//! - `Definition` — the identifier right after a `fun` / `class` / `interface` /
//!   `object` / `typealias` / `val` / `var` introducer.

use super::scanner::{BlockComment, LangSpec, LineComment, Number, WordRule};

pub(super) const SPEC: LangSpec = LangSpec {
    line: LineComment::Slashes,
    block: BlockComment::Nested,
    string_at,
    number: Number::Shared(super::NumOpts {
        radix: b"xXbB",
        radix_extra: b"",
        dot_dot_stops: true,
    }),
    ident_start: super::is_ident_start,
    ident_continue: super::is_ident_continue,
    // `val`/`var` cover the let-binding case; `enum class X` is caught by `class`.
    def_kws: &[
        "fun",
        "class",
        "interface",
        "object",
        "typealias",
        "val",
        "var",
    ],
    const_words: &["true", "false", "null"],
    words: WordRule::Standard,
    def_survives: b"",
    receiver_kw: None,
};

/// A raw/multiline `"""` string, a normal `"` string, or a `'c'` char literal —
/// the latter two stop at a newline.
fn string_at(b: &[u8], i: usize) -> Option<usize> {
    let n = b.len();
    match b[i] {
        b'"' if i + 2 < n && b[i + 1] == b'"' && b[i + 2] == b'"' => Some(scan_triple(b, i)),
        b'"' => Some(super::scan_quoted(b, i, b'"', true)),
        b'\'' => Some(super::scan_quoted(b, i, b'\'', true)),
        _ => None,
    }
}

/// Scan a raw/multiline triple-quoted string from the opening `"""` (at `q`) to
/// just past the closing `"""` (or EOF). Raw strings have NO escapes.
fn scan_triple(b: &[u8], q: usize) -> usize {
    let n = b.len();
    let mut i = q + 3;
    while i < n {
        if b[i] == b'"' && i + 2 < n && b[i + 1] == b'"' && b[i + 2] == b'"' {
            return i + 3;
        }
        // A flush close at EOF (`..."""` with nothing after).
        if b[i] == b'"' && i + 3 == n && b[i + 1] == b'"' && b[i + 2] == b'"' {
            return n;
        }
        i += 1;
    }
    n
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
        let t = "val x = 1 // hi there\n";
        let s = spans(t);
        assert_eq!(at(t, &s, SynKind::Comment), vec!["// hi there"], "{s:?}");
    }

    #[test]
    fn block_comment_nested() {
        let t = "/* a /* b */ c */ x";
        let s = spans(t);
        // The whole nested block is ONE comment span (Kotlin nests).
        assert!(has(&s, 0, 17, SynKind::Comment), "{s:?}");
    }

    #[test]
    fn string_with_escaped_quote() {
        let t = r#"val s = "a\"b""#;
        let s = spans(t);
        assert_eq!(at(t, &s, SynKind::Str), vec![r#""a\"b""#], "{s:?}");
    }

    #[test]
    fn interpolation_is_one_string_span() {
        let t = "val s = \"hi $name and ${a.b}\"\n";
        let s = spans(t);
        assert_eq!(
            at(t, &s, SynKind::Str),
            vec!["\"hi $name and ${a.b}\""],
            "{s:?}"
        );
    }

    #[test]
    fn triple_quoted_multiline() {
        let t = "val s = \"\"\"line one\nline \"two\"\"\"\"\n";
        let s = spans(t);
        let ss = at(t, &s, SynKind::Str);
        assert!(ss.iter().any(|x| x.starts_with("\"\"\"line one")), "{ss:?}");
    }

    #[test]
    fn char_literal_and_escape() {
        let t = "val c = 'x'; val n = '\\n'";
        let s = spans(t);
        assert_eq!(at(t, &s, SynKind::Str), vec!["'x'", "'\\n'"], "{s:?}");
    }

    #[test]
    fn numbers_and_constants() {
        let t =
            "val a = 42; val b = 0xFF_u; val c = 3.14; val d = 100L; val ok = true; val z = null";
        let s = spans(t);
        let cs = at(t, &s, SynKind::Constant);
        for want in ["42", "0xFF_u", "3.14", "100L", "true", "null"] {
            assert!(cs.contains(&want), "missing {want}: {cs:?}");
        }
    }

    #[test]
    fn range_op_not_eaten_by_number() {
        let t = "for (i in 0..5) {}";
        let s = spans(t);
        let cs = at(t, &s, SynKind::Constant);
        assert!(
            cs.contains(&"0") && cs.contains(&"5"),
            "ranges split: {cs:?}"
        );
    }

    #[test]
    fn definitions_after_keywords() {
        let t = "fun frobnicate() {}\nclass Widget\ninterface Shape\nobject Single\ntypealias Alias = Int\nval count = 0\nvar total = 0";
        let s = spans(t);
        let ds = at(t, &s, SynKind::Definition);
        for want in [
            "frobnicate",
            "Widget",
            "Shape",
            "Single",
            "Alias",
            "count",
            "total",
        ] {
            assert!(ds.contains(&want), "missing def {want}: {ds:?}");
        }
    }

    #[test]
    fn keyword_itself_is_not_styled() {
        // `fun` keyword stays default ink; only the NAME is a Definition.
        let t = "fun main() {}";
        let s = spans(t);
        assert!(
            !has(&s, 0, 3, SynKind::Definition),
            "the `fun` keyword must stay plain: {s:?}"
        );
        assert!(
            has(&s, 4, 8, SynKind::Definition),
            "`main` is the definition: {s:?}"
        );
    }

    #[test]
    fn plain_code_has_no_spans() {
        // No comment / literal / def-keyword -> nothing highlighted (Alabaster).
        let t = "result = compute(a, b) + offset";
        assert!(spans(t).is_empty(), "{:?}", spans(t));
    }

    #[test]
    fn reference_snippet() {
        let t = "// sum\nfun add(a: Int, b: Int): Int {\n    val total = a + b // ok\n    return total\n}\nconst val MAX = 100\n";
        let s = spans(t);
        assert_eq!(
            at(t, &s, SynKind::Comment),
            vec!["// sum", "// ok"],
            "{s:?}"
        );
        let ds = at(t, &s, SynKind::Definition);
        assert!(ds.contains(&"add") && ds.contains(&"MAX"), "{ds:?}");
        assert!(at(t, &s, SynKind::Constant).contains(&"100"), "{s:?}");
    }
}
