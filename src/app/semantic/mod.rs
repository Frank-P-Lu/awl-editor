//! Live-state fold into the renderer-independent semantic snapshot.
//!
//! One owner, three consumers: the native AccessKit tree, `--semantic-json`,
//! and the live-App capture sidecar all read the value `semantic_snapshot`
//! returns. Surfaces are folded in two families — the ACTIVE ones that own the
//! keyboard (`surfaces.rs`, selected by the `Layer` ladder) and the PASSIVE
//! ones that are announced without taking focus (`passive.rs`). Requests come
//! back through `requests.rs`, which ends every arm in an existing owner.

use super::*;
use crate::semantic::{
    DOCUMENT_ID, ROOT_ID, SemanticAction, SemanticNode, SemanticRequest, SemanticRole,
    SemanticSelection, SemanticSnapshot,
};

#[cfg(not(target_arch = "wasm32"))]
pub(crate) mod bench;
mod passive;
mod projection;
mod requests;
mod surfaces;

pub(in crate::app) use projection::{ProjectionStats, SemanticProjection};

/// Ids that both a fold and a request arm must spell identically.
const SEARCH_ID: &str = "search";
const SEARCH_QUERY_ID: &str = "search.query";
const SEARCH_REPLACE_ID: &str = "search.replacement";
const SEARCH_CASE_ID: &str = "search.case-sensitive";
const WHICHKEY_ID: &str = "whichkey";
const MENUBAR_ID: &str = "menubar";
const NOTICE_ID: &str = "notice";

impl App {
    pub(in crate::app) fn handle_accessibility_event(&mut self, event: accesskit_winit::Event) {
        match event.window_event {
            // Posted by our own activation handler (see `frame/accessibility.rs`)
            // the moment the platform asks for a tree, from whatever thread it
            // asked on. The handler itself touches no `App` state; this is
            // where the main loop learns an assistive technology attached.
            accesskit_winit::WindowEvent::InitialTreeRequested => {
                self.frame.set_accessibility_active(true);
                self.refresh_accessibility();
            }
            accesskit_winit::WindowEvent::ActionRequested(request) => {
                // The retained projection IS the tree the platform is holding,
                // so the request is decoded against exactly the node ids that
                // were published — never against a freshly built snapshot that
                // might already name different runs.
                let decoded = self
                    .frame
                    .accessibility_projection()
                    .map(|projection| projection.snapshot())
                    .and_then(|snapshot| {
                        crate::semantic::native::decode_request(snapshot, request)
                    });
                if let Some(request) = decoded {
                    self.apply_semantic_request(request);
                }
            }
            accesskit_winit::WindowEvent::AccessibilityDeactivated => {
                self.frame.set_accessibility_active(false);
            }
        }
    }

    /// The ONE live door from a scheduled frame to the platform tree.
    ///
    /// The gate sits here rather than inside the runtime because the projection
    /// work is the expense being avoided, and only the caller can decline to do
    /// it. With no assistive technology attached the frame pays a single
    /// integer compare — enough to know the tree published for a future
    /// activation has gone stale, and nothing more.
    pub(in crate::app) fn refresh_accessibility(&mut self) {
        if !self.frame.accessibility_wants_snapshot() {
            self.frame
                .note_published_tree_currency(self.published_tree_is_current());
            return;
        }
        let mut projection = self.frame.take_accessibility_projection();
        projection.refresh(self);
        self.frame.publish_accessibility(projection);
    }

    /// Is the snapshot handed to the synchronous activation handler still a
    /// truthful description of this `App`?
    ///
    /// All cheap reads — a version counter and a handful of surface gates — so
    /// an ordinary no-screen-reader frame can answer it without building
    /// anything. A `false` here is not a bug: activation then returns `None`,
    /// the platform shows a placeholder for one frame, and the first update is
    /// a full tree, which is AccessKit's documented other branch.
    fn published_tree_is_current(&self) -> bool {
        let quiet_surface = matches!(self.workspace_state.layer(), workspace::Layer::Editor)
            && self.card_kind_open().is_none()
            && self.whichkey_panel_rows().is_none()
            && !crate::menubar::menu_bar_on()
            && self.frame.notice().text().is_none();
        quiet_surface
            && self.frame.published_document_state() == self.document.buffer().runs().state_key()
    }

