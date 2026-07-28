//! PHP — the shared definition walk ([`crate::syntax::scanner`]) under PHP's
//! constants. It emits only the four Alabaster roles and leaves everything else
//! (keywords, operators, `$variables`, identifiers, punctuation) default ink:
//!
//! - `Comment`    — `// line`, `# line`, and `/* block */` comments (PHP blocks
//!   do NOT nest). A `#[` attribute is NOT a comment.
//! - `Str`        — `'single'` / `"double"` (interpolated) strings as one span
//!   each, plus heredoc / nowdoc (`<<<LABEL … LABEL`).
//! - `Constant`   — numeric literals (`0x`/`0o`/`0b`, floats, `_` separators) and
//!   `true` / `false` / `null` (case-insensitive).
//! - `Definition` — the identifier right after a `function` / `class` /
//!   `interface` / `trait` / `enum` / `const` introducer (case-insensitive).
//!
//! A `$` sigil is not an identifier byte here, so `$name` reaches the scanner's
//! default-ink path and the variable name scans as a plain token.

use super::scanner::{BlockComment, LangSpec, LineComment, Number, WordRule};

pub(super) const SPEC: LangSpec = LangSpec {
    line: LineComment::SlashesOrHashNotAttr,
    block: BlockComment::Flat,
    string_at,
    number: Number::Shared(super::NumOpts {
        radix: b"xXoObB",
        radix_extra: b"",
        dot_dot_stops: true,
    }),
    ident_start: super::is_ident_start,
    ident_continue: super::is_ident_continue,
    def_kws: &["function", "class", "interface", "trait", "enum", "const"],
    const_words: &["true", "false", "null"],
    // PHP keywords and literals are case-insensitive: `TRUE`/`True`/`true` alike.
    words: WordRule::CaseInsensitive,
    def_survives: b"",
    receiver_kw: None,
};

/// A heredoc / nowdoc, or a `'`/`"` string (neither stops at a newline; an
/// interpolated double-quoted string is one span).
fn string_at(b: &[u8], i: usize) -> Option<usize> {
    let n = b.len();
    if b[i] == b'<' && i + 2 < n && b[i + 1] == b'<' && b[i + 2] == b'<' {
        if let Some(end) = heredoc(b, i) {
            return Some(end);
        }
    }
    matches!(b[i], b'"' | b'\'').then(|| super::scan_quoted(b, i, b[i], false))
}

