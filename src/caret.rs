mod juice;
mod morph;
mod pipeline;
mod preview;
mod spring;

pub use pipeline::*;
pub use preview::*;

pub const STIFFNESS: f32 = 1400.0;
pub const DAMPING: f32 = 55.0;

pub const SMALL_MOVE_DAMPING: f32 = 80.0;

const SMALL_MOVE_ADV: f32 = 1.5;
const LARGE_MOVE_ADV: f32 = 8.0;

pub const POS_EPSILON: f32 = 0.35;
pub const VEL_EPSILON: f32 = 6.0;

const MAX_SUBSTEP: f32 = 1.0 / 240.0;

pub const SETTLE_DIST_SCALE: f32 = 26.0;
pub const SETTLE_VEL_SCALE: f32 = 520.0;

pub const CORNER_RADIUS: f32 = 7.0;

pub const STREAK_RADIUS: f32 = 1.4;

pub const CARET_STREAK_GAP: f32 = 1.5 * crate::render::CHAR_WIDTH;

pub const HELD_GAP_FRAC: f32 = 0.15;

pub const HELD_STREAK_LEN: f32 = 2.2 * crate::render::CHAR_WIDTH;

pub const CARET_ZIP_CHARS: f32 = 4.0;

pub const CARET_ZIP_ROWS: f32 = 1.0;

pub const CARET_POP_MS: f32 = 90.0;
pub const CARET_POP_SCALE: f32 = 0.8;

pub const CARET_TRAIL_MS: f32 = 200.0;
pub const CARET_TRAIL_SWEEP_MS: f32 = 55.0;
pub const CARET_TRAIL_ALPHA: f32 = 0.5;
pub const CARET_TRAIL_MIN_CHARS: f32 = 2.0;

pub const CARET_RECOIL_IMPULSE: f32 = 200.0;

pub const CARET_DELETE_SQUASH: f32 = 0.86;

pub const CARET_GULP_SCALE: f32 = 0.66;
pub const CARET_GULP_MS: f32 = 150.0;

pub const CARET_TYPE_IMPACT_SCALE: f32 = 0.84;
pub const CARET_TYPE_IMPACT_KICK: f32 = 150.0;

pub const CARET_TYPE_IMPACT_DAMP_VEL: f32 = 300.0;

pub const CARET_LINE_LAND_SCALE: f32 = 0.80;
pub const CARET_LINE_LAND_MS: f32 = 130.0;

pub const CARET_COPY_PULSE_SCALE: f32 = 0.94;
pub const CARET_COPY_PULSE_MS: f32 = 180.0;

use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaretMode {
    Block,
    Morph,
    Ibeam,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Affinity {
    #[default]
    Downstream,
    Upstream,
}

impl CaretMode {
    fn as_u8(self) -> u8 {
        match self {
            CaretMode::Block => 0,
            CaretMode::Morph => 1,
            CaretMode::Ibeam => 2,
        }
    }

    pub const ALL: [CaretMode; 3] = [CaretMode::Block, CaretMode::Morph, CaretMode::Ibeam];

    pub fn label(self) -> &'static str {
        match self {
            CaretMode::Block => "Block",
            CaretMode::Morph => "Morph",
            CaretMode::Ibeam => "I-beam",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            CaretMode::Block => "rounded square + trailing underline",
            CaretMode::Morph => "takes the glyph silhouette",
            CaretMode::Ibeam => "an alive insertion bar",
        }
    }

    pub fn from_label(s: &str) -> Option<CaretMode> {
        Self::ALL
            .into_iter()
            .find(|m| m.label().eq_ignore_ascii_case(s))
    }
}

static MODE_OVERRIDE: AtomicU8 = AtomicU8::new(0);

pub fn font_is_mono(family: &str) -> bool {
    crate::render::facepitch::family_is_mono(family)
}

pub fn default_mode() -> CaretMode {
    if font_is_mono(crate::theme::active().font) {
        CaretMode::Block
    } else {
        CaretMode::Morph
    }
}

pub fn mode() -> CaretMode {
    match MODE_OVERRIDE.load(Ordering::Relaxed) {
        1 => CaretMode::Block,
        2 => CaretMode::Morph,
        3 => CaretMode::Ibeam,
        _ => default_mode(),
    }
}

pub fn set_mode(m: CaretMode) {
    MODE_OVERRIDE.store(m.as_u8() + 1, Ordering::Relaxed);
}

pub fn is_auto() -> bool {
    MODE_OVERRIDE.load(Ordering::Relaxed) == 0
}

pub fn clear_override() {
    MODE_OVERRIDE.store(0, Ordering::Relaxed);
}

pub fn morph_anchor_col(col: usize) -> usize {
    col.saturating_sub(1)
}

pub fn morph_line_start(col: usize) -> bool {
    col == 0
}

pub fn toggle_mode() -> CaretMode {
    let next = match mode() {
        CaretMode::Block => CaretMode::Ibeam,
        CaretMode::Ibeam => CaretMode::Block,
        CaretMode::Morph => CaretMode::Block,
    };
    set_mode(next);
    next
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecoilDir {
    Up,
    Down,
    Left,
    Right,
}

impl RecoilDir {
    pub fn impulse(self) -> (f32, f32) {
        match self {
            RecoilDir::Up => (0.0, -CARET_RECOIL_IMPULSE),
            RecoilDir::Down => (0.0, CARET_RECOIL_IMPULSE),
            RecoilDir::Left => (-CARET_RECOIL_IMPULSE, 0.0),
            RecoilDir::Right => (CARET_RECOIL_IMPULSE, 0.0),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sample {
    pub x: f32,
    pub y: f32,
}

pub struct CaretAnim {
    pub pos: Sample,
    pub vel: Sample,
    pub target: Sample,
    prev_pos: Sample,
    animating: bool,
    primed: bool,
    damping: f32,
    glyph_advance: f32,
    line_height: f32,
    streak_suppressed: bool,
    edit_move: bool,
    held: bool,
    holding: bool,
    vertical_move: bool,
    pop_t: f32,
    pop_floor: f32,
    pop_ms: f32,
    trail_present: bool,
    trail_from: Sample,
    trail_to: Sample,
    trail_t: f32,
    trail_sweep_t: f32,
    trail_vertical: bool,
    trail_held: bool,
}

impl CaretAnim {
    pub fn new() -> Self {
        Self {
            pos: Sample { x: 0.0, y: 0.0 },
            vel: Sample { x: 0.0, y: 0.0 },
            target: Sample { x: 0.0, y: 0.0 },
            prev_pos: Sample { x: 0.0, y: 0.0 },
            animating: false,
            primed: false,
            damping: DAMPING,
            glyph_advance: crate::render::CHAR_WIDTH,
            line_height: crate::render::LINE_HEIGHT,
            streak_suppressed: false,
            edit_move: false,
            held: false,
            holding: false,
            vertical_move: false,
            pop_t: 1.0,
            pop_floor: CARET_POP_SCALE,
            pop_ms: CARET_POP_MS,
            trail_present: false,
            trail_from: Sample { x: 0.0, y: 0.0 },
            trail_to: Sample { x: 0.0, y: 0.0 },
            trail_t: 1.0,
            trail_sweep_t: 1.0,
            trail_vertical: false,
            trail_held: false,
        }
    }
}

impl Default for CaretAnim {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
