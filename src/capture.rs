//! Deterministic PNG capture and JSON state sidecar. See `CAPTURE.md`.

/// Fixed headless canvas.
pub const CANVAS_WIDTH: u32 = 1200;
pub const CANVAS_HEIGHT: u32 = 800;
pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

/// `/187` — shaped-frame `layout` rows; `/188` — permissive replay `replay_skips`; history lives in Git. Bump this row with the const.
/// Sidecar schema base; timeline and held use the next two versions.
pub const SCHEMA_VERSION: u32 = 188;

/// Plain single-frame schema.
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
mod film;
#[cfg(not(target_arch = "wasm32"))]
mod frames;
pub(crate) mod gpu;
mod layout_sidecar;
mod modes;
mod opts;
mod oracle;
mod policy;
mod scroll_sidecar;
mod sidecar;

pub use animated::{HeldDir, capture_held, capture_timeline};
pub use film::{FRAME_MS, FilmRenderer};
#[cfg(not(target_arch = "wasm32"))]
pub use frames::{DEFAULT_FRAME_STEP_MS, capture_frames};
pub use modes::{capture_motion, capture_motion_diagonal, capture_motion_vertical, capture_with};
pub use opts::{BuffersInfo, CaptureInfo, CaptureOpts, DiffInfo, OverlayInfo, ProjectInfo};
pub use oracle::build_oracle;
pub(crate) use sidecar::json_string;

#[allow(unused_imports)]
pub use oracle::OraclePipeline;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod schema_ledger {
    use super::SCHEMA_VERSION;

    fn history_rows() -> Vec<u32> {
        let src = include_str!("capture.rs");
        let mut rows = Vec::new();
        for line in src.lines() {
            let Some(rest) = line.trim_start().strip_prefix("//") else {
                continue;
            };
            let rest = rest.trim_start_matches('/').trim_start();
            let Some(after) = rest.strip_prefix("`/") else {
                continue;
            };
            let Some(end) = after.find('`') else { continue };
            if let Ok(n) = after[..end].parse::<u32>() {
                rows.push(n);
            }
        }
        rows
    }

    #[test]
    fn schema_version_matches_latest_history_row() {
        let rows = history_rows();
        assert!(
            !rows.is_empty(),
            "no `/N` history rows parsed — has the table's row format changed? \
             (see the CLAIM CONVENTION doc above SCHEMA_VERSION)"
        );
        for w in rows.windows(2) {
            assert!(
                w[1] > w[0],
                "schema history rows not strictly increasing (/{} then /{}) — a \
                 duplicate or out-of-order row, almost certainly an unreconciled \
                 merge collision; renumber the later row (see CLAIM CONVENTION).",
                w[0],
                w[1]
            );
        }
        let last = *rows.last().unwrap();
        assert_eq!(
            last, SCHEMA_VERSION,
            "SCHEMA_VERSION ({SCHEMA_VERSION}) must equal the LAST history row \
             (/{last}). Bump the const AND append a matching `/N` row together \
             (see the CLAIM CONVENTION doc above SCHEMA_VERSION)."
        );
    }
}
