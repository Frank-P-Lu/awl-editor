//! THE ONE OWNER of pointer-derived render state: the OS cursor icon, the
//! fold-chevron hover reveal, the working-set stack's close-mark hover, the
//! format popover's hover ring, and the inline-image resize-handle hover grip
//! (the last two, item 483 — hover feedback drawn only where which-control-
//! fires or that-a-control-exists is genuinely ambiguous). Every door that
//! can change what these five answer for — a real `CursorMoved`, a click, a
//! drag beginning or ending, an overlay opening or closing, a modifier
//! change, a wheel scroll/zoom, or a keyboard action that scrolls or moves
//! the caret off-screen — recomputes all five together through
//! [`App::resync_pointer_derived_state`] (the pointer itself moved: the door
//! is its own evidence) or
//! [`App::resync_pointer_derived_state_if_geometry_changed`] (the pointer
//! didn't move; `sync_view`'s one frame-level seam asks whether the geometry
//! moved under it instead). The five components stay private to this module
//! tree — this file is the only door in.
//!
//! # Why one seam instead of a fourth door-patch
//!
//! Before this, `CursorMoved` recomputed all three; a handful of click/drag/
//! overlay doors hand-patched the cursor icon alone; wheel scroll, wheel zoom
//! and every keyboard-driven scroll called none of them — so the OS cursor
//! shape and the revealed chevron stayed pinned to whatever was under the
//! pointer before the content moved under it. `sync_view` already runs after
//! essentially every mutating action (it is the render push every action ends
//! on) and already knows the frame's settled scroll/zoom/document, so it is
//! the one place that can ask "did the geometry under an unmoved pointer just
//! change" without a per-door special case.

use crate::app::*;

/// The geometry facts that decide what sits under an UNMOVED pointer: which
/// document, at which edit, scrolled how far, at what viewport height and
/// zoom. The document is identified by its stable in-memory address —
/// cheap and allocation-free, unlike the registry's path-based
/// `BufferKey` (which can canonicalize against disk) — so a per-keystroke
/// `sync_view` call never pays for a filesystem round trip just to ask
/// "is this still the same buffer". Two buffers that happen to share a
/// version number (both freshly opened, both at version 0) are still told
/// apart by address, so a swap between them is never read as "nothing
/// changed" — the exact version-only collision CLAUDE.md's cache-key
/// discipline warns about.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(in crate::app::input) struct PointerGeometry {
    buffer: usize,
    version: u64,
    scroll: crate::render::ScrollPos,
    height: u32,
    zoom_bits: u32,
}

impl App {
    fn pointer_geometry(&self) -> Option<PointerGeometry> {
        if !self.document.has_active() {
            return None;
        }
        let gpu = self.frame.gpu()?;
        let buffer = self.document.buffer();
        Some(PointerGeometry {
            buffer: buffer as *const crate::buffer::Buffer as usize,
            version: buffer.version(),
            scroll: self.document.scroll(),
            height: gpu.config.height,
            zoom_bits: self.frame.zoom().to_bits(),
        })
    }

    /// THE OWNER. Recomputes the fold-chevron hover, the working-set stack
    /// hover, and the OS cursor icon from the pointer's CURRENT position —
    /// unconditionally, for a door that already knows something changed
    /// (a real `CursorMoved`, or a discrete state transition like a click,
    /// a drag edge, or an overlay opening/closing).
    pub(in crate::app) fn resync_pointer_derived_state(&mut self) {
        self.update_fold_hover();
        let (px, py) = self.input.pointer.cursor_px;
        let stack_hover_changed = self.frame.gpu_mut().is_some_and(|gpu| {
            gpu.pipeline
                .resolve_gutter_stack_hover(px, py, gpu.config.height)
        });
        // FORMAT POPOVER hover ring + inline-image resize-handle grip (item
        // 483): each reads the SAME hit-test the cursor-icon path already
        // does (`popover_hit` / `image_handle_at`), so the drawn
        // acknowledgement can never disagree with a clickable target.
        let popover_hover_changed = self
            .frame
            .gpu_mut()
            .is_some_and(|gpu| gpu.pipeline.resolve_popover_hover(px, py));
        let image_hover_changed = self
            .frame
            .gpu_mut()
            .is_some_and(|gpu| gpu.pipeline.resolve_image_hover(px, py));
        if stack_hover_changed || popover_hover_changed || image_hover_changed {
            self.request_frame();
        }
        self.sync_cursor_icon();
        self.input.pointer.resynced_geometry = self.pointer_geometry();
    }

