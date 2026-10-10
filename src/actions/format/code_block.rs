//! Parser-checked fenced-code toggling, including fences that stay owned by a
//! Markdown list item instead of swallowing its marker into the code payload.

use super::{BlockToggle, FormatResult, line_start_char, split_lines};
use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};
use std::ops::Range;

mod fence;
use fence::{fence_for, matching_fence_from, matching_top_level_fence_from};

#[derive(Clone, Debug, PartialEq, Eq)]
struct ItemContext {
    range: Range<usize>,
    marker_line: usize,
    prefix: String,
    continuation: String,
}

#[derive(Clone, Debug)]
struct ParsedFence {
    range: Range<usize>,
    item: Option<Range<usize>>,
}

#[derive(Clone)]
struct Group {
    first: usize,
    last: usize,
    item: Option<ItemContext>,
}

#[derive(Clone)]
struct PlannedFence {
    open_line: usize,
    item_marker_line: Option<usize>,
    task_marker_line: Option<usize>,
}

pub(super) fn toggle(
    text: &str,
    anchor: Option<usize>,
    cursor: usize,
    first: usize,
    last: usize,
    has_sel: bool,
) -> BlockToggle {
    let lines = split_lines(text);
    let reverse = anchor.is_some_and(|a| a > cursor);

    if let Some(result) = unwrap_list_fences(text, &lines, first, last, reverse) {
        return BlockToggle::Edit(result);
    }
    if has_unclosed_fence(text, &lines, first, last) {
        return BlockToggle::NoValidListCodeBlock;
    }

    wrap(text, &lines, first, last, has_sel, reverse)
}

fn parsed_structure(text: &str) -> (Vec<Range<usize>>, Vec<ParsedFence>) {
    let mut items = Vec::new();
    let mut item_stack: Vec<Range<usize>> = Vec::new();
    let mut fences = Vec::new();
    for (event, range) in Parser::new_ext(text, crate::markdown::PARSE_OPTIONS).into_offset_iter() {
        match event {
            Event::Start(Tag::Item) => {
                items.push(range.clone());
                item_stack.push(range);
            }
            Event::End(TagEnd::Item) => {
                item_stack.pop();
            }
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(_))) => {
                fences.push(ParsedFence {
                    range,
                    item: item_stack.last().cloned(),
                });
            }
            _ => {}
        }
    }
    (items, fences)
}

fn byte_starts(lines: &[String]) -> Vec<usize> {
    let mut starts = Vec::with_capacity(lines.len());
    let mut at = 0;
    for line in lines {
        starts.push(at);
        at += line.len() + 1;
    }
    starts
}

fn line_for_byte(starts: &[usize], byte: usize) -> usize {
    starts
        .partition_point(|&start| start <= byte)
        .saturating_sub(1)
}

fn item_context(
    lines: &[String],
    starts: &[usize],
    ranges: &[Range<usize>],
    line: usize,
) -> Option<ItemContext> {
    let line_start = starts[line];
    let range = ranges
        .iter()
        .filter(|range| {
            let marker_line = line_for_byte(starts, range.start);
            marker_line <= line && (line_start < range.end || marker_line == line)
        })
        .max_by_key(|range| range.start)?
        .clone();
    item_context_for_range(lines, starts, range)
}

fn item_context_for_range(
    lines: &[String],
    starts: &[usize],
    range: Range<usize>,
) -> Option<ItemContext> {
    let marker_line = line_for_byte(starts, range.start);
    let item = crate::markdown::list_item(&lines[marker_line])?;
    if starts[marker_line] + item.indent != range.start {
        return None;
    }
    let prefix = lines[marker_line][..item.content].to_string();
    if prefix.contains('\t') {
        return None;
    }
    Some(ItemContext {
        range,
        marker_line,
        continuation: " ".repeat(item.content),
        prefix,
    })
}

