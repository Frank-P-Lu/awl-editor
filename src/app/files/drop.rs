//! LIVE glue for winit's `DroppedFile` (native only — `crate::drop` and
//! `crate::paste_image`, both consumed below, are themselves native-only
//! modules; the web backend never constructs the event, see `crate::drop`'s
//! module doc). [`App::on_dropped_file`] is the ONE entry `app/lifecycle.rs`
//! calls per dropped path — winit emits one `DroppedFile` event per file, in
//! drop order, so calling this once per event (rather than batching) is what
//! gives multiple dropped files their decided ordering: text files open into
//! the working set in drop order (each call is its own `load_path`), images
//! insert sequentially (each call's continuation lands at wherever the caret
//! ended up after the previous one).

use crate::app::*;
use std::path::Path;

impl App {
    /// Classify a dropped path (`crate::drop::classify_drop`) and route it
    /// through the exact existing door that class owns — `Open` through
    /// [`Self::load_path`] (same door as a picker Enter / the C-x b toggle /
    /// the daemon's `open` handoff), `Image` through
    /// [`Self::insert_dropped_image`] (the paste-image pipeline). The
    /// classify-and-route step is the only new decision; neither door is
    /// reimplemented here.
    pub(in crate::app) fn on_dropped_file(&mut self, exit: &dyn schedule::Exit, path: PathBuf) {
        match crate::drop::classify_drop(&path) {
            crate::drop::DropRoute::Open => {
                self.load_path(path);
            }
            crate::drop::DropRoute::Image => self.insert_dropped_image(exit, &path),
        }
    }

    /// PASTE-IMAGE VIA DROP: mirrors [`Self::paste_image_reference`]
    /// (`app/apply.rs`) exactly, except the bytes come from the DROPPED
    /// file's own path rather than the OS clipboard, so they are COPIED
    /// verbatim into `assets/` (never decoded/re-encoded — the image crate's
    /// only compiled codec is PNG, and a raw copy needs none) through the
    /// SAME low-level owner (`paste_image::persist_bytes`, generalized from
    /// the clipboard path's `persist_png`) with the dropped file's own
    /// extension preserved. Same NO-PATH BUFFER rule as the clipboard door
    /// (`ensure_note_named_before_paste` first), same one-undoable-edit
    /// insertion (`ResolvedPaste::ImageReference` through the shared core).
    ///
    /// A drop with NO active document at all (the zero-document start
    /// surface) has nothing to insert into and is a no-op, mirroring
    /// `reject_without_document`'s invariant for every other edit — the
    /// clipboard door never needs this guard because a keyboard paste is
    /// only reachable with a document focused.
    fn insert_dropped_image(&mut self, exit: &dyn schedule::Exit, path: &Path) {
        if !self.document.has_active() {
            return;
        }
        let Ok(bytes) = crate::fs::active().read(path) else {
            return;
        };
        if self.document.buffer().path().is_none() {
            self.ensure_note_named_before_paste();
        }
        let data_root = crate::fs::data_root();
        let doc_path = self.document.buffer().path().map(Path::to_path_buf);
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(str::to_ascii_lowercase)
            .filter(|e| !e.is_empty())
            .unwrap_or_else(|| "png".to_string());
        let Some(reference) =
            crate::paste_image::persist_bytes(doc_path.as_deref(), &data_root, &bytes, &ext)
        else {
            return;
        };
        let continuation = actions::ResolvedPaste::ImageReference(reference).into_action();
        self.apply(continuation, false, exit, crate::stats::Door::Chord);
    }
}
