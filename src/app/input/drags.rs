//! src/app/input/drags.rs — the RESIZE drag state machines: the page-column
//! width drag (`begin/on/apply/end_page_resize`) and the inline-image
//! edge/corner drag-resize (`begin/on/apply/end_image_resize`, incl. the
//! [`ImageDrag`] snapshot they carry press-to-release). Split out of the
//! former `app/input.rs` monolith (2026-07 code-organization pass); see
//! `mouse` for the press/hover ARMING of these drags (`begin_*_if_hovering`
//! is called from `on_mouse_input`) and `keys` for the keyboard path.

use crate::app::*;

/// INLINE-IMAGE DRAG-RESIZE (v2, live app only): the in-flight state of an
/// edge/corner drag on an inline image. Snapshotted at press
/// ([`App::begin_image_resize_if_hovering`]) and carried until release: the image's
/// document byte `range` (the `![alt](path)` span — the write-back target), the
/// grabbed `handle` (which edge/corner drives the width) + the image's PRESS-TIME
/// on-screen `rect` (`[left, top, w, h]` — the fixed anchors + aspect the width math
/// reads), and the current live-preview `width` (pipeline state, NOT a buffer edit,
/// until the release stamps the `|NNN` hint back as one undoable edit).
#[derive(Clone, Copy, Debug)]
pub(crate) struct ImageDrag {
    /// Document byte range of the `![alt](path)` image span (write-back target).
    pub(crate) range: (usize, usize),
    /// Which edge/corner is being dragged (picks the drive axis + anchor).
    pub(crate) handle: crate::render::ImageHandle,
    /// PRESS-TIME on-screen rect `[left, top, w, h]` — the fixed anchors + aspect.
    pub(crate) rect: [f32; 4],
    /// The current live-preview DISPLAY WIDTH (px); rounded to the `|NNN` hint on release.
    pub(crate) width: f32,
}

/// ITEM 94 — SETTINGS RANGE SCRUB (live app only): the in-flight state of a drag
/// on a range row's rail. Snapshotted at press ([`App::begin_range_drag`]) and
/// carried until release: WHICH setting is being scrubbed (its typed
/// [`crate::settings::SettingId`] — the key into the ONE range spec) and the
/// track's px ENDS `(x0, x1)` at press time.
///
/// The ends are snapshotted for the same reason the page drag snapshots its
/// opposite edge: the rail is laid out relative to the value TEXT beside it, and
/// that text changes width as the value changes (`"80%"` -> `"100%"`) — re-reading
/// the live rail each move would shift the track under a stationary pointer and
/// make the scrub creep. One press, one scale, the whole gesture.
#[derive(Clone, Copy, Debug)]
pub(crate) struct RangeDrag {
    /// Which range setting is being scrubbed (the range-spec key).
    pub(crate) id: crate::settings::SettingId,
    /// The `items` index of the row being scrubbed (the row whose cell mirrors).
    pub(crate) item: usize,
    /// The track's px ends at PRESS — the fixed scale every move resolves against.
    pub(crate) x0: f32,
    pub(crate) x1: f32,
}

impl App {
    /// ITEM 94 — BEGIN a rail scrub: a left press that landed on a range row's rail
    /// (the generous hit band around the visually small thumb). Selects the row,
    /// applies the pressed step IMMEDIATELY (a click IS a set — pointing, not
    /// buttons), and arms the drag. Returns whether the press was a rail press, so
    /// `overlay_click` can skip its ordinary row-accept.
    ///
    /// NEVER persists: the whole gesture — the initial click and every resolved
    /// step of the drag that may follow — writes config exactly ONCE, in
    /// [`Self::end_range_drag`].
    pub(in crate::app) fn begin_range_drag(&mut self) -> bool {
        let (px, py) = self.cursor_px;
        let Some((item, frac)) = self
            .gpu
            .as_ref()
            .and_then(|g| g.pipeline.overlay_range_at(px, py))
        else {
            return false;
        };
        // Select the row the rail belongs to (a rail press is also a selection —
        // the same row Enter would then act on).
        if let Some(ov) = self.workspace_state.overlay_mut()
            && item < ov.items.len()
        {
            ov.selected = item;
        }
        let Some(cell) = self
            .workspace_state
            .overlay()
            .and_then(|ov| ov.range_of_item(item))
        else {
            return false;
        };
        // The track's own px ends, snapshotted for the whole gesture (see the
        // struct's doc). Falls back to nothing when the rail vanished between the
        // hit-test and here (it cannot, but a missing scale must not scrub at 0).
        let Some((x0, x1)) = self
            .gpu
            .as_ref()
            .and_then(|g| g.pipeline.overlay_range_scale(item))
        else {
            return false;
        };
        self.range_drag = Some(RangeDrag {
            id: cell.id,
            item,
            x0,
            x1,
        });
        self.apply_range_frac(frac);
        true
    }

