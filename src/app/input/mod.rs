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

#[cfg(test)]
mod tests;
