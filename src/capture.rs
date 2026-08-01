//! Deterministic PNG capture and JSON state sidecar. See `CAPTURE.md`.
/// Fixed headless canvas.
pub const CANVAS_WIDTH: u32 = 1200;
pub const CANVAS_HEIGHT: u32 = 800;
pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
/// `/188` — permissive replay `replay_skips`.
/// `/189` — `page.background`'s `deckle` arm.
/// `/190` — `page.background`'s `organic` arm gains `arrangement`.
/// `/191` — `overlay.workspace`; `overlay.diff_focus` became `overlay.detail_focus`.
/// `/193` — top-level `driver`.
/// `/194` — `page.background`'s `warped-grid` arm: authored dials and travel.
/// `/195` — pointer-anchored context menus add `overlay.context_anchor`.
/// History lives in Git. Bump this row with the const.
pub const SCHEMA_VERSION: u32 = 195;
/// Plain single-frame schema; timeline and held take the next two versions.
pub fn schema_plain() -> String {
    format!("awl-capture/{SCHEMA_VERSION}")
}
pub fn schema_timeline() -> String {
    format!("awl-capture/{}", SCHEMA_VERSION + 1)
}
pub fn schema_held() -> String {
    format!("awl-capture/{}", SCHEMA_VERSION + 2)
}

mod animated;
mod background_sidecar;
mod film;
#[cfg(not(target_arch = "wasm32"))]
mod frames;
pub(crate) mod gpu;
mod layout_sidecar;
mod modes;
mod opts;
mod oracle;
mod policy;
mod replay_sidecar;
mod scroll_sidecar;
mod sidecar;

pub use animated::{HeldDir, capture_held, capture_timeline};
pub use film::{FRAME_MS, FilmRenderer};
#[cfg(not(target_arch = "wasm32"))]
pub use frames::{DEFAULT_FRAME_STEP_MS, capture_frames};
pub use modes::{capture_motion, capture_motion_diagonal, capture_motion_vertical, capture_with};
#[cfg(not(target_arch = "wasm32"))]
pub use opts::CaptureDriver;
pub use opts::{BuffersInfo, CaptureInfo, CaptureOpts, DiffInfo, OverlayInfo, ProjectInfo};
pub use oracle::build_oracle;
pub(crate) use sidecar::json_string;

#[allow(unused_imports)]
pub use oracle::OraclePipeline;

#[cfg(test)]
mod tests;
