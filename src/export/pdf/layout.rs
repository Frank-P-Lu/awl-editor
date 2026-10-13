//! Fixed-measure top-down layout; the writer alone converts to PDF page space.

use std::collections::BTreeMap;

use glyphon::{Attrs, Buffer, Family, Metrics, Shaping, Weight, Wrap};

use super::super::model::{Block, Document, ImageSource, Inline};
use super::fonts::{FontRole, FontRoster, Fonts, ROLES};
use super::images::PdfImage;
use super::inline::{Piece, Style, flatten};
use crate::export::pdf_options::PdfPaper;

mod footnotes;
mod ornaments;
mod page_numbers;
mod pagination;
mod style;
pub(super) use style::{Appearance, PageGeometry, PdfColor};
#[cfg(test)]
pub(super) use style::{MARGIN_X, MARGIN_Y, MEASURE, PAGE_H, PAGE_W, PROSE_CHARS};

pub(super) struct Layout {
    pub pages: Vec<Page>,
    pub images: Vec<PdfImage>,
    pub unicode: [BTreeMap<u16, String>; ROLES.len()],
    pub fonts: FontRoster,
    pub geometry: PageGeometry,
    pub paper: PdfPaper,
    pub appearance: Appearance,
}

#[derive(Default)]
pub(super) struct Page {
    pub ops: Vec<Op>,
    pub links: Vec<LinkRect>,
}

pub(super) enum Op {
    Glyph(GlyphOp),
    Rect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        gray: f32,
    },
    Line {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        width: f32,
        gray: f32,
    },
    Image {
        index: usize,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
    },
    ColorRect {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        color: PdfColor,
    },
    ColorLine {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        width: f32,
        color: PdfColor,
    },
}

pub(super) struct GlyphOp {
    pub role: FontRole,
    pub glyph_id: u16,
    pub size: f32,
    pub x: f32,
    pub y: f32,
    pub italic: bool,
    pub actual: String,
    pub embolden: bool,
    pub color: Option<PdfColor>,
}

pub(super) struct LinkRect {
    pub url: String,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Clone)]
pub(super) struct ShapedGlyph {
    pub role: FontRole,
    pub id: u16,
    pub size: f32,
    pub x: f32,
    pub advance: f32,
    pub style: Style,
    pub actual: String,
}
pub(super) struct ShapedLine {
    pub glyphs: Vec<ShapedGlyph>,
    pub width: f32,
    pub baseline: f32,
    pub height: f32,
}

pub(super) fn build(
    doc: &Document,
    images: &dyn ImageSource,
    options: crate::export::PdfOptions,
    appearance: Appearance,
    roster: FontRoster,
    source_spacing: bool,
) -> Layout {
    let paper = options.paper;
    let geometry = PageGeometry::for_paper(paper);
    let mut engine = Engine {
        source_spacing,
        fonts: Fonts::new(roster),
        images_source: images,
        pages: vec![style::page(geometry, appearance)],
        page: 0,
        y: geometry.margin_y,
        images: Vec::new(),
        unicode: std::array::from_fn(|_| BTreeMap::new()),
        geometry,
        appearance,
    };
    engine.blocks(&doc.blocks, geometry.margin_x, geometry.measure);
    if options.page_numbers {
        engine.page_numbers();
    }
    Layout {
        pages: engine.pages,
        images: engine.images,
        unicode: engine.unicode,
        fonts: engine.fonts.roster,
        geometry,
        paper,
        appearance,
    }
}

pub(super) struct Engine<'a> {
    pub source_spacing: bool,
    pub fonts: Fonts,
    pub images_source: &'a dyn ImageSource,
    pub pages: Vec<Page>,
    pub page: usize,
    pub y: f32,
    pub images: Vec<PdfImage>,
    pub unicode: [BTreeMap<u16, String>; ROLES.len()],
    pub geometry: PageGeometry,
    pub appearance: Appearance,
}

