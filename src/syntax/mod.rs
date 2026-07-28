//! Deterministic Alabaster-style syntax spans. Only four roles depart from base
//! ink; amber remains caret-only. See `docs/syntax.md` for policy.

use std::ops::Range;

pub mod bash;
pub mod c;
pub mod cpp;
pub mod csharp;
pub mod css;
pub mod go;
pub mod html;
pub mod java;
pub mod javascript;
pub mod json;
pub mod kotlin;
pub mod php;
pub mod python;
pub mod ruby;
pub mod rust;
mod scanner;
pub mod sql;
pub mod swift;
pub mod toml;
pub mod typescript;
pub mod yaml;

/// One highlighted ROLE. These are the ONLY four roles awl colors (Alabaster
/// philosophy); everything else in a code buffer stays the default ink.
/// `Comment` is TWO-TIERED (the tonsky split): the lexers emit only `Comment`,
/// and the central post-pass in [`spans`] reclassifies a comment whose body READS
/// AS CODE (a disabled statement) to [`SynKind::CommentCode`] — prose comments
/// stay `Comment` and render PROMINENT (full content ink + the comment wash;
/// comments are the prose in the code), while commented-out code recedes to the
/// muted grey it always had.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SynKind {
    /// PROSE-tier comments (explanations, TODOs, doc prose). Renders PROMINENT:
    /// full content ink + the per-world comment wash.
    Comment,
    /// COMMENTED-OUT CODE — a comment whose body reads as a disabled statement
    /// ([`looks_like_code`], default-to-prose). Recedes to the muted ink, no wash
    /// (today's grey exactly). Never emitted by a lexer; only the [`spans`]
    /// post-pass produces it.
    CommentCode,
    /// String + char literals (incl. raw / triple where the language has them).
    Str,
    /// Numbers, booleans, and `nil`/`null`/`None`-style literals.
    Constant,
    /// The NAME being defined — the identifier right after a `fn`/`def`/`class`/
    /// `struct`/`type`/… introducer (best-effort per language).
    Definition,
}

impl SynKind {
    /// Stable tag string for the capture sidecar's `syn_spans` block.
    pub fn tag(self) -> &'static str {
        match self {
            SynKind::Comment => "comment",
            SynKind::CommentCode => "comment_code",
            SynKind::Str => "string",
            SynKind::Constant => "constant",
            SynKind::Definition => "definition",
        }
    }
}

/// A recognized CODE language. Detected purely by file extension
/// ([`Lang::from_path`] / [`Lang::from_extension`]); a buffer whose extension is
/// not one of these (incl. `.env`, `.md`, `.txt`) has no `Lang` and is NOT
/// highlighted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lang {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Go,
    C,
    Cpp,
    Java,
    CSharp,
    Ruby,
    Php,
    Swift,
    Kotlin,
    Bash,
    Html,
    Css,
    Json,
    Yaml,
    Toml,
    Sql,
}

impl Lang {
    /// Map a file extension (WITHOUT the dot, any case) to a language, or `None`
    /// for an unrecognized / explicitly-excluded extension. This is the GATE: the
    /// excluded prose/work-file cases (`env`, `md`, `markdown`, `txt`) deliberately
    /// fall through to `None` so they render byte-identically to a plain buffer.
    pub fn from_extension(ext: &str) -> Option<Lang> {
        let e = ext.to_ascii_lowercase();
        Some(match e.as_str() {
            "rs" => Lang::Rust,
            "py" => Lang::Python,
            "js" | "mjs" | "cjs" | "jsx" => Lang::JavaScript,
            "ts" | "tsx" => Lang::TypeScript,
            "go" => Lang::Go,
            "c" | "h" => Lang::C,
            "cpp" | "cc" | "cxx" | "hpp" | "hh" => Lang::Cpp,
            "java" => Lang::Java,
            "cs" => Lang::CSharp,
            "rb" => Lang::Ruby,
            "php" => Lang::Php,
            "swift" => Lang::Swift,
            "kt" | "kts" => Lang::Kotlin,
            "sh" | "bash" | "zsh" => Lang::Bash,
            "html" | "htm" => Lang::Html,
            "css" => Lang::Css,
            "json" => Lang::Json,
            "yaml" | "yml" => Lang::Yaml,
            "toml" => Lang::Toml,
            "sql" => Lang::Sql,
            _ => return None,
        })
    }

