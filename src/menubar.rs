//! Awl-rendered web/Linux menu bar. It shares the native menu roster and action
//! dispatch, so it adds no menu-only behavior. Layout consumes shaped title extents
//! for both drawing and hit testing. It defaults off on macOS, preserving normal
//! native captures unless explicitly enabled.
//!
//! **THE PLATFORM DEFAULT IS AN AXIS, AND IT HAS A KNOB.** `MENU_BAR_ON` is the one
//! platform-forked sticky default in the tree, so a law or fixture about the drawn
//! bar sweeps NOTHING on a macOS host while being live on every Linux one — the
//! asymmetry that once fired a global-leak audit on sixty CI tests and zero local
//! ones, and that took this repo a gating CI RED (a picker drawing zero candidate
//! rows on Linux, because the bar's height comes off every card's budget). A
//! DEV-ONLY `AWL_MENU_BAR_FORCE=on|off` override — [`menu_bar_force`], the
//! `AWL_CONVENTION_FORCE` precedent exactly — forces the DEFAULT the flag starts
//! from, so the other branch is one env var away instead of a source edit away.
//! No config key, no public CLI flag, a total no-op unless set; `scripts/
//! native-gate.sh` runs BOTH arms every gate, which is what makes the axis swept
//! by the gate rather than by whoever remembers.

use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::render::Logical;
use crate::toggle::Toggle;

/// The drawn bar's default per platform — off on macOS (a real system bar
/// already exists), on elsewhere. Split out of the `cfg!` below so BOTH arms
/// stay readable from ANY build: the generated reference documents one
/// `menu_bar` default for every platform, and a bare `cfg!` would make that
/// document differ between the machine that writes it and the host that checks it.
pub(crate) const MENU_BAR_DEFAULT_MACOS: bool = false;
pub(crate) const MENU_BAR_DEFAULT_OTHER: bool = true;

/// The platform fork as a PURE function of "is this macOS" — the ONE owner of
/// which const each platform reads, and the reason a `cfg!` may not appear in a
/// second place (`Config::menu_bar_on` asserted one `cfg!` against an identical
/// one and could not fail on any host). Taking the platform as an ARGUMENT rather
/// than asking `cfg!` inside is what makes both arms gradable from a single host:
/// a law on any machine can ask for the macOS answer and the non-macOS one and
/// require them to differ, which is the claim that dies if either const is
/// flipped to match the other.
pub(crate) fn platform_default(is_macos: bool) -> bool {
    if is_macos {
        MENU_BAR_DEFAULT_MACOS
    } else {
        MENU_BAR_DEFAULT_OTHER
    }
}

/// THE DEFAULT THIS RUNNING BUILD STARTS FROM: [`platform_default`] for the host,
/// with the dev-only [`menu_bar_force`] override applied. Every reader of the
/// bar's default — the flag's own initialiser below, and `Config::menu_bar_on`'s
/// absent-key fallback — routes through here, so the forcing knob moves them
/// together and neither can carry a second copy of the platform fork.
pub(crate) fn menu_bar_default() -> bool {
    match menu_bar_force() {
        Some(forced) => forced,
        None => platform_default(cfg!(target_os = "macos")),
    }
}

/// PURE classifier for the knob's value: `on`/`off` force that default, and
/// EVERYTHING else — an unset variable, an empty one, a typo, a capitalised
/// `ON` — leaves the platform fork alone. Split out of the env read for the same
/// reason `convention::classify_ua` is: a classifier that takes its input as an
/// argument is swept over every input shape by a unit test, while a fn that
/// reads a memoized process global can only ever be asked about the one value
/// this process launched with.
#[cfg(any(test, not(target_arch = "wasm32")))]
fn classify_force(value: Option<&str>) -> Option<bool> {
    match value {
        Some("on") => Some(true),
        Some("off") => Some(false),
        _ => None,
    }
}

/// The `AWL_MENU_BAR_FORCE=on|off` dev knob, read ONCE and memoized (the
/// `AWL_CONVENTION_FORCE` / `AWL_CJK_FORCE` precedent: `menu_bar_on()` is read
/// every frame, so a per-call `std::env::var` would put an env-var thread-safety
/// hazard on a hot path). Never read on wasm: env vars are a process concept.
#[cfg(not(target_arch = "wasm32"))]
fn menu_bar_force() -> Option<bool> {
    static ONCE: OnceLock<Option<bool>> = OnceLock::new();
    *ONCE.get_or_init(|| classify_force(std::env::var("AWL_MENU_BAR_FORCE").ok().as_deref()))
}

