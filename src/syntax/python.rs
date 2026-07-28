//! Python — the shared definition walk ([`crate::syntax::scanner`]) under
//! Python's constants: `#` line comments, no block-comment form. Everything
//! outside the four Alabaster roles stays the default ink:
//!
//! - `Comment`    — `# line` comments.
//! - `Str`        — `'...'` / `"..."` and triple-quoted `'''...'''` / `"""..."""`,
//!   including the `r`/`b`/`f`/`u` string prefixes (and combos).
//! - `Constant`   — numeric literals and `True` / `False` / `None`.
//! - `Definition` — the identifier right after a `def` or `class`.

use super::scanner::{BlockComment, LangSpec, LineComment, Number, WordRule};

pub(super) const SPEC: LangSpec = LangSpec {
    line: LineComment::Hash,
    block: BlockComment::None,
    string_at,
    number: Number::Shared(super::NumOpts {
        radix: b"xXoObB",
        radix_extra: b"",
        dot_dot_stops: true,
    }),
    ident_start: super::is_ident_start,
    ident_continue: super::is_ident_continue,
    def_kws: &["def", "class"],
    const_words: &["True", "False", "None"],
    words: WordRule::Standard,
    def_survives: b"",
    receiver_kw: None,
};

/// A valid Python string-prefix letter (`r`/`b`/`f`/`u`, any case).
fn is_prefix(c: u8) -> bool {
    matches!(c, b'r' | b'b' | b'f' | b'u' | b'R' | b'B' | b'F' | b'U')
}

/// A triple- or single-quoted string, behind up to two prefix letters.
fn string_at(b: &[u8], i: usize) -> Option<usize> {
    let (quote, triple) = string_start(b, i)?;
    Some(if triple {
        scan_triple(b, quote)
    } else {
        super::scan_quoted(b, quote, b[quote], true)
    })
}

/// If a string literal starts at `i` — an optional `r`/`b`/`f`/`u` prefix (up to
/// two letters) immediately followed by a quote — return `(quote_index, is_triple)`;
/// else `None`. A bare quote at `i` (no prefix) also matches.
fn string_start(b: &[u8], i: usize) -> Option<(usize, bool)> {
    let n = b.len();
    let mut j = i;
    let mut k = 0;
    while k < 2 && j < n && is_prefix(b[j]) {
        j += 1;
        k += 1;
    }
    if j < n && (b[j] == b'"' || b[j] == b'\'') {
        let q = b[j];
        let triple = j + 2 < n && b[j + 1] == q && b[j + 2] == q;
        Some((j, triple))
    } else {
        None
    }
}

/// Scan a triple-quoted string from the opening quote `q` (the first of three) to
/// just past the closing triple (or EOF). Honors `\\` escapes.
fn scan_triple(b: &[u8], q: usize) -> usize {
    let n = b.len();
    let quote = b[q];
    let mut i = q + 3;
    while i < n {
        if b[i] == b'\\' {
            i += 2;
        } else if b[i] == quote && i + 2 < n && b[i + 1] == quote && b[i + 2] == quote {
            return i + 3;
        } else if b[i] == quote && i + 2 == n && i + 1 < n && b[i + 1] == quote {
            // Closing triple flush at EOF.
            return n;
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
    fn comment() {
        let t = "x = 1  # set x\n";
        assert_eq!(at(t, &spans(t), SynKind::Comment), vec!["# set x"]);
    }

    #[test]
    fn single_and_double_strings() {
        let t = "a = 'hi'\nb = \"yo\"\n";
        let s = spans(t);
        assert_eq!(at(t, &s, SynKind::Str), vec!["'hi'", "\"yo\""], "{s:?}");
    }

    #[test]
    fn triple_string_multiline() {
        let t = "doc = \"\"\"line one\nline two\"\"\"\n";
        let s = spans(t);
        assert_eq!(
            at(t, &s, SynKind::Str),
            vec!["\"\"\"line one\nline two\"\"\""],
            "{s:?}"
        );
    }

    #[test]
    fn prefixed_strings() {
        let t = "p = r'\\d+'\nq = f\"{x}\"\nr = rb'bytes'\n";
        let s = spans(t);
        let ss = at(t, &s, SynKind::Str);
        assert!(ss.contains(&"r'\\d+'"), "{ss:?}");
        assert!(ss.contains(&"f\"{x}\""), "{ss:?}");
        assert!(ss.contains(&"rb'bytes'"), "{ss:?}");
    }

    #[test]
    fn f_prefix_does_not_swallow_function_call() {
        // `format(` must NOT be read as an f-string prefix.
        let t = "format(x)";
        assert!(spans(t).is_empty(), "{:?}", spans(t));
    }

    #[test]
    fn numbers_and_constants() {
        let t = "a = 42\nb = 0xFF\nc = 3.14\nd = 1_000\nok = True\nz = None\n";
        let s = spans(t);
        let cs = at(t, &s, SynKind::Constant);
        for want in ["42", "0xFF", "3.14", "1_000", "True", "None"] {
            assert!(cs.contains(&want), "missing {want}: {cs:?}");
        }
    }

    #[test]
    fn def_and_class_names() {
        let t = "def frobnicate(x):\n    pass\nclass Widget:\n    pass\n";
        let s = spans(t);
        let ds = at(t, &s, SynKind::Definition);
        assert!(ds.contains(&"frobnicate"), "{ds:?}");
        assert!(ds.contains(&"Widget"), "{ds:?}");
        // The `def`/`class` keywords themselves stay plain.
        assert!(!has(&s, 0, 3, SynKind::Definition), "{s:?}");
    }

    #[test]
    fn plain_code_has_no_spans() {
        let t = "result = compute(a, b) + offset";
        assert!(spans(t).is_empty(), "{:?}", spans(t));
    }

    #[test]
    fn reference_snippet() {
        let t =
            "# add two\ndef add(a, b):\n    total = a + b  # sum\n    return total\nMAX = 100\n";
        let s = spans(t);
        assert_eq!(
            at(t, &s, SynKind::Comment),
            vec!["# add two", "# sum"],
            "{s:?}"
        );
        assert!(at(t, &s, SynKind::Definition).contains(&"add"), "{s:?}");
        assert!(at(t, &s, SynKind::Constant).contains(&"100"), "{s:?}");
    }
}