    /// `sync_view`'s own door: recompute pointer-derived state only when the
    /// geometry it depends on actually moved since the last recompute —
    /// scroll, viewport height, zoom, or the active document itself. Cheap (a
    /// handful of `Copy` field comparisons against facts `sync_view` already
    /// has in hand — no extra geometry query), so a `sync_view` pass that
    /// changed none of them (an unrelated overlay field, a notice) skips
    /// every hit-test outright. Safe to call from every `sync_view` pass for
    /// the same reason: `sync_view` is event-driven, never polled per
    /// animation frame (a caret-spring settle or a theme crossfade advances
    /// through the redraw loop directly, never through here — see the module
    /// doc), so this can never fire on an animation's behalf.
    pub(in crate::app) fn resync_pointer_derived_state_if_geometry_changed(&mut self) {
        if self.pointer_geometry() == self.input.pointer.resynced_geometry {
            return;
        }
        self.resync_pointer_derived_state();
    }

    /// Clear EVERY pointer-hover render fact — never just one. The two
    /// lifecycle edges that lose the pointer outright (the OS cursor leaving
    /// the window, the window losing focus) route through this one owner so
    /// none can independently forget another, the way `on_cursor_left` once
    /// forgot the chevron and `on_focus_lost` forgot both.
    pub(in crate::app) fn clear_pointer_hover_state(&mut self) -> bool {
        let Some(gpu) = self.frame.gpu_mut() else {
            return false;
        };
        let chevron_cleared = gpu.pipeline.set_hover_line(None);
        let stack_cleared = gpu.pipeline.clear_gutter_stack_hover();
        let popover_cleared = gpu.pipeline.clear_popover_hover();
        let image_cleared = gpu.pipeline.clear_image_hover();
        chevron_cleared || stack_cleared || popover_cleared || image_cleared
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> PointerGeometry {
        PointerGeometry {
            buffer: 1,
            version: 0,
            scroll: crate::render::ScrollPos::default(),
            height: 800,
            zoom_bits: 1.0_f32.to_bits(),
        }
    }

    #[test]
    fn geometry_signature_is_sensitive_to_scroll() {
        // THE CORE OF THE FIX: this is the exact field a wheel scroll or a
        // keyboard-driven caret-chase changes without ever moving the
        // pointer — if the signature didn't compare it, the gated resync
        // would never fire for either.
        let a = base();
        let b = PointerGeometry {
            scroll: crate::render::ScrollPos::at_row(5),
            ..a
        };
        assert_ne!(a, b, "a scroll change must change the signature");
    }

    #[test]
    fn geometry_signature_is_sensitive_to_height_and_zoom() {
        let a = base();
        assert_ne!(
            a,
            PointerGeometry { height: 801, ..a },
            "a viewport resize must change the signature"
        );
        assert_ne!(
            a,
            PointerGeometry {
                zoom_bits: 1.1_f32.to_bits(),
                ..a
            },
            "a wheel-zoom step must change the signature"
        );
    }

    #[test]
    fn geometry_signature_is_sensitive_to_buffer_identity_not_just_its_version() {
        // TWO DIFFERENT BUFFERS can share a version (both freshly opened,
        // both at version 0): the signature must still tell them apart by
        // address, or switching between them at the same scroll/height/zoom
        // would read as "nothing changed" and leave the OLD document's
        // chevron hover lit on the new one.
        let a = base();
        let b = PointerGeometry { buffer: 2, ..a };
        assert_ne!(
            a, b,
            "two same-versioned buffers must not collide in the signature"
        );
    }

    #[test]
    fn identical_geometry_compares_equal() {
        // The other half of the dirty-check: no spurious difference, or
        // every ordinary keystroke would pay for a pointless re-hit-test.
        assert_eq!(base(), base());
    }
}
