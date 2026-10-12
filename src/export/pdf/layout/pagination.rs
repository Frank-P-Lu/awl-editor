//! Pagination decisions derived from the same shaped lines that are painted.

use super::{Block, Engine, FontRole, Inline, Op, Piece, ShapedLine, Style, flatten};

impl Engine<'_> {
    pub(in crate::export::pdf) fn blocks(&mut self, blocks: &[Block], x: f32, width: f32) {
        for (index, block) in blocks.iter().enumerate() {
            self.block(block, blocks.get(index + 1), x, width);
        }
    }

    pub(super) fn block(&mut self, block: &Block, next: Option<&Block>, x: f32, width: f32) {
        match block {
            Block::Heading { level, inlines } => {
                let size = match level {
                    1 => 22.0,
                    2 => 17.6,
                    3 => 14.85,
                    4 => 12.65,
                    5 => 11.0,
                    _ => 9.9,
                };
                let mut style = Style::body();
                style.role = FontRole::SerifBold;
                style.size = size;
                style.leading = (size * 1.28).max(14.0);
                let reserve =
                    self.heading_reserve(inlines, &style, next, Style::body(), width, 5.0);
                self.ensure(reserve);
                self.rich(inlines, style, x, width, 5.0, true, false);
            }
            Block::Paragraph(inlines) => {
                self.rich(inlines, Style::body(), x, width, 8.0, false, false)
            }
            Block::BlockQuote(blocks) => self.blockquote(blocks, x, width, 18.0),
            Block::CodeBlock { lang: _, code } => {
                let mut style = Style::body();
                style.role = FontRole::Mono;
                style.size = 9.35;
                style.leading = 13.0;
                let pieces = vec![Piece::Text {
                    text: code.clone(),
                    style,
                }];
                self.text_pieces(&pieces, x + 9.0, width - 18.0, 8.0, false, true);
            }
            Block::List(list) => self.list(list, x, width),
            Block::Rule => {
                self.ensure(12.0);
                self.pages[self.page].ops.push(Op::Line {
                    x1: x,
                    y1: self.y + 4.0,
                    x2: x + width,
                    y2: self.y + 4.0,
                    width: 0.7,
                    gray: 0.72,
                });
                self.y += 12.0;
            }
            Block::Table(table) => self.table(table, x, width),
            Block::FootnoteDefinition { number, blocks, .. } => {
                self.footnote_definition(*number, blocks, x, width)
            }
        }
    }

    pub(super) fn heading_reserve(
        &mut self,
        inlines: &[Inline],
        style: &Style,
        next: Option<&Block>,
        follower_style: Style,
        width: f32,
        after: f32,
    ) -> f32 {
        let heading = flatten(inlines, style.clone(), &self.fonts.roster);
        let heading_height = self
            .shape(&heading, width)
            .iter()
            .map(|line| line.height)
            .sum::<f32>();
        let follower_height = match next {
            Some(Block::Paragraph(inlines)) => {
                let pieces = flatten(inlines, follower_style, &self.fonts.roster);
                let text = pieces
                    .into_iter()
                    .take_while(|piece| matches!(piece, Piece::Text { .. }))
                    .collect::<Vec<_>>();
                let lines = self.shape(&text, width);
                let minimum = if lines.len() <= 3 { lines.len() } else { 2 };
                lines
                    .iter()
                    .take(minimum)
                    .map(|line| line.height)
                    .sum::<f32>()
                    .max(16.0)
            }
            _ => 16.0,
        };
        heading_height + after + follower_height
    }

    pub(in crate::export::pdf) fn shaped_segment(
        &mut self,
        pieces: &[Piece],
        x: f32,
        width: f32,
        after: f32,
        keep: bool,
        panel: bool,
    ) {
        let lines = self.shape(pieces, width);
        let total = lines.iter().map(|line| line.height).sum::<f32>();
        if keep {
            self.ensure(total + 16.0);
        } else if self.needs_fresh_page(&lines, total) {
            self.new_page();
        }
        for (index, line) in lines.iter().enumerate() {
            // When only two shaped lines remain, move the pair as one unit so
            // the final line cannot sit alone on a continuation page.
            if !keep && lines.len() - index == 2 {
                let tail_height = lines[index..].iter().map(|line| line.height).sum::<f32>();
                let printable = self.geometry.height - 2.0 * self.geometry.margin_y;
                if tail_height <= printable
                    && tail_height > self.remaining()
                    && self.y > self.geometry.margin_y
                {
                    self.new_page();
                }
            }
            self.ensure(line.height);
            if panel {
                self.pages[self.page].ops.push(Op::Rect {
                    x: x - 5.0,
                    y: self.y,
                    w: width + 10.0,
                    h: line.height,
                    gray: 0.955,
                });
            }
            self.place_line(line, x, self.y);
            self.y += line.height;
        }
        self.y += after;
    }

    fn needs_fresh_page(&self, lines: &[ShapedLine], total: f32) -> bool {
        if lines.len() < 2 || total <= self.remaining() || self.y <= self.geometry.margin_y {
            return false;
        }
        let mut used = 0.0;
        let fitting = lines
            .iter()
            .take_while(|line| {
                used += line.height;
                used <= self.remaining()
            })
            .count();
        let printable = self.geometry.height - 2.0 * self.geometry.margin_y;
        fitting < 2 || (lines.len() <= 3 && total <= printable)
    }
}