    /// Detect the language of a file path by its extension. `None` for a path with
    /// no extension or an unrecognized one (the gate above). A `.env` file has the
    /// FILE NAME `.env` (no real extension), so it yields `None` here too.
    pub fn from_path(path: &std::path::Path) -> Option<Lang> {
        path.extension()
            .and_then(|e| e.to_str())
            .and_then(Lang::from_extension)
    }

    /// Map a language NAME (as written in a GitHub-style fenced code-block info
    /// string — `rust`, `bash`, `python`, `c++`, `golang`, …) to a language, or
    /// `None` for an unrecognized / absent one. This is the NAME half of the gate
    /// that [`Lang::from_extension`] is the extension half of: a fenced block tags
    /// its language by name, not by a file extension. Canonical names + the few
    /// common aliases match here; anything else falls back to [`Lang::from_extension`]
    /// so the extension-style tags (`rs`, `py`, `sh`, `kt`, `yml`, …) resolve too.
    /// Case-insensitive.
    pub fn from_name(name: &str) -> Option<Lang> {
        let n = name.trim().to_ascii_lowercase();
        let named = match n.as_str() {
            "rust" => Lang::Rust,
            "python" => Lang::Python,
            "javascript" | "node" => Lang::JavaScript,
            "typescript" => Lang::TypeScript,
            "go" | "golang" => Lang::Go,
            "c" => Lang::C,
            "cpp" | "c++" => Lang::Cpp,
            "java" => Lang::Java,
            "csharp" | "c#" => Lang::CSharp,
            "ruby" => Lang::Ruby,
            "php" => Lang::Php,
            "swift" => Lang::Swift,
            "kotlin" => Lang::Kotlin,
            "bash" | "shell" | "shellscript" | "console" => Lang::Bash,
            "html" | "xhtml" => Lang::Html,
            "css" => Lang::Css,
            "json" => Lang::Json,
            "yaml" => Lang::Yaml,
            "toml" => Lang::Toml,
            "sql" => Lang::Sql,
            // Not a known NAME — try the extension table (rs/py/sh/kt/yml/…) so the
            // extension-style fence tags resolve through the SAME shared table.
            _ => return Lang::from_extension(&n),
        };
        Some(named)
    }

    /// Map a fenced code-block INFO STRING (everything after the opening ```` ``` ````,
    /// e.g. `"rust"`, `"rust,ignore"`, `"sh title=…"`) to a language. Per GitHub, only
    /// the FIRST whitespace-delimited token names the language; the rest are
    /// attributes. An empty / attribute-only info string yields `None` (no syntax —
    /// the body stays plain mono `Code`). Delegates the token to [`Lang::from_name`].
    pub fn from_info(info: &str) -> Option<Lang> {
        let token = info.split_whitespace().next()?;
        // A `,`/`;`-separated attribute form (```` ```rust,ignore ````) tucks the lang
        // before the first separator too; split on either so both spellings resolve.
        let token = token.split([',', ';']).next().unwrap_or(token);
        Lang::from_name(token)
    }

