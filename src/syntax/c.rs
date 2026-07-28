//! C — the shared definition walk ([`crate::syntax::scanner`]) under C's
//! constants. It recognizes only what the four Alabaster roles need and leaves
//! everything else (keywords, operators, identifiers, punctuation, preprocessor
//! directives) as the default ink:
//!
//! - `Comment`    — `// line` and `/* block */` comments (C blocks do NOT nest —
//!   the first `*/` closes).
//! - `Str`        — `"strings"` and `'c'` char literals, including the encoding
//!   prefixes `L`, `u`, `U`, and `u8` (`L"..."`, `u8"..."`, `U'x'`, …).
//! - `Constant`   — numeric literals (decimal, `0x`/`0b` radix, octal, floats,
//!   `u`/`l`/`f` suffixes) and `true` / `false` / `NULL` / `nullptr`.
//! - `Definition` — the identifier right after a `struct` / `union` / `enum`
//!   introducer (C tags the name right after the keyword; full function /
//!   typedef-name detection needs a real parser, so we stay best-effort here).

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
    ident_start: super::is_ident_start,
    ident_continue: super::is_ident_continue,
    // C has no `fn`/`func`; the reliably-positioned names are the tag types. This
    // also handles `typedef struct Foo`: `typedef` is plain and `struct` arms.
    def_kws: &["struct", "union", "enum"],
    const_words: &["true", "false", "NULL", "nullptr"],
    words: WordRule::Standard,
    def_survives: b"",
    receiver_kw: None,
};

/// A `"`/`'` literal, optionally behind an `L`/`u`/`U`/`u8` encoding prefix.
/// Neither form stops at a newline in C.
fn string_at(b: &[u8], i: usize) -> Option<usize> {
    let q = match encoding_prefix(b, i) {
        Some(q) => q,
        None if b[i] == b'"' || b[i] == b'\'' => i,
        None => return None,
    };
    Some(super::scan_quoted(b, q, b[q], false))
}

