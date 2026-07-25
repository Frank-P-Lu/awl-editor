//! Unit + hermetic-App tests for `crate::app`, grouped by feature area.

use super::*;

mod buffers;
mod common;
// THE DOCK-CHURN LAW: a theme PREVIEW must never restamp the app icon; only a
// commit (and startup) may. Counts adoptions across a full preview sweep.
// Native-only, like `crate::app_icon` itself — a browser tab has no Dock.
#[cfg(not(target_arch = "wasm32"))]
mod dock_icon;
mod files;
mod history;
mod lifecycle;
mod openable;
mod source_audit;
mod spell;
mod which_key;

use common::*;
