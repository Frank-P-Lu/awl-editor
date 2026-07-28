use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gran {
    Word,
    Sentence,
}

#[derive(Clone, Copy, Debug)]
pub struct Params {
    pub gran: Gran,
    pub coalesce: f32,
}

impl Default for Params {
    fn default() -> Self {
        Params {
            gran: Gran::Word,
            coalesce: 0.5,
        }
    }
}

impl Params {
    /// The SHIPPING recipe the live "Compare with version…" view uses — the gate's
    /// pick (user, 2026-07-18): SENTENCE-level granularity (a touched sentence swaps
    /// whole rather than showing word-by-word surgery — calmer for prose) at a 0.5
    /// coalescing threshold (a paragraph reworded past halfway reads as a clean
    /// old-struck / new-washed rewrite). The one owner both the live App and the
    /// capture harness read, so they can never diverge on what a shipped diff looks
    /// like.
    pub const fn shipping() -> Self {
        Params {
            gran: Gran::Sentence,
            coalesce: 0.5,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Seg {
    Same(String),
    Ins(String),
    Del(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoveDir {
    Up,
    Down,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Block {
    Fold(usize),
    Modified(Vec<Seg>),
    Rewritten { old: String, new: String },
    Inserted(String),
    Deleted(String),
    Moved { text: String, dir: MoveDir },
}

const BACKBONE_SIM_MIN: f32 = 0.25;
/// A leftover (off-backbone) pair must clear this HIGHER bar to read as a move — a
/// relocation carries most of its words with it.
const MOVE_SIM_MIN: f32 = 0.55;

fn paragraphs(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur: Vec<&str> = Vec::new();
    for line in s.lines() {
        if line.trim().is_empty() {
            if !cur.is_empty() {
                out.push(cur.join("\n"));
                cur.clear();
            }
        } else {
            cur.push(line);
        }
    }
    if !cur.is_empty() {
        out.push(cur.join("\n"));
    }
    out
}

fn word_tokens(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut chars = s.chars().peekable();
    while let Some(&c) = chars.peek() {
        let ws = c.is_whitespace();
        let mut tok = String::new();
        while let Some(&c2) = chars.peek() {
            if c2.is_whitespace() != ws {
                break;
            }
            tok.push(c2);
            chars.next();
        }
        out.push(tok);
    }
    out
}

fn sentence_tokens(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        cur.push(c);
        if matches!(c, '.' | '!' | '?') {
            let mut j = i + 1;
            while j < chars.len() && matches!(chars[j], '.' | '!' | '?') {
                cur.push(chars[j]);
                j += 1;
            }
            let followed_by_break = j >= chars.len() || chars[j].is_whitespace();
            if followed_by_break {
                while j < chars.len() && chars[j].is_whitespace() {
                    cur.push(chars[j]);
                    j += 1;
                }
                out.push(std::mem::take(&mut cur));
                i = j;
                continue;
            }
            i = j;
            continue;
        }
        i += 1;
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn tokens(s: &str, gran: Gran) -> Vec<String> {
    match gran {
        Gran::Word => word_tokens(s),
        Gran::Sentence => sentence_tokens(s),
    }
}

/// Content (non-blank) tokens only — the unit of similarity + density, so leading
/// indentation and inter-word spacing never dominate the measure.
fn content_tokens(toks: &[String]) -> Vec<&str> {
    toks.iter()
        .filter(|t| !t.trim().is_empty())
        .map(|t| t.as_str())
        .collect()
}

// ---------------------------------------------------------------------------
// LCS (the one shared primitive: paragraph alignment AND within-paragraph diff)
// ---------------------------------------------------------------------------

fn lcs_table<T: PartialEq>(a: &[T], b: &[T]) -> Vec<Vec<u32>> {
    let mut dp = vec![vec![0u32; b.len() + 1]; a.len() + 1];
    for i in (0..a.len()).rev() {
        for j in (0..b.len()).rev() {
            dp[i][j] = if a[i] == b[j] {
                dp[i + 1][j + 1] + 1
            } else {
                dp[i + 1][j].max(dp[i][j + 1])
            };
        }
    }
    dp
}

fn ratio(a: &[&str], b: &[&str]) -> f32 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    if a == b {
        return 1.0;
    }
    let dp = lcs_table(a, b);
    let common = dp[0][0] as f32;
    2.0 * common / (a.len() + b.len()) as f32
}

/// A free UPPER BOUND on [`ratio`] from lengths alone: `|LCS| ≤ min(|a|,|b|)`, so
/// `ratio ≤ 2·min/(|a|+|b|)`. Lets a caller with a similarity FLOOR skip the
/// quadratic table for pairs that provably can't clear it (exact — a skipped pair
/// was never going to match).
fn ratio_upper_bound(a: &[&str], b: &[&str]) -> f32 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    2.0 * a.len().min(b.len()) as f32 / (a.len() + b.len()) as f32
}

fn seg_diff(old: &str, new: &str, gran: Gran) -> Vec<Seg> {
    let a = tokens(old, gran);
    let b = tokens(new, gran);
    let dp = lcs_table(&a, &b);
    let (mut i, mut j) = (0usize, 0usize);
    let mut segs: Vec<Seg> = Vec::new();
    let push = |segs: &mut Vec<Seg>, mk: fn(String) -> Seg, s: &str, tag: u8| {
        if let Some(last) = segs.last_mut() {
            let same_tag = match (last, tag) {
                (Seg::Same(x), 0) | (Seg::Del(x), 1) | (Seg::Ins(x), 2) => {
                    x.push_str(s);
                    true
                }
                _ => false,
            };
            if same_tag {
                return;
            }
        }
        segs.push(mk(s.to_string()));
    };
    while i < a.len() && j < b.len() {
        if a[i] == b[j] {
            push(&mut segs, Seg::Same, &a[i], 0);
            i += 1;
            j += 1;
        } else if dp[i + 1][j] >= dp[i][j + 1] {
            push(&mut segs, Seg::Del, &a[i], 1);
            i += 1;
        } else {
            push(&mut segs, Seg::Ins, &b[j], 2);
            j += 1;
        }
    }
    while i < a.len() {
        push(&mut segs, Seg::Del, &a[i], 1);
        i += 1;
    }
    while j < b.len() {
        push(&mut segs, Seg::Ins, &b[j], 2);
        j += 1;
    }
    segs
}

fn seg_density(segs: &[Seg], gran: Gran) -> f32 {
    let count = |s: &str| content_tokens(&tokens(s, gran)).len();
    let (mut same, mut changed) = (0usize, 0usize);
    for seg in segs {
        match seg {
            Seg::Same(s) => same += count(s),
            Seg::Ins(s) | Seg::Del(s) => changed += count(s),
        }
    }
    let total = same * 2 + changed;
    if total == 0 {
        0.0
    } else {
        changed as f32 / total as f32
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Role {
    Same,
    Edit,
    Move,
}

#[derive(Clone, Copy, Debug)]
struct Pair {
    oi: usize,
    ni: usize,
    role: Role,
}

/// The ordered BACKBONE of in-place paragraph matches (Same when identical, Edit
/// otherwise). Increasing in both indices by construction, so a relocation is
/// deliberately NOT captured here — it falls to the leftovers.
///
/// PERF (the DIFF-AS-PREVIEW round): the original single similarity-DP over ALL
/// paragraph pairs was O(P² · tokens²) — ~6 s per call on a 5k-line draft, which
/// the History picker now pays PER ARROW. The fix is patience-diff shaped and
/// output-preserving for ordinary prose: paragraphs whose content tokens are
/// EXACTLY equal and UNIQUE in both documents become anchors ([`anchor_pairs`]),
/// the longest increasing subsequence of those anchors is kept as the ordered
/// `Same` spine, and the expensive similarity DP ([`sim_backbone`]) runs only in
/// the (typically tiny) GAP WINDOWS between consecutive anchors. An identical
/// document costs O(P); a localized edit costs the window around it. A document
/// with NO usable anchors (every paragraph duplicated / everything rewritten)
/// falls back to the original whole-document DP — never worse, just never
/// faster. A dropped out-of-order anchor (a relocated paragraph) lands in the
/// leftovers exactly as before, where [`detect_moves`] claims it.
fn backbone(old: &[Vec<&str>], new: &[Vec<&str>]) -> Vec<Pair> {
    let anchors = lis_anchors(&anchor_pairs(old, new));
    if anchors.is_empty() {
        return sim_backbone(old, new, 0..old.len(), 0..new.len());
    }
    let mut pairs = Vec::new();
    let (mut oi, mut ni) = (0usize, 0usize);
    for &(ao, an) in &anchors {
        pairs.extend(sim_backbone(old, new, oi..ao, ni..an));
        pairs.push(Pair {
            oi: ao,
            ni: an,
            role: Role::Same,
        });
        oi = ao + 1;
        ni = an + 1;
    }
    pairs.extend(sim_backbone(old, new, oi..old.len(), ni..new.len()));
    pairs
}

fn anchor_pairs(old: &[Vec<&str>], new: &[Vec<&str>]) -> Vec<(usize, usize)> {
    use std::collections::HashMap;
    let mut on_old: HashMap<&[&str], (usize, usize)> = HashMap::new();
    for (i, p) in old.iter().enumerate() {
        let e = on_old.entry(p.as_slice()).or_insert((0, i));
        e.0 += 1;
    }
    let mut on_new: HashMap<&[&str], (usize, usize)> = HashMap::new();
    for (j, p) in new.iter().enumerate() {
        let e = on_new.entry(p.as_slice()).or_insert((0, j));
        e.0 += 1;
    }
    let mut out: Vec<(usize, usize)> = Vec::new();
    for (i, p) in old.iter().enumerate() {
        if on_old.get(p.as_slice()).map(|e| e.0) != Some(1) {
            continue;
        }
        if let Some(&(1, j)) = on_new.get(p.as_slice()) {
            // Empty partitions are separators, not anchors.
            if !p.is_empty() {
                out.push((i, j));
            }
        }
    }
    out
}

fn lis_anchors(cands: &[(usize, usize)]) -> Vec<(usize, usize)> {
    if cands.is_empty() {
        return Vec::new();
    }
    let mut tails: Vec<usize> = Vec::new();
    let mut prev: Vec<Option<usize>> = vec![None; cands.len()];
    for (idx, &(_, ni)) in cands.iter().enumerate() {
        let pos = tails.partition_point(|&t| cands[t].1 < ni);
        if pos > 0 {
            prev[idx] = Some(tails[pos - 1]);
        }
        if pos == tails.len() {
            tails.push(idx);
        } else {
            tails[pos] = idx;
        }
    }
    let mut chain = Vec::with_capacity(tails.len());
    let mut cur = tails.last().copied();
    while let Some(i) = cur {
        chain.push(cands[i]);
        cur = prev[i];
    }
    chain.reverse();
    chain
}

fn sim_backbone(
    old: &[Vec<&str>],
    new: &[Vec<&str>],
    or: std::ops::Range<usize>,
    nr: std::ops::Range<usize>,
) -> Vec<Pair> {
    let (ob, nb) = (or.start, nr.start);
    let old = &old[or];
    let new = &new[nr];
    let (n, m) = (old.len(), new.len());
    if n == 0 || m == 0 {
        return Vec::new();
    }
    let mut score = vec![vec![0.0f32; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            let r = ratio(&old[i], &new[j]);
            let diag = if r >= BACKBONE_SIM_MIN {
                score[i + 1][j + 1] + r
            } else {
                f32::MIN
            };
            score[i][j] = diag.max(score[i + 1][j]).max(score[i][j + 1]);
        }
    }
    let mut pairs = Vec::new();
    let (mut i, mut j) = (0usize, 0usize);
    while i < n && j < m {
        let r = ratio(&old[i], &new[j]);
        let diag = if r >= BACKBONE_SIM_MIN {
            score[i + 1][j + 1] + r
        } else {
            f32::MIN
        };
        if diag >= score[i + 1][j] && diag >= score[i][j] && r >= BACKBONE_SIM_MIN {
            let role = if r >= 0.999 { Role::Same } else { Role::Edit };
            pairs.push(Pair {
                oi: ob + i,
                ni: nb + j,
                role,
            });
            i += 1;
            j += 1;
        } else if score[i + 1][j] >= score[i][j + 1] {
            i += 1;
        } else {
            j += 1;
        }
    }
    pairs
}

fn detect_moves(
    old: &[Vec<&str>],
    new: &[Vec<&str>],
    used_old: &mut [bool],
    used_new: &mut [bool],
) -> Vec<Pair> {
    let mut cands: Vec<(f32, usize, usize)> = Vec::new();
    for (oi, o) in old.iter().enumerate() {
        if used_old[oi] {
            continue;
        }
        for (ni, nw) in new.iter().enumerate() {
            if used_new[ni] {
                continue;
            }
            if ratio_upper_bound(o, nw) < MOVE_SIM_MIN {
                continue;
            }
            let r = ratio(o, nw);
            if r >= MOVE_SIM_MIN {
                cands.push((r, oi, ni));
            }
        }
    }
    cands.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let mut moves = Vec::new();
    for (_, oi, ni) in cands {
        if used_old[oi] || used_new[ni] {
            continue;
        }
        used_old[oi] = true;
        used_new[ni] = true;
        moves.push(Pair {
            oi,
            ni,
            role: Role::Move,
        });
    }
    moves
}

pub fn diff(old: &str, new: &str, p: Params) -> Vec<Block> {
    let old_ps = paragraphs(old);
    let new_ps = paragraphs(new);
    let old_words: Vec<Vec<String>> = old_ps.iter().map(|s| word_tokens(s)).collect();
    let new_words: Vec<Vec<String>> = new_ps.iter().map(|s| word_tokens(s)).collect();
    let old_tok: Vec<Vec<&str>> = old_words.iter().map(|w| content_tokens(w)).collect();
    let new_tok: Vec<Vec<&str>> = new_words.iter().map(|w| content_tokens(w)).collect();

    let bb = backbone(&old_tok, &new_tok);
    let mut used_old = vec![false; old_ps.len()];
    let mut used_new = vec![false; new_ps.len()];
    let mut old_role: Vec<Option<Pair>> = vec![None; old_ps.len()];
    let mut new_role: Vec<Option<Pair>> = vec![None; new_ps.len()];
    for pr in &bb {
        used_old[pr.oi] = true;
        used_new[pr.ni] = true;
        old_role[pr.oi] = Some(*pr);
        new_role[pr.ni] = Some(*pr);
    }
    let moves = detect_moves(&old_tok, &new_tok, &mut used_old, &mut used_new);
    for pr in &moves {
        old_role[pr.oi] = Some(*pr);
        new_role[pr.ni] = Some(*pr);
    }

    // Two-pointer merge → reading order. Old-only (deletes / moved-away) flush first
    // within a gap, then new-only (inserts / moved-in), then the shared anchor.
    let mut blocks: Vec<Block> = Vec::new();
    let (mut i, mut j) = (0usize, 0usize);
    let (no, nn) = (old_ps.len(), new_ps.len());
    while i < no || j < nn {
        if i < no {
            match old_role[i] {
                None => {
                    blocks.push(Block::Deleted(old_ps[i].clone()));
                    i += 1;
                    continue;
                }
                Some(pr) if pr.role == Role::Move => {
                    i += 1;
                    continue;
                }
                _ => {}
            }
        }
        if j < nn {
            match new_role[j] {
                None => {
                    blocks.push(Block::Inserted(new_ps[j].clone()));
                    j += 1;
                    continue;
                }
                Some(pr) if pr.role == Role::Move => {
                    let dir = if pr.ni <= pr.oi {
                        MoveDir::Up
                    } else {
                        MoveDir::Down
                    };
                    blocks.push(Block::Moved {
                        text: new_ps[pr.ni].clone(),
                        dir,
                    });
                    j += 1;
                    continue;
                }
                _ => {}
            }
        }
        // both anchors — must be partners in the backbone
        if i < no
            && j < nn
            && let (Some(a), Some(b)) = (old_role[i], new_role[j])
            && a.oi == b.oi
            && a.ni == b.ni
        {
            match a.role {
                Role::Same => blocks.push(Block::Fold(1)),
                Role::Edit => {
                    let segs = seg_diff(&old_ps[i], &new_ps[j], p.gran);
                    if seg_density(&segs, p.gran) > p.coalesce {
                        blocks.push(Block::Rewritten {
                            old: old_ps[i].clone(),
                            new: new_ps[j].clone(),
                        });
                    } else {
                        blocks.push(Block::Modified(segs));
                    }
                }
                Role::Move => {}
            }
            i += 1;
            j += 1;
            continue;
        }
        // safety fall-through (shouldn't trigger): advance the laggard
        if i < no {
            i += 1;
        } else {
            j += 1;
        }
    }

    merge_folds(blocks)
}

fn merge_folds(blocks: Vec<Block>) -> Vec<Block> {
    let mut out: Vec<Block> = Vec::new();
    for b in blocks {
        if let Block::Fold(n) = b
            && let Some(Block::Fold(m)) = out.last_mut()
        {
            *m += n;
            continue;
        }
        out.push(b);
    }
    out
}

/// Struck deletions speak REAL markdown now: `~~…~~`, wrapped per line by
/// [`wrap_inline`] — routed through the renderer's own `MdKind::Strikethrough`
/// (the strikethrough-render round), whose muted ink + drawn strike line come
/// from THE ONE strike owner (`render::spans::strike_line_band` /
/// `strike_ink`), the same fns the format popover's `S` button reads.
///
/// HISTORY (the retired mechanism): before the renderer could draw `~~strike~~`
/// at all, deletions were struck by inserting a COMBINING LONG STROKE OVERLAY
/// (`\u{0336}`) after every non-whitespace char — genuine struck glyphs with
/// zero render-path code, at the cost of per-word gaps (whitespace had to stay
/// unstruck or read as "- - -" leaders) and of the transcript carrying invisible
/// combining marks. With real strikethrough in the render vocabulary the
/// serializer says what it means; the drawn line crosses spaces cleanly, so the
/// whitespace exception died with the mechanism.
fn strike(s: &str) -> String {
    wrap_inline(s, "~~")
}

fn highlight_lines(s: &str) -> String {
    s.lines()
        .map(|line| {
            let trimmed = line.trim_start();
            if trimmed.is_empty() {
                line.to_string()
            } else {
                let indent = &line[..line.len() - trimmed.len()];
                format!("{indent}=={trimmed}==")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Wrap ONE inline insertion run in the highlight wash markers — see
/// [`wrap_inline`] (the shared shape [`strike`] rides too).
fn highlight_inline(s: &str) -> String {
    wrap_inline(s, "==")
}

fn wrap_inline(s: &str, marker: &str) -> String {
    let mut out = String::new();
    for (k, piece) in s.split('\n').enumerate() {
        if k > 0 {
            out.push('\n');
        }
        let trimmed = piece.trim();
        if trimmed.is_empty() {
            out.push_str(piece);
            continue;
        }
        let lead = &piece[..piece.len() - piece.trim_start().len()];
        let tail = &piece[piece.trim_end().len()..];
        out.push_str(lead);
        out.push_str(marker);
        out.push_str(trimmed);
        out.push_str(marker);
        out.push_str(tail);
    }
    out
}

/// Reduce markdown SOURCE to plain prose for the manuscript diff: a marked-up
/// manuscript shows WORDS, not syntax — and, pragmatically, stripping the inline
/// markers means the `==wash==` / strike serialization never has to wrap nested
/// markdown (a `==**bold**==` pair is fragile). Applied identically to both versions
/// before diffing, so the alignment is unaffected. Deliberately light: it neutralizes
/// emphasis/code/link/marker syntax, nothing semantic.
fn strip_markdown(s: &str) -> String {
    let mut out = String::new();
    for (li, line) in s.lines().enumerate() {
        if li > 0 {
            out.push('\n');
        }
        let mut rest = line;
        let trimmed = rest.trim_start();
        let indent_len = rest.len() - trimmed.len();
        let indent = &rest[..indent_len];
        let after: Option<&str> = if let Some(a) = trimmed
            .strip_prefix("- ")
            .or_else(|| trimmed.strip_prefix("* "))
            .or_else(|| trimmed.strip_prefix("+ "))
        {
            Some(a)
        } else if trimmed.starts_with('#') {
            Some(trimmed.trim_start_matches('#').trim_start())
        } else if let Some(a) = trimmed.strip_prefix("> ") {
            Some(a)
        } else {
            trimmed
                .find(". ")
                .filter(|&i| trimmed[..i].chars().all(|c| c.is_ascii_digit()) && i > 0)
                .map(|i| &trimmed[i + 2..])
        };
        out.push_str(indent);
        if let Some(a) = after {
            rest = a;
        }
        let bytes: Vec<char> = rest.chars().collect();
        let mut i = 0;
        while i < bytes.len() {
            let c = bytes[i];
            match c {
                '`' => {}
                '*' | '_' => {}
                '~' => {}
                '[' | ']' => {}
                '(' if i > 0 && bytes[i - 1] == ']' => {
                    while i < bytes.len() && bytes[i] != ')' {
                        i += 1;
                    }
                }
                _ => out.push(c),
            }
            i += 1;
        }
    }
    out
}

fn blockquote(s: &str) -> String {
    s.lines()
        .map(|l| format!("> {l}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Serialize a block list into a marked-up-markdown transcript. The leading heading
/// parks the headless caret (byte 0) on a throwaway line so every diff marker below
/// stays WYSIWYG-concealed (clean preview), never revealed-raw.
pub fn render_markdown_blocks(blocks: &[Block], title: &str) -> String {
    let mut out = format!("# {title}\n\n");
    for b in blocks {
        match b {
            Block::Fold(n) => {
                let unit = if *n == 1 { "paragraph" } else { "paragraphs" };
                out.push_str(&format!("> ⋯  {n} {unit} unchanged  ⋯\n\n"));
            }
            Block::Modified(segs) => {
                for seg in segs {
                    match seg {
                        Seg::Same(s) => out.push_str(s),
                        Seg::Ins(s) => out.push_str(&highlight_inline(s)),
                        Seg::Del(s) => out.push_str(&strike(s)),
                    }
                }
                out.push_str("\n\n");
            }
            Block::Rewritten { old, new } => {
                out.push_str(&blockquote(&strike(old)));
                out.push_str("\n\n");
                out.push_str(&highlight_lines(new));
                out.push_str("\n\n");
            }
            Block::Inserted(s) => {
                out.push_str(&highlight_lines(s));
                out.push_str("\n\n");
            }
            Block::Deleted(s) => {
                out.push_str(&blockquote(&strike(s)));
                out.push_str("\n\n");
            }
            Block::Moved { text, dir } => {
                let arrow = match dir {
                    MoveDir::Up => "↑",
                    MoveDir::Down => "↓",
                };
                let body = text.replace('\n', " ");
                out.push_str(&format!("> *⇄  moved {arrow} — {body}*\n\n"));
            }
        }
    }
    out
}

/// One-call convenience: strip both docs to plain prose, diff, and render the
/// transcript. The strip is a RENDER-path concern (the pure [`diff`] stays raw).
/// (Non-test callers all want the counts too and ride [`diff_and_render`]
/// directly — the History preview owner, the `AWL_DIFF_*` capture harness —
/// so this convenience is exercised by the unit tests alone today.)
#[cfg_attr(not(test), allow(dead_code))]
pub fn render_markdown(old: &str, new: &str, p: Params, title: &str) -> String {
    diff_and_render(old, new, p, title).0
}

/// Like [`render_markdown`], but ALSO returns the [`DiffCounts`] of the block list
/// — the shared owner both the transcript AND the capture sidecar's `diff` state
/// block derive from, so they can never disagree about what the transcript contains.
pub fn diff_and_render(old: &str, new: &str, p: Params, title: &str) -> (String, DiffCounts) {
    let (o, n) = (strip_markdown(old), strip_markdown(new));
    let blocks = diff(&o, &n, p);
    let md = render_markdown_blocks(&blocks, title);
    (md, count_blocks(&blocks))
}

// ---------------------------------------------------------------------------
// Sidecar counts — the diff-view STATE oracle (the capture reports these; the
// pixel assertions verify the APPEARANCE, per the sidecar-vs-appearance tripwire).
// ---------------------------------------------------------------------------

/// A count of the diff's block kinds — the compact STATE the capture sidecar's
/// `diff` block reports so an agent can verify "am I looking at a diff, and does it
/// carry deletions / insertions / moves / folds". Pure over the [`Block`] list.
/// APPEARANCE ("the struck region is muted", "the wash is present") is asserted
/// over the PNG's pixels, never inferred from these — this is a state oracle only.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DiffCounts {
    /// Paragraphs shown STRUCK whole (a coalesced rewrite's old side, or a pure
    /// deletion) — the marks that must render muted+struck.
    pub struck: usize,
    /// Paragraphs shown WASHED whole (a coalesced rewrite's new side, or a pure
    /// insertion) — the marks that must render in the highlight wash.
    pub washed: usize,
    pub modified: usize,
    pub moved: usize,
    pub folds: usize,
}

pub fn count_blocks(blocks: &[Block]) -> DiffCounts {
    let mut c = DiffCounts::default();
    for b in blocks {
        match b {
            Block::Fold(_) => c.folds += 1,
            Block::Modified(_) => c.modified += 1,
            Block::Rewritten { .. } => {
                c.struck += 1;
                c.washed += 1;
            }
            Block::Inserted(_) => c.washed += 1,
            Block::Deleted(_) => c.struck += 1,
            Block::Moved { .. } => c.moved += 1,
        }
    }
    c
}

// ---------------------------------------------------------------------------
// Capture harness entry (capture-only) — mirrors the `AWL_POPOVER` / `AWL_CJK_FORCE`
// precedent: read ONCE, a total no-op unless both version paths are set. This is
// how the headless `--screenshot` renders the diff VIEW (a live App feature) so it
// is pixel-verifiable; a normal capture never touches it.
// ---------------------------------------------------------------------------

/// A resolved capture-harness diff request: the two version texts + the shipping
/// params + a title. Built from the `AWL_DIFF_*` env vars ([`env_capture`]).
pub struct EnvCapture {
    pub old: String,
    pub new: String,
    pub params: Params,
    pub title: String,
}

/// Parse the `AWL_DIFF_*` env vars. Returns `Some` only when BOTH version paths are
/// set and readable — otherwise the capture path behaves exactly as today (byte-
/// identical). `AWL_DIFF_GRAN=word` overrides the shipping SENTENCE default;
/// `AWL_DIFF_COALESCE` overrides the 0.5 threshold; `AWL_DIFF_TITLE` the heading.
pub fn env_capture() -> Option<&'static Option<EnvCapture>> {
    static ONCE: OnceLock<Option<EnvCapture>> = OnceLock::new();
    Some(ONCE.get_or_init(|| {
        let old_p = std::env::var("AWL_DIFF_OLD").ok()?;
        let new_p = std::env::var("AWL_DIFF_NEW").ok()?;
        let old = std::fs::read_to_string(&old_p).ok()?;
        let new = std::fs::read_to_string(&new_p).ok()?;
        let mut params = Params::shipping();
        if let Ok("word") = std::env::var("AWL_DIFF_GRAN").as_deref() {
            params.gran = Gran::Word;
        }
        if let Some(c) = std::env::var("AWL_DIFF_COALESCE")
            .ok()
            .and_then(|s| s.parse::<f32>().ok())
        {
            params.coalesce = c.clamp(0.0, 1.0);
        }
        let title = std::env::var("AWL_DIFF_TITLE").unwrap_or_else(|_| "Comparing versions".into());
        Some(EnvCapture {
            old,
            new,
            params,
            title,
        })
    }))
}

/// The marked-up transcript + counts + title for the active capture-harness diff
/// request, if any — called by the capture path in place of loading a file so
/// `--screenshot` renders the read-only diff view and the sidecar reports its
/// state. `None` everywhere else (byte-identical ordinary capture).
pub fn env_capture_render() -> Option<(String, DiffCounts, String)> {
    match env_capture() {
        Some(Some(p)) => {
            let (md, counts) = diff_and_render(&p.old, &p.new, p.params, &p.title);
            Some((md, counts, p.title.clone()))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wp(s: &str) -> Vec<String> {
        word_tokens(s)
    }

    #[test]
    #[ignore]
    fn perf_probe() {
        let scope = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("PHILOSOPHY.md"),
        )
        .unwrap_or_default();
        let scope_new = {
            let mut s = scope.replace("audience", "readership");
            s.push_str("\n\nA freshly appended closing paragraph for the probe.\n");
            s
        };
        let mut big_old = String::new();
        for i in 0..1000 {
            big_old.push_str(&format!(
                "Paragraph {i} begins with its own opening line here.\nThe second line of paragraph {i} carries on the thought.\nA third line follows, still inside paragraph {i} itself.\nLine four of paragraph {i} keeps the rhythm moving along.\nAnd the fifth line closes paragraph {i} with a full stop.\n\n"
            ));
        }
        let big_light = big_old.replace(
            "Paragraph 500 begins with its own opening line here.",
            "Paragraph 500 now opens with an entirely reworded first line.",
        );
        let mut big_heavy = big_old.replace(
            "inside paragraph 7 itself",
            "inside the seventh stanza of this draft",
        );
        let moved = "Paragraph 900 begins with its own opening line here.\nThe second line of paragraph 900 carries on the thought.\nA third line follows, still inside paragraph 900 itself.\nLine four of paragraph 900 keeps the rhythm moving along.\nAnd the fifth line closes paragraph 900 with a full stop.\n\n";
        big_heavy = format!("{moved}{}", big_heavy.replace(moved, ""));
        let time = |name: &str, old: &str, new: &str| {
            let t0 = std::time::Instant::now();
            let (md, counts) = diff_and_render(old, new, Params::shipping(), "Probe");
            let dt = t0.elapsed();
            println!(
                "perf_probe {name}: {:.2} ms  (old {} lines, new {} lines, transcript {} bytes, {counts:?})",
                dt.as_secs_f64() * 1000.0,
                old.lines().count(),
                new.lines().count(),
                md.len(),
            );
        };
        time("scope_md_edit", &scope, &scope_new);
        time("scope_md_identical", &scope, &scope);
        time("5k_light_edit", &big_old, &big_light);
        time("5k_identical", &big_old, &big_old);
        time("5k_heavy_rewrite_plus_move", &big_old, &big_heavy);
    }

    #[test]
    fn word_tokens_join_losslessly() {
        let s = "The  quick\nbrown fox.";
        assert_eq!(wp(s).concat(), s);
    }

    #[test]
    fn sentence_tokens_join_losslessly() {
        let s = "One sentence. Two! Three? A trailing bit";
        assert_eq!(sentence_tokens(s).concat(), s);
        assert_eq!(sentence_tokens(s).len(), 4);
    }

    #[test]
    fn paragraphs_split_on_blank_lines() {
        let s = "a\nb\n\n\nc\n";
        assert_eq!(paragraphs(s), vec!["a\nb".to_string(), "c".to_string()]);
    }

    #[test]
    fn ratio_bounds() {
        let aw = wp("the cat sat");
        let a = content_tokens(&aw);
        assert_eq!(ratio(&a, &a), 1.0);
        let bw = wp("a totally different string here");
        let b = content_tokens(&bw);
        assert!(ratio(&a, &b) < 0.34);
    }

    #[test]
    fn seg_diff_word_level_marks_ins_and_del() {
        let segs = seg_diff("the quick brown fox", "the slow brown fox", Gran::Word);
        // must be lossless on each side
        let old: String = segs
            .iter()
            .filter_map(|s| match s {
                Seg::Same(x) | Seg::Del(x) => Some(x.clone()),
                _ => None,
            })
            .collect();
        let new: String = segs
            .iter()
            .filter_map(|s| match s {
                Seg::Same(x) | Seg::Ins(x) => Some(x.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(old, "the quick brown fox");
        assert_eq!(new, "the slow brown fox");
        assert!(
            segs.iter()
                .any(|s| matches!(s, Seg::Del(x) if x.contains("quick")))
        );
        assert!(
            segs.iter()
                .any(|s| matches!(s, Seg::Ins(x) if x.contains("slow")))
        );
    }

    #[test]
    fn seg_diff_sentence_swaps_whole_sentence() {
        let old = "First stays. Second changes a lot here.";
        let new = "First stays. A totally rewritten second one.";
        let segs = seg_diff(old, new, Gran::Sentence);
        assert!(
            segs.iter()
                .any(|s| matches!(s, Seg::Same(x) if x.contains("First stays")))
        );
        assert!(segs.iter().any(|s| matches!(s, Seg::Del(_))));
        assert!(segs.iter().any(|s| matches!(s, Seg::Ins(_))));
    }

    #[test]
    fn density_low_for_small_edit_high_for_rewrite() {
        let small = seg_diff(
            "the quick brown fox jumps",
            "the quick brown fox leaps",
            Gran::Word,
        );
        let big = seg_diff(
            "the quick brown fox jumps",
            "an entirely new clause appears",
            Gran::Word,
        );
        assert!(seg_density(&small, Gran::Word) < 0.25);
        assert!(seg_density(&big, Gran::Word) > 0.6);
    }

    #[test]
    fn coalesce_threshold_flips_modified_to_rewritten() {
        let old = "The cat sat quietly on the warm mat by the old door.";
        let new = "The cat sat nervously on the cold floor near the new window.";
        let lo = diff(
            old,
            new,
            Params {
                gran: Gran::Word,
                coalesce: 0.3,
            },
        );
        assert!(
            lo.iter().any(|b| matches!(b, Block::Rewritten { .. })),
            "low threshold should coalesce: {lo:?}"
        );
        let hi = diff(
            old,
            new,
            Params {
                gran: Gran::Word,
                coalesce: 0.95,
            },
        );
        assert!(!hi.iter().any(|b| matches!(b, Block::Rewritten { .. })));
        assert!(hi.iter().any(|b| matches!(b, Block::Modified(_))));
    }

    #[test]
    fn three_thresholds_give_three_distinct_outputs() {
        // The HONEST-AXIS property: two edited paragraphs at DIFFERENT change
        // densities, so a threshold sweep between/around them yields three
        // provably-distinct rewrite counts (2 / 1 / 0) — never the c50≡c70
        // collapse that a fixture whose densities all sit below 0.5 exhibits.
        // A lightly-edited paragraph (~1/4 words swapped) and a heavily-edited
        // one (~2/3 words swapped), both still sharing enough spine to ALIGN.
        let old = "\
The quiet river wound its slow way past the sleeping village every single morning.

She counted the coins twice and wrote the total in the ledger before she left.";
        let new = "\
The quiet river wound its lazy course past the drowsy hamlet under each grey dawn.

She, by long habit, tallied the takings and inked the sum into the ledger before departing.";
        let count_rw = |c: f32| {
            diff(
                old,
                new,
                Params {
                    gran: Gran::Word,
                    coalesce: c,
                },
            )
            .iter()
            .filter(|b| matches!(b, Block::Rewritten { .. }))
            .count()
        };
        let blocks = diff(
            old,
            new,
            Params {
                gran: Gran::Word,
                coalesce: 0.5,
            },
        );
        assert!(
            !blocks
                .iter()
                .any(|b| matches!(b, Block::Deleted(_) | Block::Inserted(_)))
        );
        let (lo, mid, hi) = (count_rw(0.30), count_rw(0.55), count_rw(0.80));
        assert_eq!(
            (lo, mid, hi),
            (2, 1, 0),
            "expected 2/1/0 rewrites, got {lo}/{mid}/{hi}"
        );
    }

    #[test]
    fn alignment_same_insert_delete() {
        let old = "Alpha paragraph one.\n\nBeta paragraph two.";
        let new = "Alpha paragraph one.\n\nA brand new middle paragraph.\n\nBeta paragraph two.";
        let blocks = diff(old, new, Params::default());
        assert!(
            blocks
                .iter()
                .any(|b| matches!(b, Block::Inserted(x) if x.contains("brand new")))
        );
        assert!(blocks.iter().any(|b| matches!(b, Block::Fold(_))));
        assert!(!blocks.iter().any(|b| matches!(b, Block::Deleted(_))));
    }

    #[test]
    fn pure_deletion_is_a_deleted_block() {
        let old = "Keep this one.\n\nDrop this whole paragraph entirely.";
        let new = "Keep this one.";
        let blocks = diff(old, new, Params::default());
        assert!(
            blocks
                .iter()
                .any(|b| matches!(b, Block::Deleted(x) if x.contains("Drop this")))
        );
    }

    #[test]
    fn relocated_paragraph_reads_as_moved_not_delete_plus_insert() {
        let old = "\
Anchor one stays put here.

Anchor two also stays.

The movable paragraph about migrating birds.";
        let new = "\
The movable paragraph about migrating birds.

Anchor one stays put here.

Anchor two also stays.";
        let blocks = diff(old, new, Params::default());
        assert!(
            blocks.iter().any(
                |b| matches!(b, Block::Moved { text, .. } if text.contains("migrating birds"))
            ),
            "expected a Moved block, got {blocks:?}"
        );
        assert!(
            !blocks
                .iter()
                .any(|b| matches!(b, Block::Deleted(x) if x.contains("migrating birds")))
        );
        assert!(
            !blocks
                .iter()
                .any(|b| matches!(b, Block::Inserted(x) if x.contains("migrating birds")))
        );
    }

    #[test]
    fn unrelated_swap_is_not_a_move() {
        let old = "The quantum theory of gravitation.\n\nRecipes for sourdough bread.";
        let new = "Recipes for sourdough bread.\n\nThe quantum theory of gravitation.";
        let blocks = diff(old, new, Params::default());
        let moved = blocks
            .iter()
            .filter(|b| matches!(b, Block::Moved { .. }))
            .count();
        let del = blocks
            .iter()
            .filter(|b| matches!(b, Block::Deleted(_)))
            .count();
        let ins = blocks
            .iter()
            .filter(|b| matches!(b, Block::Inserted(_)))
            .count();
        assert!(moved >= 1);
        assert_eq!(del, 0);
        assert_eq!(ins, 0);
    }

    #[test]
    fn transcript_uses_awls_vocabulary() {
        let old = "Keep me.\n\nDelete me completely please.";
        let new = "Keep me.\n\nA fresh inserted paragraph here.";
        let md = render_markdown(old, new, Params::default(), "T");
        assert!(md.starts_with("# T\n\n"));
        assert!(md.contains("~~")); // struck deletion — REAL markdown strikethrough
        assert!(md.contains("==")); // highlight-washed insertion
        assert!(md.contains("> ")); // blockquote (dim) for the deletion + folds
    }

    #[test]
    fn strike_wraps_per_line_and_keeps_whitespace_outside_markers() {
        let s = strike("A short  digression\nspanning two lines.");
        assert_eq!(s, "~~A short  digression~~\n~~spanning two lines.~~");
        assert_eq!(strike("  indented tail  "), "  ~~indented tail~~  ");
        assert_eq!(strike("word\n\nnext"), "~~word~~\n\n~~next~~");
        assert!(!s.contains('\u{0336}'));
        assert_eq!(highlight_inline("a b\nc"), "==a b==\n==c==");
    }

    #[test]
    fn strip_markdown_removes_tildes_so_the_strike_wrap_never_nests() {
        assert_eq!(
            strip_markdown("approx ~40 chars, ~~old style~~"),
            "approx 40 chars, old style"
        );
    }

    #[test]
    fn render_is_deterministic() {
        let old = "One two three.\n\nFour five six seven.";
        let new = "One two three changed.\n\nFour five six seven.";
        let a = render_markdown(old, new, Params::default(), "T");
        let b = render_markdown(old, new, Params::default(), "T");
        assert_eq!(a, b);
    }

    #[test]
    fn shipping_recipe_is_sentence_half() {
        let p = Params::shipping();
        assert_eq!(p.gran, Gran::Sentence);
        assert_eq!(p.coalesce, 0.5);
    }

    #[test]
    fn count_blocks_tallies_each_kind() {
        let old = "Keep me here.\n\nDrop this whole paragraph entirely.\n\nAnd keep this.";
        let new = "Keep me here.\n\nA fresh inserted paragraph here.\n\nAnd keep this.";
        let blocks = diff(old, new, Params::shipping());
        let c = count_blocks(&blocks);
        assert_eq!(c.struck, 1, "one struck deletion: {blocks:?}");
        assert_eq!(c.washed, 1, "one washed insertion: {blocks:?}");
        assert!(c.folds >= 1, "at least one fold: {blocks:?}");
        assert_eq!(c.moved, 0);
    }

    #[test]
    fn count_blocks_rewrite_is_both_struck_and_washed() {
        let old = "The cat sat quietly on the warm mat by the old door.";
        let new = "The cat sat nervously on the cold floor near the new window.";
        let blocks = diff(
            old,
            new,
            Params {
                gran: Gran::Word,
                coalesce: 0.3,
            },
        );
        assert!(blocks.iter().any(|b| matches!(b, Block::Rewritten { .. })));
        let c = count_blocks(&blocks);
        assert_eq!((c.struck, c.washed), (1, 1), "{blocks:?}");
    }
}
