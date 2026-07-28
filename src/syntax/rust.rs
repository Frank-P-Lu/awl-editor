//! Rust — the shared definition walk ([`crate::syntax::scanner`]) under Rust's
//! constants. It recognizes only what the four Alabaster roles need and leaves
//! everything else (keywords, operators, identifiers, punctuation) default ink:
//!
//! - `Comment`    — `// line` and `/* block */` (nested) comments.
//! - `Str`        — `"strings"`, `'c'` char literals, and raw strings (`r"..."`,
//!   `r#"..."#`, plus the `b`-prefixed byte variants).
//! - `Constant`   — numeric literals (incl. `0x`/`0o`/`0b`, floats, `_`
//!   separators, type suffixes) and `true` / `false` / `None`.
//! - `Definition` — the identifier right after a `fn` / `struct` / `enum` /
//!   `trait` / `type` / `union` / `const` / `static` / `mod` introducer.

use super::scanner::{BlockComment, LangSpec, LineComment, Number, WordRule};

pub(super) const SPEC: LangSpec = LangSpec {
    line: LineComment::Slashes,
    block: BlockComment::Nested,
    string_at,
    number: Number::Shared(super::NumOpts {
        radix: b"xXoObB",
        radix_extra: b"",
        dot_dot_stops: true,
    }),
    ident_start: super::is_ident_start,
    ident_continue: super::is_ident_continue,
    def_kws: &[
        "fn", "struct", "enum", "trait", "type", "union", "const", "static", "mod",
    ],
    const_words: &["true", "false", "None"],
    words: WordRule::Standard,
    // A `'` that opens a LIFETIME rather than a char literal is not a token that
    // cancels a pending name — it rides through to the identifier behind it.
    def_survives: b"'",
    receiver_kw: None,
};

/// A raw string (`r"…"`, `r#"…"#`, `b`-prefixed too), a `"`/`b"` string, or a
/// char literal. A `'` that opens a LIFETIME yields `None`, so it rides the
/// default-ink path and the name behind it scans as a plain token.
fn string_at(b: &[u8], i: usize) -> Option<usize> {
    if let Some(end) = raw_string(b, i) {
        return Some(end);
    }
    let n = b.len();
    match b[i] {
        b'"' => Some(super::scan_quoted(b, i, b'"', false)),
        b'b' if i + 1 < n && b[i + 1] == b'"' => Some(super::scan_quoted(b, i + 1, b'"', false)),
        b'\'' => char_literal(b, i),
        _ => None,
    }
}

/// If a raw string literal starts at `i` (`r"`, `r#"`, …, optionally `b`-prefixed),
/// return the byte index just past its close; else `None`. Handles any number of
/// `#` hashes (closing requires the matching `"###`).
fn raw_string(b: &[u8], i: usize) -> Option<usize> {
    let n = b.len();
    let mut j = i;
    if j < n && b[j] == b'b' {
        j += 1;
    }
    if j >= n || b[j] != b'r' {
        return None;
    }
    j += 1;
    let mut hashes = 0usize;
    while j < n && b[j] == b'#' {
        hashes += 1;
        j += 1;
    }
    if j >= n || b[j] != b'"' {
        return None;
    }
    j += 1; // past opening quote
    // Scan for a closing `"` followed by `hashes` `#`s.
    while j < n {
        if b[j] == b'"' {
            let mut k = j + 1;
            let mut h = 0;
            while h < hashes && k < n && b[k] == b'#' {
                h += 1;
                k += 1;
            }
            if h == hashes {
                return Some(k);
            }
        }
        j += 1;
    }
    Some(n) // unterminated: run to EOF
}

/// If a CHAR literal starts at `i` (`'x'`, `'\n'`, `'\u{1F}'`), return the index
/// just past the closing quote; `None` if it is actually a lifetime (`'a`).
fn char_literal(b: &[u8], i: usize) -> Option<usize> {
    let n = b.len();
    debug_assert_eq!(b[i], b'\'');
    let mut j = i + 1;
    if j >= n {
        return None;
    }
    if b[j] == b'\\' {
        // Escape: skip the backslash + escape body, then require a closing quote.
        j += 1;
        if j < n && b[j] == b'u' {
            // `\u{..}` — skip to the closing brace.
            while j < n && b[j] != b'}' {
                j += 1;
            }
            if j < n {
                j += 1;
            }
        } else if j < n {
            j += 1;
        }
        if j < n && b[j] == b'\'' {
            return Some(j + 1);
        }
        return None;
    }
    // Unescaped: a single (possibly multibyte) char then a closing quote means a
    // char literal; otherwise it is a lifetime.
    let ch_len = utf8_len(b[j]);
    let close = j + ch_len;
    if close < n && b[close] == b'\'' {
        Some(close + 1)
    } else {
        None
    }
}

