//! **"SEARCH IN FOLDER…"'S MATCHER + GROUPING SEAM.** PHILOSOPHY.md §1 promises
//! "the simple file operations, navigation, search, and version history needed
//! to sustain writing," and Cmd-F/Cmd-R stop at the buffer -- this is the
//! full-text search over the whole active folder that answers "where did I
//! write about X". Pure logic, no filesystem and no GPU: the caller loads a
//! bounded CORPUS (`(path, content)` pairs, already read once at summon —
//! [`crate::overlay::OverlayState::new_search_folder`]) and this module turns
//! a typed query into ranked, grouped, snippeted [`Hit`]s on every keystroke
//! (`OverlayState::refilter`'s `SearchFolder` branch), never touching disk
//! itself so both the corpus load and the re-match stay independently
//! testable and the match stays off the render path entirely.
//!
//! **Case folding / Unicode.** Matching rides [`crate::search::find_all`] —
//! the SAME literal-substring matcher Cmd-F's in-buffer isearch already uses,
//! called with `case_sensitive: false` (isearch's own default). That gives
//! Unicode-aware case folding for free (`char::to_lowercase`, with an ASCII
//! fast path and an explicit Kelvin-sign exemption already proven there) —
//! "same behavior ⇒ same code" rather than a second, competing case-fold
//! policy for the one other place this codebase does literal text search.
//! [`tests::case_insensitive_unicode_query_matches_via_the_shared_matcher`]
//! records the decision.

/// One matching LINE inside one file: enough to land the caret
/// ([`Self::line`]/[`Self::col`], CHAR-indexed exactly like
/// [`crate::search::Match`]) and enough to draw the row (a bounded
/// [`Self::snippet`] with the match's own BYTE range inside it,
/// [`Self::hl_start`]/[`Self::hl_end`] — the figure/ground split
/// `render/chrome/overlay_shape` draws in content ink against a muted lead,
/// DESIGN.md's own figure/ground-by-value law applied to a search result
/// rather than a bespoke highlight color).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hit {
    pub path: String,
    /// Zero-based, matching `RowMeta::GotoHeading`/`GotoLine` and
    /// `Effect::JumpToLine`'s own convention.
    pub line: usize,
    /// The match's CHAR column within the original line — `line_col_to_char`'s
    /// unit, so the caret lands exactly on the match, not just the line start.
    pub col: usize,
    pub snippet: String,
    pub hl_start: usize,
    pub hl_end: usize,
}

/// The scan/result BUDGET — enforced, not aspirational: a folder larger than
/// this never hangs the picker, it just stops finding more. All five numbers
/// are named here so the tradeoff is one place to retune.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchBudget {
    /// Distinct files whose content the CALLER will load into the corpus
    /// (enforced by [`load_corpus`], not by [`search`] — the corpus handed to
    /// `search` is already the loaded, bounded set).
    pub max_files: usize,
    /// Cumulative bytes [`load_corpus`] will read across every file before it
    /// stops loading MORE files (a large folder is bounded by total work, not
    /// only by file count).
    pub max_total_bytes: usize,
    /// A single file over this size is skipped by [`load_corpus`] outright
    /// (an accidentally-included log/binary never dominates the budget).
    pub max_file_bytes: usize,
    /// Total hit ROWS [`search`] returns across the whole corpus.
    pub max_hits: usize,
    /// Hit rows per FILE — keeps one file with the query on every line from
    /// crowding out every other file's matches.
    pub max_hits_per_file: usize,
    /// The drawn snippet's width, in chars, centered on the match.
    pub snippet_chars: usize,
}

impl Default for SearchBudget {
    fn default() -> Self {
        Self {
            max_files: 300,
            max_total_bytes: 20_000_000,
            max_file_bytes: 1_000_000,
            max_hits: 200,
            max_hits_per_file: 20,
            snippet_chars: 80,
        }
    }
}

/// Load a bounded CORPUS of `(root-relative path, content)` pairs from
/// `files` (already gitignore-aware — `index::build_index`'s own roster),
/// via caller-supplied `read` (production: `crate::fs::active().read_to_string`;
/// tests: an in-memory map, no filesystem at all). Stops loading once
/// `budget.max_files` or `budget.max_total_bytes` is reached; a single file
/// over `budget.max_file_bytes`, or one `read` can't decode (binary, gone,
/// permission-denied), is skipped rather than aborting the whole load.
///
/// This is where the FILE READ happens, once, at summon — never inside
/// [`search`], which is pure and re-runs on every keystroke against the
/// corpus this returns.
pub fn load_corpus(
    files: &[String],
    budget: &SearchBudget,
    mut read: impl FnMut(&str) -> Option<String>,
) -> Vec<(String, String)> {
    let mut corpus = Vec::new();
    let mut total_bytes = 0usize;
    for path in files {
        if corpus.len() >= budget.max_files || total_bytes >= budget.max_total_bytes {
            break;
        }
        let Some(content) = read(path) else { continue };
        if content.len() > budget.max_file_bytes {
            continue;
        }
        total_bytes += content.len();
        corpus.push((path.clone(), content));
    }
    corpus
}