fn selected_groups(
    lines: &[String],
    starts: &[usize],
    item_ranges: &[Range<usize>],
    first: usize,
    last: usize,
) -> Option<Vec<Group>> {
    let mut groups: Vec<Group> = Vec::new();
    for line in first..=last {
        let item = item_context(lines, starts, item_ranges, line);
        if item_ranges.iter().any(|range| {
            let marker_line = line_for_byte(starts, range.start);
            marker_line <= line && starts[line] < range.end
        }) && item.is_none()
        {
            return None; // a parser-owned tab-indented item is deliberately refused
        }
        if let Some(group) = groups.last_mut()
            && group.item == item
        {
            group.last = line;
        } else {
            groups.push(Group {
                first: line,
                last: line,
                item,
            });
        }
    }
    Some(groups)
}

fn task_parts(rest: &str) -> Option<(&str, &str, &str)> {
    for task in ["[ ]", "[x]", "[X]"] {
        if let Some(body) = rest.strip_prefix(&format!("{task} ")) {
            return Some((task, body, " "));
        }
    }
    None
}

fn has_tab_list_marker(lines: &[String]) -> bool {
    lines.iter().any(|line| {
        let trimmed = line.trim_start_matches([' ', '\t']);
        let marker_end = if trimmed.starts_with(['-', '*', '+']) {
            1
        } else {
            trimmed
                .find(['.', ')'])
                .filter(|&at| trimmed[..at].bytes().all(|byte| byte.is_ascii_digit()))
                .map_or(0, |at| at + 1)
        };
        marker_end > 0 && trimmed.as_bytes().get(marker_end) == Some(&b'\t')
    })
}

fn wrap(
    text: &str,
    lines: &[String],
    first: usize,
    last: usize,
    _has_sel: bool,
    reverse: bool,
) -> BlockToggle {
    if has_tab_list_marker(&lines[first..=last]) {
        return BlockToggle::NoValidListCodeBlock;
    }
    let (item_ranges, _) = parsed_structure(text);
    let starts = byte_starts(lines);
    let Some(groups) = selected_groups(lines, &starts, &item_ranges, first, last) else {
        return BlockToggle::NoValidListCodeBlock;
    };
    let blank_target = lines[first..=last]
        .iter()
        .all(|line| line.trim().is_empty());

    let mut out = Vec::with_capacity(lines.len() + groups.len() * 2);
    out.extend_from_slice(&lines[..first]);
    let out_first = out.len();
    let mut planned = Vec::with_capacity(groups.len());
    for group in groups {
        let fence = fence_for(lines, group.first, group.last);
        match group.item {
            None => {
                if !blank_target
                    && lines[group.first..=group.last]
                        .iter()
                        .all(|line| line.trim().is_empty())
                {
                    out.extend_from_slice(&lines[group.first..=group.last]);
                    continue;
                }
                let open_line = out.len();
                out.push(fence.clone());
                out.extend_from_slice(&lines[group.first..=group.last]);
                out.push(fence);
                planned.push(PlannedFence {
                    open_line,
                    item_marker_line: None,
                    task_marker_line: None,
                });
            }
            Some(item) => {
                let starts_at_marker = group.first == item.marker_line;
                for line in lines
                    .iter()
                    .take(group.last + 1)
                    .skip(group.first + usize::from(starts_at_marker))
                {
                    if !line.is_empty() && !line.starts_with(&item.continuation) {
                        return BlockToggle::NoValidListCodeBlock;
                    }
                }
                if starts_at_marker {
                    let rest = &lines[group.first][item.prefix.len()..];
                    if matches!(rest, "[ ]" | "[x]" | "[X]") {
                        return BlockToggle::NoValidListCodeBlock;
                    }
                    if let Some((task, body, task_separator)) = task_parts(rest) {
                        let task_marker_line = out.len();
                        // Keep the task checkbox's required following space on
                        // its own source line. Without it pulldown keeps `[x]`
                        // as paragraph text and the task silently loses state.
                        out.push(format!("{}{task}{task_separator}", item.prefix));
                        let open_line = out.len();
                        out.push(format!("{}{fence}", item.continuation));
                        out.push(format!("{}{body}", item.continuation));
                        planned.push(PlannedFence {
                            open_line,
                            item_marker_line: Some(task_marker_line),
                            task_marker_line: Some(task_marker_line),
                        });
                    } else {
                        let open_line = out.len();
                        out.push(format!("{}{fence}", item.prefix));
                        out.push(format!("{}{}", item.continuation, rest));
                        planned.push(PlannedFence {
                            open_line,
                            item_marker_line: Some(open_line),
                            task_marker_line: None,
                        });
                    }
                    out.extend_from_slice(&lines[group.first + 1..=group.last]);
                } else {
                    if item.marker_line >= first {
                        return BlockToggle::NoValidListCodeBlock;
                    }
                    let open_line = out.len();
                    out.push(format!("{}{fence}", item.continuation));
                    out.extend_from_slice(&lines[group.first..=group.last]);
                    planned.push(PlannedFence {
                        open_line,
                        item_marker_line: Some(item.marker_line),
                        task_marker_line: None,
                    });
                }
                out.push(format!("{}{fence}", item.continuation));
            }
        }
    }
    out.extend_from_slice(&lines[last + 1..]);
    let out_last = out.len() - (lines.len() - last - 1) - 1;
    let new_text = out.join("\n");
    if !plans_parse(&new_text, &out, &planned) {
        return BlockToggle::NoValidListCodeBlock;
    }
    BlockToggle::Edit(selected_result(out, new_text, out_first, out_last, reverse))
}