/// If a string/char encoding prefix (`L`, `u`, `U`, `u8`) begins at `i` and is
/// immediately followed by a quote, return the byte index of that quote; else
/// `None`.
fn encoding_prefix(b: &[u8], i: usize) -> Option<usize> {
    let n = b.len();
    match b[i] {
        b'L' | b'U' if i + 1 < n && (b[i + 1] == b'"' || b[i + 1] == b'\'') => Some(i + 1),
        b'u' => {
            if i + 2 < n && b[i + 1] == b'8' && (b[i + 2] == b'"' || b[i + 2] == b'\'') {
                Some(i + 2)
            } else if i + 1 < n && (b[i + 1] == b'"' || b[i + 1] == b'\'') {
                Some(i + 1)
            } else {
                None
            }
        }
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
        let t = "int x = 1; // hi there\n";
        let s = spans(t);
        assert!(at(t, &s, SynKind::Comment) == vec!["// hi there"], "{s:?}");
    }

    #[test]
    fn block_comment_does_not_nest() {
        let t = "/* a /* b */ c */ x";
        let s = spans(t);
        // C closes at the FIRST `*/` — `/* a /* b */` is the whole comment.
        assert!(has(&s, 0, 12, SynKind::Comment), "{s:?}");
    }

    #[test]
    fn string_with_escaped_quote() {
        let t = "char *s = \"a\\\"b\";";
        let s = spans(t);
        assert_eq!(at(t, &s, SynKind::Str), vec!["\"a\\\"b\""], "{s:?}");
    }

    #[test]
    fn wide_and_utf8_string_prefixes() {
        let t = "L\"wide\" u8\"utf\" U\"big\"";
        let s = spans(t);
        let strs = at(t, &s, SynKind::Str);
        assert!(strs.contains(&"L\"wide\""), "{strs:?}");
        assert!(strs.contains(&"u8\"utf\""), "{strs:?}");
        assert!(strs.contains(&"U\"big\""), "{strs:?}");
    }

    #[test]
    fn char_literal_and_escape() {
        let t = "char c = 'x'; char n = '\\n';";
        let s = spans(t);
        let strs = at(t, &s, SynKind::Str);
        assert!(strs.contains(&"'x'"), "{strs:?}");
        assert!(strs.contains(&"'\\n'"), "{strs:?}");
    }

    #[test]
    fn numbers_and_constants() {
        let t = "int a = 42; unsigned b = 0xFFu; double c = 3.14; long d = 0b1010; void *p = NULL; _Bool ok = true;";
        let s = spans(t);
        let cs = at(t, &s, SynKind::Constant);
        assert!(cs.contains(&"42"), "{cs:?}");
        assert!(cs.contains(&"0xFFu"), "{cs:?}");
        assert!(cs.contains(&"3.14"), "{cs:?}");
        assert!(cs.contains(&"0b1010"), "{cs:?}");
        assert!(cs.contains(&"NULL"), "{cs:?}");
        assert!(cs.contains(&"true"), "{cs:?}");
    }

    #[test]
    fn member_access_not_eaten_by_number() {
        // A leading-digit token must stop before a `.field` member access.
        let t = "x = a1.field;";
        let s = spans(t);
        // `a1` is an identifier (not a number start) -> nothing highlighted here.
        assert!(at(t, &s, SynKind::Constant).is_empty(), "{s:?}");
    }

    #[test]
    fn definition_after_struct_enum_union() {
        let t = "struct Widget { int x; };\nenum Color { RED };\nunion Pad { int i; };";
        let s = spans(t);
        let ds = at(t, &s, SynKind::Definition);
        assert!(ds.contains(&"Widget"), "{ds:?}");
        assert!(ds.contains(&"Color"), "{ds:?}");
        assert!(ds.contains(&"Pad"), "{ds:?}");
    }

    #[test]
    fn typedef_struct_names_the_tag() {
        // `typedef struct Node` -> `Node` is the (re-armed) definition; `struct`
        // itself stays plain.
        let t = "typedef struct Node Node;";
        let s = spans(t);
        assert!(at(t, &s, SynKind::Definition).contains(&"Node"), "{s:?}");
        assert!(
            !has(&s, 8, 14, SynKind::Definition),
            "the `struct` keyword must stay plain: {s:?}"
        );
    }

    #[test]
    fn keyword_itself_is_not_styled() {
        // The `struct` keyword stays default ink; only the NAME is a Definition.
        let t = "struct Foo {};";
        let s = spans(t);
        assert!(
            !has(&s, 0, 6, SynKind::Definition),
            "the `struct` keyword must stay plain: {s:?}"
        );
        assert!(
            has(&s, 7, 10, SynKind::Definition),
            "`Foo` is the definition: {s:?}"
        );
    }

    #[test]
    fn plain_code_and_keywords_have_no_spans() {
        // No comment / literal / def-keyword -> nothing highlighted (Alabaster).
        // `int`, `compute`, etc. are keywords/identifiers and ride the default ink.
        let t = "int result = compute(a, b) + offset;";
        assert!(spans(t).is_empty(), "{:?}", spans(t));
    }

    #[test]
    fn include_directive_rides_default_ink() {
        // The `#include` directive stays plain; only the quoted header is a Str.
        let t = "#include <stdio.h>\n#include \"local.h\"\n";
        let s = spans(t);
        assert_eq!(at(t, &s, SynKind::Str), vec!["\"local.h\""], "{s:?}");
    }

    #[test]
    fn reference_snippet() {
        // A compact end-to-end snippet asserting all four roles at once.
        let t = "// sum\nstruct Acc { int n; };\nint add(int a, int b) {\n    int total = a + b; // ok\n    return total;\n}\n#define MAX 100\n";
        let s = spans(t);
        assert_eq!(
            at(t, &s, SynKind::Comment),
            vec!["// sum", "// ok"],
            "{s:?}"
        );
        assert!(at(t, &s, SynKind::Definition).contains(&"Acc"), "{s:?}");
        assert!(at(t, &s, SynKind::Constant).contains(&"100"), "{s:?}");
    }
}
