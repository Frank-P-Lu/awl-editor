//! src/app/files/export.rs — WHERE an export lands, and how the user is shown
//! it. `Effect::Export` renders the active markdown buffer through the pure
//! [`crate::export`] emitters; everything about the DESTINATION lives here.
//!
//! **One owner for the destination.** [`export_target`] is a pure function of
//! "does this document have a path, what folder is active, what does the buffer
//! call itself, which format" — no `App`, no filesystem, no clock. It is the
//! only place that answers the question, so the notice that names the file, the
//! bytes that get written, and any future platform save panel offering a
//! DEFAULT folder + name all read the same arithmetic instead of three copies
//! of it.
//!
//! **The reveal is live-only.** On macOS a successful write asks the Finder to
//! select the file (`mac_chrome::reveal_in_file_viewer`) — but only when a real
//! window exists. A headless `App` (`--screenshot-app`, every tier-2 test) has
//! no surface, so it takes the identical write path and reveals nothing: the
//! destination and the bytes are the same on both, which is what makes the
//! headless arm a trustworthy oracle for this verb at all.

use crate::app::*;

/// Where one export lands, and how much of that path the notice says out loud.
#[cfg(not(target_arch = "wasm32"))]
pub(in crate::app) struct ExportTarget {
    /// The absolute file the bytes are written to.
    pub path: std::path::PathBuf,
    /// Whether the notice names the WHOLE path rather than the bare filename.
    /// A sibling export beside a saved document needs no directory — the user
    /// is looking at the document it sat next to. An export from a buffer with
    /// no path of its own landed somewhere the user never named, so the notice
    /// says where; that line is the only thing standing between this verb and a
    /// file the writer cannot find.
    pub show_full: bool,
}

/// THE ONE answer to "where does an export land".
///
/// A document with a path exports as its own SIBLING (`doc.md` → `doc.pdf`):
/// the folder is the one the writer chose when they saved, and the stem is the
/// document's own, so a repeat export overwrites its previous snapshot instead
/// of accumulating copies. A buffer with no path has no folder of its own, so
/// the export lands in the ACTIVE folder under the name the buffer already
/// calls itself (`crate::web_export::export_stem`, which is `display_name()`
/// with the markdown extension taken off — never a second naming rule).
#[cfg(not(target_arch = "wasm32"))]
pub(in crate::app) fn export_target(
    doc_path: Option<&std::path::Path>,
    root: &std::path::Path,
    stem: &str,
    format: crate::export::Format,
) -> ExportTarget {
    match doc_path {
        Some(p) => ExportTarget {
            path: p.with_extension(format.ext()),
            show_full: false,
        },
        None => ExportTarget {
            path: root.join(format!("{stem}.{}", format.ext())),
            show_full: true,
        },
    }
}

impl App {
    /// EXPORT (`Effect::Export`): render the active markdown buffer to `.docx`,
    /// standalone `.html`, or native `.pdf` and land it where [`export_target`]
    /// says. Images embedded in the export are read off the doc's own `assets/`
    /// directory through the filesystem seam (`export::FsImages`). A calm toast
    /// names the target on success; a write failure raises a sticky notice
    /// (export never crashes). On the WEB build there is no real filesystem, so
    /// DOCX/HTML bytes are handed to the browser download shim
    /// (`web_export::trigger_download_bytes`) instead; PDF has no web command or
    /// format variant.
    pub(in crate::app) fn export_document(&mut self, format: crate::export::Format) {
        let markdown = self.document.buffer().text();
        let doc_dir = self
            .document
            .buffer()
            .path()
            .and_then(|p| p.parent())
            .map(|p| p.to_path_buf());
        let images = crate::export::FsImages { doc_dir };
        let bytes = crate::export::to_bytes(&markdown, format, &images);

        #[cfg(target_arch = "wasm32")]
        {
            let name = crate::web_export::export_name(self.document.buffer(), format);
            crate::web_export::trigger_download_bytes(&name, format.mime(), &bytes);
            self.set_toast_notice(format!("downloaded {name}"));
        }
        #[cfg(not(target_arch = "wasm32"))]
        self.write_export(format, &bytes);
    }

    /// The native half of [`Self::export_document`]: resolve the destination,
    /// write the bytes atomically, then say where they went — in the notice, and
    /// (macOS, live only) by revealing the file in the Finder.
    #[cfg(not(target_arch = "wasm32"))]
    fn write_export(&mut self, format: crate::export::Format, bytes: &[u8]) {
        let stem = crate::web_export::export_stem(self.document.buffer());
        let target = export_target(
            self.document.buffer().path(),
            &self.project_location.root,
            &stem,
            format,
        );
        if let Some(parent) = target.path.parent() {
            let _ = crate::fs::active().create_dir_all(parent);
        }
        match crate::fs::write_atomic(&target.path, bytes) {
            Ok(()) => {
                let shown = if target.show_full {
                    target.path.display().to_string()
                } else {
                    target
                        .path
                        .file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_default()
                };
                self.set_toast_notice(format!("exported {shown}"));
                let _revealed = self.reveal_export(&target.path);
            }
            Err(e) => self.set_sticky_notice(format!("export failed: {e}")),
        }
    }

    /// Point the platform's own file viewer at a just-written export. Returns
    /// whether the reveal actually fired, which is the seam a test reads: the
    /// AppKit call itself cannot be observed from a test process, but "the
    /// headless arm never reaches it" can be, and that is the property the
    /// hermetic tiers depend on.
    ///
    /// Gated on a real surface, not on `cfg`: a live window means a person is
    /// watching and the Finder coming forward is the answer they asked for,
    /// while a GPU-less `App` is a capture or a test and must stay hermetic —
    /// the same "no surface, no live-only side effect" rule `set_toast_notice`
    /// applies to its own expiry deadline. Non-macOS has no file-viewer door, so
    /// the toast's path is the whole answer there.
    #[cfg(not(target_arch = "wasm32"))]
    fn reveal_export(&self, path: &std::path::Path) -> bool {
        if self.frame.gpu().is_none() {
            return false;
        }
        #[cfg(target_os = "macos")]
        {
            crate::mac_chrome::reveal_in_file_viewer(path);
            true
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = path;
            false
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests;