fn plans_parse(text: &str, lines: &[String], plans: &[PlannedFence]) -> bool {
    let (_, parsed) = parsed_structure(text);
    let starts = byte_starts(lines);
    let spans = crate::markdown::spans(text);
    plans.iter().all(|plan| {
        let Some(fence) = parsed
            .iter()
            .find(|fence| line_for_byte(&starts, fence.range.start) == plan.open_line)
        else {
            return false;
        };
        let item_matches = match (plan.item_marker_line, &fence.item) {
            (None, None) => true,
            (Some(marker_line), Some(parsed_item)) => {
                line_for_byte(&starts, parsed_item.start) == marker_line
            }
            _ => false,
        };
        let task_matches = plan.task_marker_line.is_none_or(|line| {
            let start = starts[line];
            let end = start + lines[line].len();
            spans.iter().any(|(range, kind)| {
                range.start >= start
                    && range.end <= end
                    && matches!(kind, crate::markdown::MdKind::Task(_))
            })
        });
        item_matches && task_matches
    })
}

fn unwrap_list_fences(
    text: &str,
    lines: &[String],
    first: usize,
    last: usize,
    reverse: bool,
) -> Option<FormatResult> {
    let (items, fences) = parsed_structure(text);
    let starts = byte_starts(lines);
    let mut edits: Vec<(usize, usize, Vec<String>)> = Vec::new();
    let mut covered = vec![false; last - first + 1];
    for fence in fences {
        let open = line_for_byte(&starts, fence.range.start);
        let close = line_for_byte(&starts, fence.range.end.saturating_sub(1));
        if open < first || close > last || close <= open {
            continue;
        }
        let open_offset = fence.range.start.saturating_sub(starts[open]);
        let closes = if fence.item.is_some() {
            matching_fence_from(&lines[open], open_offset, &lines[close])
        } else {
            matching_top_level_fence_from(&lines[open], open_offset, &lines[close])
        };
        if !closes {
            continue;
        }
        let Some(item_range) = fence.item.clone() else {
            let replacement = lines[open + 1..close].to_vec();
            for line in open..=close {
                covered[line - first] = true;
            }
            edits.push((open, close, replacement));
            continue;
        };
        let item = item_context_for_range(lines, &starts, item_range)?;
        let (replace_first, replacement) = if open == item.marker_line {
            let mut replacement = if close == open + 1 {
                vec![item.prefix.clone()]
            } else {
                let body = lines.get(open + 1)?.strip_prefix(&item.continuation)?;
                vec![format!("{}{body}", item.prefix)]
            };
            if close > open + 2 {
                replacement.extend_from_slice(&lines[open + 2..close]);
            }
            (open, replacement)
        } else if open == item.marker_line + 1 {
            let marker_rest = lines[item.marker_line].strip_prefix(&item.prefix)?;
            let task_restore = ["[ ]", "[x]", "[X]"].into_iter().find_map(|task| {
                marker_rest
                    .strip_prefix(task)
                    .filter(|rest| *rest == " " || *rest == "  ")
                    .map(|spaces| (task, spaces.len()))
            });
            if let Some((task, separator_len)) = task_restore {
                let separator = " ".repeat(separator_len);
                let mut replacement = if close == open + 1 {
                    vec![format!("{}{task}{separator}", item.prefix)]
                } else {
                    let body = lines.get(open + 1)?.strip_prefix(&item.continuation)?;
                    vec![format!("{}{task}{separator}{body}", item.prefix)]
                };
                if close > open + 2 {
                    replacement.extend_from_slice(&lines[open + 2..close]);
                }
                (item.marker_line, replacement)
            } else {
                let replacement = lines[open + 1..close].to_vec();
                (open, replacement)
            }
        } else {
            (open, lines[open + 1..close].to_vec())
        };
        for line in replace_first..=close {
            if (first..=last).contains(&line) {
                covered[line - first] = true;
            }
        }
        edits.push((replace_first, close, replacement));
    }
    if edits.is_empty()
        || covered
            .iter()
            .enumerate()
            .any(|(offset, covered)| !covered && !lines[first + offset].trim().is_empty())
    {
        return None;
    }

    let mut out = lines.to_vec();
    edits.sort_by_key(|(start, _, _)| *start);
    for (start, end, replacement) in edits.into_iter().rev() {
        out.splice(start..=end, replacement);
    }
    let out_first = first;
    let removed = lines.len() - out.len();
    let out_last = last.saturating_sub(removed);
    let new_text = out.join("\n");
    let _ = items; // item ranges are consumed through each parsed fence's owner
    Some(selected_result(out, new_text, out_first, out_last, reverse))
}

