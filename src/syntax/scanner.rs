//! The ONE definition-walk scanner, and the [`LangSpec`] table that drives it.
//!
//! Twelve lexers (`c`, `cpp`, `csharp`, `go`, `java`, `javascript`, `kotlin`,
//! `php`, `python`, `rust`, `swift`, `typescript`) shipped the same loop —
//! comment, string, number, identifier, drop-the-pending-name — differing only in
//! constants. That loop lives here; each of those files is now a `LangSpec` plus
//! the one thing it genuinely owns, its [`LangSpec::string_at`] hook. The other
//! eight lexers scan on a different shape (Ruby's heredoc state, SQL's skip-words,
//! HTML's tags, …) and stay their own; `syntax::lexer` names them.
//!
//! Adding a language means adding a `LangSpec`, not a loop — `syntax::mod`'s
//! `no_lexer_module_writes_its_own_definition_walk` law enforces that.

use super::{NumOpts, SynKind};
use std::ops::Range;

/// What opens a line comment.
pub(super) enum LineComment {
    /// `//` — the C family.
    Slashes,
    /// `#` — Python.
    Hash,
    /// `//` or `#`, except `#[`, which is a PHP 8 attribute rather than a comment.
    SlashesOrHashNotAttr,
}

/// Whether the language has a `/* … */` form, and whether it nests.
pub(super) enum BlockComment {
    /// No block-comment form at all (Python).
    None,
    /// The FIRST `*/` closes (C, C++, C#, Java, JS, TS, Go, PHP).
    Flat,
    /// An inner `/*` must be matched by its own `*/` (Rust, Swift, Kotlin).
    Nested,
}

/// How an identifier is judged against the language's two word tables.
pub(super) enum WordRule {
    /// Pending name, then constant, then introducer — [`super::ident_role`].
    Standard,
    /// Constant, then introducer, then pending name, so `enum class Name` chains
    /// past the inner `class` to `Name` (C++ only).
    IntroducerFirst,
    /// [`WordRule::Standard`] with case-INsensitive table matching (PHP only).
    CaseInsensitive,
}

/// How a numeric literal is scanned.
#[derive(Clone, Copy)]
pub(super) enum Number {
    /// The shared [`super::scan_number`] under these knobs.
    Shared(NumOpts),
    /// A scanner of its own: C++'s `'` digit separators, Swift's hex floats.
    Own(fn(&[u8], usize) -> usize),
}

/// Everything a definition-walk language varies by. Data only — the behavior is
/// [`scan`].
pub(super) struct LangSpec {
    pub line: LineComment,
    pub block: BlockComment,
    /// Where a string/char/template/raw literal opening at `i` ends, or `None` if
    /// none opens there. The genuinely per-language half: encoding prefixes, raw
    /// delimiters, heredocs, text blocks, Rust's char-vs-lifetime call.
    pub string_at: fn(&[u8], usize) -> Option<usize>,
    pub number: Number,
    pub ident_start: fn(u8) -> bool,
    pub ident_continue: fn(u8) -> bool,
    /// Introducers after which the next identifier is the DEFINITION name.
    pub def_kws: &'static [&'static str],
    /// Identifiers that are [`SynKind::Constant`] literals.
    pub const_words: &'static [&'static str],
    pub words: WordRule,
    /// Non-whitespace bytes that do NOT cancel a pending definition name: the `*`
    /// of `function* gen` (JS), the `'` that opens a Rust lifetime.
    pub def_survives: &'static [u8],
    /// The introducer whose name may sit behind a parenthesized receiver —
    /// `func (r *T) Name()` (Go).
    pub receiver_kw: Option<&'static str>,
}

