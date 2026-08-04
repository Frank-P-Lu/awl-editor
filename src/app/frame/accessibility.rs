//! Window-lifecycle owner for the native AccessKit adapter.
//!
//! Two things live here that a screen reader's responsiveness depends on.
//!
//! **A synchronous activation handler.** The event-loop-proxy adapter cannot
//! answer `request_initial_tree` on the spot — it posts an event and returns
//! `None` — so a platform adapter holds a placeholder tree, and every update
//! afterwards is required to carry a FULL tree. `with_mixed_handlers` lets the
//! activation handler answer from a thread-safe slot the main loop filled
//! before the window was shown, which is the ordinary case for a VoiceOver user
//! (the screen reader is already running when awl launches).
//!
//! **Incremental updates afterwards.** AccessKit expects a full tree once, at
//! activation, and changed nodes from then on. Republishing the whole document
//! on every redraw is what a user hears as a stall: while an assistive
//! technology is attached, a one-character edit would clone the rope, run
//! UAX #29 over the entire document and re-send every node — enough, on a book,
//! for VoiceOver to report the app as not responding.
//!
//! **The activation handler runs on a platform thread and touches no `App`.**
//! It reads a mutex and posts one winit event; every transition still happens
//! on the main loop, which is the same rule the action handler has always
//! followed.

use super::*;
#[cfg(test)]
use crate::app::semantic::ProjectionStats;
use crate::app::semantic::SemanticProjection;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

/// The tree an assistive technology may ask for at any moment, on any thread.
struct SharedTree {
    slot: Mutex<Option<crate::semantic::SemanticSnapshot>>,
    /// Does `slot` still describe the live `App`? Maintained by the main loop
    /// with one integer compare per frame; never by anything that builds.
    fresh: AtomicBool,
    /// Did the activation handler serve a real tree? When it did, the platform
    /// holds exactly what the retained projection published and the first
    /// update may be a diff. When it did not, the platform holds a placeholder
    /// and AccessKit requires the next update to be a full tree.
    served: AtomicBool,
}

impl SharedTree {
    fn new() -> Self {
        Self {
            slot: Mutex::new(None),
            fresh: AtomicBool::new(false),
            served: AtomicBool::new(false),
        }
    }
}

/// The synchronous activation handler. Deliberately tiny, and deliberately
/// unable to reach an `App`: a platform callback that ran a transition would be
/// a re-entrancy bug on whatever thread the platform chose.
struct AwlActivationHandler {
    shared: Arc<SharedTree>,
    proxy: winit::event_loop::EventLoopProxy<AwlEvent>,
    window_id: winit::window::WindowId,
}

/// The activation decision, with no winit in it, so a law can drive the real
/// one rather than a re-implementation: serve the parked tree when it still
/// describes the app, otherwise `None` — and record which branch was taken,
/// because that is what decides whether the first update owes a full tree.
fn serve_initial_tree(shared: &SharedTree) -> Option<accesskit::TreeUpdate> {
    let tree = if shared.fresh.load(Ordering::SeqCst) {
        shared
            .slot
            .lock()
            .ok()
            .and_then(|slot| slot.as_ref().map(crate::semantic::native::tree_update))
    } else {
        None
    };
    shared.served.store(tree.is_some(), Ordering::SeqCst);
    tree
}

impl accesskit::ActivationHandler for AwlActivationHandler {
    fn request_initial_tree(&mut self) -> Option<accesskit::TreeUpdate> {
        let tree = serve_initial_tree(&self.shared);
        // Wake the main loop so frames begin refreshing the tree. The winit
        // adapter's own proxy path posts this same event; reusing it keeps one
        // door into `App::handle_accessibility_event`.
        self.proxy
            .send_event(AwlEvent::from(accesskit_winit::Event {
                window_id: self.window_id,
                window_event: accesskit_winit::WindowEvent::InitialTreeRequested,
            }))
            .ok();
        tree
    }
}

pub(super) struct AccessibilityRuntime {
    proxy: Option<winit::event_loop::EventLoopProxy<AwlEvent>>,
    adapter: Option<accesskit_winit::Adapter>,
    shared: Arc<SharedTree>,
    /// The retained projection. Moved out for the duration of a refresh so the
    /// refresh can borrow the whole `App` immutably.
    projection: Option<SemanticProjection>,
    /// The retained NATIVE half — the document's child ids, which travel with
    /// the document node on every keystroke and are the one document-sized
    /// thing an incremental update would otherwise rebuild.
    projector: crate::semantic::native::TreeProjector,
    /// The document state the slot's snapshot was built from — identity AND
    /// revision, so a buffer swap that restarts `version` at 0 cannot be
    /// mistaken for the same document.
    published_state: (u64, u64),
    /// The focus this runtime last published, so a focus move with no node
    /// change still reaches the platform.
    published_focus: String,
    /// Is an assistive technology listening? `false` until the platform asks
    /// for an initial tree, and `false` again the moment it lets go.
    active: bool,
    /// Does the platform hold a placeholder rather than something this
    /// projection published? Then it is owed a full tree, once.
    owes_full: bool,
    /// What the platform was handed, in order: `Some(update)` for each tree
    /// published, and `None` where an activation could not be served — which is
    /// not nothing, it is the platform DROPPING what it held for a placeholder
    /// (`AdapterState::Pending` carries no tree, on both backends). A law
    /// replays these the way an adapter does and asks what a screen reader would
    /// find, which is the one question `ProjectionStats` cannot answer.
    #[cfg(test)]
    published: Vec<Option<accesskit::TreeUpdate>>,
}

