//! Driver-owned facts folded into the renderer and sidecar views.

use super::CaptureOpts;

impl CaptureOpts {
    /// Fold the driver's working-set facts into the capture's view state: the
    /// persistent changed-elsewhere affordance, widened bottom identity, the
    /// active file's remembered root, and the set-level outline reservation.
    pub(in crate::capture) fn fold_gutter(&self, view: &mut crate::render::ViewState) {
        view.set_wants_outline_rail = self.set_wants_outline_rail;
        view.document_active = !self.document_absent;
        if self.document_absent {
            view.gutter_name.clear();
            view.gutter_project.clear();
            view.gutter_files.clear();
            // With no active file, the ambient project is the only folder
            // identity; its name is already reduced to one path component.
            view.start_folder = self.project.as_ref().map(|p| p.name.clone());
        } else {
            view.gutter_files.clone_from(&self.working_set);
            if let Some(root) = &self.gutter_project_root {
                view.gutter_project = crate::project::folder_name(root);
            }
        }
        view.gutter_changed = self.gutter_changed;
    }

    /// The `semantic` sidecar field: the live-App semantic tree serialized
    /// verbatim, or JSON `null`.
    pub(in crate::capture) fn semantic_json(&self) -> String {
        self.semantic
            .as_ref()
            .map(|snapshot| serde_json::to_string(snapshot).expect("semantic snapshot serializes"))
            .unwrap_or_else(|| "null".to_string())
    }
}