/// Byte length of the UTF-8 char whose lead byte is `c`.
fn utf8_len(c: u8) -> usize {
    if c < 0x80 {
        1
    } else if c >> 5 == 0b110 {
        2
    } else if c >> 4 == 0b1110 {
        3
    } else {
        4
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
        assert!(at(t, &s, SynKind::Comment) == vec!["// hi there"], "{s:?}");
    }

    #[test]
    fn block_comment_nested() {
        let t = "/* a /* b */ c */ x";
        let s = spans(t);
        // The whole nested block is ONE comment span.
        assert!(has(&s, 0, 17, SynKind::Comment), "{s:?}");
    }

    #[test]
    fn string_with_escaped_quote() {
        let t = r#"let s = "a\"b";"#;
        let s = spans(t);
        assert_eq!(at(t, &s, SynKind::Str), vec![r#""a\"b""#], "{s:?}");
    }

    #[test]
    fn raw_string_with_hashes() {
        let t = r####"let s = r#"he said "hi""#;"####;
        let s = spans(t);
        assert_eq!(
            at(t, &s, SynKind::Str),
            vec![r##"r#"he said "hi""#"##],
            "{s:?}"
        );
    }

    #[test]
    fn char_literal_and_lifetime() {
        let t = "let c = 'x'; fn f<'a>(r: &'a str) {}";
        let s = spans(t);
        // 'x' is a Str; 'a (a lifetime) is NOT.
        assert_eq!(at(t, &s, SynKind::Str), vec!["'x'"], "{s:?}");
    }

    #[test]
    fn char_escape_literal() {
        let t = r"let n = '\n';";
        let s = spans(t);
        assert_eq!(at(t, &s, SynKind::Str), vec![r"'\n'"], "{s:?}");
    }

    #[test]
    fn numbers_and_bools() {
        let t = "let a = 42; let b = 0xFF_u8; let c = 3.14; let ok = true;";
        let s = spans(t);
        let cs = at(t, &s, SynKind::Constant);
        assert!(cs.contains(&"42"), "{cs:?}");
        assert!(cs.contains(&"0xFF_u8"), "{cs:?}");
        assert!(cs.contains(&"3.14"), "{cs:?}");
        assert!(cs.contains(&"true"), "{cs:?}");
    }

    #[test]
    fn range_op_not_eaten_by_number() {
        let t = "for i in 0..5 {}";
        let s = spans(t);
        let cs = at(t, &s, SynKind::Constant);
        assert!(
            cs.contains(&"0") && cs.contains(&"5"),
            "ranges split: {cs:?}"
        );
    }

    #[test]
    fn definition_after_fn_and_struct() {
        let t = "pub fn frobnicate(x: i32) {}\nstruct Widget;\nenum E {}\ntype Alias = u8;";
        let s = spans(t);
        let ds = at(t, &s, SynKind::Definition);
        assert!(ds.contains(&"frobnicate"), "{ds:?}");
        assert!(ds.contains(&"Widget"), "{ds:?}");
        assert!(ds.contains(&"E"), "{ds:?}");
        assert!(ds.contains(&"Alias"), "{ds:?}");
    }

    #[test]
    fn definition_with_a_non_ascii_letter_is_one_token() {
        // A Unicode (XID-ish) identifier is highlighted as ONE definition token —
        // the accented / non-Latin letter is part of the name, not a token
        // boundary. Without the shared helpers' `>= 0x80` broadening the name
        // would be under-lexed (split or dropped) where an ASCII one is whole.
        let t = "fn café() {}\nstruct Δelta;";
        let s = spans(t);
        let ds = at(t, &s, SynKind::Definition);
        assert!(
            ds.contains(&"café"),
            "the accented fn name is one definition: {ds:?}"
        );
        assert!(
            ds.contains(&"Δelta"),
            "a Greek-initial type name is one definition: {ds:?}"
        );
    }

    #[test]
    fn keyword_itself_is_not_styled() {
        // `fn` keyword stays default ink; only the NAME is a Definition.
        let t = "fn main() {}";
        let s = spans(t);
        assert!(
            !has(&s, 0, 2, SynKind::Definition),
            "the `fn` keyword must stay plain: {s:?}"
        );
        assert!(
            has(&s, 3, 7, SynKind::Definition),
            "`main` is the definition: {s:?}"
        );
    }

    #[test]
    fn plain_code_has_no_spans() {
        // No comment / literal / def-keyword -> nothing highlighted (Alabaster).
        let t = "let result = compute(a, b) + offset;";
        assert!(spans(t).is_empty(), "{:?}", spans(t));
    }

    #[test]
    fn reference_snippet() {
        // A compact end-to-end snippet asserting all four roles at once.
        let t = "// sum\nfn add(a: i32, b: i32) -> i32 {\n    let total = a + b; // ok\n    return total;\n}\nconst MAX: u32 = 100;\n";
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
