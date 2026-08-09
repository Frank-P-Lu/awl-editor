//! src/app/files/export.rs — WHERE an export lands, and how the user is shown
//! it. `Effect::Export` renders the active markdown buffer through the pure
//! [`crate::export`] emitters; everything about the DESTINATION lives here.
//!
//! **One owner for the destination.** [`export_target`] is a pure function of
//! "does this document have a path, what folder is active, what does the buffer
//! call itself, which format, and did the writer choose a folder" — no `App`, no
//! filesystem, no clock. It is the only place that answers the question, so the
//! notice that names the file, the bytes that get written, the folder the
//! destination navigator opens at and the folder a platform save panel opens at
//! all read the same arithmetic instead of four copies of it.
//!
//! **Two live-only doors, ONE gate.** On macOS a successful write asks the
//! Finder to select the file (`mac_chrome::reveal_in_file_viewer`), and the menu
//! bar's Export rows ask `NSSavePanel` where to put it — both only when a real
//! window exists. A headless `App` (`--screenshot-app`, every tier-2 test) has no
//! surface, so it takes the identical write path, reveals nothing, and never
//! opens a modal: the destination and the bytes are the same on both, which is
//! what makes the headless arm a trustworthy oracle for this verb at all. For the
//! panel the gate is not merely hygiene — `runModal` blocks the process main
//! thread, so a surfaceless App reaching it would hang the suite forever.

use crate::app::*;

/// Where one export lands, and how much of that path the notice says out loud.
#[cfg(not(target_arch = "wasm32"))]
pub(in crate::app) struct ExportTarget {
    /// The absolute file the bytes are written to.
    pub path: std::path::PathBuf,
    /// Whether the notice names the WHOLE path rather than the bare filename.
    /// Derived by [`ExportTarget::at`] — never set by a caller.
    pub show_full: bool,
}

#[cfg(not(target_arch = "wasm32"))]
impl ExportTarget {
    /// THE ONE RULE for how much of the destination the notice says out loud:
    /// name the whole path unless the file landed in the folder the DOCUMENT
    /// itself lives in, which is the one folder the writer is already looking at.
    ///
    /// Stated as a relation between two paths rather than as a flag set per arm,
    /// so every way of arriving at a destination — the sibling default, the
    /// active folder a path-less buffer falls back to, a folder chosen in the
    /// navigator, a folder chosen in a platform save panel — reads one rule.
    /// A flag per arm had already reached two arms, and each new door would have
    /// set it by guess: that line is the only thing standing between this verb
    /// and a file the writer cannot find.
    fn at(path: std::path::PathBuf, doc_path: Option<&std::path::Path>) -> Self {
        let show_full = path.parent() != doc_path.and_then(|p| p.parent());
        Self { path, show_full }
    }
}

/// THE ONE answer to "where does an export land".
///
/// `dest_dir` is the root-relative FOLDER the writer chose in the destination
/// navigator ([`crate::overlay::OverlayKind::ExportDest`]). `None` means nobody
/// chose, and the two original defaults stand:
///
/// A document with a path exports as its own SIBLING (`doc.md` → `doc.pdf`):
/// the folder is the one the writer chose when they saved, and the stem is the
/// document's own, so a repeat export overwrites its previous snapshot instead
/// of accumulating copies. A buffer with no path has no folder of its own, so
/// the export lands in the ACTIVE folder under the name the buffer already
/// calls itself (`crate::web_export::export_stem`, which is `display_name()`
/// with the markdown extension taken off — never a second naming rule).
///
/// A CHOSEN folder keeps the document's own stem for a saved document and the
/// buffer's derived one otherwise — the same two naming rules, so exporting one
/// document into two folders reads as one document in two places.
#[cfg(not(target_arch = "wasm32"))]
pub(in crate::app) fn export_target(
    doc_path: Option<&std::path::Path>,
    root: &std::path::Path,
    stem: &str,
    format: crate::export::Format,
    dest_dir: Option<&str>,
) -> ExportTarget {
    let named = |stem: &str| format!("{stem}.{}", format.ext());
    match (dest_dir, doc_path) {
        (Some(rel), _) => {
            let stem = doc_path
                .and_then(|p| p.file_stem())
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| stem.to_string());
            ExportTarget::at(root.join(rel).join(named(&stem)), doc_path)
        }
        (None, Some(p)) => ExportTarget::at(p.with_extension(format.ext()), doc_path),
        (None, None) => ExportTarget::at(root.join(named(stem)), doc_path),
    }
}

