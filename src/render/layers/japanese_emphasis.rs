//! Prepare the Japanese emphasis decoration using cached shaped advances.

use super::*;

impl TextPipeline {
    pub(crate) fn prepare_japanese_emphasis_layer(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) {
        let dots = self.japanese_emphasis_dot_rects();
        self.japanese_emphasis_dot_pipeline
            .set_corner(self.metrics.px(rects::JA_EMPHASIS_DOT_SIZE) * 0.5);
        self.japanese_emphasis_dot_pipeline
            .prepare(device, queue, width, height, &dots);
    }
}
