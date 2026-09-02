//! DRAG-AND-DROP CLASSIFICATION (native only — winit's `DroppedFile` is
//! emitted only by the macOS, X11 and Windows backends; the web backend never
//! constructs it, so this module has no wasm32 caller and mirrors
//! [`crate::paste_image`]'s own native-only gate).
//!
//! DECIDED semantics (user's words): a markdown/text file dropped on the
//! window is "the same thing as open file, with the file" — it routes
//! through the EXACT `Open file…` door (`App::load_path`), never a second
//! implementation of opening. An image "should put the image in the file" —
//! it routes through the paste-image pipeline (`crate::paste_image`), which
//! already owns naming/dir resolution/insertion; a drop's own live glue
//! (`App::on_dropped_file`, `app/files/drop.rs`) copies the dropped file's
//! OWN bytes there rather than re-encoding, since the pipeline's PNG encoder
//! is the only image codec compiled in (`Cargo.toml`'s `image` crate carries
//! only the `"png"` feature) and a copy is simpler than a decode this crate
//! cannot always perform anyway.
//!
//! [`classify_drop`] is the ONLY new decision: which of the two existing
//! doors a dropped path routes through. It does NOT re-decide "is this
//! actually text" — `crate::openable::classify` already answers that by
//! CONTENT once the `Open` door is reached, so an extensionless or
//! unfamiliar-extension file still gets a fair (and, if binary, calmly
//! refused) read there. This module decides only IMAGE-vs-EVERYTHING-ELSE,
//! by extension, reusing [`crate::assets::IMAGE_EXTS`] — the SAME roster the
//! asset-cleaner scan already treats as "an image, whatever its bytes" — so
//! the "is this an image" answer has exactly one owner across the two
//! features.
//!
//! Pure and total: no filesystem access, no clock, so classification is
//! unit-testable as `path in, decision out`.

use std::path::Path;

/// Which existing door a dropped path routes through.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropRoute {
    /// Route through `App::load_path` — the exact `Open file…` door.
    Open,
    /// Route through the paste-image pipeline — copy the file's bytes into
    /// `assets/` beside the document and insert a reference.
    Image,
}

/// Classify a dropped path by its LEAF extension (case-insensitive), against
/// [`crate::assets::IMAGE_EXTS`]. An extensionless or unrecognized-extension
/// path is never guessed at by content here — it falls to [`DropRoute::Open`],
/// where `openable::classify` reads the actual bytes and refuses it by
/// CONTENT if it isn't text, so a renamed image with a stripped extension is
/// never misrouted into the document as a broken reference.
pub fn classify_drop(path: &Path) -> DropRoute {
    let is_image = path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| crate::assets::IMAGE_EXTS.contains(&ext.to_ascii_lowercase().as_str()));
    if is_image {
        DropRoute::Image
    } else {
        DropRoute::Open
    }
}

#[cfg(test)]
#[path = "drop_tests.rs"]
mod tests;
