//! Caret motion, preview, and report helpers.

use super::*;

impl TextPipeline {
    pub fn caret_pixel_rect(&self) -> (f32, f32, f32, f32) {
        // Affinity-aware so the OS composition cell sits at the caret's REAL screen
        // position when it is parked at a shared wrap boundary (matches `caret_cell_top`,
        // which is affinity-aware too); `Downstream` for any ordinary caret.
        let (gx, adv) =
            self.col_x_and_advance_aff(self.cursor_line, self.cursor_col, self.caret_affinity);
        let x = self.text_left() + gx;
        let y = self.caret_cell_top(self.cursor_col);
        (x, y, adv.max(self.metrics.caret_w), self.metrics.caret_h)
    }

    pub fn set_caret_target(&mut self, is_edit: bool, held: bool) {
        self.caret.set_glyph_advance(self.metrics.char_width);
        self.caret.set_line_height(self.metrics.line_height);
        self.caret.set_edit_move(is_edit);
        self.caret.set_held(held);
        let (x, y) = self.caret_target_xy();
        if is_edit {
            self.caret.jump_to(x, y);
        } else {
            self.caret.nav_to(x, y);
        }
    }

    pub fn step_caret(&mut self, dt: f32) -> bool {
        if crate::motion::reduced() {
            self.caret.snap_to_target();
            return false;
        }
        self.caret.step(dt);
        let popping = self.caret.step_pop(dt);
        let trailing = self.caret.step_trail(dt);
        self.caret.is_animating() | popping | trailing
    }

    pub fn caret_snapshot(&self) -> ((f32, f32), (f32, f32), f32, bool) {
        (
            (self.caret.pos.x, self.caret.pos.y),
            (self.caret.target.x, self.caret.target.y),
            self.caret.settle_factor(),
            self.caret.is_animating(),
        )
    }

    pub fn caret_pop_report(&mut self) -> (f32, f32, f32) {
        let s = self.caret.pop_scale();
        let (_cx, _cy, w, h, _c, _ax, _ay) = self.caret_geometry();
        (s, w * s, h * s)
    }

    pub fn caret_trail_report(&mut self) -> (bool, f32, (f32, f32), (f32, f32)) {
        let (cx, cy, w, _h, _corner, ax, ay) = self.caret_geometry();
        let half = w * 0.5;
        let tail = (cx - ax * half, cy - ay * half);
        let head = (cx + ax * half, cy + ay * half);
        (self.caret.is_holding(), w, tail, head)
    }

    #[allow(clippy::type_complexity)]
    pub(crate) fn caret_trail_geometry(&self) -> Option<(f32, f32, f32, f32, f32, f32, f32, f32)> {
        if !self.caret.trail_active() {
            return None;
        }
        let m = &self.metrics;
        let center_x_drop = if self.caret_is_bar_form() {
            self.ibeam_bar_dims().0 * 0.5
        } else {
            self.caret_block_w() * 0.5
        };
        let (center, half_along, half_across, axis) = self.caret.trail_geometry(
            m.caret_streak_h,
            m.caret_streak_gap,
            m.caret_trail_drop,
            center_x_drop,
        );
        let w = half_along * 2.0;
        if w <= 0.0 {
            return None;
        }
        let corner = m.px(STREAK_RADIUS);
        Some((
            center.x,
            center.y,
            w,
            half_across * 2.0,
            corner,
            axis.0,
            axis.1,
            self.caret.trail_alpha(),
        ))
    }

    #[allow(clippy::type_complexity)]
    pub fn caret_cosmetic_report(
        &self,
    ) -> (bool, f32, bool, bool, f32, f32, (f32, f32), (f32, f32)) {
        let held = self.caret.is_trail_held();
        let sweep = self.caret.trail_sweep_p();
        match self.caret_trail_geometry() {
            Some((cx, cy, w, _h, _c, ax, ay, alpha)) => {
                let half = w * 0.5;
                let tail = (cx - ax * half, cy - ay * half);
                let head = (cx + ax * half, cy + ay * half);
                (
                    true,
                    w,
                    self.caret.is_trail_vertical(),
                    held,
                    alpha,
                    sweep,
                    tail,
                    head,
                )
            }
            None => (
                false,
                0.0,
                self.caret.is_trail_vertical(),
                held,
                0.0,
                sweep,
                (0.0, 0.0),
                (0.0, 0.0),
            ),
        }
    }

    pub fn caret_type_impact(&mut self) {
        self.caret.type_impact();
    }

    pub fn caret_delete_squash(&mut self) {
        self.caret.delete_squash();
    }

    pub fn caret_gulp(&mut self) {
        self.caret.gulp();
    }

    pub fn caret_line_land(&mut self) {
        self.caret.line_land();
    }

    pub fn caret_recoil(&mut self, dir: crate::caret::RecoilDir) {
        self.caret.recoil(dir);
    }

    pub fn settle_caret(&mut self) {
        self.set_caret_target(false, false);
        self.caret.snap_to_target();
    }

    pub fn inject_motion_demo(&mut self) {
        let demo_line = 2usize.min(self.line_count().saturating_sub(1));
        let line_chars = self.line_glyph_xs(demo_line).len().saturating_sub(1);
        self.cursor_line = demo_line;
        self.cursor_col = 24usize.min(line_chars);
        self.set_caret_target(false, false);
        let (tx, ty) = self.caret_target_xy();
        let target = Sample { x: tx, y: ty };

        let back: f32 = 9.0 * self.metrics.char_width; // ~9 cells left of target
        const PHASE: f32 = 0.55; // fraction of the gap still remaining to the left
        let pos = Sample {
            x: tx - back * PHASE,
            y: ty,
        };
        let vel = Sample { x: 1900.0, y: 0.0 };
        self.caret.inject_motion(target, pos, vel);
    }

    pub fn inject_motion_demo_vertical(&mut self) {
        let demo_line = 6usize.min(self.line_count().saturating_sub(1));
        let line_chars = self.line_glyph_xs(demo_line).len().saturating_sub(1);
        self.cursor_line = demo_line;
        self.cursor_col = 12usize.min(line_chars);
        self.set_caret_target(false, false);
        let (tx, ty) = self.caret_target_xy();
        let target = Sample { x: tx, y: ty };

        let back: f32 = 5.0 * self.metrics.line_height; // ~5 lines above target
        const PHASE: f32 = 0.55; // fraction of the gap still remaining above
        let pos = Sample {
            x: tx,
            y: ty - back * PHASE,
        };
        let vel = Sample { x: 0.0, y: 1900.0 };
        self.caret.inject_motion(target, pos, vel);
    }

    pub fn inject_motion_demo_diagonal(&mut self) {
        let demo_line = 6usize.min(self.line_count().saturating_sub(1));
        let line_chars = self.line_glyph_xs(demo_line).len().saturating_sub(1);
        self.cursor_line = demo_line;
        self.cursor_col = 22usize.min(line_chars);
        self.set_caret_target(false, false);
        let (tx, ty) = self.caret_target_xy();
        let target = Sample { x: tx, y: ty };

        let back_x: f32 = 9.0 * self.metrics.char_width;
        let back_y: f32 = 4.0 * self.metrics.line_height;
        const PHASE: f32 = 0.55;
        let pos = Sample {
            x: tx - back_x * PHASE,
            y: ty - back_y * PHASE,
        };
        let vel = Sample {
            x: 1600.0,
            y: 1600.0,
        };
        self.caret.inject_motion(target, pos, vel);
    }

    pub fn settle_caret_preview(&mut self) {
        if self.caret_preview.is_some() {
            self.caret_demo.settle();
        }
    }
}
