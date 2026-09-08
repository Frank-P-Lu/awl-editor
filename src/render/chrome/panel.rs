//! SEARCH PANEL chrome — the summoned top-right find/replace card: its opaque
//! elevated card, the bordered find/replace fields, the match/navigation
//! region (counter, prev/next, the `Match case` checkbox), the `Replace` /
//! `Replace all` buttons, and the amber query caret riding the shaped
//! advance. Inherent methods on [`super::TextPipeline`] (they shape into its
//! shared panel buffers). See [`super`].
//!
//! **The reference chrome** (`references/find-replace-chrome.png`, cited on
//! the queue) calls for clear bordered fields, a separate match/navigation
//! region, and distinct Replace/Replace all controls — while the earlier
//! terse `/`-pill and the later borderless text-strip both went too far the
//! other way. This module keeps the keyboard-first character (every button
//! carries the chord that also fires it, sourced from ONE place —
//! `keyspec::Panel*` — never a hardcoded glyph) while restoring the bordered,
//! labeled-region composition. The card is still ONE text buffer
//! (`panel_buffer`) shaped as a handful of rows; what is new is that some of
//! those rows' CONTROLS also get a drawn box, computed from their own shaped
//! byte range (`chrome::panel_controls`), never a hardcoded pitch.

use super::*;

/// The search card's inner breathing room and outer canvas inset, preserving
/// their existing DEVICE-pixel behavior. Promoting either to `Logical` would
/// change the Retina composition and remains a visual decision; naming them is
/// what lets the pending 1x/2x comparison find both consumers without another
/// inline-literal census.
pub(in crate::render) const PANEL_PAD: Physical = Physical(12.0);
pub(in crate::render) const PANEL_MARGIN: Physical = Physical(12.0);

/// Field labels, padded to ONE shared width (13 ASCII bytes) so the find and
/// replace boxes start in the same column — ASCII, so byte len == char count,
/// which the caret-offset math below relies on.
const FIND_LABEL: &str = "Find         ";
const REPLACE_LABEL: &str = "Replace with ";
const _: () = assert!(
    FIND_LABEL.len() == REPLACE_LABEL.len(),
    "the two field labels must share one padded width so both boxes start in \
     the same column"
);
/// The ordinary-width VISIBLE cap (character cells) of the query/replacement
/// VALUE field: typing/pasting past this count SCROLLS the field
/// (`field_view_window`, the one clipping-rule owner shared by both fields)
/// instead of widening the card. Twenty-eight cells is wide enough for a
/// realistic search term without feeling cramped on an ordinary canvas.
const PANEL_FIELD_CHARS: usize = 28;
/// The label plus its one reserved caret cell — the fixed-cell floor every
/// responsive `field_chars` computation subtracts before granting the rest to
/// the value field.
// The label ITSELF renders in the active world's PROPORTIONAL face (never
// monospace), so its true pixel width is not exactly `label.len() *
// char_width` — the `+ 2` (one cell past the label + its one reserved caret
// cell) is slack against that mismatch, keeping the responsive field
// comfortably inside the narrow-canvas clamp instead of riding its exact edge.
const PANEL_FIXED_CELLS: usize = FIND_LABEL.len() + 2;
const PANEL_MIN_FIELD_CHARS: usize = 8;
/// Below this shaped-cell width the nav/actions rows drop their trailing
/// chord ANNOTATIONS — never their labels or buttons, which stay legible at
/// any width the card is ever clamped to. A narrow canvas still reads every
/// control; it just stops teaching the extra chord alongside it.
const PANEL_WIDE_CELLS: usize = 56;

impl TextPipeline {
    /// Shape + upload the top-right search panel for this frame: the opaque
    /// BASE_300 card, the panel text (calm BASE_CONTENT, or ERROR-red on the
    /// no-match state), the bordered field/button/checkbox boxes, the region
    /// separators, and the amber caret block at the focused field's end.
    /// Called from `prepare()` only when `search_active`.
    pub(in crate::render) fn prepare_panel(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) -> anyhow::Result<()> {
        self.panel_remetric();
        let shape = self.panel_shape_text(width);
        let (card_rect, text_left, text_top, caret_x) = self.panel_layout(
            width,
            shape.caret_byte,
            shape.caret_fallback_chars,
            shape.caret_row,
        );
        self.panel_upload_text(
            device, queue, width, height, &shape, card_rect, text_left, text_top,
        )?;
        self.panel_place_selection(device, queue, (width, height), &shape, text_left, text_top);
        self.panel_place_caret(queue, width, height, caret_x, text_top, shape.caret_row);
        Ok(())
    }

