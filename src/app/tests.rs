//! Unit + hermetic-App tests for `crate::app`, grouped by feature area.

use super::*;

mod buffers;
mod common;
/// ITEM 172's STRUCTURAL GATES: the `App` ownership map as executable data —
/// every root field classified, extracted domains kept off root `App`, and the
/// field-count ratchet. Prose map: `docs/app-domains.md`.
mod domains;
// THE DOCK-CHURN LAW: a theme PREVIEW must never restamp the app icon; only a
// commit (and startup) may. Counts adoptions across a full preview sweep.
// Native-only, like `crate::app_icon` itself — a browser tab has no Dock.
#[cfg(not(target_arch = "wasm32"))]
mod dock_icon;
mod files;
mod history;
mod lifecycle;
mod openable;
/// ITEM 183's HEADLESS PRESS DOOR: drive real chords into the real live
/// `App` off-window. Reach map: `docs/harness-reach.md`.
mod press;
mod source_audit;
mod spell;
/// ITEM 114's TIER-2 SWEEP: every setting changed and persisted through the
/// Settings WORKSPACE's own door, by real chords into the live `App`. Why this
/// tier and not a capture: `docs/harness-reach.md`.
mod workspace_item114;
mod which_key;

use common::*;
