use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RowNamePlan {
    name: glyphon::Color,
    flank: glyphon::Color,
    highlight: Option<(usize, usize)>,
}

/// Validate a search range against its fitted row and resolve the inks that
/// shaping will consume. A selected hit names only its matching span with the
/// row-primary ink; its context remains muted. A stale range deliberately
/// degrades to muted text, rather than making an unproven match look primary.
fn plan_row_name(
    content: &str,
    requested_highlight: Option<(usize, usize)>,
    selected: bool,
    selected_ink: Option<glyphon::Color>,
    ink: glyphon::Color,
    muted: glyphon::Color,
    spell_action: bool,
) -> RowNamePlan {
    let primary = selected_ink.filter(|_| selected).unwrap_or(ink);
    if spell_action && !selected {
        return RowNamePlan {
            name: muted,
            flank: muted,
            highlight: None,
        };
    }
    let highlight = requested_highlight.filter(|&(start, end)| {
        start < end
            && end <= content.len()
            && content.is_char_boundary(start)
            && content.is_char_boundary(end)
    });
    if requested_highlight.is_some() && highlight.is_none() {
        return RowNamePlan {
            name: muted,
            flank: muted,
            highlight: None,
        };
    }
    RowNamePlan {
        name: primary,
        flank: if highlight.is_some() || !selected {
            muted
        } else {
            primary
        },
        highlight,
    }
}

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
        let base = overlay_panel_attrs();
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
            let plan = plan_row_name(
                content,
                highlights.get(row).copied().flatten(),
                selected,
                selected_ink,
                ink,
                muted,
                spell_action,
            );
            if let Some((s, e)) = plan.highlight {
                if s > 0 {
                    spans.push((&content[..s], row_attrs(plan.flank)));
                }
                spans.push((&content[s..e], row_attrs(plan.name)));
                if e < content.len() {
                    spans.push((&content[e..], row_attrs(plan.flank)));
                }
            } else {
                let split = if content.ends_with('/') || !self.overlay_row_path_splits {
                    0
                } else {
                    crate::overlay::row_split(content)
                };
                if split > 0 {
                    spans.push((&content[..split], row_attrs(plan.flank)));
                }
                spans.push((&content[split..], row_attrs(plan.name)));
            }
            if let Some(cell) = trailing.get(row).filter(|cell| !cell.is_empty()) {
                push_symbol_split(spans, cell, || mk(muted), || sym(muted));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_name_plan_enrolls_selected_search_hits_and_safe_fallbacks() {
        let _guard = crate::testlock::serial();
        let content = "pre-match-post";
        let ink = glyphon::Color::rgb(1, 2, 3);
        let muted = glyphon::Color::rgb(4, 5, 6);
        let selected = glyphon::Color::rgb(7, 8, 9);
        assert_ne!(ink, muted);
        assert_ne!(selected, muted);

        assert_eq!(
            plan_row_name(
                content,
                Some((4, 9)),
                true,
                Some(selected),
                ink,
                muted,
                false
            ),
            RowNamePlan {
                name: selected,
                flank: muted,
                highlight: Some((4, 9)),
            },
            "selected hits keep only the matched span at selected strength"
        );
        assert_eq!(
            plan_row_name(
                content,
                Some((4, 9)),
                false,
                Some(selected),
                ink,
                muted,
                false
            ),
            RowNamePlan {
                name: ink,
                flank: muted,
                highlight: Some((4, 9)),
            },
            "unselected hits retain the established content-versus-context split"
        );
        assert_eq!(
            plan_row_name(content, None, true, Some(selected), ink, muted, false),
            RowNamePlan {
                name: selected,
                flank: selected,
                highlight: None,
            },
            "ordinary selected path rows retain their shared selected ink"
        );
        assert_eq!(
            plan_row_name(content, None, false, Some(selected), ink, muted, false),
            RowNamePlan {
                name: ink,
                flank: muted,
                highlight: None,
            },
            "ordinary unselected path rows retain their muted path context"
        );
        assert_eq!(
            plan_row_name(content, None, true, Some(selected), ink, muted, true),
            RowNamePlan {
                name: selected,
                flank: selected,
                highlight: None,
            },
            "a selected spell action retains selected ink"
        );
        assert_eq!(
            plan_row_name(content, None, false, Some(selected), ink, muted, true),
            RowNamePlan {
                name: muted,
                flank: muted,
                highlight: None,
            },
            "an unselected spell action remains muted"
        );
        assert_eq!(
            plan_row_name(
                content,
                Some((4, 99)),
                true,
                Some(selected),
                ink,
                muted,
                false
            ),
            RowNamePlan {
                name: muted,
                flank: muted,
                highlight: None,
            },
            "a stale fitted range cannot turn unproven context into selected ink"
        );
    }
}