    /// Build the tree the synchronous activation handler will serve, and park
    /// it. Called once, from `resumed`, BEFORE the adapter exists — a screen
    /// reader that is already running when awl launches asks for a tree the
    /// instant the adapter is constructed, and the handler cannot build one on
    /// a platform thread.
    #[cfg(not(target_arch = "wasm32"))]
    pub(in crate::app) fn seed_accessibility_tree(&mut self) {
        let mut projection = self.frame.take_accessibility_projection();
        projection.refresh(self);
        let state = self.document.buffer().runs().state_key();
        self.frame.seed_accessibility(projection, state);
    }

    /// Everything that is NOT the retained document: the active surface named
    /// by the `Layer` ladder, the passive surfaces that ride on top of it, and
    /// the notice. Bounded by what is on screen. `nodes[0]` is the root, whose
    /// `children` the folds append to.
    pub(in crate::app) fn fold_surfaces(&self, nodes: &mut Vec<SemanticNode>) -> String {
        // Exactly one ACTIVE surface owns the keyboard, and the ladder — not a
        // field — is what says which. No wildcard arm: a sixth rung must be
        // placed here before it compiles.
        let focus_id = match self.workspace_state.layer() {
            workspace::Layer::Editor => DOCUMENT_ID.to_string(),
            workspace::Layer::Popover => self.fold_popover(nodes),
            workspace::Layer::Search => self.fold_search(nodes),
            workspace::Layer::Workspace | workspace::Layer::Overlay => self.fold_overlay(nodes),
        };

        // PASSIVE surfaces ride on top of whatever holds focus and never take
        // it: any number may be up at once without disturbing the invariant
        // the projection asserts.
        self.fold_passive(nodes);

        if let Some(message) = self.frame.notice().text() {
            nodes.push(SemanticNode::new(NOTICE_ID, SemanticRole::Status, message));
            nodes[0].children.push(NOTICE_ID.to_string());
        }
        focus_id
    }

    /// A one-shot snapshot for the consumers that want a whole value rather
    /// than a live tree: `--semantic-json` and the live-`App` capture sidecar.
    /// O(document) by construction and correct to be — those are single
    /// invocations, not frames.
    pub(crate) fn semantic_snapshot(&self) -> SemanticSnapshot {
        let mut projection = SemanticProjection::new();
        projection.refresh(self);
        projection.into_snapshot()
    }

    /// The whole attach sequence a real screen reader drives, minus the window:
    /// `resumed` parks a tree, the platform asks the activation handler for it,
    /// and the posted event turns frames on. Returns whatever the handler
    /// actually served, so a law can tell the synchronous branch from the
    /// placeholder one.
    #[cfg(test)]
    pub(crate) fn attach_assistive_technology_for_test(&mut self) -> Option<accesskit::TreeUpdate> {
        self.seed_accessibility_tree();
        let served = self.frame.activate_accessibility_for_test();
        // What `handle_accessibility_event(InitialTreeRequested)` does next.
        self.refresh_accessibility();
        served
    }

    #[cfg(test)]
    pub(crate) fn accessibility_stats(&self) -> ProjectionStats {
        self.frame.accessibility_stats()
    }

    #[cfg(test)]
    pub(crate) fn set_semantic_text_for_test(&mut self, text: &str) {
        self.document.set_text(text);
        self.document.set_cursor(text.chars().count());
    }

    /// A real selection over the document, in CHAR offsets — the projection
    /// laws in `semantic::native` cannot reach `DocumentSession` themselves.
    #[cfg(test)]
    pub(crate) fn set_semantic_selection_for_test(&mut self, anchor: usize, cursor: usize) {
        self.document.set_anchor(anchor);
        self.document.set_cursor(cursor);
    }

    /// Put this App on a named surface. The projection laws live in
    /// `semantic::native`, which cannot reach `WorkspaceState`'s private
    /// transitions, so the fixtures are named here beside the folds they
    /// exercise.
    #[cfg(test)]
    pub(crate) fn install_semantic_fixture_for_test(&mut self, surface: &str) {
        match surface {
            "editor" => {}
            "overlay" => {
                self.workspace_state
                    .install_overlay_for_test(crate::overlay::OverlayState::new(
                        crate::overlay::OverlayKind::Command,
                        vec!["alpha".to_string(), "beta".to_string()],
                        Vec::new(),
                        Vec::new(),
                    ))
            }
            "search" => {
                self.workspace_state
                    .install_search_for_test(crate::search::SearchState::start(
                        0,
                        crate::search::Direction::Forward,
                    ));
            }
            other => panic!("unknown semantic fixture {other}"),
        }
    }
}

#[cfg(test)]
mod tests;
