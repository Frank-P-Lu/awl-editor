//! Awl-rendered web/Linux menu bar. It shares the native menu roster and action
//! dispatch, so it adds no menu-only behavior. Layout consumes shaped title extents
//! for both drawing and hit testing. It defaults off on macOS, preserving normal
//! native captures unless explicitly enabled.

use std::sync::atomic::{AtomicUsize, Ordering};

use crate::toggle::Toggle;

/// The drawn bar's default per platform — off on macOS (a real system bar
/// already exists), on elsewhere. Split out of the `cfg!` below so BOTH arms
/// stay readable from ANY build: the generated reference documents one
/// `menu_bar` default for every platform, and a bare `cfg!` would make that
/// document differ between the machine that writes it and the host that checks it.
pub(crate) const MENU_BAR_DEFAULT_MACOS: bool = false;
pub(crate) const MENU_BAR_DEFAULT_OTHER: bool = true;

/// Whether the rendered menu bar is drawn. It defaults on where there is no native
/// bar and off on macOS; config, command, and capture flags can override it.
static MENU_BAR_ON: Toggle = Toggle::new(if cfg!(target_os = "macos") {
    MENU_BAR_DEFAULT_MACOS
} else {
    MENU_BAR_DEFAULT_OTHER
});

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
    MENU_BAR_ON.on()
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
    MENU_BAR_ON.set(on);
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
pub const BAR_PAD_Y: f32 = 5.0;

pub const DROP_PAD_X: f32 = 10.0;
pub const DROP_PAD_Y: f32 = 6.0;