/// Match `query` (empty query -> no scan, no results) against every line of
/// every `(path, content)` pair in `corpus`, IN CORPUS ORDER — so hits for one
/// file arrive contiguously, which is what lets a flat row list still read as
/// "grouped by file" (`OverlayState::rebuild_search_rows` repeats the group's
/// path in each row's secondary column; consecutive rows sharing it is the
/// group). Bounded by `budget.max_hits`/`max_hits_per_file`; stops scanning
/// entirely once the total is reached, so a huge folder never over-runs the
/// budget even by one row.
pub fn search(corpus: &[(String, String)], query: &str, budget: &SearchBudget) -> Vec<Hit> {
    if query.is_empty() {
        return Vec::new();
    }
    let mut hits = Vec::new();
    'files: for (path, content) in corpus {
        let mut in_file = 0usize;
        for (line_idx, line) in content.split('\n').enumerate() {
            for m in crate::search::find_all(line, query, false) {
                let (snippet, hl_start, hl_end) =
                    build_snippet(line, m.start, m.end, budget.snippet_chars);
                hits.push(Hit {
                    path: path.clone(),
                    line: line_idx,
                    col: m.start,
                    snippet,
                    hl_start,
                    hl_end,
                });
                in_file += 1;
                if hits.len() >= budget.max_hits {
                    break 'files;
                }
                if in_file >= budget.max_hits_per_file {
                    continue 'files;
                }
            }
        }
    }
    hits
}

/// Window `line` down to at most `max_chars`, CENTERED on the match
/// `[char_start, char_end)` (CHAR indices, [`crate::search::find_all`]'s own
/// unit) so a long line's match is NEVER the part an ellipsis eats — unlike
/// `rowlayout::fit_primary`'s generic trailing elision, this owns its own
/// window because it must keep the match, never just the head. Returns the
/// windowed text plus the match's own BYTE range within it (`str` slicing is
/// byte-indexed; the caller draws `[..hl_start]` muted, `[hl_start..hl_end]`
/// in content ink, mirroring `render/chrome/overlay_shape::push_overlay_name_rows`'s
/// existing directory/filename split — the same figure/ground mechanism, a
/// different split point).
///
/// A match wider than `max_chars` itself is shown whole, unwindowed (the
/// budget bounds ROWS and FILES; it never truncates the very text the row
/// exists to show).
fn build_snippet(
    line: &str,
    char_start: usize,
    char_end: usize,
    max_chars: usize,
) -> (String, usize, usize) {
    let chars: Vec<char> = line.chars().collect();
    let match_len = char_end.saturating_sub(char_start);
    if chars.len() <= max_chars || match_len >= max_chars {
        let hl_start = chars[..char_start].iter().collect::<String>().len();
        let hl_end = chars[..char_end.min(chars.len())]
            .iter()
            .collect::<String>()
            .len();
        return (line.to_string(), hl_start, hl_end);
    }
    let context = max_chars - match_len;
    let before = context / 2;
    let after = context - before;
    let mut win_start = char_start.saturating_sub(before);
    let mut win_end = (char_end + after).min(chars.len());
    // Slack from clamping one edge is handed to the other, so a match near
    // either end of the line still fills the window instead of showing less
    // than `max_chars` when more text is available on the far side.
    if win_start == 0 {
        win_end = (win_start + max_chars).min(chars.len());
    }
    if win_end == chars.len() {
        win_start = win_end.saturating_sub(max_chars);
    }
    let lead_ellipsis = win_start > 0;
    let tail_ellipsis = win_end < chars.len();
    let mut snippet = String::new();
    if lead_ellipsis {
        snippet.push('\u{2026}');
    }
    let hl_start = snippet.len()
        + chars[win_start..char_start]
            .iter()
            .collect::<String>()
            .len();
    let hl_end = snippet.len()
        + chars[win_start..char_end.min(chars.len())]
            .iter()
            .collect::<String>()
            .len();
    snippet.push_str(&chars[win_start..win_end].iter().collect::<String>());
    if tail_ellipsis {
        snippet.push('\u{2026}');
    }
    (snippet, hl_start, hl_end)
}

#[cfg(test)]
mod tests;