    /// ITEM 94 — LIVE rail scrub step: resolve the pointer's x against the
    /// PRESS-TIME track scale and apply that step. No persist (see
    /// [`Self::end_range_drag`]).
    pub(in crate::app) fn on_range_drag(&mut self) {
        let Some(drag) = self.range_drag else { return };
        let frac = crate::render::rail_frac_at(self.cursor_px.0, drag.x0, drag.x1);
        self.apply_range_frac(frac);
    }

    /// THE ONE POINTER→VALUE STEP, shared by the initial click and every drag move:
    /// resolve `frac` to a value through the range SPEC (never a parallel
    /// computation), apply it through the setting's own live owner, and mirror the
    /// new readout + thumb into the still-open menu's row. Idempotent — re-applying
    /// the same fraction changes nothing, which is what makes a fast drag and a slow
    /// one settle identically.
    fn apply_range_frac(&mut self, frac: f32) {
        let Some(drag) = self.range_drag else { return };
        let Some(spec) = crate::settings::range_spec(drag.id) else {
            return;
        };
        let value = spec.value_at_frac(frac);
        self.range_apply_live(drag.id, value);
        let (step, readout) = (spec.step_of(value), spec.format(value));
        if let Some(ov) = self.workspace_state.overlay_mut() {
            // The scrubbed row STAYS the selected row for the whole gesture (the
            // drag owns the pointer, so no hover can steal the highlight
            // mid-scrub) — re-pinned from the press-time snapshot, never
            // re-derived from wherever the selection happens to be now.
            if drag.item < ov.items.len() {
                ov.selected = drag.item;
            }
            ov.set_selected_range(step, readout);
        }
        self.sync_view(true);
        if let Some(gpu) = self.gpu.as_ref() {
            gpu.window.request_redraw();
        }
    }

    /// ITEM 94 — FINISH a rail scrub on button RELEASE: drop the drag state and
    /// PERSIST the settled value EXACTLY ONCE (the sticky write the keyboard's
    /// discrete step does per step; a drag defers it to here so a 120 Hz scrub
    /// writes one line, not hundreds). Also refreshes the still-open menu from the
    /// live values, so the cell the drag mirrored and the config now agree.
    pub(in crate::app) fn end_range_drag(&mut self) {
        let Some(drag) = self.range_drag.take() else {
            return;
        };
        if let Some(key) = crate::settings::value_key(drag.id) {
            self.range_persist(key);
        }
        self.refresh_settings_overlay();
        if let Some(gpu) = self.gpu.as_ref() {
            gpu.window.request_redraw();
        }
    }

