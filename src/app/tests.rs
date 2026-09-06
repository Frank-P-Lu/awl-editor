//! Unit + hermetic-App tests for `crate::app`, grouped by feature area.

use super::*;

mod buffers;
/// THE LIVE CLIPBOARD BRIDGE ACROSS A BUFFER SWITCH: `clipboard_last_written`
/// is App-global while the kill ring it mirrors is per-buffer, so a naive
/// "already wrote this" skip can starve a buffer that switched in after the
/// copy. Native-only — the fake clipboard backend it drives only exists off
/// wasm, matching `arboard`'s own native-only role in `App`.
#[cfg(not(target_arch = "wasm32"))]
mod clipboard;
mod common;
/// STRUCTURAL GATES: the `App` ownership map as executable data —
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
mod external;
mod files;
mod history;
mod lifecycle;
/// THE LIVE-`App` EVENT→PRESENT TRACE ASSERTION: the picker-navigation
/// chain read back off the flight recorder's own lines. Native-only — the
/// recorder is (`crate::probe`), like the daemon and the live probe.
#[cfg(not(target_arch = "wasm32"))]
mod nav_trace;
mod openable;
/// The owner-derived durable-write failure matrix.  Precise filesystem faults
/// stay at tier 1; real-process-only crash and scale arms live in the paired
/// integration test.
#[cfg(not(target_arch = "wasm32"))]
mod persistence_faults;
/// TIER 2: A READING SURFACE ACCEPTS NO TEXT. The insertion doors that bypass
/// `App::apply` — the IME commit, the two assistive document writes — have no
/// chord vocabulary on any capture door, so this is the purest reachable seam.
/// Native-only, like its siblings above: two of the three doors it sweeps live
/// in `crate::app::semantic`, which the wasm build does not compile, and a
/// per-arm gate would leave the sweep's own roster different per platform —
/// which is a worse thing to be than absent.
#[cfg(not(target_arch = "wasm32"))]
mod read_only_surface;
/// THE SEMANTIC FOLD'S REACH BOUNDARY: the tree-building path sees a narrow
/// `SemanticView`, not the live `App`. Native-only, like `crate::app::semantic`.
#[cfg(not(target_arch = "wasm32"))]
mod semantic_reach;
mod source_audit;
mod spell;
/// TIER 2: A SUMMONED TEXT-ENTRY SURFACE OWNS THE EDIT VERBS. The macOS menu
/// bar's own key equivalents (and a menu/context-menu click) deliver an
/// `Action` to `App::apply` without ever touching a keymap, so the panel's
/// key guard never sees them — no chord-replay door can drive this, which is
/// what makes the live `App` the purest reachable seam.
/// Native-only, like its `read_only_surface` sibling: `install_search_for_test`
/// is not compiled for wasm.
#[cfg(not(target_arch = "wasm32"))]
mod summoned_field_actions;
mod which_key;
/// TIER-2 SWEEP: every setting changed and persisted through the
/// Settings WORKSPACE's own door, by real chords into the live `App`. Why this
/// tier and not a capture: `docs/harness-reach.md`.
mod workspace;
/// TIER 2: the summoned workspace's BACK — which key it is, that the footer
/// names that key, and that `Tab` is no longer taught as one. A report about a
/// keyboard is answered with the keyboard.
mod workspace_back;

use common::*;