pub fn bar_height(line_height: f32) -> f32 {
    line_height + 2.0 * BAR_PAD_Y
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
mod tests {
    use super::*;

    /// THE SLIVER FIX, pure: a rect flush on all three canvas-touching sides (the
    /// bar's own ground strip) bleeds top/left/right by `EDGE_BLEED_PX`, and its
    /// BOTTOM (never a canvas edge for the bar) is untouched.
    #[test]
    fn bleed_extends_every_flush_edge_and_leaves_the_bottom_alone() {
        let rect = [0.0, 0.0, 1200.0, 32.0];
        let bled = bleed_to_canvas_edges(rect, 1200.0);
        assert_eq!(bled[0], -EDGE_BLEED_PX, "left bleeds past x=0");
        assert_eq!(bled[1], -EDGE_BLEED_PX, "top bleeds past y=0");
        assert_eq!(
            bled[2],
            1200.0 + 2.0 * EDGE_BLEED_PX,
            "width bleeds on both flush sides"
        );
        assert_eq!(
            bled[3],
            32.0 + EDGE_BLEED_PX,
            "height bleeds on the top side only"
        );
        // The bottom edge (y + h) moves by exactly the top bleed, i.e. the BOTTOM
        // itself (a non-flush edge) never moved: bled_y + bled_h == rect_y + rect_h.
        assert_eq!(
            bled[1] + bled[3],
            rect[1] + rect[3],
            "the bottom edge itself is unmoved"
        );
    }

    #[test]
    fn bleed_leaves_interior_left_and_right_edges_untouched() {
        let rect = [400.0, 0.0, 80.0, 32.0]; // nowhere near x=0 or x=1200
        let bled = bleed_to_canvas_edges(rect, 1200.0);
        assert_eq!(bled[0], 400.0, "left edge is interior, untouched");
        assert_eq!(bled[2], 80.0, "width is untouched (no side bled)");
        assert_eq!(
            bled[1], -EDGE_BLEED_PX,
            "top still bleeds — it's always flush for the bar"
        );
        assert_eq!(bled[3], 32.0 + EDGE_BLEED_PX);
    }

    #[test]
    fn bleed_is_independent_per_side() {
        let rect = [1100.0, 0.0, 100.0, 32.0]; // right edge exactly at canvas_w=1200
        let bled = bleed_to_canvas_edges(rect, 1200.0);
        assert_eq!(bled[0], 1100.0, "left edge is interior, untouched");
        assert_eq!(
            bled[2],
            100.0 + EDGE_BLEED_PX,
            "right bleeds (flush to canvas_w)"
        );
        assert_eq!(bled[1], -EDGE_BLEED_PX);
    }

    /// A rect NOT touching the canvas top at all (hypothetical future caller) is
    /// left exactly alone on every side — the fix only ever touches an edge that is
    /// ACTUALLY flush with the canvas boundary, never a rect drawn purely elsewhere.
    #[test]
    fn bleed_is_a_total_no_op_off_every_canvas_edge() {
        let rect = [200.0, 50.0, 300.0, 40.0];
        assert_eq!(bleed_to_canvas_edges(rect, 1200.0), rect);
    }

    #[test]
    fn globals_toggle_and_open_close() {
        let _g = crate::testlock::serial();
        let ambient = menu_bar_on(); // not `cfg!`: that reflects the host, not the initializer
        set_menu_bar_on(true);
        assert!(menu_bar_on());
        assert_eq!(toggle_open(2), Some(2));
        assert_eq!(open_menu(), Some(2));
        assert_eq!(toggle_open(2), None);
        assert_eq!(open_menu(), None);
        set_open(Some(1));
        assert_eq!(toggle_open(3), Some(3));
        set_open(Some(0));
        set_menu_bar_on(false);
        assert!(!menu_bar_on());
        assert_eq!(open_menu(), None, "a hidden bar holds no open dropdown");
        set_open(Some(0));
        assert!(toggle(), "toggle from off -> on");
        set_open(Some(0));
        assert!(!toggle(), "toggle from on -> off closes the dropdown");
        assert_eq!(open_menu(), None);
        set_menu_bar_on(ambient);
    }

    #[test]
    fn boxes_from_extents_abut_at_midpoints() {
        let boxes = boxes_from_extents(&[(20.0, 50.0), (70.0, 96.0), (110.0, 146.0)]);
        assert_eq!(boxes.len(), 3);
        assert_eq!(boxes[0].band_left, 20.0 - TITLE_PAD_X);
        assert_eq!(boxes[0].text_left, 20.0);
        assert_eq!(boxes[0].text_right, 50.0);
        assert_eq!(boxes[0].band_right, (50.0 + 70.0) / 2.0);
        assert_eq!(boxes[1].band_left, boxes[0].band_right, "bands abut");
        assert_eq!(boxes[1].band_right, (96.0 + 110.0) / 2.0);
        assert_eq!(boxes[2].band_left, boxes[1].band_right);
        assert_eq!(boxes[2].band_right, 146.0 + TITLE_PAD_X);
    }

    #[test]
    fn title_at_maps_x_across_the_whole_bar() {
        let boxes = boxes_from_extents(&[(20.0, 50.0), (70.0, 96.0), (110.0, 146.0)]);
        let bar_h = bar_height(20.0);
        assert_eq!(
            title_at(&boxes, bar_h, boxes[0].text_left + 1.0, 4.0),
            Some(0)
        );
        assert_eq!(
            title_at(&boxes, bar_h, boxes[1].text_left + 1.0, 4.0),
            Some(1)
        );
        assert_eq!(
            title_at(&boxes, bar_h, boxes[2].band_right - 1.0, 4.0),
            Some(2)
        );
        assert_eq!(
            title_at(&boxes, bar_h, boxes[0].text_left, bar_h + 1.0),
            None
        );
        assert_eq!(title_at(&boxes, bar_h, 0.0, 4.0), None);
        assert_eq!(
            title_at(&boxes, bar_h, boxes[2].band_right + 5.0, 4.0),
            None
        );
    }

    #[test]
    fn drop_rows_stack_uniform_slots_marking_separators() {
        let (rows, total) = drop_rows(&[false, false, true, false], 22.0);
        assert_eq!(rows.len(), 4);
        assert_eq!(rows[0].top, 0.0);
        assert_eq!(rows[1].top, 22.0);
        assert_eq!(rows[2].top, 44.0);
        assert!(rows[2].separator, "the third row is the separator");
        assert_eq!(rows[3].top, 66.0);
        assert_eq!(total, 4.0 * 22.0);
    }

    #[test]
    fn drop_item_at_hits_clickable_rows_only() {
        let anchor = TitleBox {
            band_left: 40.0,
            text_left: 52.0,
            text_right: 84.0,
            band_right: 90.0,
        };
        let bar_h = bar_height(20.0);
        let (rows, total) = drop_rows(&[false, true, false], 22.0);
        let rect = drop_rect(&anchor, bar_h, 120.0, total);
        assert_eq!(rect[0], 40.0, "the dropdown left-aligns under its title");
        assert_eq!(rect[1], bar_h, "it hangs just below the bar");
        assert_eq!(rect[2], 120.0 + 2.0 * DROP_PAD_X);
        let (x, y) = (rect[0] + 5.0, rect[1] + DROP_PAD_Y + 1.0);
        assert_eq!(drop_item_at(rect, &rows, x, y), Some(0));
        // The separator row (index 1) is never a hit.
        let sep_y = rect[1] + DROP_PAD_Y + rows[1].top + 1.0;
        assert_eq!(drop_item_at(rect, &rows, x, sep_y), None);
        let third_y = rect[1] + DROP_PAD_Y + rows[2].top + 1.0;
        assert_eq!(drop_item_at(rect, &rows, x, third_y), Some(2));
        assert_eq!(drop_item_at(rect, &rows, rect[0] - 1.0, y), None);
        assert_eq!(drop_item_at(rect, &rows, x, rect[1] + 1.0), None);
    }
}