/// Parse `text` into Alabaster spans under `spec`. Single pass, pure, spans in
/// document byte order; the byte at every span boundary is ASCII, so multibyte
/// UTF-8 rides inside a span without ever splitting a char.
pub(super) fn scan(spec: &LangSpec, text: &str) -> Vec<(Range<usize>, SynKind)> {
    let b = text.as_bytes();
    let n = b.len();
    let mut out: Vec<(Range<usize>, SynKind)> = Vec::new();
    let mut i = 0usize;
    // Set when the previous significant token was a def introducer; the next
    // identifier is then the defined NAME.
    let mut expect_def = false;
    // Whether the introducer that armed `expect_def` was `spec.receiver_kw`.
    let mut after_receiver_kw = false;

    while i < n {
        let c = b[i];

        if opens_line_comment(spec, b, i) {
            let end = super::scan_line_comment(b, i);
            out.push((i..end, SynKind::Comment));
            i = end;
            continue;
        }

        if c == b'/' && i + 1 < n && b[i + 1] == b'*' {
            let nest = match spec.block {
                BlockComment::None => None,
                BlockComment::Flat => Some(false),
                BlockComment::Nested => Some(true),
            };
            if let Some(nest) = nest {
                let end = super::scan_block_comment(b, i, nest);
                out.push((i..end, SynKind::Comment));
                i = end;
                continue;
            }
        }

        if let Some(end) = (spec.string_at)(b, i) {
            out.push((i..end, SynKind::Str));
            i = end;
            expect_def = false;
            continue;
        }

        if c.is_ascii_digit() {
            let start = i;
            i = match spec.number {
                Number::Shared(opts) => super::scan_number(b, i, opts, spec.ident_start),
                Number::Own(f) => f(b, i),
            };
            out.push((start..i, SynKind::Constant));
            expect_def = false;
            continue;
        }

        if (spec.ident_start)(c) {
            let start = i;
            i += 1;
            while i < n && (spec.ident_continue)(b[i]) {
                i += 1;
            }
            let word = &text[start..i];
            let was_expecting = expect_def;
            if let Some(kind) = word_role(spec, word, &mut expect_def) {
                out.push((start..i, kind));
            } else if !was_expecting && expect_def {
                after_receiver_kw = spec.receiver_kw == Some(word);
            }
            continue;
        }

        // A receiver group `(r *T)` sits between the introducer and the NAME; step
        // over it with the expectation intact.
        if c == b'(' && expect_def && after_receiver_kw {
            i = skip_parens(b, i);
            continue;
        }

        // Any other byte (operator, punctuation, whitespace) stays default ink.
        // Whitespace between an introducer and its name must not clear the
        // expectation; other tokens mean the name never materialized.
        if !c.is_ascii_whitespace() && !spec.def_survives.contains(&c) {
            expect_def = false;
        }
        i += 1;
    }

    out
}

fn opens_line_comment(spec: &LangSpec, b: &[u8], i: usize) -> bool {
    let n = b.len();
    let slashes = b[i] == b'/' && i + 1 < n && b[i + 1] == b'/';
    match spec.line {
        LineComment::Slashes => slashes,
        LineComment::Hash => b[i] == b'#',
        LineComment::SlashesOrHashNotAttr => {
            slashes || (b[i] == b'#' && !(i + 1 < n && b[i + 1] == b'['))
        }
    }
}

/// The role of one identifier under `spec.words`, threading `expect_def`.
fn word_role(spec: &LangSpec, word: &str, expect_def: &mut bool) -> Option<SynKind> {
    match spec.words {
        WordRule::Standard => super::ident_role(word, spec.def_kws, spec.const_words, expect_def),
        WordRule::IntroducerFirst => {
            if spec.const_words.contains(&word) {
                *expect_def = false;
                Some(SynKind::Constant)
            } else if spec.def_kws.contains(&word) {
                *expect_def = true;
                None
            } else if *expect_def {
                *expect_def = false;
                Some(SynKind::Definition)
            } else {
                None
            }
        }
        WordRule::CaseInsensitive => {
            if *expect_def {
                *expect_def = false;
                Some(SynKind::Definition)
            } else if super::matches_word_ci(spec.const_words, word) {
                Some(SynKind::Constant)
            } else if super::matches_word_ci(spec.def_kws, word) {
                *expect_def = true;
                None
            } else {
                None
            }
        }
    }
}

/// Skip a balanced `(…)` group opening at `i`; returns the index just past the
/// matching close paren (or EOF).
fn skip_parens(b: &[u8], i: usize) -> usize {
    let n = b.len();
    let mut j = i + 1;
    let mut depth = 1u32;
    while j < n && depth > 0 {
        match b[j] {
            b'(' => depth += 1,
            b')' => depth -= 1,
            _ => {}
        }
        j += 1;
    }
    j
}
