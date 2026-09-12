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
    /// The empty copy a specific lens would show in this card's current mode.
    /// The right-hug roster and the live empty row both read this owner so a
    /// width can never be measured without one of the strings it may display.
    pub(super) fn empty_message_for_lens(&self, lens: &str) -> &'static str {
        if self.kind == OverlayKind::Goto && !self.files_mode {
            return match lens {
                "files" => "no files here",
                "headings" => "no headings yet",
                "folders" => "no folders here",
                "recent" => "no recent destinations",
                _ => self.kind.empty_corpus_message(),
            };
        }
        self.kind
            .empty_lens_message(lens)
            .unwrap_or_else(|| self.kind.empty_corpus_message())
    }

    pub fn empty_message(&self) -> String {
        if !self.query.is_empty() {
            return "no matches".to_string();
        }
        self.active_facet_id()
            .map(|lens| self.empty_message_for_lens(lens))
            .unwrap_or_else(|| self.kind.empty_corpus_message())
            .to_string()
    }

    pub fn empty_notice(&self) -> Option<String> {
        if self.files_mode && !self.notice.is_empty() {
            return Some(self.notice.clone());
        }
        self.items.is_empty().then(|| self.empty_message())
    }
}
