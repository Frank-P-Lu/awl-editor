//! ITEM 188 — THE LIVE `App`'s OWN SIDECAR: the constructor a live-`App`
//! capture builds through, and the `App`'s side of the ONE per-frame fold.
//!
//! # The gap this closes
//!
//! Item 183 narrowed `App::apply`'s `&ActiveEventLoop` borrow to `app::Exit`,
//! so a real `App` became DRIVABLE headlessly (`app/press.rs`). It stayed
//! UNPHOTOGRAPHABLE: `App` never called the sidecar writer, so every transition
//! only a live `App` can perform — a Settings write, a buffer switch, a config
//! reload — could be executed but not captured, and had to be asserted in Rust
//! rather than read from the one oracle the rest of the project uses
//! (`docs/harness-reach.md`, "Left for a follow-up" #1).
//!
//! # Why there is no serializer here
//!
//! There is exactly one sidecar writer (`capture::sidecar::write_sidecar`) and
//! exactly one per-frame fold (`run::fold_capture_state`), and this file adds
//! neither. It supplies the five facts that fold asks any driver for — the
//! `CaptureSubject` impl below — plus the project block, built by
//! `run::project_info`, the one builder item 183 made for exactly this. The
//! live `App` and the shared-core `ReplaySession` therefore differ in what
//! drives them and in nothing a sidecar reports.

use super::*;
use crate::capture::CaptureOpts;

#[cfg(not(target_arch = "wasm32"))]
impl App {
    /// THE LIVE-`App` CAPTURE CONSTRUCTOR (`--screenshot-app`). Unlike its
    /// sibling [`new_headless_scheduler`](Self::new_headless_scheduler), it does
    /// NOT install a filesystem of its own: `--screenshot-app` is a SCENARIO
    /// door, so `args::parse_args` has already swapped the process fs to the
    /// seeded hermetic sandbox (`crate::scenario::install_hermetic_fs`) before
    /// the config loaded. Constructing on a fresh empty `InMemoryFs` here would
    /// throw that seeding away and, worse, leave the App's LATER writes (a
    /// settings persist, an autosave) pointed at a backend the caller restored
    /// out from under it — the whole point of this mode is that those writes
    /// really happen, in the sandbox.
    ///
    /// Session restore and reduce-motion are pinned off for the same reason the
    /// two hermetic siblings pin them: a capture may not depend on what a
    /// previous run remembered, nor on the test machine's OS accessibility
    /// preferences. Routes through `Self::new`, not the raw constructor's
    /// open-paren needle, so `app::tests::source_audit`'s accounting guard is
    /// unaffected.
    pub(crate) fn new_headless_capture(
        file: Option<PathBuf>,
        root: PathBuf,
        workspace: Option<PathBuf>,
        config: Config,
    ) -> Self {
        let config = Config {
            session_restore: Some(false),
            reduce_motion: Some(false),
            ..config
        };
        Self::new(file, root, workspace, None, config)
    }

    /// Fold THIS live `App`'s current state into the [`CaptureOpts`] the
    /// single-frame capture path renders and the one sidecar writer serializes.
    ///
    /// Both halves route through their existing one owner: the project block
    /// through `run::project_info` (item 183's builder, fed the App's OWN
    /// flag-over-config workspace fold so a `--screenshot-app` sidecar reports
    /// the location the running editor actually has), and everything else
    /// through `run::fold_capture_state`, shared with the storyboard stepper.
    pub(crate) fn capture_opts(&self) -> CaptureOpts {
        let workspace = self
            .cli_workspace
            .clone()
            .or_else(|| self.config.workspace.clone());
        let project = crate::run::project_info(
            &self.root,
            &workspace,
            Some(self.default_folder.as_path()),
            &self.config,
        );
        let mut opts = crate::run::fold_capture_state(self, project);
        opts.driver = crate::capture::CaptureDriver::LiveApp;
        opts
    }
}

/// The LIVE driver's view of itself — the same five facts `ReplaySession`
/// answers, read straight off the running `App`. Every one is a plain borrow of
/// state the App already owns: nothing is recomputed for the capture, so the
/// sidecar reports the editor rather than a model of it.
impl crate::run::CaptureSubject for App {
    fn buffer(&self) -> &crate::buffer::Buffer {
        &self.active.buffer
    }
    fn zoom(&self) -> f32 {
        self.zoom
    }
    fn search(&self) -> Option<&crate::search::SearchState> {
        self.workspace_state.search()
    }
    fn journey(&self) -> &crate::overlay::Journey {
        self.workspace_state.journey()
    }
    fn buffers_open(&self) -> usize {
        self.buffer_registry.len() + 1
    }
}