    /// If a left press landed ON a page-column edge, begin a DIRECT page-width resize
    /// drag (symmetric about center) instead of a text selection, and snap the edge to
    /// the press x — UNLESS it's the SECOND click of a DOUBLE-CLICK on the edge, in
    /// which case it RESETS the page width to the built-in default instead
    /// (pointing-not-buttons — the same affordance games/DAWs use on a divider for
    /// "back to default"). Returns whether the edge press was handled (so the caller
    /// skips `on_press`). Shares the SAME multi-click detection `on_press` uses
    /// (`bump_click_count`), so a double-click on the edge is recognized exactly like
    /// a double-click anywhere else in the document. LIVE-ONLY gesture; the hover
    /// test + measure math + the reset action itself are unit-tested.
    pub(in crate::app) fn begin_page_resize_if_hovering(
        &mut self,
        exit: &dyn schedule::Exit,
    ) -> bool {
        let edge = self
            .gpu
            .as_ref()
            .and_then(|g| g.pipeline.page_resize_edge_at(self.cursor_px.0));
        let Some(edge) = edge else {
            return false;
        };
        // A resize (or a reset) is a non-edit gesture either way: seal the open
        // undo group like a click does, before branching.
        self.active.buffer.seal_undo_group();
        if self.bump_click_count() == 2 {
            // DOUBLE-CLICK on the draggable edge: reset instead of beginning a drag.
            // Routes through the real Action via `App::apply`, so it is the exact
            // same path the palette command and a rebound `--keys` chord take. A direct
            // gesture is the fast path — `Door::Chord` for the ledger (Reset page width
            // has no native chord anyway, so it never surfaces as a candidate).
            self.apply(
                crate::keymap::Action::PageReset,
                false,
                exit,
                crate::stats::Door::Chord,
            );
            return true;
        }
        self.page_resizing = true;
        self.page_resize_edge = Some(edge);
        // STABLE REFERENCE: snapshot the OPPOSITE edge's position ONCE, now, and hold it
        // for the whole drag. The grabbed edge tracks the pointer against this fixed
        // anchor (`geometry::page_resize_measure_anchored`), so the measure stays
        // monotone. Reading the current adaptively-shifted edge each frame instead fed
        // the rail-hide shift back into the measure and oscillated it across the boundary.
        self.page_resize_anchor = self.gpu.as_ref().map(|g| {
            let left = g.pipeline.column_left();
            match edge {
                crate::render::ResizeEdge::Right => left,
                crate::render::ResizeEdge::Left => left + g.pipeline.column_width(),
            }
        });
        // The context flipped to "dragging the edge" WITHOUT any mouse motion: recompute
        // the cursor shape right now (`dragging_edge` outranks everything), not just on
        // the next `CursorMoved`.
        self.sync_cursor_icon();
        self.apply_page_resize();
        true
    }

    /// LIVE page-width drag step: re-derive the measure from the pointer and re-wrap.
    /// Only the release (`end_page_resize`) persists the sticky width.
    pub(in crate::app) fn on_page_resize_drag(&mut self) {
        if !self.page_resizing {
            return;
        }
        self.apply_page_resize();
    }

    /// Set the page MEASURE from the current pointer x (symmetric about the window
    /// center, clamped to the band), re-wrap the buffer at the new column width, and
    /// redraw. Shared by the initial press + every drag move. Re-wrap mirrors the
    /// `PageWider`/`PageNarrower` command path (`set_size` reshapes at the new width).
    fn apply_page_resize(&mut self) {
        let anchor = self.page_resize_anchor;
        let target = self
            .page_resize_edge
            .zip(anchor)
            .and_then(|(edge, anchor_x)| {
                self.gpu.as_ref().map(|g| {
                    g.pipeline
                        .page_resize_measure_at(self.cursor_px.0, edge, anchor_x)
                })
            });
        if let Some(target) = target
            && target != crate::page::measure()
        {
            crate::page::set_measure(target);
            if let Some(gpu) = self.gpu.as_mut() {
                let (w, h) = (gpu.config.width as f32, gpu.config.height as f32);
                gpu.pipeline.set_size(w, h);
            }
            self.sync_view(true);
        }
        let (px, py) = self.input.resting_pointer().px();
        if let Some(gpu) = self.gpu.as_mut() {
            // DRAG READOUT: a quiet muted char-count near the pointer while the edge
            // is held (Butterick's line-length rule made visible) — live for the
            // whole gesture (press through every move); cleared on release.
            gpu.pipeline
                .set_page_drag_readout(Some((px, py, crate::page::measure())));
            gpu.window.request_redraw();
        }
    }

    /// Finish a page-width resize on button RELEASE: drop the drag flag and PERSIST the
    /// settled width (sticky, exactly like the C-x } / C-x { keyboard commands).
    pub(in crate::app) fn end_page_resize(&mut self) {
        self.page_resizing = false;
        self.page_resize_edge = None;
        self.page_resize_anchor = None;
        self.persist_page_width();
        if let Some(gpu) = self.gpu.as_mut() {
            // Drop the drag readout — gone the instant the edge is released.
            gpu.pipeline.set_page_drag_readout(None);
            gpu.window.request_redraw();
        }
        // The context flipped off "dragging the edge" WITHOUT any mouse motion:
        // recompute now (usually resumes the edge-hover or plain-text shape rather
        // than waiting for the next `CursorMoved`).
        self.sync_cursor_icon();
    }

