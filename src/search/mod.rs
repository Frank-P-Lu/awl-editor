pub mod keys;
mod semantic;
use crate::textbox::TextBox;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Match {
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
    Forward,
    Backward,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StepOutcome {
    Moved,
    RecoiledAtBoundary(Direction),
    Wrapped,
    NoMatches,
}

/// Live isearch state. Owned by `App` as `Option<SearchState>`; the query is its
/// OWN String, never spliced into the rope.
pub struct SearchState {
    /// The search needle + its CHAR-index caret, one shared [`TextBox`].
    query: TextBox,
    case_sensitive: bool,
    matches: Vec<Match>,
    current: Option<usize>,
    direction: Direction,
    origin: usize,
    /// REPLACE mode: once revealed, the SAME panel hosts a second (replacement)
    /// field. `false` keeps the plain isearch panel; the buffer is untouched until
    /// a replace fires, so a search that never reveals replace behaves exactly as
    /// before.
    replace_active: bool,
    /// The replacement text + its OWN CHAR-index caret, one shared
    /// [`TextBox`] like `query` — never spliced into the rope until
    /// replace-current / replace-all is invoked, and its caret/word motion
    /// NEVER recomputes or jumps (the deliberate query/replacement asymmetry —
    /// see [`Self::push_replace_char`]'s own doc).
    replacement: TextBox,
    editing_replacement: bool,
    wrap_armed: Option<Direction>,
}

impl SearchState {
    pub fn start(origin: usize, direction: Direction) -> Self {
        Self {
            query: TextBox::new(),
            case_sensitive: false,
            matches: Vec::new(),
            current: None,
            direction,
            origin,
            replace_active: false,
            replacement: TextBox::new(),
            editing_replacement: false,
            wrap_armed: None,
        }
    }

    /// Begin a search anchored at `origin`, PREFILLED with `query` and
    /// immediately recomputing matches over `haystack` — ONE atomic open
    /// rather than [`Self::start`] plus a manual `push_char` loop, so there is
    /// no intermediate empty-query match state. Feeds both prefill doors the
    /// keybinding-idiom audit asks for: an active selection (Cmd-F, Xcode's
    /// "search for selection", W2) and the REMEMBERED last query (a bare
    /// Cmd-G/Cmd-Shift-G re-find, P2) — see `actions/motion.rs::start_search`,
    /// the one caller. An empty `query` behaves exactly like [`Self::start`].
    pub fn start_with_query(
        origin: usize,
        direction: Direction,
        query: &str,
        haystack: &str,
    ) -> Self {
        let mut s = Self::start(origin, direction);
        if !query.is_empty() {
            s.query = TextBox::seeded(query);
            s.recompute(haystack);
        }
        s
    }

    pub fn push_char(&mut self, c: char, haystack: &str) {
        self.query.insert(c);
        self.recompute(haystack);
    }

    pub fn pop_char(&mut self, haystack: &str) {
        self.query.delete_back();
        self.recompute(haystack);
    }

    /// QUERY char/word caret motion. Pure motion, so it does NOT
    /// recompute the match set (the text is unchanged) — the caller still
    /// re-anchors the visible caret from `self`, never the buffer.
    pub fn query_char_left(&mut self) {
        self.query.char_left();
    }
    pub fn query_char_right(&mut self) {
        self.query.char_right();
    }
    pub fn query_word_left(&mut self) {
        self.query.word_left();
    }
    pub fn query_word_right(&mut self) {
        self.query.word_right();
    }

    /// SELECT ALL in the FOCUSED field — the panel's own answer to the
    /// select-all verb, and the ONE owner both of its doors call: the raw-key
    /// ⌘A door ([`keys::intercept`]) and the routed-Action door
    /// ([`keys::intercept_action`], which is where a macOS menu-bar key
    /// equivalent lands). Pure field state: the query's TEXT is unchanged, so
    /// this deliberately does NOT recompute or jump — the match set, the
    /// parked document caret and its parked selection all stay exactly as
    /// they were, which is the whole point of the verb belonging to the
    /// field. The subsequent typing/deletion needs nothing added: every
    /// [`TextBox`] edit op already replaces an active selection.
    pub fn select_all_focused_field(&mut self) {
        if self.editing_replacement {
            self.replacement.select_all();
        } else {
            self.query.select_all();
        }
    }

    /// The FOCUSED field's active selection as CHAR indices, or `None`. What
    /// the panel DRAWS (only the focused row carries a visible band) and what
    /// a law reads back to prove select-all landed in the field rather than
    /// in the document.
    pub fn focused_selection(&self) -> Option<(usize, usize)> {
        if self.editing_replacement {
            self.replacement.selection_range()
        } else {
            self.query.selection_range()
        }
    }

    /// The two fields' own selections, named separately so a law can assert
    /// that the UNFOCUSED field was left alone.
    #[allow(dead_code)]
    pub fn query_selection(&self) -> Option<(usize, usize)> {
        self.query.selection_range()
    }

    #[allow(dead_code)]
    pub fn replacement_selection(&self) -> Option<(usize, usize)> {
        self.replacement.selection_range()
    }

    pub fn query_delete_word_back(&mut self, haystack: &str) {
        self.query.delete_word_back();
        self.recompute(haystack);
    }

    pub fn toggle_case(&mut self, haystack: &str) {
        self.case_sensitive = !self.case_sensitive;
        self.recompute(haystack);
    }

    /// Refill `matches` for the current query, then pick `current` anchored at
    /// `origin` (deterministic for capture/sidecar):
    ///   * Forward  → first match with `start >= origin`, else wrap to first.
    ///   * Backward → last match with `start <= origin`, else wrap to last.
    fn recompute(&mut self, haystack: &str) {
        self.wrap_armed = None;
        self.matches = find_all(haystack, self.query.text(), self.case_sensitive);
        self.current = if self.matches.is_empty() {
            None
        } else {
            match self.direction {
                Direction::Forward => Some(
                    self.matches
                        .iter()
                        .position(|m| m.start >= self.origin)
                        .unwrap_or(0),
                ),
                Direction::Backward => Some(
                    self.matches
                        .iter()
                        .rposition(|m| m.start <= self.origin)
                        .unwrap_or(self.matches.len() - 1),
                ),
            }
        };
    }

    pub fn step(&mut self, dir: Direction) -> StepOutcome {
        let len = self.matches.len();
        if len == 0 {
            self.direction = dir;
            self.wrap_armed = None;
            return StepOutcome::NoMatches;
        }
        if self.wrap_armed != Some(dir) {
            self.wrap_armed = None;
        }
        self.direction = dir;
        let cur = self.current.unwrap_or(0);
        let at_boundary = match dir {
            Direction::Forward => cur + 1 >= len,
            Direction::Backward => cur == 0,
        };
        if at_boundary {
            if self.wrap_armed == Some(dir) {
                self.wrap_armed = None;
                self.current = Some(match dir {
                    Direction::Forward => 0,
                    Direction::Backward => len - 1,
                });
                StepOutcome::Wrapped
            } else {
                self.wrap_armed = Some(dir);
                StepOutcome::RecoiledAtBoundary(dir)
            }
        } else {
            // A normal in-buffer step disarms any stale wrap and advances.
            self.wrap_armed = None;
            self.current = Some(match dir {
                Direction::Forward => cur + 1,
                Direction::Backward => cur - 1,
            });
            StepOutcome::Moved
        }
    }

    // --- find + replace -----------------------------------------------------
    //
    // Replace is a MODE of the same panel: the search query stays the needle, a
    // second field holds the replacement. The model never touches the rope — it
    // computes the post-replace text and the caller writes it back — so it stays
    // pure + unit-testable, mirroring `find_all`.

    pub fn toggle_replace(&mut self) {
        if self.replace_active {
            self.editing_replacement = !self.editing_replacement;
        } else {
            self.replace_active = true;
            self.editing_replacement = true;
        }
    }

    /// Reveal the labeled replace row WITHOUT moving focus off the find field — the
    /// fresh Cmd-R open state (both rows shown, the amber caret still on the query).
    /// Idempotent: a re-reveal never steals focus back to the query.
    pub fn reveal_replace(&mut self) {
        self.replace_active = true;
    }

    pub fn focus_replacement(&mut self) {
        self.replace_active = true;
        self.editing_replacement = true;
    }

    /// Move focus back to the FIND field (the query) — the click-to-focus
    /// counterpart to [`Self::focus_replacement`], for a mouse press on the find
    /// row. Leaves the replace row's revealed state untouched (a click never hides
    /// it); a no-op when the query already has focus.
    pub fn focus_query(&mut self) {
        self.editing_replacement = false;
    }

    /// Insert a char at the replacement field's caret. The replacement is NOT
    /// searched, so the match set is unchanged (no recompute) — the deliberate
    /// query/replacement asymmetry (`SearchState`'s own doc): this is the ONE
    /// field that must NEVER wire into a recompute/jump.
    pub fn push_replace_char(&mut self, c: char) {
        self.replacement.insert(c);
    }

    pub fn pop_replace_char(&mut self) {
        self.replacement.delete_back();
    }

    /// REPLACEMENT char/word caret motion + word-delete. NONE of
    /// these ever recompute or jump — the replacement is never searched, and a
    /// replace commit reads its CURRENT text regardless of where the caret
    /// sits. Preserves the deliberate query/replacement asymmetry.
    pub fn replacement_char_left(&mut self) {
        self.replacement.char_left();
    }
    pub fn replacement_char_right(&mut self) {
        self.replacement.char_right();
    }
    pub fn replacement_word_left(&mut self) {
        self.replacement.word_left();
    }
    pub fn replacement_word_right(&mut self) {
        self.replacement.word_right();
    }
    pub fn replacement_delete_word_back(&mut self) {
        self.replacement.delete_word_back();
    }

    pub fn refind(&mut self, origin: usize, haystack: &str) {
        self.origin = origin;
        self.direction = Direction::Forward;
        self.recompute(haystack);
    }

    pub fn replace_current_text(&mut self, haystack: &str) -> Option<String> {
        let m = self.current_match()?;
        let chars: Vec<char> = haystack.chars().collect();
        let replacement = self.replacement.text();
        let mut out = String::with_capacity(haystack.len() + replacement.len());
        out.extend(chars[..m.start].iter());
        out.push_str(replacement);
        out.extend(chars[m.end..].iter());
        let resume = m.start + replacement.chars().count();
        self.refind(resume, &out);
        Some(out)
    }

    pub fn replace_all_text(&self, haystack: &str) -> String {
        if self.matches.is_empty() {
            return haystack.to_string();
        }
        let chars: Vec<char> = haystack.chars().collect();
        let replacement = self.replacement.text();
        let mut out = String::with_capacity(haystack.len());
        let mut prev = 0usize;
        for m in &self.matches {
            out.extend(chars[prev..m.start].iter());
            out.push_str(replacement);
            prev = m.end;
        }
        out.extend(chars[prev..].iter());
        out
    }

    pub fn is_replace_active(&self) -> bool {
        self.replace_active
    }

    pub fn is_editing_replacement(&self) -> bool {
        self.editing_replacement
    }

    pub fn replacement(&self) -> &str {
        self.replacement.text()
    }

    // --- accessors for App + render -----------------------------------------

    #[allow(dead_code)]
    pub fn query(&self) -> &str {
        self.query.text()
    }

    pub fn query_caret(&self) -> usize {
        self.query.caret()
    }

    pub fn replacement_caret(&self) -> usize {
        self.replacement.caret()
    }

    #[allow(dead_code)]
    pub fn is_case_sensitive(&self) -> bool {
        self.case_sensitive
    }

    #[allow(dead_code)]
    pub fn matches(&self) -> &[Match] {
        &self.matches
    }

    pub fn current_match(&self) -> Option<Match> {
        self.current.map(|i| self.matches[i])
    }

    #[allow(dead_code)]
    pub fn hit_count(&self) -> usize {
        self.matches.len()
    }

    #[allow(dead_code)]
    pub fn current_index(&self) -> Option<usize> {
        self.current
    }

    pub fn origin(&self) -> usize {
        self.origin
    }
}

/// Every NON-OVERLAPPING occurrence of `needle` in `haystack`, as CHAR spans.
///
/// Two paths, ONE meaning. Byte search through [`memchr::memmem`] is the fast
/// path; the char scan below stays the fallback rather than dead legacy,
/// because the byte path is only reachable where byte equality means exactly
/// what [`chars_eq_fold`] means:
///
/// - CASE-SENSITIVE always takes it. UTF-8 is self-synchronizing — a leading
///   byte never appears as a continuation byte — so a valid needle can never
///   match starting mid-character, and every byte hit is a real char hit.
/// - CASE-INSENSITIVE takes it when the NEEDLE is ASCII and the haystack holds
///   no [`KELVIN_SIGN`]. ASCII-lowercasing the raw BYTES is length- and
///   boundary-preserving (it touches only `A`-`Z`, never a byte >= 0x80), so
///   the offset map survives it even in mixed text — the haystack itself does
///   NOT have to be ASCII. Real prose is full of curly quotes and em dashes;
///   gating on the haystack would have sent nearly every manuscript down the
///   slow path.
///
/// A non-ASCII NEEDLE keeps the char scan: Unicode folding is not a per-char
/// byte-preserving rule (`İ` lowercases to TWO chars), so it cannot be done in
/// place. The fallback is therefore live code, held to the same output by
/// `byte_path_and_char_path_agree_across_the_corpus`.
pub fn find_all(haystack: &str, needle: &str, case_sensitive: bool) -> Vec<Match> {
    if needle.is_empty() {
        return Vec::new();
    }
    if can_byte_search(haystack, needle, case_sensitive) {
        if !case_sensitive {
            let hay = haystack.as_bytes().to_ascii_lowercase();
            let ndl = needle.as_bytes().to_ascii_lowercase();
            return byte_matches(haystack, &hay, &ndl, needle);
        }
        return byte_matches(haystack, haystack.as_bytes(), needle.as_bytes(), needle);
    }
    find_all_by_char(haystack, needle, case_sensitive)
}

fn can_byte_search(haystack: &str, needle: &str, case_sensitive: bool) -> bool {
    case_sensitive || (needle.is_ascii() && !holds_kelvin_sign(haystack))
}

/// U+212A KELVIN SIGN — the ONLY non-ASCII scalar in all of Unicode that
/// case-folds equal to an ASCII char (`k`) under [`chars_eq_fold`], and so the
/// only one an ASCII-only byte fold would miss. Swept exhaustively by
/// `kelvin_sign_is_the_only_scalar_folding_to_ascii`, which re-derives the set
/// from `char::to_lowercase` rather than trusting this comment.
const KELVIN_SIGN: &str = "\u{212A}";

/// Vectorized presence test for the one scalar that defeats an ASCII byte fold.
/// Bails out immediately on pure-ASCII text, which cannot contain it.
fn holds_kelvin_sign(haystack: &str) -> bool {
    let bytes = haystack.as_bytes();
    // A 3-byte needle over a SIMD substring search: far cheaper than the
    // char-window scan it keeps us out of.
    memchr::memmem::find(bytes, KELVIN_SIGN.as_bytes()).is_some()
}

/// Byte-offset search over `hay`/`ndl` (which may be case-folded copies),
/// remapped onto `haystack`'s CHAR indices. `needle` supplies the char length,
/// so the span width stays in the caller's units.
fn byte_matches(haystack: &str, hay: &[u8], ndl: &[u8], needle: &str) -> Vec<Match> {
    if ndl.is_empty() || ndl.len() > hay.len() {
        return Vec::new();
    }
    let mut starts: Vec<usize> = Vec::new();
    let finder = memchr::memmem::Finder::new(ndl);
    let mut pos = 0usize;
    while let Some(off) = finder.find(&hay[pos..]) {
        let at = pos + off;
        starts.push(at);
        pos = at + ndl.len(); // non-overlapping, matching the char scan
    }
    if starts.is_empty() {
        return Vec::new();
    }
    let width = needle.chars().count();
    // In pure ASCII a byte offset IS a char offset, so the remap walk is skipped
    // entirely — the common case for English prose.
    if haystack.is_ascii() {
        return starts
            .iter()
            .map(|&s| Match {
                start: s,
                end: s + width,
            })
            .collect();
    }
    // Otherwise convert the (ascending) byte starts to char indices by COUNTING
    // the chars between them. Counting is a flat byte predicate the optimizer
    // can vectorize; decoding each scalar through `char_indices` cannot be, and
    // measured ~5x slower over the same span.
    let bytes = haystack.as_bytes();
    let mut out = Vec::with_capacity(starts.len());
    let mut chars_before = 0usize;
    let mut counted_to = 0usize;
    for &s in &starts {
        chars_before += char_count(&bytes[counted_to..s]);
        counted_to = s;
        out.push(Match {
            start: chars_before,
            end: chars_before + width,
        });
    }
    out
}

/// Chars in a valid-UTF-8 byte slice: every byte that is NOT a continuation
/// byte (`0b10xxxxxx`) begins exactly one scalar.
#[inline]
fn char_count(bytes: &[u8]) -> usize {
    bytes.iter().filter(|&&b| (b & 0xC0) != 0x80).count()
}

/// The exhaustive char-window scan — correct for ANY text and any fold, and the
/// oracle the byte path is tested against.
fn find_all_by_char(haystack: &str, needle: &str, case_sensitive: bool) -> Vec<Match> {
    let mut out = Vec::new();
    let hay: Vec<char> = haystack.chars().collect();
    let ndl: Vec<char> = needle.chars().collect();
    let nlen = ndl.len();
    if nlen == 0 || nlen > hay.len() {
        return out;
    }
    let mut i = 0usize;
    while i + nlen <= hay.len() {
        if char_window_matches(&hay[i..i + nlen], &ndl, case_sensitive) {
            out.push(Match {
                start: i,
                end: i + nlen,
            });
            i += nlen; // non-overlapping
        } else {
            i += 1;
        }
    }
    out
}

// --- the REMEMBERED last search query (P2's honest Cmd-G re-find) ----------
//
// A tiny process-global mirroring `commands::RECENT`'s own MRU pattern: the
// last NON-EMPTY query a search closed with (Enter accept / Esc abort — live,
// `app/input/keys.rs` — or the headless `Action::Cancel` arm, the ONE search-
// close door `--keys` replay can reach; `actions.rs`). `start_search`
// (`actions/motion.rs`) consults it as the prefill FALLBACK when there is no
// active selection to prefer, so a bare Cmd-G/Cmd-Shift-G — with the panel
// already closed and nothing selected — genuinely re-finds the last thing you
// searched for, mirroring the Safari/browser convention. A fresh process
// starts empty, so a default `--screenshot` (and every test that never
// exercises this door) is unaffected.
use std::sync::Mutex;

static LAST_QUERY: Mutex<String> = Mutex::new(String::new());

/// Remember `query` as the last search term, IF non-empty — an EMPTY close
/// (a search opened and abandoned before typing anything) never overwrites a
/// still-useful remembered query.
pub fn set_last_query(query: &str) {
    if query.is_empty() {
        return;
    }
    if let Ok(mut q) = LAST_QUERY.lock() {
        *q = query.to_string();
    }
}

pub fn last_query() -> String {
    LAST_QUERY.lock().map(|q| q.clone()).unwrap_or_default()
}

#[cfg(test)]
pub fn clear_last_query() {
    if let Ok(mut q) = LAST_QUERY.lock() {
        q.clear();
    }
}

fn char_window_matches(window: &[char], needle: &[char], case_sensitive: bool) -> bool {
    window.iter().zip(needle.iter()).all(|(a, b)| {
        if case_sensitive {
            a == b
        } else {
            chars_eq_fold(*a, *b)
        }
    })
}

fn chars_eq_fold(a: char, b: char) -> bool {
    if a == b {
        return true;
    }
    a.to_lowercase().eq(b.to_lowercase())
}

#[cfg(test)]
mod tests;
