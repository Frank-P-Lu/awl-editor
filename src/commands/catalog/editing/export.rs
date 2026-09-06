//! Export entries in the editing command catalog.

use super::{Action, Command};

pub(super) const EXPORT_WORD: Command = Command {
    name: "Export as Word…",
    action: Action::ExportWord,
    native: "",
    emacs: "",
    native_only: false,
    web_only: false,
    description: Some("Export as `.docx`; markdown buffers only, folder chosen on native."),
};

pub(super) const EXPORT_HTML: Command = Command {
    name: "Export as HTML…",
    action: Action::ExportHtml,
    native: "",
    emacs: "",
    native_only: false,
    web_only: false,
    description: Some("Export as `.html`; markdown buffers only, folder chosen on native."),
};

pub(super) const EXPORT_PDF: Command = Command {
    name: "Export as PDF…",
    action: Action::ExportPdf,
    native: "",
    emacs: "",
    native_only: true,
    web_only: false,
    description: Some(
        "Choose a folder, then export as `.pdf`; markdown buffers only, native builds only.",
    ),
};