#[cfg(target_arch = "wasm32")]
fn menu_bar_force() -> Option<bool> {
    None
}

/// Whether the rendered menu bar is drawn. It defaults on where there is no native
/// bar and off on macOS; config, command, and capture flags can override it.
///
/// LAZY, not a `const` initialiser, for exactly one reason: [`menu_bar_default`]
/// reads an environment variable and `Toggle::new` is a `const fn`. The forcing
/// knob therefore moves the DEFAULT and nothing else — `set_menu_bar_on` and
/// `toggle` keep working normally under a forcing, which is what a fixture that
/// flips the bar and restores the AMBIENT value depends on.
static MENU_BAR_ON: OnceLock<Toggle> = OnceLock::new();

fn menu_bar_flag() -> &'static Toggle {
    MENU_BAR_ON.get_or_init(|| Toggle::new(menu_bar_default()))
}

const NONE: usize = usize::MAX;

/// Which top-level menu's dropdown is currently OPEN — an index into
/// [`crate::menu::roster`], or [`NONE`]. Transient interaction state (set by a title
/// click, cleared by an item click / a click away / hiding the bar), owned here as a
/// process-global exactly like [`crate::hud`]'s held flag so both the renderer and
/// the capture sidecar read ONE source and a `--menu-open` capture can drive it.
static OPEN_MENU: AtomicUsize = AtomicUsize::new(NONE);

/// True when the menu bar is enabled (read by the renderer each frame + the capture
/// sidecar's `menubar` block, so the two can never disagree).
pub fn menu_bar_on() -> bool {
    menu_bar_flag().on()
}

/// Set the menu bar on/off explicitly — the config sticky-pref launch-apply
/// (`Config::apply_sticky_globals`), the settings-menu toggle, and the `--menu-bar`
/// capture flag. Turning it OFF also closes any open dropdown (a hidden bar can hold
/// no open menu).
pub fn set_menu_bar_on(on: bool) {
    // Self-acquiring: geometry tests share this reentrant global with the
    // page-layout state across many call sites, so the write takes the lock
    // itself (held across both this store and the nested `set_open` below)
    // rather than demanding every one of them pre-hold it.
    #[cfg(test)]
    let _g = crate::testlock::serial();
    menu_bar_flag().set(on);
    if !on {
        set_open(None);
    }
}

pub fn toggle() -> bool {
    let next = !menu_bar_on();
    set_menu_bar_on(next);
    next
}

pub fn open_menu() -> Option<usize> {
    let v = OPEN_MENU.load(Ordering::Relaxed);
    (v != NONE).then_some(v)
}

/// Open the dropdown for menu `i` (`None` closes any open one). A no-op-safe setter:
/// the renderer / hit-test tolerate an out-of-range index (nothing draws / nothing
/// hits), so a stale index can never panic.
pub fn set_open(i: Option<usize>) {
    // Share the geometry test lock; nested use from `set_menu_bar_on` is reentrant.
    #[cfg(test)]
    let _g = crate::testlock::serial();
    OPEN_MENU.store(i.unwrap_or(NONE), Ordering::Relaxed);
}

pub fn toggle_open(i: usize) -> Option<usize> {
    let next = if open_menu() == Some(i) {
        None
    } else {
        Some(i)
    };
    set_open(next);
    next
}

pub const BAR_INSET_X: f32 = 8.0;
pub const TITLE_PAD_X: f32 = 12.0;

/// The bar's own vertical breathing room, ABOVE and BELOW the title line — the one
/// role this constant has (a 45-reader-shaped census turned up a single caller:
/// [`bar_height`]). Authored in LOGICAL px like every new chrome dimension
/// ([`crate::render::Logical`]), so it cannot reach a caller unscaled: the newtype
/// has no arithmetic of its own except [`Logical::px`], which demands the same
/// `scale` the line-height argument was already computed with.
pub const BAR_PAD_Y: Logical = Logical(5.0);

pub const DROP_PAD_X: f32 = 10.0;
pub const DROP_PAD_Y: f32 = 6.0;

