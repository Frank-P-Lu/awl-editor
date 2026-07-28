//! TypeScript — the shared definition walk ([`crate::syntax::scanner`]) under
//! TS's constants. Everything outside the four Alabaster roles (keywords,
//! operators, identifiers, punctuation) stays the default ink:
//!
//! - `Comment`    — `// line` and `/* block */` comments (TS blocks do NOT nest —
//!   the first `*/` closes).
//! - `Str`        — `"..."`, `'...'`, and `` `...` `` template literals
//!   (multiline; an interpolated `${…}` rides inside the one Str span).
//! - `Constant`   — numeric literals (`0x`/`0o`/`0b`, floats, `_` separators,
//!   exponents, the `n` BigInt suffix) and `true` / `false` / `null` /
//!   `undefined`.
//! - `Definition` — the identifier right after a `function` / `class` /
//!   `interface` / `type` / `enum` / `namespace` / `module` introducer or a
//!   `const` / `let` / `var` binding.

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
    def_kws: &[
        "function",
        "class",
        "interface",
        "type",
        "enum",
        "namespace",
        "module",
        "const",
        "let",
        "var",
    ],
    const_words: &["true", "false", "null", "undefined"],
    words: WordRule::Standard,
    def_survives: b"",
    receiver_kw: None,
};

/// A `` ` `` template (spans newlines, `${…}` rides inside the one span) or a
/// `"`/`'` string (which does not cross an unescaped newline).
fn string_at(b: &[u8], i: usize) -> Option<usize> {
    match b[i] {
        b'`' => Some(super::scan_quoted(b, i, b'`', false)),
        b'"' | b'\'' => Some(super::scan_quoted(b, i, b[i], true)),
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
        let t = "let x = 1; // hi there\n";
        let s = spans(t);
        assert!(at(t, &s, SynKind::Comment) == vec!["// hi there"], "{s:?}");
    }

    #[test]
    fn block_comment_does_not_nest() {
        let t = "/* a /* b */ c */ x";
        let s = spans(t);
        // The FIRST `*/` closes the block (TS comments don't nest).
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
        let t = "const c = 'hi';";
        let s = spans(t);
        assert_eq!(at(t, &s, SynKind::Str), vec!["'hi'"], "{s:?}");
    }

    #[test]
    fn template_literal_multiline_and_interpolation() {
        let t = "const g = `line one ${x + 1}\nline two`;";
        let s = spans(t);
        // The whole template — newline + `${…}` — is ONE Str span.
        assert_eq!(
            at(t, &s, SynKind::Str),
            vec!["`line one ${x + 1}\nline two`"],
            "{s:?}"
        );
    }

    #[test]
    fn numbers_and_constants() {
        let t = "let a = 42; let b = 0xFF; let c = 3.14; let d = 1_000n; let ok = true; let z = null; let u = undefined;";
        let s = spans(t);
        let cs = at(t, &s, SynKind::Constant);
        for want in ["42", "0xFF", "3.14", "1_000n", "true", "null", "undefined"] {
            assert!(cs.contains(&want), "missing {want}: {cs:?}");
        }
    }

    #[test]
    fn spread_not_eaten_by_number() {
        let t = "const xs = [0, ...rest];";
        let s = spans(t);
        let cs = at(t, &s, SynKind::Constant);
        assert!(cs.contains(&"0"), "{cs:?}");
    }

    #[test]
    fn definitions_after_introducers() {
        let t = "function frobnicate(x: number) {}\nclass Widget {}\ninterface Shape {}\ntype Alias = number;\nenum Color {}\nconst MAX = 100;";
        let s = spans(t);
        let ds = at(t, &s, SynKind::Definition);
        for want in ["frobnicate", "Widget", "Shape", "Alias", "Color", "MAX"] {
            assert!(ds.contains(&want), "missing {want}: {ds:?}");
        }
    }

    #[test]
    fn keyword_itself_is_not_styled() {
        // `function` keyword stays default ink; only the NAME is a Definition.
        let t = "function main() {}";
        let s = spans(t);
        assert!(
            !has(&s, 0, 8, SynKind::Definition),
            "the keyword must stay plain: {s:?}"
        );
        assert!(
            has(&s, 9, 13, SynKind::Definition),
            "`main` is the definition: {s:?}"
        );
    }

    #[test]
    fn plain_code_has_no_spans() {
        // No comment / literal / def-keyword -> nothing highlighted (Alabaster).
        let t = "result = compute(a, b) + offset;";
        assert!(spans(t).is_empty(), "{:?}", spans(t));
    }

    #[test]
    fn reference_snippet() {
        let t = "// sum\nfunction add(a: number, b: number): number {\n    const total = a + b; // ok\n    return total;\n}\nconst MAX = 100;\n";
        let s = spans(t);
        assert_eq!(
            at(t, &s, SynKind::Comment),
            vec!["// sum", "// ok"],
            "{s:?}"
        );
        let ds = at(t, &s, SynKind::Definition);
        assert!(
            ds.contains(&"add") && ds.contains(&"total") && ds.contains(&"MAX"),
            "{ds:?}"
        );
        assert!(at(t, &s, SynKind::Constant).contains(&"100"), "{s:?}");
    }
}
