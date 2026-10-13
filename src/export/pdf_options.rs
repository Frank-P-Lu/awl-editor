//! Native PDF choices shared by the export dialog and the in-process engine.

/// Physical paper size for PDF output.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PdfPaper {
    A4,
    Letter,
}

impl PdfPaper {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::A4 => "A4",
            Self::Letter => "Letter",
        }
    }

    /// Page dimensions in PDF points (1/72 inch), in portrait orientation.
    pub const fn size(self) -> (f32, f32) {
        match self {
            Self::A4 => (595.276, 841.890),
            Self::Letter => (612.0, 792.0),
        }
    }
}

/// Visual treatment for a PDF document.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PdfAppearance {
    /// Neutral white paper with the established Bitter/IBM Plex type system.
    Plain,
    /// Snapshot the immutable theme at this roster index when the document is built.
    CurrentTheme(usize),
}

/// Complete set of choices that can affect PDF layout or paint.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PdfOptions {
    pub paper: PdfPaper,
    pub appearance: PdfAppearance,
    /// Show centered page numbers in the bottom margin.
    pub page_numbers: bool,
    /// Keep authored single newlines; blank paragraphs and hard breaks always remain.
    pub preserve_line_breaks: bool,
}

impl Default for PdfOptions {
    fn default() -> Self {
        Self {
            paper: PdfPaper::A4,
            appearance: PdfAppearance::Plain,
            page_numbers: false,
            preserve_line_breaks: true,
        }
    }
}