    /// Re-metric the shared panel buffer to the current zoom so its glyph
    /// line-height matches the caret/layout rects (which use m.line_height).
    fn panel_remetric(&mut self) {
        let m = self.metrics;
        self.panel_buffer
            .set_metrics(&mut self.font_system, m.glyph_metrics());
    }

    /// Compose + shape the labeled find/replace panel text into `panel_buffer`,
    /// returning the colors the card draws with and the FOCUSED field's
    /// reserved-caret-cell offsets. Also resolves every drawn CONTROL's
    /// shaped byte-span into `self.panel_control_spans` (read back by
    /// `panel_hit` and the sidecar's `panel_geometry`), so a click and a
    /// capture can never disagree with what this function just shaped.
    ///
    /// Row plan (each row a real shaped LINE, addressed by `f32` row index —
    /// `panel_rows`'s own contract):
    ///   * row 0 — **find**: label, the fixed-width windowed query, one
    ///     reserved caret cell. Always present.
    ///   * row 1 — **replace** (only once revealed): label, the windowed
    ///     replacement, one reserved caret cell. No trailing hint text — this
    ///     row's own width is what the responsive `field_chars` budget below
    ///     is computed against, so a fixed-length hint appended here would
    ///     ride outside that budget (see the nav row's own `Tab switch` hint
    ///     for why this matters).
    ///   * the **nav row** (`find    ` -> row 1, replace revealed -> row 2;
    ///     wraps to a second line under narrow pressure): the `N of M`
    ///     counter, the `^`/`v` step buttons, and the `Aa` match-case
    ///     checkbox with its `Match case` label — plus, at ordinary widths, a
    ///     `Tab switch` hint and, with no replace row up, the `Esc close`
    ///     hint (there is no actions row to carry it otherwise).
    ///   * the **actions row** (nav row + its line count, only once replace is
    ///     revealed; also wraps under narrow pressure): the `Replace` /
    ///     `Replace all` buttons and the `Esc close` hint.
    pub(in crate::render) fn panel_shape_text(&mut self, width: u32) -> PanelShape {
        let m = self.metrics;
        let no_match = self.search_no_matches();
        let ink = theme::base_content().to_glyphon();
        let muted = theme::muted().to_glyphon();
        let red = theme::error().to_glyphon();
        let total = self.search_matches.len();
        let n = self.search_current.map(|i| i + 1).unwrap_or(0);
        let query = self.search_query.clone();

        // The query never shapes as its raw, unbounded self: it is a fixed
        // visible field, scrolled/padded by `field_view_window`.
        let panel_text_w =
            (width as f32 - 2.0 * m.px_physical(PANEL_MARGIN) - 2.0 * m.px_physical(PANEL_PAD))
                .max(m.char_width);
        let panel_cells = (panel_text_w / m.char_width).floor() as usize;
        let field_chars = PANEL_FIELD_CHARS.min(
            panel_cells
                .saturating_sub(PANEL_FIXED_CELLS)
                .max(PANEL_MIN_FIELD_CHARS),
        );
        let (query_view, query_view_caret) =
            field_view_window(&query, self.search_query_caret, field_chars);

        let base = panel_attrs();
        let mk = |c| base.clone().color(c);
        // The macOS modifier glyphs (⌘ ⌥) in a chord label shape from the
        // bundled SYMBOL_FAMILY face (the display/mono faces render them as
        // tofu), the same treatment the overlay chord column gives them.
        let sym = |c| Attrs::new().family(Family::Name(SYMBOL_FAMILY)).color(c);
        // The query/replacement VALUE spans shape in a MONOSPACE family (never
        // the active world's proportional `base`), so `field_view_window`'s
        // fixed CHAR-COUNT contract yields a fixed PIXEL width too.
        let field = |c| Attrs::new().family(Family::Monospace).color(c);

        let replacement = self.search_replacement.clone();
        let (replacement_view, replacement_view_caret) =
            field_view_window(&replacement, self.search_replacement_caret, field_chars);
        let replace_active = self.search_replace_active;
        let editing_replacement = replace_active && self.search_editing_replacement;

        // Calm visual hierarchy via per-run color: muted labels, full-ink
        // query/replacement, and an "Aa" indicator that brightens from muted
        // to full ink when case-sensitivity is ON — state carried by VALUE,
        // never amber (the caret alone owns that accent).
        let (c_query, c_counter, c_toggle) = if no_match {
            (red, red, muted)
        } else if self.search_case_sensitive {
            (ink, muted, ink)
        } else {
            (ink, muted, muted)
        };
        let wide = panel_cells >= PANEL_WIDE_CELLS;
        let case_hint_on = self.search_case_sensitive && !no_match;

        let mut spans: Vec<(&str, Attrs)> = Vec::new();
        let mut controls = PanelControlSpans::default();

        // ROW 0 — FIND.
        spans.push((FIND_LABEL, mk(muted)));
        let find_start = FIND_LABEL.len();
        spans.push((query_view.as_str(), field(c_query)));
        let find_end = find_start + query_view.len();
        controls.find_field = Some(ControlSpan {
            row: 0.0,
            byte_start: find_start,
            byte_end: find_end,
        });
        spans.push((" ", mk(muted))); // the reserved caret cell

        // ROW 1 — REPLACE (only once revealed). No trailing hint text here:
        // this row's own width is what the responsive `field_chars` budget is
        // computed against, so any FIXED-length text appended after the
        // reserved caret cell rides for free on top of that budget and can
        // push the row past the narrow-canvas clamp `panel_layout` derives
        // from the very same width. The `Tab switch field` hint lives on the
        // nav row instead, which already carries its own wide/narrow wrap.
        let mut nav_row = 1.0_f32;
        if replace_active {
            spans.push(("\n", mk(muted)));
            spans.push((REPLACE_LABEL, mk(muted)));
            let rep_start = REPLACE_LABEL.len();
            spans.push((replacement_view.as_str(), field(ink)));
            let rep_end = rep_start + replacement_view.len();
            controls.replace_field = Some(ControlSpan {
                row: 1.0,
                byte_start: rep_start,
                byte_end: rep_end,
            });
            spans.push((" ", mk(ink))); // the reserved caret cell
            nav_row = 2.0;
        }

        // THE NAV ROW — counter, step buttons, match-case checkbox. At
        // ordinary widths this is ONE line; under narrow pressure the
        // checkbox (+ its label) moves to a SECOND line rather than letting
        // the line's natural width outgrow the card's own narrow-canvas
        // clamp (`panel_layout` sizes the card from the SHAPED rows, so a
        // row that does not shrink here would draw past the card it is
        // supposedly inside) — the same "wrap rather than overflow" policy
        // `field_view_window` already applies to the value fields.
        spans.push(("\n", mk(muted)));
        let counter = format!("{n} of {total}");
        spans.push((counter.as_str(), mk(c_counter)));
        let mut off = counter.len();
        spans.push(("  ", mk(muted)));
        off += 2;
        let prev_start = off;
        spans.push(("^", mk(ink)));
        off += 1;
        controls.nav_prev = Some(ControlSpan {
            row: nav_row,
            byte_start: prev_start,
            byte_end: off,
        });
        // A real gap between the two step buttons: each box outsets its own
        // tight glyph span by `CONTROL_BOX_PAD_X` on every side, so a single
        // reserved column between two one-glyph controls would let their
        // outset boxes touch or overlap.
        spans.push(("   ", mk(muted)));
        off += 3;
        let next_start = off;
        spans.push(("v", mk(ink)));
        off += 1;
        controls.nav_next = Some(ControlSpan {
            row: nav_row,
            byte_start: next_start,
            byte_end: off,
        });
        let case_row = if wide { nav_row } else { nav_row + 1.0 };
        if wide {
            spans.push(("   ", mk(muted)));
            off += 3;
        } else {
            spans.push(("\n", mk(muted)));
            off = 0;
        }
        let case_start = off;
        spans.push(("Aa", mk(c_toggle)));
        off += 2;
        controls.case_box = Some(ControlSpan {
            row: case_row,
            byte_start: case_start,
            byte_end: off,
        });
        const CASE_LABEL: &str = " Match case";
        spans.push((CASE_LABEL, mk(muted)));
        let case_hint_owned;
        if wide {
            case_hint_owned = format!(" {}", crate::keyspec::PANEL_MATCH_CASE.label());
            let case_hint_color = if case_hint_on { ink } else { muted };
            push_symbol_split(
                &mut spans,
                &case_hint_owned,
                move || mk(case_hint_color),
                move || sym(case_hint_color),
            );
        }
        let switch_hint_owned;
        if wide {
            switch_hint_owned = format!("   {} switch", crate::keyspec::PANEL_SWITCH_FIELD.label());
            push_symbol_split(&mut spans, &switch_hint_owned, || mk(muted), || sym(muted));
        }
        let nav_close_owned;
        if !replace_active {
            nav_close_owned = format!("   {} close", crate::keyspec::PANEL_CLOSE.label());
            push_symbol_split(&mut spans, &nav_close_owned, || mk(muted), || sym(muted));
        }
        let nav_lines = if wide { 1.0 } else { 2.0 };

        // THE ACTIONS ROW (only once replace is revealed): Replace / Replace
        // all, each a real click target with its own chord annotation, and
        // the `Esc close` hint. Same wrap policy as the nav row: one line at
        // ordinary widths, `Replace all` moves to a second line under narrow
        // pressure.
        let actions_row = nav_row + nav_lines;
        let replace_hint_owned;
        let replace_all_hint_owned;
        let actions_close_owned;
        if replace_active {
            spans.push(("\n", mk(muted)));
            let mut off2 = 0usize;
            const REPLACE_BTN: &str = "Replace";
            let rb_start = off2;
            spans.push((REPLACE_BTN, mk(ink)));
            off2 += REPLACE_BTN.len();
            controls.replace_button = Some(ControlSpan {
                row: actions_row,
                byte_start: rb_start,
                byte_end: off2,
            });
            spans.push((" ", mk(muted)));
            off2 += 1;
            if wide {
                replace_hint_owned = crate::keyspec::PANEL_REPLACE_NEXT.label();
                push_symbol_split(&mut spans, &replace_hint_owned, || mk(muted), || sym(muted));
                off2 += replace_hint_owned.len();
            }
            let replace_all_row = if wide {
                spans.push(("   ", mk(muted)));
                off2 += 3;
                actions_row
            } else {
                spans.push(("\n", mk(muted)));
                off2 = 0;
                actions_row + 1.0
            };
            const REPLACE_ALL_BTN: &str = "Replace all";
            let rab_start = off2;
            spans.push((REPLACE_ALL_BTN, mk(ink)));
            off2 += REPLACE_ALL_BTN.len();
            controls.replace_all_button = Some(ControlSpan {
                row: replace_all_row,
                byte_start: rab_start,
                byte_end: off2,
            });
            spans.push((" ", mk(muted)));
            if wide {
                replace_all_hint_owned = crate::keyspec::PANEL_REPLACE_ALL.label();
                push_symbol_split(
                    &mut spans,
                    &replace_all_hint_owned,
                    || mk(muted),
                    || sym(muted),
                );
            }
            actions_close_owned = format!("   {} close", crate::keyspec::PANEL_CLOSE.label());
            push_symbol_split(
                &mut spans,
                &actions_close_owned,
                || mk(muted),
                || sym(muted),
            );
        }
        let actions_lines = if replace_active {
            if wide { 1.0 } else { 2.0 }
        } else {
            0.0
        };

        let rows = actions_row + actions_lines;
        // Give the buffer generous width + one line height per row so it never wraps.
        self.panel_buffer.set_size(
            &mut self.font_system,
            Some(width as f32 * 2.0),
            Some(m.line_height * rows),
        );
        let default_attrs = base.clone().color(ink);
        self.panel_buffer.set_rich_text(
            &mut self.font_system,
            spans,
            &default_attrs,
            Shaping::Advanced,
            None,
        );
        self.panel_buffer
            .shape_until_scroll(&mut self.font_system, false);

        // Byte offset + char-prefix of the FOCUSED field's caret, at its OWN
        // CHAR-index position (`TextBox::caret`) — LINE-relative (cosmic-text
        // resets `LayoutGlyph::start` to 0 after every `\n`), computed
        // against the WINDOWED `query_view`/`replacement_view`, never the raw
        // field text, so the caret always lands on a real shaped glyph.
        let (caret_byte, caret_fallback_chars, caret_row) = if editing_replacement {
            (
                REPLACE_LABEL.len() + field_caret_byte(&replacement_view, replacement_view_caret),
                REPLACE_LABEL.chars().count() + replacement_view_caret,
                1.0_f32,
            )
        } else {
            (
                FIND_LABEL.len() + field_caret_byte(&query_view, query_view_caret),
                FIND_LABEL.chars().count() + query_view_caret,
                0.0_f32,
            )
        };
        // THE SELECTION BAND's own crossing, through the one owner beside
        // this file (`panel_selection`), asked of the SAME focused field the
        // caret row above was derived from.
        let (label, view, field_caret, field_len) = if editing_replacement {
            let caret = self.search_replacement_caret;
            (
                REPLACE_LABEL,
                &replacement_view,
                caret,
                replacement.chars().count(),
            )
        } else {
            let caret = self.search_query_caret;
            (FIND_LABEL, &query_view, caret, query.chars().count())
        };
        let selection_span = panel_selection_span(
            self.search_field_selection,
            label,
            view,
            field_caret,
            field_len,
            field_chars,
        );

        self.panel_control_spans = controls;

        PanelShape {
            no_match,
            ink,
            red,
            caret_byte,
            caret_fallback_chars,
            caret_row,
            selection_span,
        }
    }
}
