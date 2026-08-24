//! src/app/input/ — INPUT handling, split by natural seam (2026-07
//! code-organization pass) out of the former `app/input.rs` monolith:
//! [`keys`] (the keyboard path — held-HUD/peek, whichkey, incremental
//! search, zoom, page scroll, `KeyboardInput`/`ModifiersChanged`
//! dispatch), [`mouse`] (the pointer path — hit-test, click/drag-select,
//! outline/link/overlay/panel/menu-bar clicks, the cursor icon,
//! wheel scroll/zoom/table-pan, `CursorMoved`/`MouseInput`/`MouseWheel`
//! dispatch), and [`drags`] (the page-column and inline-image RESIZE drag
//! state machines, incl. [`ImageDrag`]). Everything `window_event`
//! dispatches into; every external path (`app::input::ImageDrag`) is
//! unchanged — this file only re-exports.

mod context_menu;
mod drags;
mod gutter;
mod ime;
mod keys;
mod mouse;
mod mouse_button;
mod wheel;

use drags::ImageDrag;
#[cfg(test)]
pub(in crate::app) use drags::RangeDrag;
#[cfg(not(test))]
use drags::RangeDrag;
pub(in crate::app) use wheel::initial_sensitivity as initial_scroll_sensitivity;

/// The live input handle. It is the one `App` field for input, while its two
/// private substates own the different invariants advanced by a window event.
pub(in crate::app) struct InputRuntime {
    keyboard: KeyboardInput,
    pointer: PointerInput,
}

/// The timer facts the frame scheduler may observe from live input state.
/// Copying them at the poll boundary prevents the scheduler from reaching
/// through `InputRuntime` while it is also driving input-owned transitions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::app) struct SchedulingSnapshot {
    pub(in crate::app) prefix_pending_at: Option<crate::clock::Instant>,
    pub(in crate::app) whichkey_shown: bool,
    pub(in crate::app) peek_armed_at: Option<crate::clock::Instant>,
    pub(in crate::app) zoom_persist_held: bool,
}

/// Key resolution and the transient keyboard surfaces coupled to it.
struct KeyboardInput {
    keymap: crate::keymap::KeymapState,
    mods: winit::event::Modifiers,
    prefix_pending_at: Option<crate::clock::Instant>,
    whichkey_shown: bool,
    hud_key: Option<winit::keyboard::Key>,
    hud_mods: winit::keyboard::ModifiersState,
    peek_arm: crate::peek::PeekArm,
    peek_armed_at: Option<crate::clock::Instant>,
    preedit: String,
    ime_enabled: bool,
}

/// Pointer visibility, gesture lifetimes, click cadence, and wheel state.
struct PointerInput {
    pointer_hide: crate::pointer_hide::PointerHide,
    cursor_px: (f32, f32),
    dragging: bool,
    drag_press_px: (f32, f32),
    drag_armed: bool,
    /// Last time a drag-scroll step actually advanced the scroll, so the next
    /// step's `dt` is real elapsed time rather than a fixed guess (see the
    /// tick helpers below). `None` between drags and whenever the pointer
    /// sits back inside the band.
    drag_scroll_last_tick: Option<crate::clock::Instant>,
    page_resizing: bool,
    page_resize_edge: Option<crate::render::ResizeEdge>,
    page_resize_anchor: Option<f32>,
    image_resizing: Option<ImageDrag>,
    range_drag: Option<RangeDrag>,
    cursor_icon: winit::window::CursorIcon,
    drag_granularity: DragGranularity,
    last_click_time: Option<crate::clock::Instant>,
    last_click_px: (f32, f32),
    click_count: u32,
    scroll_px_accum: f32,
    scroll_sensitivity: f32,
}

impl InputRuntime {
    pub(in crate::app) fn new(keymap: crate::keymap::KeymapState, scroll_sensitivity: f32) -> Self {
        Self {
            keyboard: KeyboardInput {
                keymap,
                mods: winit::event::Modifiers::default(),
                prefix_pending_at: None,
                whichkey_shown: false,
                hud_key: None,
                hud_mods: winit::keyboard::ModifiersState::empty(),
                peek_arm: crate::peek::PeekArm::default(),
                peek_armed_at: None,
                preedit: String::new(),
                ime_enabled: false,
            },
            pointer: PointerInput {
                pointer_hide: crate::pointer_hide::PointerHide::Visible,
                cursor_px: (0.0, 0.0),
                dragging: false,
                drag_press_px: (0.0, 0.0),
                drag_armed: false,
                drag_scroll_last_tick: None,
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
                scroll_sensitivity,
            },
        }
    }

    /// The last pointer position is the only input fact an overlay resync may
    /// borrow. It is a value snapshot, so the overlay cannot retain or mutate
    /// live pointer state.
    pub(in crate::app) fn resting_pointer(&self) -> RestingPointer {
        RestingPointer(self.pointer.cursor_px)
    }

    /// Finish a text-selection gesture. The next press must always begin
    /// below drag slop; leaving `drag_armed` true leaks a completed drag into
    /// the next click.
    pub(in crate::app) fn finish_text_drag(&mut self) {
        self.pointer.finish_text_drag();
    }

    #[cfg(any(not(target_arch = "wasm32"), test))]
    pub(in crate::app) fn set_modifiers(&mut self, mods: winit::event::Modifiers) {
        self.keyboard.mods = mods;
    }

    #[cfg(any(not(target_arch = "wasm32"), test))]
    pub(in crate::app) fn clear_modifiers(&mut self) {
        self.keyboard.mods = winit::event::Modifiers::default();
    }

