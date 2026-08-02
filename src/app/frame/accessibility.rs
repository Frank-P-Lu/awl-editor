//! Window-lifecycle owner for the native AccessKit adapter.

use super::*;

pub(super) struct AccessibilityRuntime {
    proxy: Option<winit::event_loop::EventLoopProxy<AwlEvent>>,
    adapter: Option<accesskit_winit::Adapter>,
    last: Option<crate::semantic::SemanticSnapshot>,
    /// Is an assistive technology listening? `false` until the platform asks
    /// for an initial tree, and `false` again the moment it lets go. A
    /// snapshot costs a whole-rope `String` plus a UAX #29 pass over it, so
    /// the ordinary no-AT frame must not build one — this bit, not the
    /// equality dedup below, is what keeps per-frame work off the document.
    active: bool,
}

impl AccessibilityRuntime {
    pub(super) fn new() -> Self {
        Self {
            proxy: None,
            adapter: None,
            last: None,
            active: false,
        }
    }

    pub(super) fn set_active(&mut self, active: bool) {
        self.active = active;
        if !active {
            self.last = None;
        }
    }

    /// The one question a frame asks before paying for a snapshot.
    pub(super) fn wants_snapshot(&self) -> bool {
        self.active
    }

    pub(super) fn set_proxy(&mut self, proxy: winit::event_loop::EventLoopProxy<AwlEvent>) {
        self.proxy = Some(proxy);
    }

    pub(super) fn install(&mut self, event_loop: &ActiveEventLoop, window: &Window) {
        let Some(proxy) = self.proxy.take() else {
            return;
        };
        // AccessKit must own the window before the platform can see it: on
        // macOS VoiceOver caches the accessibility parent of a newly ordered-in
        // window, so an adapter installed after `set_visible(true)` is not
        // asked for a tree until the window is cycled.
        self.adapter = Some(accesskit_winit::Adapter::with_event_loop_proxy(
            event_loop, window, proxy,
        ));
    }

    pub(super) fn process_window_event(&mut self, window: &Window, event: &WindowEvent) {
        if let Some(adapter) = self.adapter.as_mut() {
            adapter.process_event(window, event);
        }
    }

    pub(super) fn update(&mut self, snapshot: crate::semantic::SemanticSnapshot, force: bool) {
        if !semantic_update_needed(self.last.as_ref(), &snapshot, force) {
            return;
        }
        if let Some(adapter) = self.adapter.as_mut() {
            adapter.update_if_active(|| crate::semantic::native::tree_update(&snapshot));
        }
        self.last = Some(snapshot);
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
    pub(in crate::app) fn update_accessibility(
        &mut self,
        snapshot: crate::semantic::SemanticSnapshot,
        force: bool,
    ) {
        self.accessibility.update(snapshot, force);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(in crate::app) fn set_accessibility_active(&mut self, active: bool) {
        self.accessibility.set_active(active);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(in crate::app) fn accessibility_wants_snapshot(&self) -> bool {
        self.accessibility.wants_snapshot()
    }
}

fn semantic_update_needed(
    previous: Option<&crate::semantic::SemanticSnapshot>,
    next: &crate::semantic::SemanticSnapshot,
    force: bool,
) -> bool {
    force || previous != Some(next)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn animation_only_frames_do_not_emit_semantic_updates() {
        let _guard = crate::testlock::serial();
        let app = App::new_hermetic(None, PathBuf::from("/"), Config::empty());
        let snapshot = app.semantic_snapshot();
        assert!(semantic_update_needed(None, &snapshot, false));
        assert!(!semantic_update_needed(Some(&snapshot), &snapshot, false));
        assert!(semantic_update_needed(Some(&snapshot), &snapshot, true));
    }

    /// The dedup above runs AFTER a snapshot exists, so it cannot answer "did
    /// this frame pay for one". `wants_snapshot` is the gate that can, and it
    /// follows the platform's own two signals rather than a heuristic.
    #[test]
    fn a_frame_pays_for_a_snapshot_only_while_an_assistive_technology_listens() {
        let mut runtime = AccessibilityRuntime::new();
        assert!(!runtime.wants_snapshot());
        runtime.set_active(true);
        assert!(runtime.wants_snapshot());
        runtime.set_active(false);
        assert!(!runtime.wants_snapshot());
    }

    /// Deactivation must forget the last snapshot, or a reattached screen
    /// reader whose first frame happens to be state-identical would be handed
    /// the initial tree and then nothing — the dedup would suppress the very
    /// update that repopulates the adapter.
    #[test]
    fn deactivation_forgets_the_last_snapshot_so_a_reattach_is_not_deduped_away() {
        let _guard = crate::testlock::serial();
        let app = App::new_hermetic(None, PathBuf::from("/"), Config::empty());
        let mut runtime = AccessibilityRuntime::new();
        runtime.set_active(true);
        runtime.last = Some(app.semantic_snapshot());
        runtime.set_active(false);
        assert!(runtime.last.is_none());
    }
}