impl Engine<'_> {
    fn blocks_italic(&mut self, blocks: &[Block], x: f32, width: f32) {
        for (index, block) in blocks.iter().enumerate() {
            match block {
                Block::Paragraph(inlines) => {
                    let mut s = Style::body();
                    s.italic = true;
                    self.rich(
                        inlines,
                        s,
                        x,
                        width,
                        if self.source_spacing { 0.0 } else { 6.0 },
                        false,
                        false,
                    );
                }
                Block::Heading { level: _, inlines } => {
                    let mut s = Style::body();
                    s.role = FontRole::SerifBold;
                    s.italic = true;
                    let mut follower = Style::body();
                    follower.italic = true;
                    let (next, gap) = pagination::following(&blocks[index + 1..]);
                    let reserve =
                        self.heading_reserve(inlines, &s, next, follower, width, 6.0 + gap);
                    self.ensure(reserve);
                    self.rich(inlines, s, x, width, 6.0, true, false);
                }
                Block::BlockQuote(b) => self.blockquote(b, x, width, 14.0),
                Block::CodeBlock { lang: _, code } => {
                    let mut s = Style::body();
                    s.role = FontRole::Mono;
                    s.size = 9.35;
                    s.leading = 13.0;
                    self.text_pieces(
                        &[Piece::Text {
                            text: code.clone(),
                            style: s,
                        }],
                        x,
                        width,
                        6.0,
                        false,
                        true,
                    );
                }
                Block::List(l) => self.list(l, x, width),
                Block::Rule(_) | Block::BlankLines(_) => self.block(block, None, x, width, 0.0),
                Block::Table(t) => self.table(t, x, width),
                Block::FootnoteDefinition { number, blocks, .. } => {
                    self.footnote_definition(*number, blocks, x, width)
                }
            }
        }
    }

    /// Paint a quote's vertical affordance once per occupied page segment. The
    /// quote body can page independently; recording the endpoints around that
    /// layout keeps the rule in the printable box without coupling pagination
    /// to a particular child block. Nested quotes use their existing tighter
    /// indent while passing through this same segment owner.
    fn blockquote(&mut self, blocks: &[Block], x: f32, width: f32, indent: f32) {
        let start_page = self.page;
        let start_y = self.y;
        self.y += 3.0;
        self.blocks_italic(blocks, x + indent, (width - indent).max(40.0));
        let end_page = self.page;
        let end_y = (self.y - 5.0).max(self.geometry.margin_y);
        for page in start_page..=end_page {
            let y1 = if page == start_page {
                start_y
            } else {
                self.geometry.margin_y
            };
            let y2 = if page == end_page {
                end_y
            } else {
                self.geometry.bottom()
            };
            if y2 > y1 {
                self.pages[page].ops.push(Op::Line {
                    x1: x + 4.0,
                    y1,
                    x2: x + 4.0,
                    y2,
                    width: 1.2,
                    gray: 0.65,
                });
            }
        }
    }

    // Rich block layout keeps the markdown-derived properties explicit at its one call boundary.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn rich(
        &mut self,
        inlines: &[Inline],
        style: Style,
        x: f32,
        width: f32,
        after: f32,
        keep: bool,
        panel: bool,
    ) {
        let pieces = flatten(inlines, style, &self.fonts.roster);
        self.text_pieces(&pieces, x, width, after, keep, panel);
    }

    fn text_pieces(
        &mut self,
        pieces: &[Piece],
        x: f32,
        width: f32,
        after: f32,
        keep: bool,
        panel: bool,
    ) {
        let mut text = Vec::new();
        for piece in pieces {
            match piece {
                Piece::Text { .. } => text.push(piece.clone()),
                Piece::Image {
                    src,
                    alt,
                    width_hint,
                } => {
                    if !text.is_empty() {
                        self.shaped_segment(&text, x, width, 3.0, keep, panel);
                        text.clear();
                    }
                    self.image(src, alt, *width_hint, x, width);
                }
            }
        }
        if !text.is_empty() {
            self.shaped_segment(&text, x, width, 0.0, keep, panel);
        }
        self.y += after;
    }

    pub(super) fn shape(&mut self, pieces: &[Piece], width: f32) -> Vec<ShapedLine> {
        let styles: Vec<Style> = pieces
            .iter()
            .filter_map(|p| match p {
                Piece::Text { style, .. } => Some(style.clone()),
                Piece::Image { .. } => None,
            })
            .collect();
        if styles.is_empty() {
            return Vec::new();
        }
        let spans: Vec<(&str, Attrs<'_>)> = pieces
            .iter()
            .filter_map(|piece| match piece {
                Piece::Text { text, .. } => Some(text.as_str()),
                Piece::Image { .. } => None,
            })
            .enumerate()
            .map(|(i, text)| {
                let style = &styles[i];
                let a = self.fonts.roster.asset(style.role);
                (
                    text,
                    Attrs::new()
                        .family(Family::Name(a.family))
                        .weight(Weight(a.weight))
                        .metadata(i)
                        .metrics(Metrics::new(style.size, style.leading)),
                )
            })
            .collect();
        let base = &styles[0];
        let base_asset = self.fonts.roster.asset(base.role);
        let default = Attrs::new()
            .family(Family::Name(base_asset.family))
            .weight(Weight(base_asset.weight))
            .metrics(Metrics::new(base.size, base.leading));
        let mut buffer = Buffer::new(
            &mut self.fonts.system,
            Metrics::new(base.size, base.leading),
        );
        buffer.set_size(&mut self.fonts.system, Some(width), None);
        buffer.set_wrap(&mut self.fonts.system, Wrap::WordOrGlyph);
        buffer.set_rich_text(
            &mut self.fonts.system,
            spans,
            &default,
            Shaping::Advanced,
            None,
        );
        buffer.shape_until_scroll(&mut self.fonts.system, false);
        let mut previous_run_end = 0;
        buffer
            .layout_runs()
            .map(|run| {
                // cosmic-text can omit a literal whitespace glyph at a rich-span
                // or wrapped-line boundary while preserving its advance. Carry
                // any source bytes between adjacent shaped clusters into the NEXT
                // cluster's ActualText, so positioned text still extracts as the original
                // prose ("a [link]" must not become "areal link").
                // Cluster byte offsets are paragraph-relative, so the cursor must
                // span layout runs; resetting it on a wrapped line either loses the
                // wrap-space or repeats every earlier source byte.
                let mut previous_end = previous_run_end;
                let glyphs = run
                    .glyphs
                    .iter()
                    .map(|g| {
                        let role = self
                            .fonts
                            .role_for_id(g.font_id)
                            .expect("PDF shaping escaped closed font roster");
                        let style = styles.get(g.metadata).cloned().unwrap_or_else(Style::body);
                        let skipped = run.text.get(previous_end..g.start).unwrap_or("");
                        let cluster = style.actual.clone().unwrap_or_else(|| {
                            run.text.get(g.start..g.end).unwrap_or("").to_string()
                        });
                        let actual = format!("{skipped}{cluster}");
                        previous_end = previous_end.max(g.end);
                        ShapedGlyph {
                            role,
                            id: g.glyph_id,
                            size: g.font_size,
                            x: g.x + g.font_size * g.x_offset,
                            advance: g.w,
                            style,
                            actual,
                        }
                    })
                    .collect();
                previous_run_end = previous_end;
                ShapedLine {
                    glyphs,
                    width: run.line_w,
                    baseline: run.line_y - run.line_top,
                    height: run.line_height,
                }
            })
            .collect()
    }

    pub(super) fn remaining(&self) -> f32 {
        self.geometry.bottom() - self.y
    }
    pub(super) fn ensure(&mut self, h: f32) {
        if h > self.remaining() && self.y > self.geometry.margin_y {
            self.new_page();
        }
    }
    pub(super) fn new_page(&mut self) {
        self.pages.push(style::page(self.geometry, self.appearance));
        self.page += 1;
        self.y = self.geometry.margin_y;
    }
}
