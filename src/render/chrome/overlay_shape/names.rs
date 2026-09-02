use super::*;

impl TextPipeline {
    pub(super) fn push_overlay_name_rows<'a>(
        &self,
        spans: &mut Vec<(&'a str, glyphon::Attrs<'a>)>,
        rows: &'a [String],
        trailing: &'a [String],
        has_query: bool,
        inks: OverlaySpanInks,
        vis: &VisualSelection,
    ) {
        let highlights = &self.overlay_match_highlights;
        let OverlaySpanInks {
            ink,
            muted,
            selected: selected_ink,
        } = inks;
        let base = panel_attrs();
        let mk = |c| base.clone().color(c);
        let sym = |c| Attrs::new().family(Family::Name(SYMBOL_FAMILY)).color(c);
        let italic = crate::render::overlay_slant().is_some_and(|slant| slant.italic);
        let row_attrs = |c| {
            if italic {
                mk(c).style(glyphon::cosmic_text::Style::Italic)
            } else {
                mk(c)
            }
        };
        for (row, content) in rows.iter().enumerate() {
            if has_query || row != 0 {
                spans.push(("\n", mk(ink)));
            }
            let selected = vis.reads_selected(row);
            // Spell's terminal dictionary action is muted only at rest; selected
            // ink still follows the same visual-selection transaction as every row.
            let spell_action = !has_query && row + 1 == rows.len();
            let (name_color, directory_color) = match selected_ink {
                Some(color) if selected => (color, color),
                _ if spell_action => (muted, muted),
                _ => (ink, muted),
            };
            // SEARCH-IN-FOLDER's own split: the query MATCH reads in content
            // ink, everything before AND after it muted -- figure/ground-by-
            // value applied to a matched substring rather than a directory
            // prefix, DESIGN.md's one existing row-split idiom reused for a
            // different split point (never `row_split`'s `/`-based one, which
            // free-form matched prose could false-trigger on). Defensively
            // re-validated against THIS row's own fitted text -- elision can
            // shrink `content` after the range was computed -- so a stale or
            // out-of-bounds range degrades to the ordinary unsplit row rather
            // than panicking on a non-char-boundary slice.
            let highlight = highlights.get(row).copied().flatten().filter(|&(s, e)| {
                s <= e
                    && e <= content.len()
                    && content.is_char_boundary(s)
                    && content.is_char_boundary(e)
            });
            if let Some((s, e)) = highlight {
                if s > 0 {
                    spans.push((&content[..s], row_attrs(directory_color)));
                }
                spans.push((&content[s..e], row_attrs(name_color)));
                if e < content.len() {
                    spans.push((&content[e..], row_attrs(directory_color)));
                }
            } else {
                let split = if content.ends_with('/') || !self.overlay_row_path_splits {
                    0
                } else {
                    crate::overlay::row_split(content)
                };
                if split > 0 {
                    spans.push((&content[..split], row_attrs(directory_color)));
                }
                spans.push((&content[split..], row_attrs(name_color)));
            }
            if let Some(cell) = trailing.get(row).filter(|cell| !cell.is_empty()) {
                push_symbol_split(spans, cell, || mk(muted), || sym(muted));
            }
        }
    }
}
