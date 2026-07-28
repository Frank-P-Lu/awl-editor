//! Java — the shared definition walk ([`crate::syntax::scanner`]) under Java's
//! constants. It emits only the four Alabaster roles and leaves everything else
//! (keywords, operators, identifiers, punctuation) as the default ink:
//!
//! - `Comment`    — `// line` and `/* block */` comments (Java blocks do NOT nest
//!   — the first `*/` closes).
//! - `Str`        — `"strings"`, `'c'` char literals, and `"""` text blocks
//!   (Java 13+ multi-line strings).
//! - `Constant`   — numeric literals (incl. `0x`/`0b`/octal, `_` separators,
//!   `L`/`f`/`d` suffixes, floats/exponents) and `true` / `false` / `null`.
//! - `Definition` — the identifier right after a `class` / `interface` / `enum` /
//!   `record` introducer.

use super::scanner::{BlockComment, LangSpec, LineComment, Number, WordRule};

pub(super) const SPEC: LangSpec = LangSpec {
    line: LineComment::Slashes,
    block: BlockComment::Flat,
    string_at,
    number: Number::Shared(super::NumOpts {
        radix: b"xXbB",
        radix_extra: b"",
        dot_dot_stops: false,
    }),
    ident_start: super::is_ident_start_dollar,
    ident_continue: super::is_ident_continue_dollar,
    def_kws: &["class", "interface", "enum", "record"],
    const_words: &["true", "false", "null"],
    words: WordRule::Standard,
    def_survives: b"",
    receiver_kw: None,
};

/// A `"""` text block, or a `"`/`'` literal (neither of which crosses a newline).
fn string_at(b: &[u8], i: usize) -> Option<usize> {
    let n = b.len();
    match b[i] {
        b'"' if i + 2 < n && b[i + 1] == b'"' && b[i + 2] == b'"' => Some(text_block(b, i)),
        b'"' => Some(super::scan_quoted(b, i, b'"', true)),
        b'\'' => Some(super::scan_quoted(b, i, b'\'', true)),
        _ => None,
    }
}

/// Scan a text block from the opening `"""` (at `q`) to just past the closing
/// `"""` (or EOF). Honors `\\` escapes.
fn text_block(b: &[u8], q: usize) -> usize {
    let n = b.len();
    let mut i = q + 3;
    while i < n {
        if b[i] == b'\\' {
            i += 2;
        } else if b[i] == b'"' && i + 2 < n && b[i + 1] == b'"' && b[i + 2] == b'"' {
            return i + 3;
        } else if b[i] == b'"' && i + 2 == n && i + 1 < n && b[i + 1] == b'"' {
            return n; // closing triple flush at EOF
        } else {
            i += 1;
        }
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
        let t = "int x = 1; // hi there\n";
        let s = spans(t);
        assert_eq!(at(t, &s, SynKind::Comment), vec!["// hi there"], "{s:?}");
    }

    #[test]
    fn block_comment_does_not_nest() {
        let t = "/* a /* b */ c */ x";
        let s = spans(t);
        // The FIRST `*/` closes (Java block comments do not nest).
        assert!(has(&s, 0, 12, SynKind::Comment), "{s:?}");
    }

    #[test]
    fn string_with_escaped_quote() {
        let t = "String s = \"a\\\"b\";";
        let s = spans(t);
        assert_eq!(at(t, &s, SynKind::Str), vec!["\"a\\\"b\""], "{s:?}");
    }

    #[test]
    fn char_literal_and_escape() {
        let t = "char c = 'x'; char n = '\\n'; char u = '\\u0041';";
        let s = spans(t);
        let ss = at(t, &s, SynKind::Str);
        assert!(ss.contains(&"'x'"), "{ss:?}");
        assert!(ss.contains(&"'\\n'"), "{ss:?}");
        assert!(ss.contains(&"'\\u0041'"), "{ss:?}");
    }

    #[test]
    fn text_block_multiline() {
        let t = "String d = \"\"\"\nline one\nline two\"\"\";";
        let s = spans(t);
        assert_eq!(
            at(t, &s, SynKind::Str),
            vec!["\"\"\"\nline one\nline two\"\"\""],
            "{s:?}"
        );
    }

    #[test]
    fn numbers_bools_and_null() {
        let t = "int a = 42; long b = 0xFF_L; double c = 3.14e2; var f = 1_000L; boolean ok = true; Object z = null; boolean no = false;";
        let s = spans(t);
        let cs = at(t, &s, SynKind::Constant);
        for want in ["42", "0xFF_L", "3.14e2", "1_000L", "true", "null", "false"] {
            assert!(cs.contains(&want), "missing {want}: {cs:?}");
        }
    }

    #[test]
    fn member_access_not_eaten_by_number() {
        // `0.toString()` style: the `.` before an identifier must not be consumed.
        let t = "x = 5.length;";
        let s = spans(t);
        assert!(at(t, &s, SynKind::Constant).contains(&"5"), "{s:?}");
    }

    #[test]
    fn definition_after_class_and_friends() {
        let t = "class Widget {}\ninterface Drawable {}\nenum Color {}\nrecord Point(int x) {}";
        let s = spans(t);
        let ds = at(t, &s, SynKind::Definition);
        for want in ["Widget", "Drawable", "Color", "Point"] {
            assert!(ds.contains(&want), "missing {want}: {ds:?}");
        }
    }

    #[test]
    fn keyword_itself_is_not_styled() {
        // `class` keyword stays default ink; only the NAME is a Definition.
        let t = "class Foo {}";
        let s = spans(t);
        assert!(
            !has(&s, 0, 5, SynKind::Definition),
            "the `class` keyword must stay plain: {s:?}"
        );
        assert!(
            has(&s, 6, 9, SynKind::Definition),
            "`Foo` is the definition: {s:?}"
        );
    }

    #[test]
    fn plain_code_has_no_spans() {
        // No comment / literal / def-keyword -> nothing highlighted (Alabaster).
        let t = "int result = compute(a, b) + offset;";
        assert!(spans(t).is_empty(), "{:?}", spans(t));
    }

    #[test]
    fn reference_snippet() {
        let t = "// sum\nclass Adder {\n    int add(int a, int b) {\n        return a + b; // ok\n    }\n}\nstatic final int MAX = 100;\n";
        let s = spans(t);
        assert_eq!(
            at(t, &s, SynKind::Comment),
            vec!["// sum", "// ok"],
            "{s:?}"
        );
        assert!(at(t, &s, SynKind::Definition).contains(&"Adder"), "{s:?}");
        assert!(at(t, &s, SynKind::Constant).contains(&"100"), "{s:?}");
    }
}