fn has_unclosed_fence(text: &str, lines: &[String], first: usize, last: usize) -> bool {
    let (_, fences) = parsed_structure(text);
    let starts = byte_starts(lines);
    fences.into_iter().any(|fence| {
        let open = line_for_byte(&starts, fence.range.start);
        let close = line_for_byte(&starts, fence.range.end.saturating_sub(1));
        open >= first
            && open <= last
            && close <= last
            && !(if fence.item.is_some() {
                matching_fence_from(
                    &lines[open],
                    fence.range.start.saturating_sub(starts[open]),
                    &lines[close],
                )
            } else {
                matching_top_level_fence_from(
                    &lines[open],
                    fence.range.start.saturating_sub(starts[open]),
                    &lines[close],
                )
            })
    })
}

fn selected_result(
    lines: Vec<String>,
    text: String,
    first: usize,
    last: usize,
    reverse: bool,
) -> FormatResult {
    if lines.is_empty() || first >= lines.len() || last >= lines.len() {
        return FormatResult {
            text,
            anchor: None,
            cursor: line_start_char(&lines, first.min(lines.len())),
        };
    }
    let start = line_start_char(&lines, first);
    let end = line_start_char(&lines, last) + lines[last].chars().count();
    let (anchor, cursor) = if reverse {
        (Some(end), start)
    } else {
        (Some(start), end)
    };
    FormatResult {
        text,
        anchor,
        cursor,
    }
}
