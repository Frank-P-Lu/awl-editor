use super::*;

impl SelectionPipeline {
    /// Configure a static scanline-only material. A negative `halftone` value
    /// selects the shader's transparent raster branch without widening the
    /// uniform layout used by every inert consumer. Phase is absolute canvas y.
    pub fn set_scanlines(&mut self, strength: f32, pitch_px: f32, line_px: f32, ink: [u8; 4]) {
        let strength = strength.clamp(0.0, 1.0);
        self.halftone = if strength > 0.0 { -strength } else { 0.0 };
        self.halftone_angle = line_px.max(0.5);
        self.halftone_cell = pitch_px.max(self.halftone_angle + 0.5);
        self.dot_color = srgba_u8_to_linear(ink);
    }

    #[cfg(test)]
    pub fn scanlines(&self) -> Option<(f32, f32, f32)> {
        (self.halftone < 0.0).then_some((-self.halftone, self.halftone_cell, self.halftone_angle))
    }
}
