//! Deterministic PNG capture and JSON state sidecar. See `CAPTURE.md`.
/// Fixed headless canvas.
pub const CANVAS_WIDTH: u32 = 1200;
pub const CANVAS_HEIGHT: u32 = 800;
pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
/// `/189` — `page.background`'s `deckle` arm.
/// `/190` — `page.background`'s `organic` arm gains `arrangement`.
/// `/191` — `overlay.workspace`; `overlay.diff_focus` became `overlay.detail_focus`.
/// `/193` — top-level `driver`.
/// `/194` — `page.background`'s `warped-grid` arm: authored dials and travel.
/// `/195` — pointer-anchored context menus add `overlay.context_anchor`.
/// `/196` — top-level `semantic`: the live-App semantic tree, else `null`.
/// `/197` — `overlay.preview_view`; `gutter.changed`.
/// `/198` — `readout.unit` / `hud.unit`: `"words"` or `"characters"`.
/// `/199` — `page.background` LOSES three keys with the ground dials that fed
///          them: `organic.arrangement`, `deckle.anchor`, `lava.edge`. Each
///          enum had collapsed to one arm, so the key reported a constant.
///          A key REMOVAL is a shape change (a reader keying on it breaks),
///          unlike `/198`'s predecessor, which only narrowed a value space.
///          Every world's PNG is byte-identical across this bump.
/// `/200` — top-level `notice`: `{ text, kind }` for the calm notice on screen,
///          or `null`. Added because no capture door could see the channel at
///          all: `CaptureOpts` had no slot for it, so a driven editor that had
///          raised a notice produced a PNG byte-identical to one that had not.
/// `/201` — `overlay.window` gains `band` + `rows`: every candidate display
///          line's PLANNED rect, in the physical pixels the pointer and the PNG
///          already speak. A row's geometry was measurable only from the PNG.
/// History lives in Git. Bump this row with the const.
pub const SCHEMA_VERSION: u32 = 201;
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
mod plan_sidecar;
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
