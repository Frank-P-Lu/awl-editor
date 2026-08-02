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
/// THE LIVE-`App` EVENT→PRESENT TRACE ASSERTION: the picker-navigation
/// chain read back off the flight recorder's own lines. Native-only — the
/// recorder is (`crate::probe`), like the daemon and the live probe.
#[cfg(not(target_arch = "wasm32"))]
mod nav_trace_item211;
mod openable;
mod source_audit;
mod spell;
/// ITEM 202: the shared `theme_font_at` debounce that decouples the
/// colors-only preview present from the deferred font reshape. Pins the
/// scheduling mechanism a `THEME_FONT_DEBOUNCE_DEFAULT_MS == 0` default broke
/// (docs/fonts.md's "Theme-preview debounce").
mod theme_debounce_item202;
mod which_key;
/// ITEM 114's TIER-2 SWEEP: every setting changed and persisted through the
/// Settings WORKSPACE's own door, by real chords into the live `App`. Why this
/// tier and not a capture: `docs/harness-reach.md`.
mod workspace_item114;

use common::*;
