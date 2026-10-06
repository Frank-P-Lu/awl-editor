//! The clipboard surface `app/apply.rs`'s kill-ring bridge
//! (`sync_kill_to_clipboard`/`refresh_kill_from_clipboard`/
//! `paste_image_reference`) needs, abstracted behind a trait so a test can
//! inject a deterministic fake instead of reaching the real OS pasteboard —
//! `arboard::Clipboard` has no test backend of its own, and reaching the
//! real one from a unit test is either flaky (a shared, host-ambient
//! resource under parallel test threads) or a silent no-op (a headless CI
//! runner with no clipboard service at all), neither of which can prove a
//! buffer-switch/clipboard-bridge law.
pub(super) trait ClipboardBackend {
    fn set_text(&mut self, text: String) -> Result<(), ()>;
    fn get_text(&mut self) -> Result<String, ()>;
    fn get_image(&mut self) -> Result<arboard::ImageData<'static>, ()>;
}

impl ClipboardBackend for arboard::Clipboard {
    fn set_text(&mut self, text: String) -> Result<(), ()> {
        self.set_text(text).map_err(|_| ())
    }
    fn get_text(&mut self) -> Result<String, ()> {
        self.get_text().map_err(|_| ())
    }
    fn get_image(&mut self) -> Result<arboard::ImageData<'static>, ()> {
        self.get_image().map_err(|_| ())
    }
}

/// A deterministic, hermetic stand-in for the OS clipboard. `Clone`s share
/// the same backing cell, so a test can keep one handle installed on `App`
/// and a second in its own scope to simulate an EXTERNAL app changing the
/// clipboard behind awl's back — the exact shape a buffer-switch law needs
/// to drive independently of anything `App` itself last wrote.
#[derive(Clone, Default)]
pub(crate) struct MemoryClipboard(std::sync::Arc<std::sync::Mutex<Option<String>>>);

impl MemoryClipboard {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Set the clipboard's content as if another application had just
    /// written it — never touches `App`'s own `clipboard_last_written`.
    #[cfg(test)]
    pub(crate) fn set_external(&self, text: &str) {
        *self.0.lock().unwrap() = Some(text.to_string());
    }

    /// What the (fake) OS clipboard currently holds, for assertions.
    #[cfg(test)]
    pub(crate) fn current(&self) -> Option<String> {
        self.0.lock().unwrap().clone()
    }
}

impl ClipboardBackend for MemoryClipboard {
    fn set_text(&mut self, text: String) -> Result<(), ()> {
        *self.0.lock().unwrap() = Some(text);
        Ok(())
    }
    fn get_text(&mut self) -> Result<String, ()> {
        self.0.lock().unwrap().clone().ok_or(())
    }
    fn get_image(&mut self) -> Result<arboard::ImageData<'static>, ()> {
        Err(()) // text-only fake: the OS clipboard never holds an image
    }
}

/// Construction routes select ownership before any platform handle is acquired.
#[derive(Clone, Copy, Debug)]
pub(super) enum Route {
    Live,
    #[cfg(test)]
    Unit,
    Capture,
    Scheduler,
    Persistence,
}

impl Route {
    pub(super) fn default_app() -> Self {
        #[cfg(test)]
        {
            Self::Unit
        }
        #[cfg(not(test))]
        {
            Self::Live
        }
    }
}

fn select<T>(route: Route, system: impl FnOnce() -> T, memory: impl FnOnce() -> T) -> T {
    match route {
        Route::Live => system(),
        #[cfg(test)]
        Route::Unit => memory(),
        Route::Capture | Route::Scheduler | Route::Persistence => memory(),
    }
}

pub(super) fn for_route(route: Route) -> Option<super::ClipboardHandle> {
    select(route, system, || Some(Box::new(MemoryClipboard::new())))
}

#[cfg(not(test))]
fn system() -> Option<super::ClipboardHandle> {
    match arboard::Clipboard::new() {
        Ok(c) => Some(Box::new(c)),
        Err(e) => {
            super::clipboard_disabled(e);
            None
        }
    }
}

#[cfg(test)]
fn system() -> Option<super::ClipboardHandle> {
    panic!("unit App construction attempted a system clipboard factory");
}

#[cfg(test)]
mod tests;
