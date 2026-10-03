//! Stable launcher identity, independent of the executable filename and title.

use winit::event_loop::ActiveEventLoop;
use winit::platform::wayland::ActiveEventLoopExtWayland;
use winit::window::WindowAttributes;

const X11_CLASS: &str = "awl";
const WAYLAND_APP_ID: &str = "dev.franklu.awl";

pub(super) fn apply(attrs: WindowAttributes, event_loop: &ActiveEventLoop) -> WindowAttributes {
    for_backend(attrs, event_loop.is_wayland())
}

fn for_backend(attrs: WindowAttributes, wayland: bool) -> WindowAttributes {
    // Both extension traits write winit's same ApplicationName field. Applying
    // both would let the last call overwrite the active backend's identity.
    if wayland {
        winit::platform::wayland::WindowAttributesExtWayland::with_name(
            attrs,
            WAYLAND_APP_ID,
            X11_CLASS,
        )
    } else {
        winit::platform::x11::WindowAttributesExtX11::with_name(attrs, X11_CLASS, X11_CLASS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_launcher_metadata_matches_both_backend_identities() {
        let _guard = crate::testlock::serial();
        let packager = include_str!("../../scripts/package-appimage.sh");
        assert!(packager.contains(&format!("StartupWMClass={X11_CLASS}\n")));
        assert!(packager.contains(&format!("APP_ID=\"${{AWL_BUNDLE_ID:-{WAYLAND_APP_ID}}}\"")));
        assert!(packager.contains("DESKTOP=\"$APPDIR/$APP_ID.desktop\""));
    }

    #[test]
    fn each_backend_sets_the_actual_winit_application_name() {
        let _guard = crate::testlock::serial();
        for (wayland, general) in [(false, "awl"), (true, "dev.franklu.awl")] {
            let result = for_backend(WindowAttributes::default(), wayland);
            // Winit exposes its platform fields only through Debug. Inspect the
            // assigned field, not a parallel table of intended names.
            let expected = format!(
                "name: Some(ApplicationName {{ general: {general:?}, instance: \"awl\" }})"
            );
            assert!(format!("{result:?}").contains(&expected), "{result:?}");
        }
    }

    #[test]
    fn backend_assignment_preserves_the_other_window_attributes() {
        let _guard = crate::testlock::serial();
        for wayland in [false, true] {
            let attrs = WindowAttributes::default()
                .with_title("a changing document title")
                .with_visible(false)
                .with_inner_size(winit::dpi::LogicalSize::new(720.0, 480.0));
            let expected_size = attrs.inner_size;
            let result = for_backend(attrs, wayland);
            assert_eq!(result.title, "a changing document title");
            assert!(!result.visible);
            assert_eq!(result.inner_size, expected_size);
        }
    }
}