// Append to src/export/pdf/layout/pagination.rs. Keeping the test here permits
// positioning the real Engine at an exact shaped boundary without a costly,
// brittle search over padding paragraphs or duplicating layout formulas.
#[cfg(test)]
mod boundary_tests {
    use super::*;
    use crate::export::pdf::fonts::{FontRoster, Fonts};
    use crate::export::pdf::layout::{Appearance, PageGeometry};
    use crate::export::{PdfAppearance, PdfPaper};

    struct NoImages;
    impl crate::export::ImageSource for NoImages {
        fn resolve(&self, _: &str) -> Option<crate::export::ExportImage> {
            None
        }
    }

    #[test]
    fn heading_follows_a_three_line_paragraph_onto_its_next_page() {
        let _serial = crate::testlock::serial();
        let appearances = std::iter::once(PdfAppearance::Plain)
            .chain((0..crate::theme::THEMES.len()).map(PdfAppearance::CurrentTheme));
        for paper in [PdfPaper::A4, PdfPaper::Letter] {
            for appearance in appearances.clone() {
                let (palette, roster) = match appearance {
                    PdfAppearance::Plain => (Appearance::plain(), FontRoster::plain()),
                    PdfAppearance::CurrentTheme(index) => {
                        let theme = &crate::theme::THEMES[index];
                        (
                            Appearance::theme(theme),
                            FontRoster::for_theme(theme).unwrap(),
                        )
                    }
                };
                let geometry = PageGeometry::for_paper(paper);
                let mut engine = Engine {
                    fonts: Fonts::new(roster),
                    images_source: &NoImages,
                    pages: vec![super::super::style::page(geometry, palette)],
                    page: 0,
                    y: geometry.margin_y,
                    images: Vec::new(),
                    unicode: std::array::from_fn(|_| Default::default()),
                    geometry,
                    appearance: palette,
                };
                let heading = vec![Inline::Text("KEEP".into())];
                let follower = vec![
                    Inline::Text("First line".into()),
                    Inline::HardBreak,
                    Inline::Text("Second line".into()),
                    Inline::HardBreak,
                    Inline::Text("Third line".into()),
                ];
                let pieces = flatten(&follower, Style::body(), &engine.fonts.roster);
                let lines = engine.shape(&pieces, geometry.measure);
                assert_eq!(lines.len(), 3, "{paper:?} {appearance:?}");
                let follower_height: f32 = lines.iter().map(|line| line.height).sum();

                // Measure the heading through the production block path, then
                // clear its operations so only the actual boundary is examined.
                let block = Block::Heading {
                    level: 2,
                    inlines: heading,
                };
                engine.block(&block, None, geometry.margin_x, geometry.measure);
                let heading_height_with_gap = engine.y - geometry.margin_y;
                engine.pages[0].ops.clear();
                engine.pages[0].links.clear();
                let available = heading_height_with_gap
                    + lines[0].height
                    + lines[1].height
                    + lines[2].height * 0.5;
                assert!(available < heading_height_with_gap + follower_height);
                engine.y = geometry.bottom() - available;
                engine.blocks(
                    &[block, Block::Paragraph(follower)],
                    geometry.margin_x,
                    geometry.measure,
                );
                assert_eq!(engine.pages.len(), 2, "{paper:?} {appearance:?}");
                assert!(
                    !engine.pages[0]
                        .ops
                        .iter()
                        .any(|op| matches!(op, Op::Glyph(_))),
                    "{paper:?} {appearance:?}: heading was stranded on the previous page"
                );
                let baselines = engine.pages[1]
                    .ops
                    .iter()
                    .filter_map(|op| match op {
                        Op::Glyph(glyph) => Some(glyph.y.to_bits()),
                        _ => None,
                    })
                    .collect::<std::collections::BTreeSet<_>>();
                assert_eq!(baselines.len(), 4, "heading and all three follower lines");
            }
        }
    }
}