/// If a heredoc / nowdoc starts at `i` (`<<<LABEL`, `<<<"LABEL"`, or `<<<'LABEL'`
/// for a nowdoc), return the byte index just past the closing label; else `None`.
/// The closing label is matched at the start of a line, allowing PHP 7.3+ leading
/// indentation, and must not be glued to a longer identifier.
fn heredoc(b: &[u8], i: usize) -> Option<usize> {
    let n = b.len();
    let mut j = i + 3; // past `<<<`
    // PHP forbids space here, but be lenient with a stray space/tab.
    while j < n && (b[j] == b' ' || b[j] == b'\t') {
        j += 1;
    }
    let quote = if j < n && (b[j] == b'"' || b[j] == b'\'') {
        let q = b[j];
        j += 1;
        Some(q)
    } else {
        None
    };
    // The label is a normal identifier.
    let label_start = j;
    if j >= n || !super::is_ident_start(b[j]) {
        return None;
    }
    while j < n && super::is_ident_continue(b[j]) {
        j += 1;
    }
    let label = &b[label_start..j];
    // A quoted label must close with the same quote.
    if let Some(qc) = quote {
        if j < n && b[j] == qc {
            j += 1;
        } else {
            return None;
        }
    }
    // The rest of the opening line must be only whitespace before the newline.
    while j < n && b[j] != b'\n' {
        if b[j] != b' ' && b[j] != b'\t' && b[j] != b'\r' {
            return None;
        }
        j += 1;
    }
    if j >= n {
        return Some(n); // unterminated: run to EOF
    }
    j += 1; // past the opening newline

    // Scan body lines for the closing label.
    while j < n {
        let mut k = j;
        while k < n && (b[k] == b' ' || b[k] == b'\t') {
            k += 1;
        }
        if k + label.len() <= n && &b[k..k + label.len()] == label {
            let after = k + label.len();
            // The closer must not be part of a longer identifier (`LABEL` vs
            // `LABELX`); `;`, `,`, a newline, or EOF all end it cleanly.
            if after >= n || !super::is_ident_continue(b[after]) {
                return Some(after);
            }
        }
        // Advance to the next line.
        while j < n && b[j] != b'\n' {
            j += 1;
        }
        if j < n {
            j += 1;
        }
    }
    Some(n) // unterminated: run to EOF
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
    fn line_comments_slash_and_hash() {
        let t = "$x = 1; // slash\n$y = 2; # hash\n";
        let s = spans(t);
        assert_eq!(
            at(t, &s, SynKind::Comment),
            vec!["// slash", "# hash"],
            "{s:?}"
        );
    }

    #[test]
    fn hash_attribute_is_not_a_comment() {
        // PHP 8 attribute `#[...]` must NOT recede as a comment.
        let t = "#[Route('/x')]\nfunction f() {}";
        let s = spans(t);
        assert!(at(t, &s, SynKind::Comment).is_empty(), "{s:?}");
        // The string inside the attribute still styles.
        assert!(at(t, &s, SynKind::Str).contains(&"'/x'"), "{s:?}");
    }

    #[test]
    fn block_comment_is_not_nested() {
        let t = "/* a /* b */ c */ $x";
        let s = spans(t);
        // PHP block comments do NOT nest: the first `*/` closes it.
        assert!(has(&s, 0, 12, SynKind::Comment), "{s:?}");
    }

    #[test]
    fn strings_single_and_double() {
        let t = "$a = 'hi';\n$b = \"yo $a\";\n";
        let s = spans(t);
        let ss = at(t, &s, SynKind::Str);
        assert!(ss.contains(&"'hi'"), "{ss:?}");
        assert!(ss.contains(&"\"yo $a\""), "{ss:?}");
    }

    #[test]
    fn string_with_escaped_quote() {
        let t = r#"$s = "a\"b";"#;
        let s = spans(t);
        assert_eq!(at(t, &s, SynKind::Str), vec![r#""a\"b""#], "{s:?}");
    }

    #[test]
    fn heredoc_and_nowdoc() {
        let t = "$a = <<<EOT\nhello $x\nEOT;\n$b = <<<'RAW'\nliteral $x\nRAW;\n";
        let s = spans(t);
        let ss = at(t, &s, SynKind::Str);
        assert!(
            ss.iter()
                .any(|x| x.starts_with("<<<EOT") && x.ends_with("EOT")),
            "{ss:?}"
        );
        assert!(
            ss.iter()
                .any(|x| x.starts_with("<<<'RAW'") && x.ends_with("RAW")),
            "{ss:?}"
        );
    }

    #[test]
    fn numbers_and_constants() {
        let t = "$a = 42; $b = 0xFF; $c = 3.14; $d = 1_000; $e = true; $f = null;";
        let s = spans(t);
        let cs = at(t, &s, SynKind::Constant);
        for want in ["42", "0xFF", "3.14", "1_000", "true", "null"] {
            assert!(cs.contains(&want), "missing {want}: {cs:?}");
        }
    }

    #[test]
    fn constants_are_case_insensitive() {
        let t = "$a = TRUE; $b = Null; $c = False;";
        let s = spans(t);
        let cs = at(t, &s, SynKind::Constant);
        assert!(
            cs.contains(&"TRUE") && cs.contains(&"Null") && cs.contains(&"False"),
            "{cs:?}"
        );
    }

    #[test]
    fn definitions_after_introducers() {
        let t = "function frobnicate($x) {}\nclass Widget {}\ninterface Shape {}\ntrait T {}\nenum Suit {}\nconst MAX = 1;\n";
        let s = spans(t);
        let ds = at(t, &s, SynKind::Definition);
        for want in ["frobnicate", "Widget", "Shape", "T", "Suit", "MAX"] {
            assert!(ds.contains(&want), "missing {want}: {ds:?}");
        }
    }

    #[test]
    fn keyword_itself_is_not_styled() {
        // `function` stays default ink; only the NAME is a Definition.
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
    fn variables_are_not_highlighted() {
        let t = "$result = compute($a, $b) + $offset;";
        assert!(spans(t).is_empty(), "{:?}", spans(t));
    }

    #[test]
    fn reference_snippet() {
        let t = "<?php\n// add two\nfunction add($a, $b) {\n    $total = $a + $b; # sum\n    return $total;\n}\nconst MAX = 100;\n";
        let s = spans(t);
        assert_eq!(
            at(t, &s, SynKind::Comment),
            vec!["// add two", "# sum"],
            "{s:?}"
        );
        let ds = at(t, &s, SynKind::Definition);
        assert!(ds.contains(&"add") && ds.contains(&"MAX"), "{ds:?}");
        assert!(at(t, &s, SynKind::Constant).contains(&"100"), "{s:?}");
    }
}
