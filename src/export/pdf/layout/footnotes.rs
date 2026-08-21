//! Footnote-definition layout, including its visible reference marker.

use super::{Block, Engine, Inline, Style};

impl Engine<'_> {
    pub(super) fn footnote_definition(
        &mut self,
        number: usize,
        blocks: &[Block],
        x: f32,
        width: f32,
    ) {
        let indent = 16.0;
        let mut rest = blocks;
        if let Some(Block::Paragraph(inlines)) = rest.first() {
            let mut composed = Vec::with_capacity(inlines.len() + 2);
            composed.push(Inline::FootnoteReference {
                label: String::new(),
                number,
                occurrence: 0,
            });
            composed.push(Inline::Text(" ".to_string()));
            composed.extend(inlines.iter().cloned());
            let mut style = Style::body();
            style.size = 9.35;
            style.leading = 13.0;
            self.rich(&composed, style, x, width, 5.0, false, false);
            rest = &rest[1..];
        } else {
            let mut style = Style::body();
            style.size = 9.35;
            style.leading = 13.0;
            self.rich(
                &[Inline::FootnoteReference {
                    label: String::new(),
                    number,
                    occurrence: 0,
                }],
                style,
                x,
                width,
                3.0,
                false,
                false,
            );
        }
        self.blocks(rest, x + indent, (width - indent).max(40.0));
    }
}
