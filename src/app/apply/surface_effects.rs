//! Platform-owned surfaces emitted by the shared action core.

use crate::app::*;

impl App {
    pub(super) fn apply_surface_effect(&mut self, effect: crate::actions::SurfaceEffect) {
        match effect {
            crate::actions::SurfaceEffect::ShowAbout => self.show_about_surface(),
            crate::actions::SurfaceEffect::OpenFileChooser => self.open_file_chooser_surface(),
            crate::actions::SurfaceEffect::OpenFolderChooser => {
                self.open_folder_chooser_surface();
            }
        }
    }

    fn show_about_surface(&mut self) {
        #[cfg(target_os = "macos")]
        crate::mac_about::show();
        #[cfg(not(target_os = "macos"))]
        crate::about::set_open(true);
    }

    fn open_file_chooser_surface(&mut self) {
        #[cfg(target_os = "macos")]
        if self.frame.gpu().is_some()
            && let Some(path) =
                crate::mac_chrome::pick_file_to_open(Some(&self.project_location.root))
        {
            self.apply_file_choice(Some(path));
        }
        #[cfg(not(target_os = "macos"))]
        self.open_fallback_chooser(crate::overlay::OverlayKind::Browse);
    }

    fn open_folder_chooser_surface(&mut self) {
        #[cfg(target_os = "macos")]
        if self.frame.gpu().is_some()
            && let Some(path) = crate::mac_chrome::pick_folder_to_open(
                self.project_location.workspace_root.as_deref(),
            )
        {
            self.apply_folder_choice(Some(path));
        }
        #[cfg(not(target_os = "macos"))]
        self.open_fallback_chooser(crate::overlay::OverlayKind::ProjectBrowse);
    }

    #[cfg(not(target_os = "macos"))]
    fn open_fallback_chooser(&mut self, kind: crate::overlay::OverlayKind) {
        let overlay = crate::overlay::browse_level(
            kind,
            None,
            &self.project_location.root,
            self.project_location.workspace_root.as_deref(),
            &[],
        );
        self.workspace_state.core_slots().1.enter(overlay);
    }
}