impl AccessibilityRuntime {
    pub(super) fn new() -> Self {
        Self {
            proxy: None,
            adapter: None,
            shared: Arc::new(SharedTree::new()),
            projection: None,
            projector: crate::semantic::native::TreeProjector::default(),
            published_state: (0, 0),
            published_focus: String::new(),
            active: false,
            owes_full: true,
            #[cfg(test)]
            published: Vec::new(),
        }
    }

    pub(super) fn set_active(&mut self, active: bool) {
        if active {
            // Keyed on what the handler SERVED, never on the transition. An
            // activation can arrive while this runtime already believes an
            // assistive technology is attached — a re-asked initial tree, which
            // macOS issues when a window is cycled — and a repeat request that
            // could not be served leaves the platform holding a placeholder
            // exactly as a first one does. Early-returning on `active == true`
            // skipped that bookkeeping and left the platform holding whatever
            // it had, with every later update a diff against it.
            if !self.shared.served.load(Ordering::SeqCst) {
                // A placeholder tree: the diff would be against something the
                // platform never saw.
                self.invalidate();
            }
            self.active = true;
        } else if self.active {
            self.active = false;
            // A reattach gets a platform adapter that never saw what we
            // published, so nothing retained may be diffed against.
            self.invalidate();
            self.shared.served.store(false, Ordering::SeqCst);
            self.shared.fresh.store(false, Ordering::SeqCst);
        }
    }

    fn invalidate(&mut self) {
        if let Some(projection) = self.projection.as_mut() {
            projection.invalidate();
        }
        self.projector.invalidate();
        self.published_focus.clear();
        self.owes_full = true;
    }

    /// The one question a frame asks before paying for a refresh.
    pub(super) fn wants_snapshot(&self) -> bool {
        self.active
    }

    /// Record whether the tree published for a future activation still
    /// describes the live `App`. One atomic store; nothing is built.
    pub(super) fn note_published_currency(&self, current: bool) {
        self.shared.fresh.store(current, Ordering::SeqCst);
    }

    pub(super) fn published_state(&self) -> (u64, u64) {
        self.published_state
    }

    pub(super) fn take_projection(&mut self) -> SemanticProjection {
        self.projection.take().unwrap_or_default()
    }

    pub(super) fn projection(&self) -> Option<&SemanticProjection> {
        self.projection.as_ref()
    }

    #[cfg(test)]
    pub(super) fn stats(&self) -> ProjectionStats {
        self.projection
            .as_ref()
            .map(SemanticProjection::stats)
            .unwrap_or_default()
    }

    pub(super) fn set_proxy(&mut self, proxy: winit::event_loop::EventLoopProxy<AwlEvent>) {
        self.proxy = Some(proxy);
    }

    /// Park a freshly built projection and hand its snapshot to the activation
    /// handler. Called once, before the window is shown, so the handler has a
    /// real tree to serve the instant the platform asks.
    pub(super) fn seed(&mut self, projection: SemanticProjection, state: (u64, u64)) {
        if let Ok(mut slot) = self.shared.slot.lock() {
            *slot = Some(projection.snapshot().clone());
        }
        self.published_state = state;
        self.shared.fresh.store(true, Ordering::SeqCst);
        // A tree is parked for the handler to serve, so a first update that
        // finds it was served owes a diff, not another whole document.
        self.owes_full = false;
        self.projection = Some(projection);
    }

    /// Drive the REAL activation decision without a window, so the laws
    /// exercise `serve_initial_tree` rather than a second copy of it.
    #[cfg(test)]
    pub(super) fn activate_for_test(&mut self) -> Option<accesskit::TreeUpdate> {
        let tree = serve_initial_tree(&self.shared);
        // The handler's answer reaches the platform exactly as an update does,
        // and its ABSENCE reaches it too — so both branches are recorded.
        self.published.push(tree.clone());
        self.set_active(true);
        tree
    }

    /// Every tree the platform has been handed, in order.
    #[cfg(test)]
    pub(super) fn published_trees(&self) -> &[Option<accesskit::TreeUpdate>] {
        &self.published
    }

    pub(super) fn install(&mut self, event_loop: &ActiveEventLoop, window: &Window) {
        let Some(proxy) = self.proxy.take() else {
            return;
        };
        let activation = AwlActivationHandler {
            shared: Arc::clone(&self.shared),
            proxy: proxy.clone(),
            window_id: window.id(),
        };
        // AccessKit must own the window before the platform can see it: on
        // macOS VoiceOver caches the accessibility parent of a newly ordered-in
        // window, so an adapter installed after `set_visible(true)` is not
        // asked for a tree until the window is cycled.
        //
        // MIXED handlers, not the proxy constructor: the action and
        // deactivation events are fine asynchronously, but activation is the
        // one call whose answer has to be synchronous, because returning `None`
        // is what forces every later update to carry a full tree.
        self.adapter = Some(accesskit_winit::Adapter::with_mixed_handlers(
            event_loop, window, activation, proxy,
        ));
    }

