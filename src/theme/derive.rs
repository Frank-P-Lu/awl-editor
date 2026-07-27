use std::sync::atomic::{AtomicUsize, Ordering};

use super::color::Srgb;
use super::model::{Background, Elevation, ImageReveal, Lens, Theme};
use super::worlds::{DEFAULT_THEME, THEMES};

static ACTIVE: AtomicUsize = AtomicUsize::new(DEFAULT_THEME);

pub fn active() -> Theme {
    THEMES[ACTIVE.load(Ordering::Relaxed) % THEMES.len()]
}

pub fn active_index() -> usize {
    ACTIVE.load(Ordering::Relaxed) % THEMES.len()
}

pub fn set_active(index: usize) -> Theme {
    // Tests must serialize process-global theme writes.
    #[cfg(test)]
    assert!(
        crate::testlock::currently_held(),
        "theme::set_active wrote the process-global world without holding \
         crate::testlock::serial()"
    );
    let i = index % THEMES.len();
    ACTIVE.store(i, Ordering::Relaxed);
    THEMES[i]
}

pub fn cycle(step: isize) -> Theme {
    let n = THEMES.len() as isize;
    let cur = active_index() as isize;
    let next = (((cur + step) % n) + n) % n;
    set_active(next as usize)
}

pub fn set_active_by_name(name: &str) -> Option<Theme> {
    let idx = THEMES
        .iter()
        .position(|t| t.name.eq_ignore_ascii_case(name))?;
    Some(set_active(idx))
}

#[must_use = "a WorldPin restores the active world when it drops; binding it to `_` drops it immediately"]
pub struct WorldPin {
    prev: usize,
}

impl WorldPin {
    pub fn snapshot() -> Self {
        #[cfg(test)]
        assert!(
            crate::testlock::currently_held(),
            "WorldPin must be created inside crate::testlock::serial()"
        );
        WorldPin {
            prev: ACTIVE.load(Ordering::Relaxed),
        }
    }

    pub fn world(name: &str) -> Option<Self> {
        let pin = WorldPin::snapshot();
        set_active_by_name(name)?;
        Some(pin)
    }

    pub fn restores_to(&self) -> usize {
        self.prev % THEMES.len()
    }
}

impl Drop for WorldPin {
    fn drop(&mut self) {
        #[cfg(test)]
        assert!(
            crate::testlock::currently_held(),
            "WorldPin restored the process-global world without holding \
             crate::testlock::serial()"
        );
        ACTIVE.store(self.prev, Ordering::Relaxed);
    }
}

pub fn base_100() -> Srgb {
    active().base_100
}
pub fn base_200() -> Srgb {
    active().base_200
}
pub fn base_300() -> Srgb {
    active().base_300
}
pub fn base_content() -> Srgb {
    active().base_content
}
pub fn muted() -> Srgb {
    active().muted
}
pub fn faint() -> Srgb {
    active().faint
}
pub fn primary() -> Srgb {
    active().primary
}
pub fn primary_content() -> Srgb {
    active().primary_content
}
pub fn error() -> Srgb {
    active().error
}
pub fn selection() -> Srgb {
    active().selection
}

pub fn fold_afford_chevron_ink() -> Srgb {
    let t = active();
    t.muted.lerp(
        t.base_content,
        t.render_caps.fold_afford.chevron_lift.clamp(0.0, 1.0),
    )
}

pub fn fold_afford_tail_ink() -> Srgb {
    let t = active();
    t.faint.lerp(
        t.base_content,
        t.render_caps.fold_afford.tail_lift.clamp(0.0, 1.0),
    )
}

pub fn card_texture_ink() -> Srgb {
    let t = active();
    t.muted.lerp(t.base_300, 0.25)
}

const PLACARD_DARK_LIFT_FAINT: f32 = 0.75;
const PLACARD_DARK_LIFT_GHOST: f32 = 0.45;

pub fn placard_ink(ink: super::model::PlacardInk) -> Srgb {
    let t = active();
    match ink {
        super::model::PlacardInk::Faint if t.dark => faint().lerp(muted(), PLACARD_DARK_LIFT_FAINT),
        super::model::PlacardInk::Ghost if t.dark => faint().lerp(muted(), PLACARD_DARK_LIFT_GHOST),
        super::model::PlacardInk::Faint => faint(),
        super::model::PlacardInk::Ghost => faint().lerp(base_300(), 0.5),
        super::model::PlacardInk::Stipple => base_content(),
        super::model::PlacardInk::Muted => muted(),
        super::model::PlacardInk::Bold => muted().lerp(base_content(), PLACARD_BOLD_LIFT),
    }
}

const PLACARD_BOLD_LIFT: f32 = 0.5;

fn rel_lum(c: Srgb) -> f32 {
    fn lin(u: u8) -> f32 {
        let s = u as f32 / 255.0;
        if s <= 0.03928 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    }
    0.2126 * lin(c.r) + 0.7152 * lin(c.g) + 0.0722 * lin(c.b)
}

