//! `OverlayState`'s FACETING glue -- the lens strip a picker like Theme /
//! Project / History / Command shows, generalized over
//! [`crate::facets::FacetScheme`]. Split out of the former `overlay.rs`
//! monolith (2026-07 code-organization pass); every item's path is
//! unchanged -- only the file it lives in moved.

use super::{OverlayKind, OverlayState};

impl OverlayState {
    pub(super) fn filters_to_active_facet(&self) -> bool {
        self.facet_lens != 0 && !(self.kind == OverlayKind::Command && !self.query.is_empty())
    }

    /// This picker's FACETING scheme (its lens strip + item bucketing), or `None`
    /// for a non-faceting picker. GENERIC — keyed by [`Self::kind`] through the one
    /// owner [`crate::facets::scheme`], so every facet method below is picker-agnostic.
    pub fn facet_scheme(&self) -> Option<&'static crate::facets::FacetScheme> {
        crate::facets::scheme(self.kind)
    }

    /// Whether this picker facets (has a lens strip). Drives the LEFT/RIGHT
    /// lens-cycle gate in `actions` + the "draw a strip" gate in the renderer.
    pub fn is_faceting(&self) -> bool {
        self.facet_scheme().is_some()
    }

    /// The active lens's short sidecar id (`"all"`/`"time"`/…), or `None` for a
    /// non-faceting picker. Generalizes the old theme-only `theme_lens.as_str()`.
    pub fn active_facet_id(&self) -> Option<&'static str> {
        self.facet_scheme()
            .and_then(|sc| sc.strip.get(self.facet_lens))
            .map(|f| f.id)
    }

    /// **WHERE THIS PICKER IS, at the SECONDARY level** (item 220) — the active
    /// lens's label, or `None` at the All home and for a non-faceting picker.
    ///
    /// The picker's PRIMARY level is its [`OverlayKind::title`]; this is the
    /// category inside it. It is the one datum a world's own location cue is an
    /// expression of, which is why it is asked of the scheme rather than
    /// re-read off the strip: a reader that scanned [`Self::lens_strip`] for its
    /// active entry would be a second answer to the same question, free to
    /// disagree with the mark the strip itself draws.
    pub fn location(&self) -> Option<&'static str> {
        self.facet_scheme()
            .and_then(|sc| sc.location(self.facet_lens))
    }

    /// The lens STRIP for rendering + the sidecar — each lens's label with a flag
    /// marking the ACTIVE one (emphasized by VALUE, never amber). In the scheme's
    /// [`crate::facets::FacetScheme::strip`] order (All parked at the far left).
    /// Empty for every NON-faceting kind (so the pipeline knows to draw no strip).
    pub fn lens_strip(&self) -> Vec<(String, bool)> {
        match self.facet_scheme() {
            Some(sc) => sc.strip_labels(self.facet_lens),
            None => Vec::new(),
        }
    }

    /// Switch the faceting lens by `delta` steps along this picker's strip (clamped
    /// at both ends — LEFT at All / RIGHT at the last lens are no-ops), KEEPING the
    /// currently-highlighted item highlighted (it just moves to its section in the
    /// new lens). Regroups the list. A no-op for a non-faceting kind.
    pub fn cycle_lens(&mut self, delta: isize) {
        let Some(sc) = self.facet_scheme() else {
            return;
        };
        let next =
            (self.facet_lens as isize + delta).clamp(0, sc.strip.len() as isize - 1) as usize;
        self.set_facet_lens(next);
    }

    /// Switch DIRECTLY to the lens at strip index `idx` (the pointing counterpart to
    /// [`Self::cycle_lens`] — a click on a strip label), KEEPING the highlighted item.
    /// A no-op when it isn't a faceting picker, `idx` is out of range, or that lens is
    /// already active.
    pub fn set_facet_lens(&mut self, idx: usize) {
        let Some(sc) = self.facet_scheme() else {
            return;
        };
        if idx >= sc.strip.len() || idx == self.facet_lens {
            return;
        }
        let keep = self.selected_corpus_index();
        self.facet_lens = idx;
        self.refilter();
        if let Some(ci) = keep
            && let Some(pos) = self.items.iter().position(|&i| i == ci)
        {
            self.selected = pos;
        }
        self.scroll_to_selected();
    }

    /// Pre-lens a freshly-built faceting overlay onto the lens whose sidecar `id` is
    /// `id`, if this picker's scheme carries one — the door for a "go straight to a
    /// refinement" command (the palette's "Go to heading…" opens Go-to on `headings`;
    /// "Recent projects…" opens Switch project on `recent`). A no-op when the picker
    /// doesn't facet or has no lens by that id.
    pub fn focus_facet_id(&mut self, id: &str) {
        if let Some(sc) = self.facet_scheme()
            && let Some(idx) = sc.strip.iter().position(|f| f.id == id)
        {
            self.set_facet_lens(idx);
        }
    }
}
