//! THE LIVE `App`'s OWN SIDECAR: the constructor a live-`App`
//! capture builds through, and the `App`'s side of the ONE per-frame fold.
//!
//! # The gap this closes
//!
//! `App::apply`'s `&ActiveEventLoop` borrow was narrowed to `app::Exit`,
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
//! `run::project_info`, the one builder for exactly this. The
//! live `App` and the shared-core `ReplaySession` therefore differ in what
//! drives them and in nothing a sidecar reports.

use super::*;
use crate::capture::CaptureOpts;

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
    /// through `run::project_info` (fed the App's OWN
    /// flag-over-config workspace fold so a `--screenshot-app` sidecar reports
    /// the location the running editor actually has), and everything else
    /// through `run::fold_capture_state`, shared with the storyboard stepper.
    pub(crate) fn capture_opts(&self) -> CaptureOpts {
        let workspace = self.config.location_policy().workspace_override();
        let project = crate::run::project_info(
            &self.project_location.root,
            &workspace,
            Some(self.config.default_folder.as_path()),
            &self.config,
        );
        let mut opts = crate::run::fold_capture_state(self, project);
        opts.driver = crate::capture::CaptureDriver::LiveApp;
        // The which-key panel is drawn by the harness's offscreen pipeline from
        // `opts`, but ANNOUNCED from the App's own scheduling state. Fed from
        // the one gate (`whichkey_panel_rows`) so the PNG and the `semantic`
        // tree cannot disagree about whether the panel is up.
        opts.whichkey = self.whichkey_panel_rows();
        opts.semantic = Some(self.semantic_snapshot());
        // THE VISIBLE WORKING SET, scoped to the root the ACTIVE FILE remembers
        // — the same one `sync_view` draws the live margin from, so a capture and
        // the running editor cannot disagree about which files are open. Empty
        // for a single-file root, which is what keeps a one-file capture
        // byte-identical to one taken before this surface existed.
        opts.working_set = self
            .document
            .working_set()
            .active_root()
            .map(|root| self.document.working_set().stack_rows(root))
            .unwrap_or_default();
        // THE IDENTITY'S FOLDER LABEL: the same root `sync_view` draws the
        // live gutter from, so a `--screenshot-app` capture and the running
        // editor cannot disagree about which folder the open file is in —
        // `None` only for the working set's own empty startup instant, which
        // does not survive `App::new`.
        opts.gutter_project_root = self
            .document
            .working_set()
            .active_root()
            .map(std::path::Path::to_path_buf);
        opts
    }
}

/// The LIVE driver's view of itself — the same five facts `ReplaySession`
/// answers, read straight off the running `App`. Every one is a plain borrow of
/// state the App already owns: nothing is recomputed for the capture, so the
/// sidecar reports the editor rather than a model of it.
impl crate::run::CaptureSubject for App {
    fn buffer(&self) -> Option<&crate::buffer::Buffer> {
        self.document.buffer_opt()
    }
    fn zoom(&self) -> f32 {
        self.frame.zoom()
    }
    fn search(&self) -> Option<&crate::search::SearchState> {
        self.workspace_state.search()
    }
    fn journey(&self) -> &crate::overlay::Journey {
        self.workspace_state.journey()
    }
    fn buffers_open(&self) -> usize {
        self.document.open_count()
    }
    fn changed_elsewhere(&self) -> bool {
        self.change_unresolved()
    }
    /// Straight off the frame's own notice slot — the SAME snapshot
    /// `App::sync_view` folds into the `ViewState` and `App::semantic_snapshot`
    /// announces, so the PNG, the sidecar's `notice` block and the sidecar's
    /// `semantic` tree cannot disagree about what is on screen.
    fn notice(&self) -> Option<(String, crate::actions::NoticeKind)> {
        let notice = self.frame.notice();
        notice.owned().map(|text| (text, notice.kind()))
    }
}
