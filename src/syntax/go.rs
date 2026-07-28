//! Go — the shared definition walk ([`crate::syntax::scanner`]) under Go's
//! constants. It recognizes only what the four Alabaster roles need and leaves
//! everything else (keywords, operators, identifiers, punctuation) default ink:
//!
//! - `Comment`    — `// line` and `/* block */` comments (Go blocks do NOT nest).
//! - `Str`        — interpreted `"..."` strings, raw `` `...` `` strings
//!   (multiline, no escapes), and `'r'` rune literals.
//! - `Constant`   — numeric literals (incl. `0x`/`0o`/`0b`, hex floats,
//!   `_` separators, the `i` imaginary suffix) and `true` / `false` / `nil` /
//!   `iota`.
//! - `Definition` — the identifier right after a `func` / `type` / `var`
//!   / `const` / `package` introducer (a `func` method receiver in parens is
//!   skipped so the METHOD name is the one marked).

use super::scanner::{BlockComment, LangSpec, LineComment, Number, WordRule};

pub(super) const SPEC: LangSpec = LangSpec {
    line: LineComment::Slashes,
    block: BlockComment::Flat,
    string_at,
    number: Number::Shared(super::NumOpts {
        radix: b"xXoObB",
        // Go's hex floats (`0x1p-2`, `0x1.8p3`) put a `.` inside the radix body.
        radix_extra: b".",
        dot_dot_stops: false,
    }),
    ident_start: super::is_ident_start,
    ident_continue: super::is_ident_continue,
    def_kws: &["func", "type", "var", "const", "package"],
    const_words: &["true", "false", "nil", "iota"],
    words: WordRule::Standard,
    def_survives: b"",
    receiver_kw: Some("func"),
};

/// Go's three literal forms. A raw `` ` `` string takes no escapes and may span
/// newlines; an interpreted string and a rune literal both stop at a newline.
fn string_at(b: &[u8], i: usize) -> Option<usize> {
    match b[i] {
        b'`' => {
            let n = b.len();
            let mut j = i + 1;
            while j < n && b[j] != b'`' {
                j += 1;
            }
            Some(if j < n { j + 1 } else { j })
        }
        b'"' => Some(super::scan_quoted(b, i, b'"', true)),
        b'\'' => Some(super::scan_quoted(b, i, b'\'', true)),
        _ => None,
    }
}

#[cfg(test)]
fn spans(text: &str) -> super::Spans {
    super::scanner::scan(&SPEC, text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::syntax::SynKind;
    use crate::syntax::testutil::{at, has};

    #[test]
    fn line_comment() {
        let t = "x := 1 // hi there\n";
        let s = spans(t);
        assert_eq!(at(t, &s, SynKind::Comment), vec!["// hi there"], "{s:?}");
    }

    #[test]
    fn block_comment_not_nested() {
        // Go blocks do NOT nest: the FIRST `*/` closes it.
        let t = "/* a /* b */ c";
        let s = spans(t);
        assert_eq!(at(t, &s, SynKind::Comment), vec!["/* a /* b */"], "{s:?}");
    }

    #[test]
    fn interpreted_string_with_escaped_quote() {
        let t = r#"s := "a\"b""#;
        let s = spans(t);
        assert_eq!(at(t, &s, SynKind::Str), vec![r#""a\"b""#], "{s:?}");
    }

    #[test]
    fn raw_string_multiline() {
        let t = "s := `line one\nline two`\n";
        let s = spans(t);
        assert_eq!(
            at(t, &s, SynKind::Str),
            vec!["`line one\nline two`"],
            "{s:?}"
        );
    }

    #[test]
    fn rune_literals() {
        let t = "a := 'x'; b := '\\n'; c := '世'\n";
        let s = spans(t);
        let ss = at(t, &s, SynKind::Str);
        assert!(ss.contains(&"'x'"), "{ss:?}");
        assert!(ss.contains(&"'\\n'"), "{ss:?}");
        assert!(ss.contains(&"'世'"), "{ss:?}");
    }

    #[test]
    fn numbers_and_constants() {
        let t = "a := 42; b := 0xFF_u; c := 3.14; d := 1_000; e := 2i; ok := true; z := nil; i := iota\n";
        let s = spans(t);
        let cs = at(t, &s, SynKind::Constant);
        for want in ["42", "0xFF_u", "3.14", "1_000", "2i", "true", "nil", "iota"] {
            assert!(cs.contains(&want), "missing {want}: {cs:?}");
        }
    }

    #[test]
    fn int_method_call_not_eaten() {
        // `x.foo` — the `.` is an attribute access, not a fractional point.
        let t = "n := 3\nv := n.foo\n";
        let s = spans(t);
        // Only the bare `3` is a constant; nothing swallows past it.
        assert!(at(t, &s, SynKind::Constant).contains(&"3"), "{s:?}");
    }

    #[test]
    fn definitions_after_keywords() {
        let t = "package main\nfunc add(a int) int { return a }\ntype Widget struct{}\nvar count int\nconst Max = 100\n";
        let s = spans(t);
        let ds = at(t, &s, SynKind::Definition);
        for want in ["main", "add", "Widget", "count", "Max"] {
            assert!(ds.contains(&want), "missing {want}: {ds:?}");
        }
    }

    #[test]
    fn method_receiver_is_skipped() {
        // The method NAME `Area`, not the receiver `r`, is the Definition.
        let t = "func (r *Rect) Area() int { return 0 }\n";
        let s = spans(t);
        let ds = at(t, &s, SynKind::Definition);
        assert!(ds.contains(&"Area"), "{ds:?}");
        assert!(!ds.contains(&"r"), "receiver wrongly marked: {ds:?}");
    }

    #[test]
    fn keyword_itself_is_not_styled() {
        // `func` keyword stays default ink; only the NAME is a Definition.
        let t = "func main() {}";
        let s = spans(t);
        assert!(
            !has(&s, 0, 4, SynKind::Definition),
            "`func` must stay plain: {s:?}"
        );
        assert!(
            has(&s, 5, 9, SynKind::Definition),
            "`main` is the definition: {s:?}"
        );
    }

    #[test]
    fn plain_code_has_no_spans() {
        // No comment / literal / def-keyword -> nothing highlighted (Alabaster).
        let t = "result := compute(a, b) + offset";
        assert!(spans(t).is_empty(), "{:?}", spans(t));
    }

    #[test]
    fn reference_snippet() {
        // A compact end-to-end snippet asserting all four roles at once.
        let t = "// sum\nfunc add(a int, b int) int {\n\ttotal := a + b // ok\n\treturn total\n}\nconst Max = 100\n";
        let s = spans(t);
        assert_eq!(
            at(t, &s, SynKind::Comment),
            vec!["// sum", "// ok"],
            "{s:?}"
        );
        let ds = at(t, &s, SynKind::Definition);
        assert!(ds.contains(&"add") && ds.contains(&"Max"), "{ds:?}");
        assert!(at(t, &s, SynKind::Constant).contains(&"100"), "{s:?}");
    }
}
