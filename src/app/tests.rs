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
/// Every door that could write to, leave, or rename a file that changed
/// underneath awl, plus both resolutions and relaunch recovery. Why this tier
/// and not a capture: that file's own module doc.
mod external_item204;
mod files;
mod history;
mod lifecycle;
/// THE LIVE-`App` EVENT→PRESENT TRACE ASSERTION: the picker-navigation
/// chain read back off the flight recorder's own lines. Native-only — the
/// recorder is (`crate::probe`), like the daemon and the live probe.
#[cfg(not(target_arch = "wasm32"))]
mod nav_trace_item211;
mod openable;
/// THE SEMANTIC FOLD'S REACH BOUNDARY: the tree-building path sees a narrow
/// `SemanticView`, not the live `App`. Native-only, like `crate::app::semantic`.
#[cfg(not(target_arch = "wasm32"))]
mod semantic_reach;
mod source_audit;
mod spell;
mod which_key;
/// ITEM 114's TIER-2 SWEEP: every setting changed and persisted through the
/// Settings WORKSPACE's own door, by real chords into the live `App`. Why this
/// tier and not a capture: `docs/harness-reach.md`.
mod workspace_item114;

use common::*;
