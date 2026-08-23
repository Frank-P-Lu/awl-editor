use super::*;

impl TextPipeline {
    /// Prepare the one static material pass shared by the summoned pane and
    /// placard. Both pipelines receive identical display-resolved dials, and
    /// the shader keys phase to absolute canvas y, so their separate rects
    /// still read as one raster. This pass owns no clock; Reduce Motion has
    /// nothing to compress, and awl has no reduced-transparency preference.
    pub(super) fn prepare_overlay_material(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        geom: &OverlayGeom,
        placard: Option<(f32, f32, f32, f32)>,
    ) {
        let theme::SummonedMaterial::Scanlines {
            pitch_px,
            line_px,
            strength,
        } = crate::render::overrides::effective_summoned_material()
        else {
            self.panel_material
                .prepare(device, queue, width, height, &[]);
            self.placard_material
                .prepare(device, queue, width, height, &[]);
            return;
        };

        let density = self.dpi.max(1.0);
        let pitch = pitch_px * density;
        let line = line_px * density;
        let ink = theme::base_100().rgba_bytes();
        let card = [geom.card_x, geom.card_y, geom.card_w, geom.card_h];
        let (chamfer, _) = self.card_shape_texture(&[card]);

        self.panel_material.set_chamfer(chamfer.top, chamfer.bottom);
        self.panel_material
            .set_scanlines(strength, pitch, line, ink);
        self.panel_material
            .prepare(device, queue, width, height, &[card]);

        // The placard resolves its OWN corner shape through the same owner,
        // against its OWN rect — never a per-layer opinion independent of the
        // card it bleeds against (the bug this round names).
        let placard = placard.map(|(x, y, w, h)| [x, y, w, h]);
        let (placard_chamfer, _) = match &placard {
            Some(rect) => self.card_shape_texture(std::slice::from_ref(rect)),
            None => self.card_shape_texture(&[]),
        };
        self.placard_material
            .set_chamfer(placard_chamfer.top, placard_chamfer.bottom);
        self.placard_material
            .set_scanlines(strength, pitch, line, ink);
        self.placard_material
            .prepare(device, queue, width, height, placard.as_slice());
    }
}