    /// If a left press landed ON an inline image's resize EDGE/CORNER, begin a DIRECT
    /// drag-resize of that image (its width tracks the pointer, previewed live without
    /// touching the buffer) instead of a text selection. Returns whether the handle
    /// press was handled (so the caller skips the page-resize / doc-click path).
    /// Mirrors [`Self::begin_page_resize_if_hovering`]: seal the open undo group (a
    /// resize is a non-edit gesture until the release), record the drag, flip the
    /// cursor shape now, and apply the first preview step. LIVE-ONLY gesture; the hover
    /// hit-test + width math + the write-back are unit-tested.
    pub(in crate::app) fn begin_image_resize_if_hovering(&mut self) -> bool {
        let (px, py) = self.cursor_px;
        // The hit-test lives on the pipeline (where the images layout + the pure
        // `geometry::image_handle_hit` live), mirroring `page_resize_hover` — no raw
        // geometry leaks to the app. Returns the hit image's byte range, the grabbed
        // edge/corner, and the press-time rect (the width math's anchors).
        let hit = self
            .gpu
            .as_ref()
            .and_then(|g| g.pipeline.image_handle_at(px, py));
        let Some((range, handle, rect)) = hit else {
            return false;
        };
        // A resize is a non-edit gesture: seal the open undo group like a click does,
        // so the single write-back on release is its own clean undo entry.
        self.active.buffer.seal_undo_group();
        // `width` is a placeholder; `apply_image_resize` below sets it from the pointer.
        self.image_resizing = Some(ImageDrag {
            range,
            handle,
            rect,
            width: 0.0,
        });
        // The context flipped to "dragging an image" WITHOUT any mouse motion:
        // recompute the cursor shape now, not just on the next `CursorMoved`.
        self.sync_cursor_icon();
        self.apply_image_resize();
        true
    }

    /// LIVE image drag-resize step: re-derive the display width from the pointer and
    /// preview it. Only the release ([`Self::end_image_resize`]) writes the buffer.
    pub(in crate::app) fn on_image_resize_drag(&mut self) {
        if self.image_resizing.is_none() {
            return;
        }
        self.apply_image_resize();
    }

    /// Set the dragged image's live-preview DISPLAY WIDTH from the current pointer
    /// (driven by the grabbed edge/corner off the press-time rect, clamped to
    /// `[MIN_IMAGE_W, wrap]`), push it to the pipeline as a preview override (NOT a
    /// buffer edit), re-fit + redraw. Shared by the initial press + every drag move.
    /// The re-fit mirrors the page-resize dance: the pipeline's `set_image_preview`
    /// marks itself dirty so the next `sync_view` forces the reshape that re-runs the
    /// image layout at the new width.
    fn apply_image_resize(&mut self) {
        let Some(drag) = self.image_resizing else {
            return;
        };
        let pointer = self.cursor_px;
        let width = self.gpu.as_ref().map(|g| {
            g.pipeline
                .image_resize_width_at(drag.handle, drag.rect, pointer)
        });
        let Some(width) = width else {
            return;
        };
        if let Some(d) = self.image_resizing.as_mut() {
            d.width = width;
        }
        if let Some(gpu) = self.gpu.as_mut() {
            gpu.pipeline
                .set_image_preview(Some((drag.range.0, drag.range.1, width)));
        }
        self.sync_view(false);
        if let Some(gpu) = self.gpu.as_ref() {
            gpu.window.request_redraw();
        }
    }

    /// Finish an image drag-resize on button RELEASE: clear the drag flag + the
    /// pipeline preview, then WRITE the settled `|NNN` width hint back into the image's
    /// alt as ONE undoable edit ([`Self::write_back_image_width`]). Mirrors
    /// [`Self::end_page_resize`]'s clear-then-persist shape.
    pub(in crate::app) fn end_image_resize(&mut self) {
        let Some(drag) = self.image_resizing.take() else {
            return;
        };
        if let Some(gpu) = self.gpu.as_mut() {
            // Drop the live preview — the committed `|NNN` hint drives the fit now.
            gpu.pipeline.set_image_preview(None);
        }
        self.write_back_image_width(drag.range, drag.width);
        self.sync_view(false);
        // The context flipped off "dragging an image" WITHOUT any mouse motion.
        self.sync_cursor_icon();
        if let Some(gpu) = self.gpu.as_ref() {
            gpu.window.request_redraw();
        }
    }
}