/// The drawn bar's height in device px: the caller's already-scaled line height plus
/// the pad on both sides. `scale` is a required parameter, not folded into
/// `line_height` by the caller — the bypass items 314/315 closed for `TEXT_LEFT` /
/// `TEXT_TOP` (a scaled argument silently added to an unscaled constant) is closed
/// here the same way: no caller can hand this fn a pre-scaled `BAR_PAD_Y`, because
/// there is no way to scale a `Logical` except through [`Logical::px`], which this fn
/// alone calls.
pub fn bar_height(line_height: f32, scale: f32) -> f32 {
    line_height + 2.0 * BAR_PAD_Y.px(scale)
}

pub const EDGE_BLEED_PX: f32 = 4.0;

pub fn bleed_to_canvas_edges(rect: [f32; 4], canvas_w: f32) -> [f32; 4] {
    const FLUSH_EPS: f32 = 0.5;
    let [mut x, mut y, mut w, mut h] = rect;
    if y <= FLUSH_EPS {
        y -= EDGE_BLEED_PX;
        h += EDGE_BLEED_PX;
    }
    if x <= FLUSH_EPS {
        x -= EDGE_BLEED_PX;
        w += EDGE_BLEED_PX;
    }
    if x + w >= canvas_w - FLUSH_EPS {
        w += EDGE_BLEED_PX;
    }
    [x, y, w, h]
}

/// One title's laid-out horizontal extents (px, absolute canvas x), from
/// [`boxes_from_extents`]. Built from the SHAPED glyph positions the pipeline read
/// back (never a parallel layout), so the drawn glyphs and the click bands agree.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TitleBox {
    pub band_left: f32,
    pub text_left: f32,
    pub text_right: f32,
    pub band_right: f32,
}

pub fn boxes_from_extents(extents: &[(f32, f32)]) -> Vec<TitleBox> {
    let n = extents.len();
    let mut out = Vec::with_capacity(n);
    for k in 0..n {
        let (l, r) = extents[k];
        let band_left = if k == 0 {
            (l - TITLE_PAD_X).max(0.0)
        } else {
            (extents[k - 1].1 + l) * 0.5
        };
        let band_right = if k + 1 < n {
            (r + extents[k + 1].0) * 0.5
        } else {
            r + TITLE_PAD_X
        };
        out.push(TitleBox {
            band_left,
            text_left: l,
            text_right: r,
            band_right,
        });
    }
    out
}

/// Which title's band contains the point `(px, py)` — `Some(index)` when `py` is
/// within the bar's height and `px` falls in a title band, else `None`. The single
/// hit-test owner for the bar, read by the click handler AND the cursor-shape flag,
/// so a hovered title can never disagree with a clickable one.
pub fn title_at(boxes: &[TitleBox], bar_h: f32, px: f32, py: f32) -> Option<usize> {
    if py < 0.0 || py >= bar_h {
        return None;
    }
    boxes
        .iter()
        .position(|b| px >= b.band_left && px < b.band_right)
}

pub fn in_bar(bar_h: f32, py: f32) -> bool {
    py >= 0.0 && py < bar_h
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DropRow {
    pub top: f32,
    pub height: f32,
    pub separator: bool,
}

pub fn drop_rows(separators: &[bool], row_h: f32) -> (Vec<DropRow>, f32) {
    let mut rows = Vec::with_capacity(separators.len());
    let mut top = 0.0;
    for &sep in separators {
        rows.push(DropRow {
            top,
            height: row_h,
            separator: sep,
        });
        top += row_h;
    }
    (rows, top)
}

pub fn drop_rect(anchor: &TitleBox, bar_h: f32, content_w: f32, rows_total_h: f32) -> [f32; 4] {
    let w = content_w.max(0.0) + 2.0 * DROP_PAD_X;
    let h = rows_total_h + 2.0 * DROP_PAD_Y;
    [anchor.band_left, bar_h, w, h]
}

pub fn drop_item_at(rect: [f32; 4], rows: &[DropRow], px: f32, py: f32) -> Option<usize> {
    let [x, y, w, h] = rect;
    if px < x || px >= x + w || py < y || py >= y + h {
        return None;
    }
    let local_y = py - (y + DROP_PAD_Y);
    if local_y < 0.0 {
        return None;
    }
    rows.iter()
        .position(|r| !r.separator && local_y >= r.top && local_y < r.top + r.height)
}

#[cfg(test)]
mod tests;
