//! Section dividers retain the editor's syntax-specific theme vocabulary.
use super::{Engine, FontRole, Op, Piece, Style};

impl Engine<'_> {
    pub(super) fn ornament(
        &mut self,
        kind: crate::markdown::BreakKind,
        x: f32,
        width: f32,
    ) -> bool {
        let Some((marks, scale)) = self.appearance.ornaments else {
            return false;
        };
        let mut style = Style::body();
        style.role = FontRole::Ornament;
        style.size *= scale;
        style.leading *= scale;
        let lines = self.shape(
            &[Piece::Text {
                text: marks.pick(kind).into(),
                style,
            }],
            width,
        );
        let height: f32 = lines.iter().map(|line| line.height).sum();
        self.ensure(height + 8.0);
        let start = self.pages[self.page].ops.len();
        for line in &lines {
            self.place_line(line, x + (width - line.width) * 0.5, self.y);
            self.y += line.height;
        }
        for op in &mut self.pages[self.page].ops[start..] {
            if let Op::Glyph(glyph) = op {
                glyph.color = self.appearance.palette.map(|p| p.muted);
            }
        }
        self.y += 8.0;
        true
    }
}
