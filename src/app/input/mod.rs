//! src/app/input/ — INPUT handling, split by natural seam (2026-07
//! code-organization pass) out of the former `app/input.rs` monolith:
//! [`keys`] (the keyboard path — held-HUD/peek, whichkey, incremental
//! search, zoom, page scroll, IME, `KeyboardInput`/`ModifiersChanged`
//! dispatch), [`mouse`] (the pointer path — hit-test, click/drag-select,
//! outline/link/overlay/panel/menu-bar clicks, the cursor icon,
//! wheel scroll/zoom/table-pan, `CursorMoved`/`MouseInput`/`MouseWheel`
//! dispatch), and [`drags`] (the page-column and inline-image RESIZE drag
//! state machines, incl. [`ImageDrag`]). Everything `window_event`
//! dispatches into; every external path (`app::input::ImageDrag`) is
//! unchanged — this file only re-exports.

mod context_menu;
mod drags;
mod keys;
mod mouse;
mod wheel;

pub(crate) use drags::{ImageDrag, RangeDrag};
pub(in crate::app) use wheel::initial_sensitivity as initial_scroll_sensitivity;

/// The live input handle. Keyboard/prefix/IME state and the pointer gesture
/// that began at a press belong to one runtime: a window event advances one
/// coherent interaction, not a bag of `App` flags.
///
/// The handle is private to the live app. Its fields are visible inside
/// `crate::app` while the existing input seams are migrated; no renderer,
/// configuration, or shared-core code can reach them.
pub struct InputRuntime {
    pub(in crate::app) keymap: crate::keymap::KeymapState,
    pub(in crate::app) mods: winit::event::Modifiers,
    pub(in crate::app) prefix_pending_at: Option<crate::clock::Instant>,
    pub(in crate::app) whichkey_shown: bool,
    pub(in crate::app) hud_key: Option<winit::keyboard::Key>,
    pub(in crate::app) hud_mods: winit::keyboard::ModifiersState,
    pub(in crate::app) peek_arm: crate::peek::PeekArm,
    pub(in crate::app) peek_armed_at: Option<crate::clock::Instant>,
    pub(in crate::app) pointer_hide: crate::pointer_hide::PointerHide,
    pub(in crate::app) cursor_px: (f32, f32),
    pub(in crate::app) dragging: bool,
    pub(in crate::app) drag_press_px: (f32, f32),
    pub(in crate::app) drag_armed: bool,
    pub(in crate::app) page_resizing: bool,
    pub(in crate::app) page_resize_edge: Option<crate::render::ResizeEdge>,
    pub(in crate::app) page_resize_anchor: Option<f32>,
    pub(in crate::app) image_resizing: Option<ImageDrag>,
    pub(in crate::app) range_drag: Option<RangeDrag>,
    pub(in crate::app) cursor_icon: winit::window::CursorIcon,
    pub(in crate::app) drag_granularity: DragGranularity,
    pub(in crate::app) last_click_time: Option<crate::clock::Instant>,
    pub(in crate::app) last_click_px: (f32, f32),
    pub(in crate::app) click_count: u32,
    pub(in crate::app) scroll_px_accum: f32,
    pub(in crate::app) preedit: String,
    pub(in crate::app) ime_enabled: bool,
    pub(in crate::app) scroll_sensitivity: f32,
}

impl InputRuntime {
    pub(in crate::app) fn new(keymap: crate::keymap::KeymapState, scroll_sensitivity: f32) -> Self {
        Self {
            keymap,
            mods: winit::event::Modifiers::default(),
            prefix_pending_at: None,
            whichkey_shown: false,
            hud_key: None,
            hud_mods: winit::keyboard::ModifiersState::empty(),
            peek_arm: crate::peek::PeekArm::default(),
            peek_armed_at: None,
            pointer_hide: crate::pointer_hide::PointerHide::Visible,
            cursor_px: (0.0, 0.0),
            dragging: false,
            drag_press_px: (0.0, 0.0),
            drag_armed: false,
            page_resizing: false,
            page_resize_edge: None,
            page_resize_anchor: None,
            image_resizing: None,
            range_drag: None,
            cursor_icon: winit::window::CursorIcon::Default,
            drag_granularity: DragGranularity::Char,
            last_click_time: None,
            last_click_px: (0.0, 0.0),
            click_count: 0,
            scroll_px_accum: 0.0,
            preedit: String::new(),
            ime_enabled: false,
            scroll_sensitivity,
        }
    }

    /// The last pointer position is the only input fact an overlay resync may
    /// borrow. It is a value snapshot, so the overlay cannot retain or mutate
    /// live pointer state.
    pub(in crate::app) fn resting_pointer(&self) -> RestingPointer {
        RestingPointer(self.cursor_px)
    }

    /// Finish a text-selection gesture. The next press must always begin
    /// below drag slop; leaving `drag_armed` true leaks a completed drag into
    /// the next click.
    pub(in crate::app) fn finish_text_drag(&mut self) {
        self.dragging = false;
        self.drag_armed = false;
    }
}

/// A pointer position after the current input event has settled.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::app) struct RestingPointer((f32, f32));

impl RestingPointer {
    pub(in crate::app) fn px(self) -> (f32, f32) {
        self.0
    }
}

#[derive(Clone, Copy, PartialEq)]
pub(in crate::app) enum DragGranularity {
    Char,
    Word,
    Line,
}

#[cfg(test)]
mod tests;