    pub(in crate::app) fn apply_key_overrides(&mut self, overrides: &[(String, Vec<String>)]) {
        self.keyboard.keymap.apply_overrides(overrides);
    }

    pub(in crate::app) fn apply_linux_keep(&mut self, keep: &[String]) {
        self.keyboard.keymap.apply_linux_keep(keep);
    }

    /// The classic-Meta-layer sibling of [`Self::apply_linux_keep`] — called
    /// right alongside it on every door that can flip `keymap` flavor live, so
    /// both halves of the flavor land in the same reseed.
    pub(in crate::app) fn apply_linux_emacs_meta(&mut self, active: bool) {
        self.keyboard.keymap.set_linux_emacs_meta(active);
    }

    pub(in crate::app) fn clear_preedit(&mut self) {
        self.keyboard.preedit.clear();
    }

    pub(in crate::app) fn preedit(&self) -> &str {
        &self.keyboard.preedit
    }

    pub(in crate::app) fn scheduling_snapshot(&self) -> SchedulingSnapshot {
        SchedulingSnapshot {
            prefix_pending_at: self.keyboard.prefix_pending_at,
            whichkey_shown: self.keyboard.whichkey_shown,
            peek_armed_at: self.keyboard.peek_armed_at,
            zoom_persist_held: self.pointer.range_drag.is_some(),
        }
    }

    #[cfg(any(not(target_arch = "wasm32"), test))]
    pub(in crate::app) fn arm_prefix(&mut self, now: crate::clock::Instant) {
        self.keyboard.prefix_pending_at = Some(now);
    }

    #[cfg(any(not(target_arch = "wasm32"), test))]
    pub(in crate::app) fn whichkey_shown(&self) -> bool {
        self.keyboard.whichkey_shown
    }

    pub(in crate::app) fn selecting_drag(&self) -> bool {
        self.pointer.dragging
    }

    pub(in crate::app) fn set_scroll_sensitivity(&mut self, value: f32) {
        self.pointer.scroll_sensitivity = value;
    }

    pub(in crate::app) fn scroll_sensitivity(&self) -> f32 {
        self.pointer.scroll_sensitivity
    }

    /// Focus loss is a pointer-state transition. Return only the OS visibility
    /// effect that the window host must interpret.
    pub(in crate::app) fn reveal_pointer(&mut self) -> Option<bool> {
        let before = self.pointer.pointer_hide;
        self.pointer.pointer_hide = crate::pointer_hide::PointerHide::Visible;
        crate::pointer_hide::os_visibility_change(before, self.pointer.pointer_hide)
    }

    #[cfg(test)]
    pub(in crate::app) fn set_resting_pointer_for_test(&mut self, px: (f32, f32)) {
        self.pointer.cursor_px = px;
    }

    #[cfg(test)]
    pub(in crate::app) fn set_range_drag_for_test(&mut self, drag: RangeDrag) {
        self.pointer.range_drag = Some(drag);
    }

    #[cfg(test)]
    pub(in crate::app) fn range_drag_active(&self) -> bool {
        self.pointer.range_drag.is_some()
    }
}

impl PointerInput {
    fn bump_click_count(&mut self, now: crate::clock::Instant) -> u32 {
        let near = (self.cursor_px.0 - self.last_click_px.0).abs() < 4.0
            && (self.cursor_px.1 - self.last_click_px.1).abs() < 4.0;
        let recent = self.last_click_time.is_some_and(|then| {
            now.duration_since(then) < std::time::Duration::from_millis(super::MULTICLICK_MS)
        });
        self.click_count = if recent && near {
            (self.click_count % 3) + 1
        } else {
            1
        };
        self.last_click_time = Some(now);
        self.last_click_px = self.cursor_px;
        self.click_count
    }

    fn begin_text_drag(&mut self) {
        self.dragging = true;
        self.drag_press_px = self.cursor_px;
        self.drag_armed = false;
        self.drag_scroll_last_tick = None;
    }

    fn arm_text_drag_if_moved(&mut self) -> bool {
        if !self.drag_armed {
            self.drag_armed = Self::exceeds_drag_slop(self.drag_press_px, self.cursor_px);
        }
        self.drag_armed
    }

    fn exceeds_drag_slop(press: (f32, f32), current: (f32, f32)) -> bool {
        let dx = current.0 - press.0;
        let dy = current.1 - press.1;
        dx * dx + dy * dy > super::DRAG_ARM_SLOP_PX.powi(2)
    }

    fn finish_text_drag(&mut self) {
        self.dragging = false;
        self.drag_armed = false;
        self.drag_scroll_last_tick = None;
    }

    /// Elapsed time since the last drag-scroll tick (zero on the first tick
    /// past the edge, priming the clock rather than guessing a step), and
    /// stamp `now` as the new last tick. The caller only reaches this once it
    /// already knows the pointer sits beyond the band — see
    /// [`Self::clear_drag_scroll_tick`] for the other half.
    pub(super) fn drag_scroll_tick_dt(
        &mut self,
        now: crate::clock::Instant,
    ) -> std::time::Duration {
        let dt = self
            .drag_scroll_last_tick
            .map_or(std::time::Duration::ZERO, |last| {
                now.saturating_duration_since(last)
            });
        self.drag_scroll_last_tick = Some(now);
        dt
    }

    /// Forget the drag-scroll clock: called the instant the pointer re-enters
    /// the band (or the drag ends), so a LATER re-crossing primes fresh at
    /// `dt = 0` instead of replaying the gap spent inside the band as one
    /// huge scroll jump.
    pub(super) fn clear_drag_scroll_tick(&mut self) {
        self.drag_scroll_last_tick = None;
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