    pub(super) fn process_window_event(&mut self, window: &Window, event: &WindowEvent) {
        if let Some(adapter) = self.adapter.as_mut() {
            adapter.process_event(window, event);
        }
    }

    /// Hand a refreshed projection back, publishing whatever it changed.
    pub(super) fn publish(&mut self, mut projection: SemanticProjection) {
        if !self.active {
            self.projection = Some(projection);
            return;
        }
        let focus_moved = projection.snapshot().focus_id != self.published_focus;
        let full = self.owes_full || !projection.is_seeded();
        let shape = projection.shape_rev();
        // Built eagerly rather than inside `update_if_active`'s closure. The
        // expensive half — reading the rope and re-segmenting it — already
        // happened in the projection; what is left is projecting the nodes this
        // update actually names, which is the whole document only once per
        // activation and two nodes per keystroke. Paying that in the rare frame
        // where the platform adapter has gone inactive under us buys the thing
        // the closure form cannot give: the update is a VALUE, so one door can
        // see every byte that reaches the platform.
        let update = if full {
            let update = self.projector.full(projection.snapshot(), shape);
            projection.note_full_tree();
            Some(update)
        } else if !projection.changed().is_empty() || focus_moved {
            let count = projection.changed().len();
            let update =
                self.projector
                    .incremental(projection.snapshot(), projection.changed(), shape);
            projection.note_incremental(count);
            Some(update)
        } else {
            None
        };
        if let Some(update) = update {
            self.emit(update);
        }
        self.owes_full = false;
        self.published_focus = projection.snapshot().focus_id.clone();
        self.projection = Some(projection);
    }

    /// The ONE door from this runtime to the platform's tree.
    ///
    /// Every update goes through here, so "what does the screen reader actually
    /// hold" is answerable by recording at a single point rather than by
    /// reasoning about the adapter. Without a door, the laws could only assert
    /// on projection COUNTERS — and a counter says an update was published, not
    /// what was in it.
    fn emit(&mut self, update: accesskit::TreeUpdate) {
        #[cfg(test)]
        self.published.push(Some(update.clone()));
        if let Some(adapter) = self.adapter.as_mut() {
            adapter.update_if_active(move || update);
        }
    }
}

/// The runtime is private to this file; `FrameRuntime` is the door every App
/// call goes through, so "who may talk to the adapter" stays answerable with
/// one grep.
impl FrameRuntime {
    #[cfg(not(target_arch = "wasm32"))]
    pub(in crate::app) fn set_accessibility_proxy(
        &mut self,
        proxy: winit::event_loop::EventLoopProxy<AwlEvent>,
    ) {
        self.accessibility.set_proxy(proxy);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(in crate::app) fn install_accessibility(
        &mut self,
        event_loop: &ActiveEventLoop,
        window: &Window,
    ) {
        self.accessibility.install(event_loop, window);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(in crate::app) fn process_accessibility_window_event(
        &mut self,
        window: &Window,
        event: &WindowEvent,
    ) {
        self.accessibility.process_window_event(window, event);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(in crate::app) fn set_accessibility_active(&mut self, active: bool) {
        self.accessibility.set_active(active);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(in crate::app) fn accessibility_wants_snapshot(&self) -> bool {
        self.accessibility.wants_snapshot()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(in crate::app) fn note_published_tree_currency(&self, current: bool) {
        self.accessibility.note_published_currency(current);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(in crate::app) fn published_document_state(&self) -> (u64, u64) {
        self.accessibility.published_state()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(in crate::app) fn take_accessibility_projection(&mut self) -> SemanticProjection {
        self.accessibility.take_projection()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(in crate::app) fn accessibility_projection(&self) -> Option<&SemanticProjection> {
        self.accessibility.projection()
    }

    #[cfg(all(test, not(target_arch = "wasm32")))]
    pub(in crate::app) fn accessibility_stats(&self) -> ProjectionStats {
        self.accessibility.stats()
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(in crate::app) fn publish_accessibility(&mut self, projection: SemanticProjection) {
        self.accessibility.publish(projection);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(in crate::app) fn seed_accessibility(
        &mut self,
        projection: SemanticProjection,
        state: (u64, u64),
    ) {
        self.accessibility.seed(projection, state);
    }

    #[cfg(test)]
    pub(in crate::app) fn activate_accessibility_for_test(
        &mut self,
    ) -> Option<accesskit::TreeUpdate> {
        self.accessibility.activate_for_test()
    }

    #[cfg(test)]
    pub(in crate::app) fn published_accessibility_trees(&self) -> &[Option<accesskit::TreeUpdate>] {
        self.accessibility.published_trees()
    }
}

#[cfg(test)]
mod tests;
