//! Queries and inverses over one planned overlay row band.

use super::{OverlayRowPlan, PlannedRow};

impl OverlayRowPlan {
    pub(in crate::render) fn rows(&self) -> &[PlannedRow] {
        &self.rows
    }
    pub(in crate::render) fn row_top(&self, display: usize) -> Option<f32> {
        self.rows.get(display).map(|r| r.top)
    }
    pub(in crate::render) fn item_at(&self, display: usize) -> Option<usize> {
        self.rows.get(display).and_then(|r| r.item)
    }
    pub(in crate::render) fn candidate_rows(&self) -> usize {
        self.rows.len()
    }
    pub(in crate::render) fn content_rows(&self) -> usize {
        self.rows.len() + self.empty_rows
    }
    pub(in crate::render) fn first_top(&self) -> f32 {
        self.first_top
    }
    pub(in crate::render) fn lh(&self) -> f32 {
        self.lh
    }
    pub(in crate::render) fn band_bottom(&self) -> f32 {
        self.first_top + self.rows.len() as f32 * self.lh
    }
    pub(in crate::render) fn footer_top(&self) -> f32 {
        self.first_top + self.content_rows() as f32 * self.lh
    }
    pub(in crate::render) fn selected_display(&self) -> Option<usize> {
        self.selected_display
    }

    /// Invert a planned row slot without re-deriving its y arithmetic.
    pub(in crate::render) fn display_at(&self, py: f32) -> Option<usize> {
        (self.lh > 0.0).then(|| {
            self.rows
                .iter()
                .position(|r| py >= r.top && py < r.bottom())
        })?
    }

    /// A travelling band belongs to the planned row nearest its visual centre.
    pub(in crate::render) fn display_nearest(&self, py: f32) -> Option<usize> {
        self.rows
            .iter()
            .min_by(|a, b| {
                let da = (a.top + a.height * 0.5 - py).abs();
                let db = (b.top + b.height * 0.5 - py).abs();
                da.total_cmp(&db)
            })
            .map(|row| row.display)
    }

    pub(in crate::render) fn row_dx(&self, display: usize) -> f32 {
        self.rows.get(display).map_or(0.0, |r| r.dx)
    }
    pub(in crate::render) fn row_dw(&self, display: usize) -> f32 {
        self.rows.get(display).map_or(0.0, |r| r.dw)
    }

    /// The pointer inverse reads the exact horizontal span the planned row draws.
    pub(in crate::render) fn row_at(&self, px: f32, py: f32) -> Option<usize> {
        let row = self.rows.get(self.display_at(py)?)?;
        (px >= self.card_x + row.dx && px <= self.card_x + self.card_w + row.dw)
            .then_some(row.item)
            .flatten()
    }

    #[cfg(test)]
    pub(in crate::render) fn card_x_span(&self) -> (f32, f32) {
        (self.card_x, self.card_x + self.card_w)
    }
}
