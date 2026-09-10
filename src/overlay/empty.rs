//! Empty-state copy shared by rendering, semantics, and capture sidecars.

use super::{OverlayKind, OverlayState};

impl OverlayKind {
    pub fn empty_lens_message(self, lens: &str) -> Option<&'static str> {
        match (self, lens) {
            (OverlayKind::Goto, "files") => Some("this folder is empty"),
            (OverlayKind::Goto, "recent") => Some("no recent files yet"),
            (OverlayKind::Project, "recent") => Some("no recent projects yet"),
            (_, "all") => None,
            _ => Some("nothing here"),
        }
    }
}

impl OverlayState {
    pub fn empty_message(&self) -> String {
        if !self.query.is_empty() {
            return "no matches".to_string();
        }
        if self.kind == OverlayKind::Goto && !self.files_mode {
            return match self.active_facet_id() {
                Some("files") => "no files here",
                Some("headings") => "no headings yet",
                Some("folders") => "no folders here",
                Some("recent") => "no recent destinations",
                _ => self.kind.empty_corpus_message(),
            }
            .to_string();
        }
        self.active_facet_id()
            .and_then(|lens| self.kind.empty_lens_message(lens))
            .unwrap_or_else(|| self.kind.empty_corpus_message())
            .to_string()
    }

    pub fn empty_notice(&self) -> Option<String> {
        self.items.is_empty().then(|| self.empty_message())
    }
}
