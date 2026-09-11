use super::{OverlayKind, OverlayState};

impl OverlayState {
    /// The visible errand, including a root-relative destination once the
    /// navigator descends. Save Copy and Move keep their typed purpose here.
    pub fn title(&self) -> String {
        if self.save_copy && self.kind == OverlayKind::ExportDest {
            self.with_browse_dir_suffix(self.subject_errand("save a copy of", "save a copy to"))
        } else if self.save_copy_dest.is_some() && self.rename_edit.is_some() {
            "save a copy as".to_string()
        } else if let Some(name) = self
            .move_filename
            .as_deref()
            .filter(|_| self.kind == OverlayKind::MoveDest)
        {
            self.move_dest_title(name)
        } else if self.kind == OverlayKind::ExportDest {
            self.with_browse_dir_suffix(self.subject_errand("export", self.kind.title()))
        } else if self.kind == OverlayKind::History {
            self.subject_name
                .as_deref()
                .map(|name| format!("history of {name}"))
                .unwrap_or_else(|| self.kind.title().to_string())
        } else if self.kind == OverlayKind::SearchFolder {
            self.search_root
                .as_deref()
                .map(crate::project::folder_name)
                .filter(|name| !name.is_empty())
                .map(|name| format!("search in {name}"))
                .unwrap_or_else(|| self.kind.title().to_string())
        } else if let Some(title) = self.files_title() {
            title
        } else {
            self.kind.title().to_string()
        }
    }

    fn subject_errand(&self, with_subject: &str, fallback: &str) -> String {
        self.subject_name
            .as_deref()
            .map(|name| format!("{with_subject} {name} to"))
            .unwrap_or_else(|| fallback.to_string())
    }

    /// `title`'s composition for [`OverlayKind::MoveDest`]: `"move {name}"`
    /// at the level it opened at, `"move {name} to {dir}/"` once descended.
    fn move_dest_title(&self, name: &str) -> String {
        match self.browse_dir_display() {
            Some(dir) => format!("move {name} to {dir}"),
            None => format!("move {name}"),
        }
    }

    /// Append the current destination folder to `base`, unchanged at root.
    fn with_browse_dir_suffix(&self, base: String) -> String {
        match self.browse_dir_display() {
            Some(dir) => format!("{base} {dir}"),
            None => base,
        }
    }

    /// A root-relative destination with a trailing slash, or `None` at its
    /// opening level and for absolute workspace walkers.
    fn browse_dir_display(&self) -> Option<String> {
        if !super::consumer_for_card(self)
            .is_some_and(super::LocationConsumer::shows_relative_breadcrumb)
        {
            return None;
        }
        let dir = self.browse_dir.as_deref()?;
        if dir.is_empty() {
            return None;
        }
        Some(format!("{dir}/"))
    }
}
