//! The winit user-event type, lifted out of `app.rs` so the enum can grow
//! without the root file growing.

/// The winit USER EVENT type this app's event loop carries: the single-
/// instance daemon's posted events on every native platform, PLUS (macOS
/// only) a fired native menu-bar item's raw id — an uninhabited no-op on wasm
/// (the browser has no process/socket/menu-bar concept; `crate::daemon` and
/// `crate::menu` both compile out there entirely). Growing this enum (the
/// `Menu` variant) is what FORCES `user_event`'s match below to grow a
/// matching arm — the exhaustiveness check is the whole point.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) enum AwlEvent {
    #[cfg(not(feature = "mas"))]
    Daemon(crate::daemon::DaemonEvent),
    #[cfg(target_os = "macos")]
    Menu(String),
    /// A live-probe step posted by the `--live-script` driver thread (see
    /// `crate::probe`'s module doc) — a scripted chord for the real dispatch
    /// tail, a compositor-side window shot, or the terminating quit.
    Probe(crate::probe::ProbeEvent),
    Accessibility(accesskit_winit::Event),
}
#[cfg(not(target_arch = "wasm32"))]
impl From<accesskit_winit::Event> for AwlEvent {
    fn from(event: accesskit_winit::Event) -> Self {
        Self::Accessibility(event)
    }
}
#[cfg(target_arch = "wasm32")]
pub(crate) type AwlEvent = ();
