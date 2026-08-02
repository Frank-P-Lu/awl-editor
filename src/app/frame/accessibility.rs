//! Window-lifecycle owner for the native AccessKit adapter.

use super::*;

pub(super) struct AccessibilityRuntime {
    proxy: Option<winit::event_loop::EventLoopProxy<AwlEvent>>,
    adapter: Option<accesskit_winit::Adapter>,
    last: Option<crate::semantic::SemanticSnapshot>,
}

impl AccessibilityRuntime {
    pub(super) fn new() -> Self {
        Self {
            proxy: None,
            adapter: None,
            last: None,
        }
    }

    pub(super) fn set_proxy(&mut self, proxy: winit::event_loop::EventLoopProxy<AwlEvent>) {
        self.proxy = Some(proxy);
    }

    pub(super) fn install(&mut self, event_loop: &ActiveEventLoop, window: &Window) {
        let Some(proxy) = self.proxy.take() else {
            return;
        };
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
}
