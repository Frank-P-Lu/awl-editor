//! Footer numbers share the stored glyph stream used by PDF and preview.
use super::{Engine, Op, PdfColor, Piece, Style};

impl Engine<'_> {
    pub(super) fn page_numbers(&mut self) {
        let color = self
            .appearance
            .palette
            .map_or(PdfColor([90, 90, 90]), |p| p.footer);
        let mut style = Style::body();
        style.size = 9.0;
        style.leading = 12.0;
        for page in 0..self.pages.len() {
            self.page = page;
            let pieces = [Piece::Text {
                text: (page + 1).to_string(),
                style: style.clone(),
            }];
            let lines = self.shape(&pieces, self.geometry.measure);
            let start = self.pages[page].ops.len();
            for line in &lines {
                let x = (self.geometry.width - line.width) * 0.5;
                let top = self.geometry.height - self.geometry.margin_y * 0.5 - line.height * 0.5;
                self.place_line(line, x, top);
            }
            for op in &mut self.pages[page].ops[start..] {
                if let Op::Glyph(glyph) = op {
                    glyph.color = Some(color);
                }
            }
        }
    }
}
