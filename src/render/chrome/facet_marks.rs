use super::overlay_rows::{CHIP_UNDERLINE_CORNER, FACET_CHIP_RADIUS};
use super::*;

impl TextPipeline {
    pub(super) fn park_overlay_facets(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
    ) {
        self.overlay_tab_backing
            .prepare(device, queue, width, height, &[]);
        self.overlay_facet_ghost
            .prepare(device, queue, width, height, &[]);
        self.overlay_facet_material
            .prepare(device, queue, width, height, &[]);
    }

    pub(super) fn overlay_prepare_facet_marks(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        geom: &OverlayGeom,
    ) {
        let underline: Vec<[f32; 4]> = if geom.theme {
            self.overlay_theme_underline.iter().copied().collect()
        } else {
            Vec::new()
        };
        self.overlay_tab_backing
            .prepare(device, queue, width, height, &[]);
        if geom.workspace && !geom.theme {
            self.prepare_rail_mark(device, queue, width, height, geom);
            return;
        }
        let facet_style = crate::render::effective_facet_style();
        let ui = self.metrics.ui();
        let chip_radius = ui.px(FACET_CHIP_RADIUS);
        let bar_stroke = ui.px(crate::render::BAR_OUTLINE_STROKE);
        let underline_corner = ui.px(CHIP_UNDERLINE_CORNER);
        let mut ghosts = Vec::new();
        let band = super::overlay_selected_band_srgb();
        let chrome = crate::render::overlay_chrome_theme();
        match facet_style {
            theme::FacetStyle::Text => {}
            theme::FacetStyle::Band => {
                self.overlay_lens_underline.set_color(band.rgba_bytes());
                self.overlay_lens_underline.set_corner(chip_radius);
                self.overlay_lens_underline.set_stroke(0.0);
            }
            theme::FacetStyle::DockedTab => {
                self.overlay_lens_underline.set_color(
                    theme::pane_surface_for(chrome, crate::render::effective_card_elevation())
                        .rgba_bytes(),
                );
                self.overlay_lens_underline.set_corner(0.0);
                self.overlay_lens_underline.set_stroke(0.0);
                self.overlay_facet_ghost
                    .set_color(theme::surface_selected_for(chrome).rgba_bytes());
                self.overlay_facet_ghost.set_corner(0.0);
                self.overlay_facet_ghost.set_stroke(bar_stroke);
                ghosts = self.overlay_theme_facet_ghosts.clone();
                if geom.theme {
                    let plan = self.overlay_row_plan(geom);
                    if let Some(dock) = self.docked_facet_band(geom, &plan) {
                        let mut backing = vec![(
                            [geom.card_x, dock.top, geom.card_w, dock.height],
                            chrome.base_100.rgba_bytes(),
                        )];
                        backing.extend(
                            ghosts
                                .iter()
                                .copied()
                                .map(|rect| (rect, chrome.base_200.rgba_bytes())),
                        );
                        self.overlay_tab_backing.set_corner(0.0);
                        self.overlay_tab_backing
                            .prepare_multicolor(device, queue, width, height, &backing);
                    }
                }
            }
            theme::FacetStyle::Chips(v) => {
                use theme::ChipVariant as V;
                let content = chrome.base_content;
                let muted = chrome.muted;
                let (a_fill, a_corner, a_stroke) = match v {
                    V::Hairline => (band.rgba_bytes(), chip_radius, 0.0),
                    V::FilledActive => (content.rgba_bytes(), chip_radius, 0.0),
                    V::Underline => (content.rgba_bytes(), underline_corner, 0.0),
                    V::Bracket => (content.rgba_bytes(), 0.0, 0.0),
                };
                self.overlay_lens_underline.set_color(a_fill);
                self.overlay_lens_underline.set_corner(a_corner);
                self.overlay_lens_underline.set_stroke(a_stroke);
                let (g_color, g_corner, g_stroke) = match v {
                    V::Hairline => (muted.rgba_bytes(), chip_radius, bar_stroke),
                    V::Bracket => (content.rgba_bytes(), 0.0, 0.0),
                    V::FilledActive | V::Underline => (muted.rgba_bytes(), chip_radius, bar_stroke),
                };
                self.overlay_facet_ghost.set_color(g_color);
                self.overlay_facet_ghost.set_corner(g_corner);
                self.overlay_facet_ghost.set_stroke(g_stroke);
                if geom.theme {
                    ghosts = self.overlay_theme_facet_ghosts.clone();
                }
            }
        }
        self.overlay_lens_underline
            .prepare(device, queue, width, height, &underline);
        if !matches!(
            facet_style,
            theme::FacetStyle::Chips(_) | theme::FacetStyle::DockedTab
        ) {
            self.overlay_facet_ghost.set_corner(chip_radius);
            self.overlay_facet_ghost.set_stroke(bar_stroke);
        }
        self.overlay_facet_ghost
            .prepare(device, queue, width, height, &ghosts);
    }
}
