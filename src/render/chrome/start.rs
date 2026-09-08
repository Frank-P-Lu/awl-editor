//! Honest no-document chrome: its two actions, shaping, and shared hit geometry.

use super::*;

const START_ACTIONS: [&str; 2] = ["New document", "Go to"];
/// The two actions' real bound chords (`assets/keymap-defaults.toml`'s
/// `new_document`/`go_to` slugs — Cmd-N, Cmd-O), hardcoded as mac glyphs like
/// every other render-side hint string (`panel.rs`'s replace-hint, `whichkey.rs`):
/// this layer never re-derives a convention-aware label. Bare Enter is NOT one of
/// these — `resolve.rs` sends it to `Action::Newline`, and `no_document.rs::
/// reject_without_document` rejects anything but `NewDocument`/`OpenGoto`/`Quit`
/// with no document open, so a `↵` glyph here would draw a control that does
/// nothing.
const START_CHORDS: [&str; 2] = ["\u{2318}N", "\u{2318}O"];

/// The convention-truthful label for the folder line's Go-to chord — routed
/// through [`crate::keytoken::key_token_label`], the SAME owner the starting
/// docs' `{{key:go_to}}` tokens resolve through, rather than a literal glyph
/// like [`START_CHORDS`] above (an existing, documented debt this line does
/// not repeat): a hardcoded `⌘O` reads wrong on Linux and under a browser's
/// reserved-chord substitution. `""` on the (should-never-happen) chance the
/// resolver has nothing to show for a live catalog action — never a bogus
/// placeholder glyph.
pub(crate) fn goto_chord_label() -> String {
    crate::keytoken::key_token_label(
        "go_to",
        crate::convention::Convention::current(),
        crate::commands::Platform::current(),
    )
    .unwrap_or_default()
}

/// The two action rows' geometry, ALWAYS exactly [`START_ACTIONS`].len() long
/// regardless of `folder_line` — a click only ever lands on "New document" or
/// "Go to", never on the folder-naming line above them. `folder_line` only
/// shifts the block down to leave room for that extra line, so
/// [`TextPipeline::start_action_at`] and [`TextPipeline::prepare_start_surface`]
/// agree on where the two clickable rows actually sit.
fn start_rows(width: f32, height: f32, row_h: f32, folder_line: bool) -> [[f32; 4]; 2] {
    let extra_rows = if folder_line { 1 } else { 0 };
    let block_h = row_h * (START_ACTIONS.len() + extra_rows) as f32;
    let top = ((height - block_h) * 0.5).max(0.0) + row_h * extra_rows as f32;
    // The folder line's own text (name + convention chord + "Go to") runs
    // longer than either bare action row, especially under
    // `Convention::Linux`'s word-labelled chords ("Ctrl+O" vs "⌘O") — a box
    // sized for the SHORT action rows alone let that line wrap onto a
    // second visual row, silently turning "one dim line" into two and
    // pushing the row below it toward (or past) the buffer's own allocated
    // height. Widen the shared box when a folder line is drawn; the two
    // action rows only centre within more room, which does not affect
    // their own click geometry (still centred, still exactly `row_h` tall).
    let row_w = if folder_line {
        (width * 0.6).clamp(260.0, 520.0)
    } else {
        (width * 0.4).clamp(180.0, 360.0)
    };
    let left = (width - row_w) * 0.5;
    [[left, top, row_w, row_h], [left, top + row_h, row_w, row_h]]
}

/// A folder name is an OS path component, not bounded prose — cap it so the
/// folder line can never wrap regardless of how [`start_rows`] sized the box.
/// Character-count truncation (not byte-count) so a multi-byte folder name
/// is never cut mid-codepoint.
const FOLDER_NAME_CHAR_CAP: usize = 40;

