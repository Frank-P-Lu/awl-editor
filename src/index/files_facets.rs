//! Facet scheme for the dedicated Files and Recent views.

use crate::facets::{Facet, FacetItem, FacetScheme};

const FILES_FACET_STRIP: [Facet; 2] = [
    Facet {
        label: "Files",
        id: "files",
        sections: &[],
    },
    Facet {
        label: "Recent",
        id: "recent",
        sections: &["Recent"],
    },
];

/// `Recent` contains files actually used recently plus the two terminal actions.
fn files_bucket(item: FacetItem, lens_idx: usize) -> Option<&'static str> {
    match lens_idx {
        1 if matches!(item.accept, "Change folder…") => Some("Recent"),
        1 if item.accept.starts_with("New document — ") => Some("Recent"),
        1 => item.recent.then_some("Recent"),
        _ => None,
    }
}

pub static FILES_FACETS: FacetScheme = FacetScheme {
    strip: &FILES_FACET_STRIP,
    bucket: files_bucket,
};
