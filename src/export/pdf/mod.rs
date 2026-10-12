//! Native, in-process PDF export: the neutral export tree is shaped with the
//! same cosmic-text engine as awl, then painted as positioned TrueType glyphs
//! into deterministic A4 pages. No clock, host font, network, compression, or
//! platform API enters the byte stream.

mod flow;
mod fonts;
mod images;
mod inline;
mod layout;
mod manifest;
mod preview;
mod writer;

/// Fixed PDF font-coverage probe exposed only to the crate's performance harness.
pub(super) fn glyph_probe() -> usize {
    fonts::glyph_probe()
}

use super::model::{Document, ImageSource};
use super::pdf_options::{PdfAppearance, PdfOptions};

/// A fully laid-out PDF artifact. Byte emission and page preview both consume
/// this stored layout, so changing pages in the dialog never reparses or
/// reflows the document.
pub struct PdfDocument {
    layout: layout::Layout,
    metadata: Vec<u8>,
}

impl PdfDocument {
    pub fn build(
        markdown: &str,
        images: &dyn ImageSource,
        options: PdfOptions,
    ) -> Result<Self, String> {
        let doc = super::model::parse(markdown);
        Self::from_document(&doc, images, options)
    }

    fn from_document(
        doc: &Document,
        images: &dyn ImageSource,
        options: PdfOptions,
    ) -> Result<Self, String> {
        let (appearance, fonts) = match options.appearance {
            PdfAppearance::Plain => (layout::Appearance::plain(), fonts::FontRoster::plain()),
            PdfAppearance::CurrentTheme(index) => {
                let theme = crate::theme::THEMES.get(index).ok_or_else(|| {
                    format!(
                        "PDF theme index {index} is outside the {}-world roster",
                        crate::theme::THEMES.len()
                    )
                })?;
                (
                    layout::Appearance::theme(theme),
                    fonts::FontRoster::for_theme(theme)?,
                )
            }
        };
        let layout = layout::build(doc, images, options, appearance, fonts);
        let metadata = manifest::build(doc, &layout);
        Ok(Self { layout, metadata })
    }

    pub fn page_count(&self) -> usize {
        self.layout.pages.len()
    }

    /// Page dimensions in PDF points, in portrait orientation.
    pub fn page_size(&self) -> (f32, f32) {
        (self.layout.geometry.width, self.layout.geometry.height)
    }

    pub fn bytes(&self) -> Result<Vec<u8>, String> {
        writer::emit(&self.layout, &self.metadata)
    }

    /// Rasterize one stored page at `scale` pixels per PDF point.
    pub fn preview_page(&self, page: usize, scale: f32) -> Result<image::RgbaImage, String> {
        preview::render(&self.layout, page, scale)
    }
}

pub(super) fn emit(doc: &Document, images: &dyn ImageSource) -> Result<Vec<u8>, String> {
    PdfDocument::from_document(doc, images, PdfOptions::default())?.bytes()
}

#[cfg(test)]
mod tests;
