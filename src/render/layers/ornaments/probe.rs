use super::*;

pub(crate) struct RuleRunShapeProbe {
    pub text: String,
    pub layout_runs: usize,
    pub glyphs: usize,
    pub width: f32,
    pub faces: Vec<(String, u16)>,
}

impl TextPipeline {
    pub(crate) fn rule_run_shape_probe(&mut self) -> Vec<RuleRunShapeProbe> {
        let rules = RuleOrnaments::shape(
            self,
            self.metrics,
            theme::muted().to_glyphon(),
            self.text_wrap_width().max(1.0),
        );
        rules
            .glyphs
            .iter()
            .map(|(text, buffer)| {
                let runs: Vec<_> = buffer.layout_runs().collect();
                let glyph_count = runs.iter().map(|run| run.glyphs.len()).sum();
                let width = runs.iter().map(|run| run.line_w).fold(0.0_f32, f32::max);
                let faces = runs
                    .iter()
                    .flat_map(|run| run.glyphs.iter())
                    .map(|glyph| {
                        let face = self
                            .font_system
                            .db()
                            .face(glyph.font_id)
                            .expect("shaped ornament face remains registered");
                        (face.families[0].0.clone(), face.weight.0)
                    })
                    .collect();
                RuleRunShapeProbe {
                    text: (*text).to_string(),
                    layout_runs: runs.len(),
                    glyphs: glyph_count,
                    width,
                    faces,
                }
            })
            .collect()
    }
}
