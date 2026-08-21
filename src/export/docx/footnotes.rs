//! DOCX bookmark anchors and visible notes for the export model's footnotes.

use super::{Block, Docx, RunProps, text_element};

impl Docx<'_> {
    pub(super) fn footnote_definition(
        &mut self,
        number: usize,
        blocks: &[Block],
        list_depth: usize,
    ) {
        let bookmark = format!("_awl_footnote_{number}");
        let bookmark_id = 10_000usize + number;
        self.body
            .push_str("<w:p><w:pPr><w:pStyle w:val=\"FootnoteText\"/></w:pPr>");
        self.body.push_str(&format!(
            "<w:bookmarkStart w:id=\"{bookmark_id}\" w:name=\"{bookmark}\"/>"
        ));
        let props = RunProps {
            superscript: true,
            ..RunProps::default()
        };
        self.body.push_str("<w:r>");
        self.body.push_str(&props.rpr());
        self.body.push_str(&text_element(&format!("{number} ")));
        self.body.push_str("</w:r>");
        self.body
            .push_str(&format!("<w:bookmarkEnd w:id=\"{bookmark_id}\"/>"));
        let mut rest = blocks;
        if let Some(Block::Paragraph(inlines)) = rest.first() {
            let mut runs = String::new();
            for inline in inlines {
                self.inline(&mut runs, inline, RunProps::default());
            }
            self.body.push_str(&runs);
            rest = &rest[1..];
        }
        self.body.push_str("</w:p>");
        for block in rest {
            self.block(block, list_depth);
        }
    }
}
