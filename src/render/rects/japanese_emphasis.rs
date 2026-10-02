//! Japanese Markdown emphasis uses small dots; Latin keeps its italic face.

use super::*;

pub(in crate::render) const JA_EMPHASIS_DOT_SIZE: Logical = Logical(2.4);

/// Kana and kanji letters take a mark; punctuation, combining voicing marks,
/// whitespace and Latin do not own independent marked cells.
fn markable(c: char) -> bool {
    matches!(c as u32,
        0x3041..=0x3096 | 0x30A1..=0x30FA | 0x31F0..=0x31FF
        | 0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xF900..=0xFAFF)
}

fn emphasis_prose_spans(
    text: &str,
    md_spans: &[(std::ops::Range<usize>, crate::markdown::MdKind)],
) -> Vec<std::ops::Range<usize>> {
    use crate::markdown::{ConcealKind, MdKind};
    let tables: Vec<_> = md_spans
        .iter()
        .filter(|(_, kind)| *kind == MdKind::ConcealMarkup(ConcealKind::Table))
        .map(|(range, _)| range)
        .collect();
    // Structural emphasis survives heading/link/quote style priority. Intersect
    // with real prose Text spans so code, URLs and markup cannot acquire dots.
    let emphasis = crate::markdown::emphasis_content_spans(text);
    let mut spans = Vec::new();
    for (range, kind) in emphasis {
        if !matches!(kind, MdKind::Italic | MdKind::BoldItalic) {
            continue;
        }
        for (content, kind) in md_spans {
            if !matches!(
                kind,
                MdKind::Italic
                    | MdKind::BoldItalic
                    | MdKind::Bold
                    | MdKind::Heading(_)
                    | MdKind::Quote
                    | MdKind::LinkText
                    | MdKind::TaskDone
            ) {
                continue;
            }
            let start = range.start.max(content.start);
            let end = range.end.min(content.end);
            if start < end
                && !tables
                    .iter()
                    .any(|table| table.start < end && start < table.end)
            {
                spans.push(start..end);
            }
        }
    }
    spans
}

impl TextPipeline {
    /// Cache against the same shaped geometry as the other content decorations.
    /// Cursor reveal changes advances through reshape; scroll only offsets dots.
    pub(super) fn rebuild_japanese_emphasis_protos(&self) {
        let Some(text) = self.shaped_key.as_deref() else {
            return;
        };
        let spans = emphasis_prose_spans(text, &self.md_spans);
        if spans.is_empty() {
            self.wash_cache
                .japanese_emphasis_protos
                .borrow_mut()
                .clear();
            return;
        }
        let mut line_starts = Vec::with_capacity(self.buffer.lines.len());
        let mut start = 0usize;
        for line in self.buffer.lines.iter() {
            line_starts.push(start);
            start += line.text().len() + 1;
        }
        let mut segments = Vec::new();
        for range in spans {
            let mut line = line_starts
                .partition_point(|start| *start <= range.start)
                .saturating_sub(1);
            while line < self.buffer.lines.len() && line_starts[line] < range.end {
                let line_start = line_starts[line];
                let text = self.buffer.lines[line].text();
                let lo = range.start.max(line_start);
                let hi = range.end.min(line_start + text.len());
                if lo < hi {
                    let col = |byte| {
                        text.char_indices()
                            .take_while(|(index, _)| *index < byte)
                            .count()
                    };
                    segments.push((line, col(lo - line_start), col(hi - line_start)));
                }
                line += 1;
            }
        }
        let lines = segments.iter().map(|(line, ..)| *line).collect();
        let rows_by_line = self.visual_rows_for_lines(&lines);
        let priority = crate::script::effective_cjk_priority(self.han_evidence, &self.cjk_priority);
        let mut marked = std::collections::BTreeSet::new();
        let mut protos = Vec::new();
        for (line, start, end) in segments {
            let Some(rows) = rows_by_line.get(&line) else {
                continue;
            };
            let chars: Vec<_> = self.buffer.lines[line].text().chars().collect();
            for row in rows {
                let a = start.max(row.start_col).min(row.xs.len().saturating_sub(1));
                let b = end.min(row.end_col).min(row.xs.len().saturating_sub(1));
                for col in a..b {
                    let Some(&c) = chars.get(col) else {
                        continue;
                    };
                    if markable(c)
                        && crate::script::resolve_font_id(
                            self.doc_lang,
                            crate::script::classify_char(c),
                            &priority,
                        ) == theme::FontId::Ja
                        && row.xs[col + 1] > row.xs[col]
                        && marked.insert((line, col))
                    {
                        protos.push(UnderlineProto {
                            line,
                            start_col: col,
                            end_col: col + 1,
                            line_top: row.line_top,
                            line_height: row.line_height,
                            xs_s: row.xs[col],
                            xs_e: row.xs[col + 1],
                        });
                    }
                }
            }
        }
        protos.sort_by(|a, b| {
            a.line_top
                .total_cmp(&b.line_top)
                .then_with(|| a.xs_s.total_cmp(&b.xs_s))
        });
        *self.wash_cache.japanese_emphasis_protos.borrow_mut() = protos;
    }

    pub(in crate::render) fn japanese_emphasis_dot_rects(&self) -> Vec<[f32; 4]> {
        if self.md_spans.is_empty() {
            return Vec::new();
        }
        self.ensure_wash_protos();
        let protos = self.wash_cache.japanese_emphasis_protos.borrow();
        let size = self.metrics.px(JA_EMPHASIS_DOT_SIZE);
        let doc_top = self.doc_top();
        let text_left = self.text_left();
        let margin = self.metrics.line_height * 8.0;
        let first =
            protos.partition_point(|proto| doc_top + proto.line_top + proto.line_height <= -margin);
        let visible = protos[first..]
            .iter()
            .take_while(|proto| doc_top + proto.line_top < self.window_h + margin);
        let mut out = Vec::new();
        for proto in visible {
            let line_top = doc_top + proto.line_top;
            if !self.proto_visible(line_top, proto.line_height) {
                continue;
            }
            let (y, _) = self.row_band_for(proto.line, proto.line_height, line_top);
            let x = text_left + (proto.xs_s + proto.xs_e) * 0.5 - size * 0.5;
            out.push([x, y, size, size]);
        }
        self.clip_decorative_rects_to_band(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn letters_only_receive_independent_dots() {
        let _g = crate::testlock::serial();
        for c in ['日', '本', '語', 'ひ', 'ら', 'カ', 'ナ', 'ㇰ'] {
            assert!(markable(c), "{c:?}");
        }
        for c in [
            '、', '。', '「', '」', '（', '）', '！', '？', ' ', '*', 'a', '\u{3099}',
        ] {
            assert!(!markable(c), "punctuation/Latin/combining {c:?}");
        }
    }
}
