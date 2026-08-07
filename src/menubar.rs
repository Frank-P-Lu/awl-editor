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

use crate::render::{Logical, Physical};
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

/// The bar's own horizontal breathing room: the x the FIRST title's glyphs begin at,
/// measured from the canvas's left edge. LOGICAL, on the usage evidence — it is added
/// to shaped glyph x-offsets that were themselves produced at device metrics
/// (`m.font_size * LABEL`), so at 2x an 8-device-px inset reads as four logical px
/// and the titles crowd the edge as the display gets denser. Same role, same answer
/// as [`BAR_PAD_Y`] one axis over. (`readout.rs`'s `CANVAS_INSET` is the same
/// number declared `Physical`, for a different reason: promoting THAT one moves six
/// bottom/right-anchored chrome call sites at once and owes a sweep across every
/// anchor arm. This one has a single reader.)
pub const BAR_INSET_X: Logical = Logical(8.0);

/// How far a title's CLICKABLE band reaches past its own ink, on the two outer edges
/// only — the first title's left and the last title's right. LOGICAL, on the usage
/// evidence: every INTERIOR band edge is the midpoint between two device-scaled glyph
/// extents, so it widens with DPI on its own, and a device-fixed pad here would make
/// the two outer bands shrink relative to every interior one as the display gets
/// denser. The bands stay derived from the same shaped positions the pixels use.
pub const TITLE_PAD_X: Logical = Logical(12.0);

/// The bar's own vertical breathing room, ABOVE and BELOW the title line — the one
/// role this constant has (a 45-reader-shaped census turned up a single caller:
/// [`bar_height`]). Authored in LOGICAL px like every new chrome dimension
/// ([`crate::render::Logical`]), so it cannot reach a caller unscaled: the newtype
/// has no arithmetic of its own except [`Logical::px`], which demands the same
/// `scale` the line-height argument was already computed with.
pub const BAR_PAD_Y: Logical = Logical(5.0);

/// The open dropdown card's own padding, around its item rows. Both LOGICAL, on the
/// usage evidence: each is added to a quantity that is ALREADY device-scaled and
/// whose siblings are already enrolled — the card's width is a char-count estimate
/// off `m.char_width * LABEL` floored by `DROP_MIN_WIDTH.px(scale)`, and its height
/// is a `Rows`-derived row pitch off the LABEL line height. A device-fixed pad beside
/// scaled content is the exact defect items 314/315/321 closed three times over: the
/// card's ink would grow with the display and its breathing room would not.
pub const DROP_PAD_X: Logical = Logical(10.0);
pub const DROP_PAD_Y: Logical = Logical(6.0);

/// The drawn bar's height in device px: the caller's already-scaled line height plus
/// the pad on both sides. `scale` is a required parameter, not folded into
/// `line_height` by the caller — the bypass items 314/315 closed for `TEXT_LEFT` /
/// `TEXT_TOP` (a scaled argument silently added to an unscaled constant) is closed
/// here the same way: no caller can hand this fn a pre-scaled `BAR_PAD_Y`, because
/// there is no way to scale a `Logical` except through [`Logical::px`], which this fn
/// alone calls.
///
/// ⚠️ **NOT THE DOOR. `TextPipeline::menubar_reserve` IS.** This is the ARITHMETIC;
/// the question "how tall is the menu bar on this frame" has exactly one answer, and
/// it lives in `render/geometry.rs` where the gate on `menu_bar_on()` and the
/// LABEL-scaled line height live with it. The reserve and the drawn strip each used
/// to spell `bar_height(metrics.line_height * LABEL, metrics.scale)` for themselves
/// and agreed only by both remembering to — the same shape `TEXT_TOP +
/// menubar_reserve()` had at SIX call sites, where a real bug then survived at half
/// of them. `pub(crate)` narrows the reach; `tests::bar_height_has_exactly_one_
/// non_test_caller` is what actually holds it to one, with no wildcard, so a second
/// consumer fails by name instead of quietly becoming a second owner.
pub(crate) fn bar_height(line_height: f32, scale: f32) -> f32 {
    line_height + 2.0 * BAR_PAD_Y.px(scale)
}

