use crate::app::*;

impl App {
    pub(in crate::app) fn normalize_and_repush_scroll(
        &mut self,
        view: &mut ViewState,
        previous: crate::render::ScrollPos,
        height: f32,
    ) {
        let pipeline = &self.gpu.as_ref().unwrap().pipeline;
        self.active.extra.scroll = pipeline.scroll_by_px(self.active.extra.scroll, 0.0, height);
        if self.active.extra.scroll != previous {
            view.scroll = self.active.extra.scroll;
            self.gpu.as_mut().unwrap().pipeline.set_view(view);
        }
    }
}