    /// Stable lowercase name for this language, for the capture sidecar's
    /// `syn_lang` field (so a code capture reports the DETECTED language alongside
    /// the `syn_spans` it emitted, rather than leaving the language implicit). One
    /// canonical word per variant — `cpp`/`csharp` spell out the awkward
    /// extensions; the rest match their common name.
    pub fn name(self) -> &'static str {
        match self {
            Lang::Rust => "rust",
            Lang::Python => "python",
            Lang::JavaScript => "javascript",
            Lang::TypeScript => "typescript",
            Lang::Go => "go",
            Lang::C => "c",
            Lang::Cpp => "cpp",
            Lang::Java => "java",
            Lang::CSharp => "csharp",
            Lang::Ruby => "ruby",
            Lang::Php => "php",
            Lang::Swift => "swift",
            Lang::Kotlin => "kotlin",
            Lang::Bash => "bash",
            Lang::Html => "html",
            Lang::Css => "css",
            Lang::Json => "json",
            Lang::Yaml => "yaml",
            Lang::Toml => "toml",
            Lang::Sql => "sql",
        }
    }
}

/// What a lexer returns: styling spans in document byte coordinates.
pub type Spans = Vec<(Range<usize>, SynKind)>;

/// Which scanner a language's spans come from — the ONE place a language is
/// wired in. The match below carries no wildcard, so a new [`Lang`] variant fails
/// to compile until someone decides between the two.
enum Lexer {
    /// Data through the shared definition walk ([`scanner::scan`]) — the default,
    /// and what a new C-family-shaped language must use.
    Table(&'static scanner::LangSpec),
    /// A scanner of its own, because the language's shape is not the definition
    /// walk. Every member is listed with its reason in `BESPOKE`.
    Own(fn(&str) -> Spans),
}

fn lexer(lang: Lang) -> Lexer {
    match lang {
        Lang::Rust => Lexer::Table(&rust::SPEC),
        Lang::Python => Lexer::Table(&python::SPEC),
        Lang::JavaScript => Lexer::Table(&javascript::SPEC),
        Lang::TypeScript => Lexer::Table(&typescript::SPEC),
        Lang::Go => Lexer::Table(&go::SPEC),
        Lang::C => Lexer::Table(&c::SPEC),
        Lang::Cpp => Lexer::Table(&cpp::SPEC),
        Lang::Java => Lexer::Table(&java::SPEC),
        Lang::CSharp => Lexer::Table(&csharp::SPEC),
        Lang::Php => Lexer::Table(&php::SPEC),
        Lang::Swift => Lexer::Table(&swift::SPEC),
        Lang::Kotlin => Lexer::Table(&kotlin::SPEC),
        Lang::Ruby => Lexer::Own(ruby::spans),
        Lang::Bash => Lexer::Own(bash::spans),
        Lang::Html => Lexer::Own(html::spans),
        Lang::Css => Lexer::Own(css::spans),
        Lang::Json => Lexer::Own(json::spans),
        Lang::Yaml => Lexer::Own(yaml::spans),
        Lang::Toml => Lexer::Own(toml::spans),
        Lang::Sql => Lexer::Own(sql::spans),
    }
}

/// THE DISPATCH: parse `text` into syntax styling spans for `lang`, in DOCUMENT
/// byte coordinates. [`lexer`] picks the scanner; spans may be returned in any
/// order and may overlap; the renderer applies them in order (last-wins on
/// overlap), so a lexer that pushes a coarse span then a finer one inside it gets
/// the finer styling.
///
/// TWO-TIER COMMENT POST-PASS (the ONE owner of the split): the lexers keep
/// emitting plain [`SynKind::Comment`]; after the per-language lexer returns,
/// each comment span's body is judged by [`looks_like_code`] (on
/// [`comment_body`], markers stripped) and reclassified to
/// [`SynKind::CommentCode`] when it reads as a DISABLED STATEMENT rather than
/// prose. Central here — not per lexer — so all ~20 languages split identically,
/// and markdown FENCES inherit it for free (`markdown/` calls this same fn).
pub fn spans(lang: Lang, text: &str) -> Spans {
    let mut out = match lexer(lang) {
        Lexer::Table(spec) => scanner::scan(spec, text),
        Lexer::Own(f) => f(text),
    };
    for (r, k) in out.iter_mut() {
        if *k == SynKind::Comment
            && text
                .get(r.clone())
                .is_some_and(|t| looks_like_code(comment_body(t)))
        {
            *k = SynKind::CommentCode;
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Two-tier comment classification (tonsky: comments are the prose in the code)
//
// Prose comments are PROMINENT (full content ink + wash — see render/spans.rs);
// COMMENTED-OUT CODE stays the muted grey. The split is a LIGHT heuristic,
// deliberately simple + unit-tested; when unsure, a comment is PROSE.
// ---------------------------------------------------------------------------

/// The language-agnostic keyword table for [`looks_like_code`] rule (2): a
/// comment whose body STARTS with one of these (and carries a code-shaped
/// symbol) reads as a disabled statement. One shared table — never per-language.
const CODE_LEAD_KEYWORDS: &[&str] = &[
    "let", "fn", "return", "if", "for", "while", "const", "var", "import", "use", "pub", "def",
    "class", "struct", "impl", "match", "function", "echo", "foreach", "select", "insert",
    "update", "delete", "print", "elif",
];

/// Strip the LEADING comment markers (`//`+ incl. `///`/`//!`, `/*`, `#`+,
/// `--`+, `<!--`, a block-comment interior line's leading `*`) and the TRAILING
/// closers (`*/`, `-->`) off a comment span's text, then trim. What remains is
/// the comment's BODY, the text [`looks_like_code`] judges. Pure, zero-copy.
pub fn comment_body(text: &str) -> &str {
    let mut s = text.trim_start();
    if let Some(rest) = s.strip_prefix("<!--") {
        s = rest;
    } else if let Some(rest) = s.strip_prefix("/*") {
        s = rest.trim_start_matches('*'); // `/**` doc openers
    } else if s.starts_with("//") {
        s = s.trim_start_matches('/');
        s = s.strip_prefix('!').unwrap_or(s); // `//!` inner doc marker
    } else if s.starts_with("--") {
        s = s.trim_start_matches('-');
    } else if s.starts_with('#') {
        s = s.trim_start_matches('#');
    } else if s.starts_with('*') {
        // A block-comment INTERIOR continuation line (` * like this`).
        s = s.trim_start_matches('*');
    }
    let s = s.trim_end();
    let s = s.strip_suffix("*/").unwrap_or(s);
    let s = s.strip_suffix("-->").unwrap_or(s);
    s.trim()
}

/// Does a comment BODY (markers already stripped via [`comment_body`]) read as
/// COMMENTED-OUT CODE rather than prose? CODE iff ANY of:
///
/// 1. the trimmed body ends with `;`, `{`, or `}` (a disabled statement);
/// 2. the FIRST WORD is in the shared [`CODE_LEAD_KEYWORDS`] table AND the body
///    carries at least one code-shaped symbol (`=(){}[]<>*` — `*` included so
///    `select * from users` reads as SQL, while a keyword alone — "return early
///    here" — stays prose);
/// 3. symbol density ≥ 0.30 over a body of length ≥ 8, where symbols are
///    `(){}[];=<>+*/&|` — deliberately EXCLUDING `.` `,` `'` `?` `!` `:` so
///    prose punctuation never trips it.
///
/// DEFAULT-TO-PROSE: an empty body, a short body, and anything not matching a
/// rule is prose (prominent). A MULTI-LINE body (block comments) is judged
/// per-line (each line re-stripped for `*` continuations): it is code only when
/// EVERY non-empty line reads as code — when mixed, prose wins.
pub fn looks_like_code(body: &str) -> bool {
    let body = body.trim();
    if body.is_empty() {
        return false;
    }
    if body.contains('\n') {
        // Block comment spanning lines: judged whole-span, prose wins on a mix.
        let mut any = false;
        for line in body.lines() {
            let l = comment_body(line);
            if l.is_empty() {
                continue;
            }
            if !looks_like_code_line(l) {
                return false;
            }
            any = true;
        }
        return any;
    }
    looks_like_code_line(body)
}

/// One line of [`looks_like_code`] — the three rules over a single-line body.
fn looks_like_code_line(l: &str) -> bool {
    let l = l.trim();
    if l.is_empty() {
        return false;
    }
    // (1) Statement punctuation: a trailing `;` / `{` / `}` is code, not prose.
    if l.ends_with(';') || l.ends_with('{') || l.ends_with('}') {
        return true;
    }
    // (2) A leading code keyword + a code-shaped symbol. The first WORD is the
    // leading identifier run (so `print(x)` yields `print`), matched
    // case-SENSITIVELY against the lowercase table — capitalized prose ("If you
    // set x = 3...") never trips it (default-to-prose).
    let word_end = l
        .char_indices()
        .find(|(_, c)| !(c.is_ascii_alphanumeric() || *c == '_'))
        .map(|(i, _)| i)
        .unwrap_or(l.len());
    let first = &l[..word_end];
    if CODE_LEAD_KEYWORDS.contains(&first) && l.chars().any(|c| "=(){}[]<>*".contains(c)) {
        return true;
    }
    // (3) Symbol density over a long-enough body.
    let n = l.chars().count();
    if n >= 8 {
        let sym = l.chars().filter(|c| "(){}[];=<>+*/&|".contains(*c)).count();
        if sym as f32 / n as f32 >= 0.30 {
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Shared lexer primitives
//
// The hand-written scanners below were copy-pasted across nearly every
// `<lang>.rs`; they are gathered here so a lexer just `use`s or calls them
// instead of carrying its own identical copy. They add NO behavior — each is the
// exact body the lexers already shipped, lifted verbatim and (where languages
// differed only in a constant) parameterized. A lexer with a genuine quirk
// (extra identifier chars, a hex-float exponent, a digit separator, …) keeps its
// own local version and does not reach for these.
// ---------------------------------------------------------------------------

/// True if the byte `c` can START an identifier in the common case: ASCII letter
/// or `_`. The lexers whose identifier rule is exactly this `use
/// super::is_ident_start`; the few with extra chars (`$` in JS/TS/Java, `-` in
/// CSS, its own non-ASCII rule in PHP) keep a local override instead.
///
/// UNICODE (pragmatic, no `unicode-xid` crate): any NON-ASCII UTF-8 byte
/// (`>= 0x80`, the lead byte of a multi-byte scalar) also starts an identifier,
/// so Unicode (XID-ish) names — `café`, `Δx`, `naïve` — are highlighted as one
/// token instead of under-lexed. Because these helpers drive a byte scan
/// (`while is_ident_continue(b[i])`), treating every byte of a multi-byte scalar
/// as identifier-continue makes the scan swallow the whole scalar, so the word
/// slice always lands on a char boundary. ASCII fast-path first.
pub(super) fn is_ident_start(c: u8) -> bool {
    c == b'_' || c.is_ascii_alphabetic() || c >= 0x80
}

/// True if the byte `c` can CONTINUE an identifier in the common case: ASCII
/// alphanumeric or `_`. Companion to [`is_ident_start`]; same sharing + Unicode
/// (`>= 0x80`) rule.
pub(super) fn is_ident_continue(c: u8) -> bool {
    c == b'_' || c.is_ascii_alphanumeric() || c >= 0x80
}

/// The JS-family identifier-START rule: the common case plus `$`. Shared by the
/// JavaScript / TypeScript / Java lexers, which all admit `$` in identifiers
/// (`use super::is_ident_start_dollar as is_ident_start`). CSS (`-`) and PHP
/// (its own non-ASCII rule) keep their own distinct overrides. Same Unicode
/// (`>= 0x80`) broadening as [`is_ident_start`].
pub(super) fn is_ident_start_dollar(c: u8) -> bool {
    c == b'_' || c == b'$' || c.is_ascii_alphabetic() || c >= 0x80
}

/// The JS-family identifier-CONTINUE rule: the common case plus `$`. Companion to
/// [`is_ident_start_dollar`]; same sharing + Unicode (`>= 0x80`) rule.
pub(super) fn is_ident_continue_dollar(c: u8) -> bool {
    c == b'_' || c == b'$' || c.is_ascii_alphanumeric() || c >= 0x80
}

/// Case-insensitive membership test: true if `word` equals any entry of `list`
/// ignoring ASCII case. Shared by the case-insensitive-keyword lexers (PHP, SQL),
/// which match their keyword tables regardless of source casing.
pub(super) fn matches_word_ci(list: &[&str], word: &str) -> bool {
    list.iter().any(|k| word.eq_ignore_ascii_case(k))
}

/// Classify an identifier `word` under the "definition-introducer" model nearly
/// every C-family lexer wrote by hand: a keyword in `def_kws`
/// (`fn`/`class`/`struct`/`def`/…) introduces a NAME, so it ARMS `expect_def` and
/// styles nothing itself ([`SynKind::Definition`] is reserved for the name); the
/// next identifier — reached here with `expect_def` already set — IS that
/// definition; a word in `const_words` (`true`/`false`/`null`/`None`-style) is a
/// [`SynKind::Constant`]; everything else stays the default ink (`None`).
///
/// `expect_def` threads through by `&mut`: it is CONSUMED (cleared) when a name is
/// emitted and SET when an introducer is seen. The precedence is pending-name
/// FIRST, then constant, then introducer — the order the converted lexers
/// (`c`/`java`/`csharp`/`go`/`javascript`/`typescript`/`kotlin`/`swift`/`rust`/
/// `python`) already shared. Lexers with a genuine quirk keep their own arm:
/// `cpp` checks the introducer FIRST so `enum class Name` chains past the inner
/// `class` to `Name`; `php`/`sql` match their keyword tables case-INsensitively.
/// `go` calls this AND separately notes whether the introducer was `func`.
pub(super) fn ident_role(
    word: &str,
    def_kws: &[&str],
    const_words: &[&str],
    expect_def: &mut bool,
) -> Option<SynKind> {
    if *expect_def {
        *expect_def = false;
        Some(SynKind::Definition)
    } else if const_words.contains(&word) {
        Some(SynKind::Constant)
    } else if def_kws.contains(&word) {
        *expect_def = true;
        None
    } else {
        None
    }
}

/// Scan a `//`-style (or `--`, `#`, …) LINE comment whose body runs to the end of
/// the line; return the index of the terminating `\n` (or EOF). The caller has
/// already matched the comment marker and passes `start` at the marker's first
/// byte, so the returned span is `start..scan_line_comment(b, start)`. This is the
/// shared body behind the `while i < n && b[i] != b'\n'` loop every lexer carried.
pub(super) fn scan_line_comment(b: &[u8], start: usize) -> usize {
    let n = b.len();
    let mut i = start;
    while i < n && b[i] != b'\n' {
        i += 1;
    }
    i
}

/// Scan a `/* … */` BLOCK comment starting at the opening `/` (`b[start]` is `/`
/// and `b[start + 1]` is `*`, already matched by the caller); return the index
/// just past the closing `*/` (or EOF if unterminated). When `nest` is set the
/// scanner tracks depth so an inner `/*` must be matched by its own `*/` (Rust /
/// Swift / Kotlin); otherwise the FIRST `*/` closes (C-family, Go, SQL, …). This
/// is the shared body behind the two copy-pasted block-comment loops.
pub(super) fn scan_block_comment(b: &[u8], start: usize, nest: bool) -> usize {
    let n = b.len();
    let mut i = start + 2;
    if nest {
        let mut depth = 1u32;
        while i < n && depth > 0 {
            if b[i] == b'/' && i + 1 < n && b[i + 1] == b'*' {
                depth += 1;
                i += 2;
            } else if b[i] == b'*' && i + 1 < n && b[i + 1] == b'/' {
                depth -= 1;
                i += 2;
            } else {
                i += 1;
            }
        }
    } else {
        while i < n {
            if b[i] == b'*' && i + 1 < n && b[i + 1] == b'/' {
                i += 2;
                break;
            }
            i += 1;
        }
    }
    i
}

/// Scan a quoted literal starting at the opening quote `open`; return the index
/// just past the closing `quote` byte (or, when `stop_at_newline`, the newline
/// that terminates an unclosed single-line literal — or EOF). A `\\` escapes the
/// next byte so an escaped quote does not close the literal. This is the shared
/// body behind ~15 per-language `scan_string` scanners; the caller supplies the
/// `quote` byte (a fixed `"` for most, `b[open]` for the languages that also
/// accept `'`) and whether a newline ends an unterminated literal.
pub(super) fn scan_quoted(b: &[u8], open: usize, quote: u8, stop_at_newline: bool) -> usize {
    let n = b.len();
    let mut i = open + 1;
    while i < n {
        match b[i] {
            b'\\' => i += 2,
            b'\n' if stop_at_newline => return i,
            c if c == quote => return i + 1,
            _ => i += 1,
        }
    }
    n
}

/// Per-language knobs for the shared [`scan_number`] — the small set of constants
/// the otherwise-identical numeric scanners varied by.
#[derive(Clone, Copy)]
pub(super) struct NumOpts {
    /// Letters that, right after a leading `0`, open a radix-prefixed integer
    /// (e.g. `b"xXoObB"` for hex/octal/binary, `b"xXbB"` where there is no `0o`).
    pub radix: &'static [u8],
    /// Extra bytes allowed inside a radix body beyond ASCII alphanumerics + `_`
    /// (Go alone allows a `.` for its hex floats; most pass `b""`).
    pub radix_extra: &'static [u8],
    /// Whether a `..` after the integer part stops the scan (the range op): when
    /// set, a `.` followed by another `.` ends the number; otherwise only a `.`
    /// before an identifier start does.
    pub dot_dot_stops: bool,
}

/// Scan a numeric literal beginning at the digit `i`; return the index just past
/// it. The shared body behind the "decimal + radix-prefix" family of scanners:
/// a `0x`/`0o`/`0b` (per `opts.radix`) integer consumes alphanumerics, `_`, and
/// `opts.radix_extra`; otherwise the decimal loop consumes alphanumerics + `_`
/// and a fractional `.` unless that `.` opens a range (`opts.dot_dot_stops`) or a
/// member access (a `.` before `is_ident_start`). The `is_ident_start` predicate
/// is passed in so a language with extra identifier chars (`$`, non-ASCII) keeps
/// its exact member-access boundary. Languages with a hex-float exponent or a
/// digit separator keep their own local scanner.
pub(super) fn scan_number(
    b: &[u8],
    i: usize,
    opts: NumOpts,
    is_ident_start: fn(u8) -> bool,
) -> usize {
    let n = b.len();
    let mut j = i + 1;
    if b[i] == b'0' && j < n && opts.radix.contains(&b[j]) {
        j += 1;
        while j < n
            && (b[j].is_ascii_alphanumeric() || b[j] == b'_' || opts.radix_extra.contains(&b[j]))
        {
            j += 1;
        }
        return j;
    }
    while j < n {
        let c = b[j];
        if c.is_ascii_alphanumeric() || c == b'_' {
            j += 1;
        } else if c == b'.' {
            if j + 1 < n && ((opts.dot_dot_stops && b[j + 1] == b'.') || is_ident_start(b[j + 1])) {
                break;
            }
            j += 1;
        } else {
            break;
        }
    }
    j
}

/// Shared assertion helpers for the per-language lexer tests. Every `<lang>.rs`
/// test module reaches for the same two: [`has`] (an exact `start..end` span of a
/// role exists) and [`at`] (the substrings a role covers, for readable failures).
/// Colocated here so each lexer just `use crate::syntax::testutil::{has, at};`
/// instead of copy-pasting the pair. Test-only — no runtime impact.
#[cfg(test)]
pub(crate) mod testutil {
    use super::{Range, SynKind};

    pub(crate) fn has(s: &[(Range<usize>, SynKind)], lo: usize, hi: usize, k: SynKind) -> bool {
        s.iter()
            .any(|(r, kk)| r.start == lo && r.end == hi && *kk == k)
    }

    pub(crate) fn at<'a>(text: &'a str, s: &[(Range<usize>, SynKind)], k: SynKind) -> Vec<&'a str> {
        s.iter()
            .filter(|(_, kk)| *kk == k)
            .map(|(r, _)| &text[r.clone()])
            .collect()
    }
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod verifier_probe {
    use super::*;

    #[test]
    fn verifier_adversarial_two_tier() {
        let cases: &[(&str, bool)] = &[
            ("// TODO: fix the wrap", false),
            ("// let x = foo(bar);", true),
            ("// return early here", false),
            ("# print(x)", true),
            ("// x += 1;", true),
            ("-- select * from users", true),
            ("// use two spaces here", false),
            ("// wraps at 53 -> 2 rows", false),
            ("// e.g. `foo();` disables the cache entirely", false),
            ("// If you set x = 3, the cache invalidates", false),
            ("// the answer is 42 (see DESIGN.md, section 3)", false),
            ("// a + b, then c - d: simple arithmetic in prose", false),
            ("// O(visible) not O(doc) per frame", false),
            (
                "/// Returns the width in px (physical, not logical).",
                false,
            ),
            ("// -----------------------------------------", false),
            ("// return None;", true),
            ("//     let mut out = Vec::new();", true),
            ("// if x > 3 { bail() }", true),
            ("// fn old_hook() -> bool {", true),
            ("# import os", false),
            ("// foo(bar, baz);", true),
            ("/* let x = 1;\n   but this line is prose */", false),
            ("/* let x = 1;\n * let y = 2; */", true),
            ("//", false),
            ("/* */", false),
        ];
        for (c, want) in cases {
            assert_eq!(
                looks_like_code(comment_body(c)),
                *want,
                "two-tier misclassified: {c:?} (want code={want})"
            );
        }
    }

    #[test]
    fn verifier_keyword_plus_backtick_edges() {
        let greyed: &[&str] = &[
            "// use `mono_safe_weight()` to dodge the trap",
            "// if the cache is stale, rebuild() it",
            "// match the surrounding style (table-driven)",
            "// for details see resolve_cjk() and its weight trap",
        ];
        for c in greyed {
            assert!(
                looks_like_code(comment_body(c)),
                "expectation drifted: {c:?} currently classifies CODE (greyed)"
            );
        }
        assert!(looks_like_code(comment_body(
            "// ============================"
        )));
    }

    #[test]
    fn verifier_corpus_report() {
        let mut flagged: Vec<String> = Vec::new();
        let mut total = 0usize;
        for f in [
            "src/render/spans.rs",
            "src/render/rects.rs",
            "src/spell.rs",
            "src/theme/model.rs",
            "src/app.rs",
            "src/markdown/spans.rs",
        ] {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(f);
            let text = std::fs::read_to_string(&path).unwrap();
            for line in text.lines() {
                let t = line.trim_start();
                if !t.starts_with("//") {
                    continue;
                }
                total += 1;
                if looks_like_code(comment_body(t)) {
                    flagged.push(format!("{f}: {t}"));
                }
            }
        }
        println!(
            "corpus: {total} comment lines, {} classified code",
            flagged.len()
        );
        for f in &flagged {
            println!("  CODE: {f}");
        }
        assert!(
            (flagged.len() as f32) / (total as f32) < 0.05,
            "heuristic greys {}/{total} of this repo's own prose comments",
            flagged.len()
        );
    }
}