pub fn heatmap_colors() -> [Srgb; crate::streaks::LEVELS] {
    let empty = base_200();
    let ink = base_content();
    if active().is_one_bit() {
        let mut out = [empty; crate::streaks::LEVELS];
        for c in out.iter_mut().skip(1) {
            *c = ink;
        }
        return out;
    }
    let stops = [0.0_f32, 0.34, 0.56, 0.78, 1.0];
    let mut out = [empty; crate::streaks::LEVELS];
    for (i, t) in stops.iter().enumerate() {
        out[i] = empty.lerp(ink, *t);
    }
    out
}

const PLACARD_STIPPLE_DENSITY_FLOOR: f32 = 0.12;
const PLACARD_STIPPLE_DENSITY_CEILING: f32 = 0.55;

pub fn placard_stipple_density() -> f32 {
    let ground = rel_lum(base_100());
    let ink = rel_lum(base_content());
    let target = rel_lum(placard_ink(super::model::PlacardInk::Faint));
    let span = ink - ground;
    let density = if span.abs() < 1e-6 {
        0.0
    } else {
        (target - ground) / span
    };
    density.clamp(
        PLACARD_STIPPLE_DENSITY_FLOOR,
        PLACARD_STIPPLE_DENSITY_CEILING,
    )
}

pub fn page_frame_ink() -> Srgb {
    base_content()
}

pub(super) const SELECTED_BAND_STEPS: i32 = 2;

pub(super) const OVERLAY_SELROW_EXTRA_STEPS: i32 = 1;

fn surface_step_band(extra_steps: i32) -> Srgb {
    let a = active();
    if a.base_200 == a.base_300 {
        return a.base_content;
    }
    let steps = SELECTED_BAND_STEPS + extra_steps;
    let step = |lo: u8, hi: u8| -> u8 {
        let d = hi as i32 - lo as i32;
        (hi as i32 + d * steps).clamp(0, 255) as u8
    };
    Srgb::rgb(
        step(a.base_200.r, a.base_300.r),
        step(a.base_200.g, a.base_300.g),
        step(a.base_200.b, a.base_300.b),
    )
}

pub fn overlay_band_overlap() -> Srgb {
    surface_step_band(OVERLAY_SELROW_EXTRA_STEPS + 1)
}

pub fn overlay_selected_band() -> Srgb {
    surface_step_band(OVERLAY_SELROW_EXTRA_STEPS)
}

fn contrast_ratio(a: Srgb, b: Srgb) -> f32 {
    let (la, lb) = (rel_lum(a), rel_lum(b));
    let (hi, lo) = if la >= lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

pub fn pane_surface(elevation: Elevation) -> Srgb {
    match elevation {
        Elevation::Flat | Elevation::Bordered => base_300(),
        Elevation::Recessed => base_200(),
    }
}

pub(super) const SELECTED_ROW_INK_CONTRAST_FLOOR: f32 = 3.0;

pub fn selected_row_ink(band: Srgb) -> Srgb {
    let content = base_content();
    if contrast_ratio(band, content) >= SELECTED_ROW_INK_CONTRAST_FLOOR {
        return content;
    }
    let ground = base_100();
    if contrast_ratio(band, ground) > contrast_ratio(band, content) {
        ground
    } else {
        content
    }
}

pub fn selected_row_secondary_ink(band: Srgb) -> Srgb {
    let dim = muted();
    if contrast_ratio(band, dim) >= SELECTED_ROW_INK_CONTRAST_FLOOR {
        return dim;
    }
    let ground = base_100();
    let content = base_content();
    if contrast_ratio(band, ground) > contrast_ratio(band, content) {
        ground
    } else {
        content
    }
}

pub fn overlay_bar_unselected() -> Srgb {
    base_200()
}

pub fn overlay_bars_scrim() -> Srgb {
    base_100()
}

pub fn surface_selected() -> Srgb {
    surface_step_band(0)
}

const OVERLAY_SCRIM_ALPHA: u8 = 0x80;

pub fn overlay_scrim() -> Srgb {
    let b = active().base_100;
    Srgb::rgba(b.r, b.g, b.b, OVERLAY_SCRIM_ALPHA)
}

const IMAGE_REVEAL_SCRIM_ALPHA: u8 = 0xB8;

pub fn image_reveal_scrim() -> Srgb {
    let b = active().base_100;
    if active().render_caps.image_reveal == ImageReveal::Opaque {
        return Srgb::rgba(b.r, b.g, b.b, 0xFF);
    }
    Srgb::rgba(b.r, b.g, b.b, IMAGE_REVEAL_SCRIM_ALPHA)
}
pub fn background() -> Background {
    active().background
}

pub fn tag_for(name: &str, lens: Lens) -> Option<&'static str> {
    THEMES
        .iter()
        .find(|t| t.name == name)
        .and_then(|t| t.tags.section(lens))
}