/// ⚠️ **THE ANNOTATED EXCEPTION IN THIS FILE: `Physical`, and the reason is the
/// RASTERIZER.** This is how far a rect is pushed PAST a canvas edge it already runs
/// flush to, and the only thing it has to cover is what the shader would otherwise
/// feather on a visible pixel. Both quantities it has to clear are fixed in DEVICE
/// pixels and neither moves with DPI: `shaders/selection.wgsl` antialiases its
/// rounded-rect SDF with `smoothstep(-1.0, 1.0, d)` — a ~1 px band each side of the
/// edge, in framebuffer px — and the ordinary fill pipeline's corner radius is
/// `selection.rs`'s `CORNER_RADIUS: f32 = 2.5`, uploaded once at construction and
/// never multiplied by `scale`. ~3.5 device px to clear, cleared by 4. Declaring this
/// `Logical` would grow the off-canvas overdraw on every Retina display while the
/// thing it is hiding stayed the same size — the reader's eye is not the reference
/// here, the device grid is. (`Physical` is deliberately outside the no-bare-field
/// half of the declaration law: `px_physical` is the identity, so a `.0` here loses
/// nothing, and `FLUSH_EPS` below is read inside a `const` where no method call is
/// legal.)
pub const EDGE_BLEED_PX: Physical = Physical(4.0);

pub fn bleed_to_canvas_edges(rect: [f32; 4], canvas_w: f32) -> [f32; 4] {
    /// SUB-PIXEL TOLERANCE on a device-grid coincidence test, so `Physical` for a
    /// second reason: this is not breathing room, it is "does this edge land on the
    /// boundary pixel". Half a DEVICE pixel is the whole question; scaled, a rect
    /// sitting 1.4 device px clear of the edge on a 3x display would start counting
    /// as flush and get bled — a different rect, not a better-tuned one.
    const FLUSH_EPS: Physical = Physical(0.5);
    let [mut x, mut y, mut w, mut h] = rect;
    if y <= FLUSH_EPS.0 {
        y -= EDGE_BLEED_PX.0;
        h += EDGE_BLEED_PX.0;
    }
    if x <= FLUSH_EPS.0 {
        x -= EDGE_BLEED_PX.0;
        w += EDGE_BLEED_PX.0;
    }
    if x + w >= canvas_w - FLUSH_EPS.0 {
        w += EDGE_BLEED_PX.0;
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

/// The clickable bands, from the shaped extents. `scale` is a REQUIRED parameter
/// rather than a pad the caller resolves, for the reason [`bar_height`]'s doc gives:
/// a `Logical` has no arithmetic but [`Logical::px`], so no caller can reach the
/// outer pad unscaled and this fn is its only resolver.
pub fn boxes_from_extents(extents: &[(f32, f32)], scale: f32) -> Vec<TitleBox> {
    let n = extents.len();
    let pad = TITLE_PAD_X.px(scale);
    let mut out = Vec::with_capacity(n);
    for k in 0..n {
        let (l, r) = extents[k];
        let band_left = if k == 0 {
            (l - pad).max(0.0)
        } else {
            (extents[k - 1].1 + l) * 0.5
        };
        let band_right = if k + 1 < n {
            (r + extents[k + 1].0) * 0.5
        } else {
            r + pad
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

/// The open dropdown card's outer rect: its anchor title's band left edge, hanging
/// off the bar's bottom, sized to the item rows plus [`DROP_PAD_X`]/[`DROP_PAD_Y`].
/// Takes `scale` for the same reason [`boxes_from_extents`] does.
pub fn drop_rect(
    anchor: &TitleBox,
    bar_h: f32,
    content_w: f32,
    rows_total_h: f32,
    scale: f32,
) -> [f32; 4] {
    let w = content_w.max(0.0) + 2.0 * DROP_PAD_X.px(scale);
    let h = rows_total_h + 2.0 * DROP_PAD_Y.px(scale);
    [anchor.band_left, bar_h, w, h]
}

/// The card's INNER top-left — where row 0's ink begins. The one owner of the card
/// pad's resolution, so the drawn rows ([`crate::render`]'s dropdown prepare) and the
/// hit-test below can never disagree about where the row grid starts.
pub fn drop_inner_origin(rect: [f32; 4], scale: f32) -> (f32, f32) {
    (
        rect[0] + DROP_PAD_X.px(scale),
        rect[1] + DROP_PAD_Y.px(scale),
    )
}

pub fn drop_item_at(
    rect: [f32; 4],
    rows: &[DropRow],
    px: f32,
    py: f32,
    scale: f32,
) -> Option<usize> {
    let [x, y, w, h] = rect;
    if px < x || px >= x + w || py < y || py >= y + h {
        return None;
    }
    let local_y = py - drop_inner_origin(rect, scale).1;
    if local_y < 0.0 {
        return None;
    }
    rows.iter()
        .position(|r| !r.separator && local_y >= r.top && local_y < r.top + r.height)
}

#[cfg(test)]
mod tests;
