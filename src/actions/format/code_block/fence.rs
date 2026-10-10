//! Raw fence-pair validation layered under pulldown's structural parse.

fn longest_backtick_run(lines: &[String], first: usize, last: usize) -> usize {
    lines[first..=last]
        .iter()
        .flat_map(|line| line.split(|ch| ch != '`'))
        .map(str::len)
        .max()
        .unwrap_or(0)
}

pub(super) fn fence_for(lines: &[String], first: usize, last: usize) -> String {
    "`".repeat(3.max(longest_backtick_run(lines, first, last) + 1))
}

fn run_at(line: &str, offset: usize) -> Option<(char, usize)> {
    let rest = line.get(offset..)?;
    let marker = rest.chars().next()?;
    if marker != '`' && marker != '~' {
        return None;
    }
    let run = rest.chars().take_while(|&ch| ch == marker).count();
    (run >= 3).then_some((marker, run))
}

pub(super) fn matching_fence_from(open: &str, offset: usize, close: &str) -> bool {
    let Some((marker, opening_run)) = run_at(open, offset) else {
        return false;
    };
    if close.len() - close.trim_start_matches(' ').len() != offset {
        return false;
    }
    let rest = &close[offset..];
    let run = rest.chars().take_while(|&ch| ch == marker).count();
    run >= opening_run && rest[run..].chars().all(|ch| ch == ' ' || ch == '\t')
}

pub(super) fn matching_top_level_fence_from(open: &str, offset: usize, close: &str) -> bool {
    let Some((marker, opening_run)) = run_at(open, offset) else {
        return false;
    };
    let close_offset = close.len() - close.trim_start_matches(' ').len();
    if close_offset > 3 {
        return false;
    }
    let rest = &close[close_offset..];
    let run = rest.chars().take_while(|&ch| ch == marker).count();
    run >= opening_run && rest[run..].chars().all(|ch| ch == ' ' || ch == '\t')
}