fn truncated_folder_name(name: &str) -> std::borrow::Cow<'_, str> {
    if name.chars().count() <= FOLDER_NAME_CHAR_CAP {
        std::borrow::Cow::Borrowed(name)
    } else {
        let head: String = name.chars().take(FOLDER_NAME_CHAR_CAP).collect();
        std::borrow::Cow::Owned(format!("{head}\u{2026}"))
    }
}

impl TextPipeline {
    pub(in crate::render) fn prepare_start_surface(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) -> anyhow::Result<()> {
        self.gutter_stack_plate
            .prepare(device, queue, width, height, &[]);
        let label = crate::markdown::type_scale::LABEL;
        let row_h = self.metrics.line_height * label;
        self.gutter_buffer.set_metrics(
            &mut self.font_system,
            GlyphMetrics::new(self.metrics.font_size * label, row_h),
        );
        let folder = self.start_folder.clone();
        let has_folder = folder.is_some();
        let rows = start_rows(width as f32, height as f32, row_h, has_folder);
        let [left, top, row_w, _] = rows[0];
        let block_top = top - if has_folder { row_h } else { 0.0 };
        let block_rows = if has_folder { 3.0 } else { 2.0 };
        self.gutter_buffer.set_size(
            &mut self.font_system,
            Some(row_w),
            Some(row_h * block_rows + 1.0),
        );
        // Widening the box (above) narrows the wrap risk but does
        // not close it — a folder name long enough (still short of
        // `FOLDER_NAME_CHAR_CAP`) on a narrow window can still overflow it.
        // WRAPPING is the actually unacceptable outcome here: it silently
        // turns "one dim line" into two and pushes the action row below it
        // down (found live, under `AWL_CONVENTION_FORCE=linux`'s longer
        // "Ctrl+O" word label). Disabling wrap trades that for a stray name
        // simply running past its box on an extreme name/window
        // combination — a far smaller defect, and one `truncated_folder_name`
        // already bounds. Reset to the ordinary wrap on the no-folder path,
        // since this buffer is reused across calls.
        self.gutter_buffer.set_wrap(
            &mut self.font_system,
            if has_folder {
                Wrap::None
            } else {
                Wrap::WordOrGlyph
            },
        );
        let base = panel_attrs();
        // BOTH actions read in the SAME full ink — hierarchy is order alone, not
        // ink (DECIDED: neither reads as disabled). Each row's chord rides beside
        // its verb in the established quiet-chord/full-ink-verb split every
        // secondary-column reads (`shape_overlay_right`'s `ink`-primary/
        // `muted`-chord pairing): the chord glyph through `push_symbol_split` (⌘
        // is `is_symbol`, tofu on the display face without the symbol-family
        // split) in `muted`, the verb that follows in `ink`.
        let ink = theme::base_content().to_glyphon();
        let muted = theme::muted().to_glyphon();
        let sym = |c| Attrs::new().family(Family::Name(SYMBOL_FAMILY)).color(c);
        let mut spans: Vec<(&str, glyphon::Attrs)> = Vec::new();
        // The open folder's own name, ONE dim line above the two
        // actions (never a panel) — entirely in `muted`, chord glyph included,
        // so it reads as quiet CONTEXT rather than a third action competing
        // with the ink-bearing rows below. `folder`/`chord` are declared here
        // (rather than inline) so their owned `String`s outlive `spans`'
        // borrows through to `set_rich_text` below.
        let folder_prefix;
        let chord_owned;
        if let Some(name) = &folder {
            chord_owned = goto_chord_label();
            folder_prefix = format!("{}    ", truncated_folder_name(name));
            spans.push((folder_prefix.as_str(), base.clone().color(muted)));
            push_symbol_split(
                &mut spans,
                &chord_owned,
                || base.clone().color(muted),
                || sym(muted),
            );
            spans.push((" Go to\n", base.clone().color(muted)));
        }
        push_symbol_split(
            &mut spans,
            START_CHORDS[0],
            || base.clone().color(muted),
            || sym(muted),
        );
        let verb0 = format!(" {}\n", START_ACTIONS[0]);
        spans.push((verb0.as_str(), base.clone().color(ink)));
        push_symbol_split(
            &mut spans,
            START_CHORDS[1],
            || base.clone().color(muted),
            || sym(muted),
        );
        let verb1 = format!(" {}", START_ACTIONS[1]);
        spans.push((verb1.as_str(), base.clone().color(ink)));
        self.gutter_buffer.set_rich_text(
            &mut self.font_system,
            spans,
            &base.color(ink),
            Shaping::Advanced,
            Some(glyphon::cosmic_text::Align::Center),
        );
        self.gutter_buffer
            .shape_until_scroll(&mut self.font_system, false);
        let area = TextArea {
            buffer: &self.gutter_buffer,
            left,
            top: block_top,
            scale: 1.0,
            bounds: TextBounds {
                left: 0,
                top: 0,
                right: width as i32,
                bottom: height as i32,
            },
            default_color: ink,
            custom_glyphs: &[],
        };
        self.gutter_renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                [area],
                &mut self.swash_cache,
            )
            .map_err(|e| anyhow::anyhow!("glyphon start-surface prepare failed: {e:?}"))
    }

    pub fn start_action_at(&self, x: f32, y: f32) -> Option<crate::keymap::Action> {
        if self.document_active {
            return None;
        }
        let row_h = self.metrics.line_height * crate::markdown::type_scale::LABEL;
        let rows = start_rows(
            self.window_w,
            self.window_h,
            row_h,
            self.start_folder.is_some(),
        );
        let row = rows.iter().position(|[rx, ry, rw, rh]| {
            x >= *rx && x <= *rx + *rw && y >= *ry && y <= *ry + *rh
        })?;
        Some(match row {
            0 => crate::keymap::Action::NewDocument,
            1 => crate::keymap::Action::OpenGoto,
            _ => unreachable!("start surface has exactly two rows"),
        })
    }

    pub fn start_actions(&self) -> &'static [&'static str] {
        if self.document_active {
            &[]
        } else {
            &START_ACTIONS
        }
    }

    pub fn document_active(&self) -> bool {
        self.document_active
    }

    /// The open folder's own name for the no-document start surface's dim
    /// line — `None` when there is nothing worth naming (mirrors
    /// [`crate::render::ViewState::start_folder`]; see that field's doc for
    /// the shared-owner rationale). Read by the capture sidecar's
    /// `document.start_folder` so a capture can verify exactly what this
    /// surface named.
    pub fn start_folder(&self) -> Option<&str> {
        self.start_folder.as_deref()
    }

    /// The folder-naming dim line's own rect (`None` when
    /// [`Self::start_folder`] is `None`) — the EXACT geometry
    /// `prepare_start_surface` draws it at, exposed so a pixel law can sample
    /// precisely this line rather than re-deriving its own geometry and
    /// drifting from what the surface actually draws. `width`/`height` must
    /// match the values just passed to `prepare_start_surface`. Test-only: no
    /// production caller needs this geometry back.
    #[cfg(test)]
    pub(in crate::render) fn folder_line_rect(&self, width: u32, height: u32) -> Option<[f32; 4]> {
        self.start_folder.as_ref()?;
        let row_h = self.metrics.line_height * crate::markdown::type_scale::LABEL;
        let rows = start_rows(width as f32, height as f32, row_h, true);
        let [left, action0_top, row_w, _] = rows[0];
        Some([left, action0_top - row_h, row_w, row_h])
    }

    pub(in crate::render) fn background_bounds(&self, width: u32) -> (f32, f32) {
        if !self.document_active {
            return (0.0, 0.0);
        }
        let (page_on, _measure, col_left, col_w) = self.page_geometry();
        if page_on {
            (col_left, col_w)
        } else {
            (0.0, width as f32)
        }
    }
}