impl App {
    /// EXPORT (`Effect::Export`): render the active markdown buffer to `.docx`,
    /// standalone `.html`, or native `.pdf` and land it where [`export_target`]
    /// says — inside `dest_dir` when the destination navigator chose one. Images
    /// embedded in the export are read off the doc's own `assets/` directory
    /// through the filesystem seam (`export::FsImages`). A calm toast names the
    /// target on success; a write failure raises a sticky notice (export never
    /// crashes). On the WEB build there is no real filesystem, so DOCX/HTML bytes
    /// are handed to the browser download shim
    /// (`web_export::trigger_download_bytes`) instead — which is also why nothing
    /// chooses a folder there (`actions::export_picks_destination`); PDF has no
    /// web command or format variant.
    pub(in crate::app) fn export_document(
        &mut self,
        format: crate::export::Format,
        dest_dir: Option<&str>,
    ) {
        let bytes = self.export_bytes(format);

        #[cfg(target_arch = "wasm32")]
        {
            let _ = dest_dir;
            let name = crate::web_export::export_name(self.document.buffer(), format);
            crate::web_export::trigger_download_bytes(&name, format.mime(), &bytes);
            self.set_toast_notice(format!("downloaded {name}"));
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let target = self.export_destination(format, dest_dir);
            self.write_export(&bytes, target);
        }
    }

    /// The rendered bytes for one export — the pure emitters, plus the one
    /// filesystem seam they read embedded images through. Shared by every
    /// destination door, so no door can render differently from another.
    pub(in crate::app) fn export_bytes(&self, format: crate::export::Format) -> Vec<u8> {
        let markdown = self.document.buffer().text();
        let doc_dir = self
            .document
            .buffer()
            .path()
            .and_then(|p| p.parent())
            .map(|p| p.to_path_buf());
        let images = crate::export::FsImages { doc_dir };
        crate::export::to_bytes(&markdown, format, &images)
    }

    /// Resolve this App's export destination through the ONE pure owner.
    #[cfg(not(target_arch = "wasm32"))]
    fn export_destination(
        &self,
        format: crate::export::Format,
        dest_dir: Option<&str>,
    ) -> ExportTarget {
        let stem = crate::web_export::export_stem(self.document.buffer());
        export_target(
            self.document.buffer().path(),
            &self.project_location.root,
            &stem,
            format,
            dest_dir,
        )
    }

    /// Ask the PLATFORM where this export goes — macOS' real `NSSavePanel`,
    /// opened at the folder and under the name [`export_target`] would have
    /// chosen on its own — and write there. Returns whether the panel door was
    /// actually taken, which is the seam the law reads: the AppKit modal itself
    /// cannot be observed from a test process, but "no hermetic tier ever reaches
    /// it" can be.
    ///
    /// ⚠️ **THE SURFACE GATE IS LOAD-BEARING, not defensive.** `runModal` blocks
    /// the process MAIN THREAD until a human closes the panel, so a surfaceless
    /// `App` reaching this would hang `cargo test` and `--screenshot-app` forever
    /// — no output, no timeout, no diagnosis. It is deliberately the SAME gate
    /// [`Self::reveal_export`] already applies rather than a second one: a live
    /// window means a person is watching, and everything else is a capture.
    ///
    /// Non-macOS has no platform save panel; the in-app destination navigator is
    /// the whole answer there, on every door including the drawn menu bar's.
    #[cfg(not(target_arch = "wasm32"))]
    #[cfg_attr(not(any(target_os = "macos", test)), allow(dead_code))]
    pub(in crate::app) fn export_via_platform_panel(
        &mut self,
        format: crate::export::Format,
    ) -> bool {
        if self.frame.gpu().is_none() {
            return false;
        }
        #[cfg(target_os = "macos")]
        {
            let target = self.export_destination(format, None);
            let dir = target
                .path
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| self.project_location.root.clone());
            let name = target
                .path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            if let Some(chosen) = crate::mac_chrome::pick_export_destination(&dir, &name) {
                let bytes = self.export_bytes(format);
                let doc_path = self.document.buffer().path().map(|p| p.to_path_buf());
                self.write_export(&bytes, ExportTarget::at(chosen, doc_path.as_deref()));
            }
            true
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = format;
            false
        }
    }

    /// The native half of [`Self::export_document`]: write the bytes atomically
    /// at an already-resolved destination, then say where they went — in the
    /// notice, and (macOS, live only) by revealing the file in the Finder.
    #[cfg(not(target_arch = "wasm32"))]
    fn write_export(&mut self, bytes: &[u8], target: ExportTarget) {
        if let Some(parent) = target.path.parent() {
            let _ = crate::fs::active().create_dir_all(parent);
        }
        match crate::durable::write(crate::durable::Owner::Export, &target.path, bytes) {
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
