use super::TextPipeline;

pub(super) fn pipelines(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
) -> (
    crate::background::BackgroundPipeline,
    crate::lava::LavaPipeline,
) {
    (
        crate::background::BackgroundPipeline::new(
            device,
            format,
            crate::background::active_desc(),
        ),
        crate::lava::LavaPipeline::new(device, format),
    )
}

impl TextPipeline {
    pub fn lava_render_phase(&self) -> f32 {
        crate::lava::lava_phase_for(
            self.lava_phase,
            crate::motion::reduced(),
            crate::lava::env_phase(),
        )
    }

    pub fn stars_render_phase(&self) -> f32 {
        crate::lava::lava_phase_for(
            self.lava_phase,
            crate::motion::reduced(),
            crate::stars::env_phase(),
        )
    }

    pub fn waves_render_phase(&self) -> f32 {
        crate::lava::lava_phase_for(self.lava_phase, crate::motion::reduced(), None)
    }

    pub fn warp_grid_render_phase(&self) -> f32 {
        crate::warpgrid::phase_for(
            self.warp_grid_phase,
            crate::motion::reduced(),
            crate::warpgrid::env_phase(),
        )
    }

    pub(super) fn background_render_phase(&self) -> f32 {
        let background = self.effective_background();
        if background.is_waves() {
            crate::background::waves_drift_radians(self.waves_render_phase())
        } else if background.is_organic() {
            self.waves_render_phase() * std::f32::consts::TAU / crate::lava::LAVA_LOOP_CYCLES
        } else if background.is_warped_grid() {
            self.warp_grid_render_phase()
        } else {
            0.0
        }
    }

    pub fn advance_lava(&mut self, dt: f32) {
        self.lava_phase = crate::lava::advance_phase(self.lava_phase, dt);
        self.warp_grid_phase = crate::warpgrid::advance_phase(self.warp_grid_phase, dt);
    }

    pub fn freeze_lava(&mut self) {
        self.lava_phase = crate::lava::LAVA_FROZEN_PHASE;
        self.warp_grid_phase = crate::warpgrid::FROZEN_PHASE;
    }
}
