use glyphon::{
    Attrs, Buffer as GlyphBuffer, Cache, CacheKey, Family, FontSystem, Metrics as GlyphMetrics,
    Resolution, Shaping, SwashCache, SwashContent, TextArea, TextAtlas, TextBounds, TextRenderer,
    Viewport, Wrap,
};

use crate::background::{BackgroundPipeline, BgDesc};
use crate::caret::{CORNER_RADIUS, CaretAnim, CaretMode, CaretPipeline, STREAK_RADIUS, Sample};
use crate::caret_glyph::{CaretGlyphPipeline, GlyphMask};
use crate::selection::SelectionPipeline;
use crate::spell::Misspelling;
use crate::spellunderline::{SpellUnderlinePipeline, Squiggle};
use crate::theme;

/// Layout-dependent caret geometry; animation and GPU pipelines live in crate modules.
mod caret;
mod caret_body;

/// Measured bundled-face pitch used by the caret's mono/proportional fork.
/// [`FONT_THEME_FACES`] declares it alongside each face's bytes.
pub(crate) mod facepitch;

/// Geometric curly-quote orientation check — the permanent roster law that a
/// font-file bug like a transposed pair of raised quote outlines cannot ship
/// silently again. Test-only: its one consumer is
/// `render::tests::quote_orientation`, a font-QA sweep, not a
/// runtime path.
#[cfg(test)]
pub(crate) mod quotecheck;

pub(crate) mod dither;

mod spans;
#[cfg(test)]
pub(crate) use spans::wysiwyg_reveals;
use spans::*;

mod rowgeom;

mod chrome;
#[cfg(test)]
pub(crate) use chrome::POPOVER_VPAD;
pub use chrome::PanelHit;

/// The `AWL_*_FORCE` dev-only render/theme override knobs, consolidated into
/// ONE [`overrides::RenderOverrides`] struct. See that module's doc.
pub(crate) mod overrides;
#[cfg(test)]
pub(crate) use overrides::{
    ForcedKnob, RenderOverrides, classify_forced_knob, parse_bar_config_force,
    parse_facet_style_force, parse_list_style_force, parse_motion_force, parse_overlay_align,
    parse_overlay_anchor_force, parse_overlay_density_force, parse_overlay_motion_force,
    parse_overlay_slant_force, parse_overlay_style_force, set_bar_config_test_override,
    set_card_anchor_test_override, set_chrome_face_test_override, set_facet_style_test_override,
    set_list_style_test_override, set_motion_test_override, set_overlay_density_test_override,
    set_overlay_motion_test_override, set_pane_split_test_override, set_slant_test_override,
    set_test_override, set_title_style_test_override,
};
pub(crate) use overrides::{OverlayMotionProbe, SlantProbe, TypeDensity};

mod rowlayout;
pub use rowlayout::rail_frac_at;
mod blur;
pub(crate) mod plan;

mod geometry;
use geometry::*;
pub use geometry::{ImageHandle, ResizeEdge, visible_lines_z};
// Test-only: lets `crate::lava`'s geometry-sweep laws read the SAME column
// formula the live app does, never a parallel computation.
#[cfg(test)]
pub use geometry::{column_left_for, column_width_for};

mod layout_report;
pub(crate) use layout_report::LayoutReport;

mod text;
pub use text::ScriptFontReports;

mod reports;

/// LAYER GEOMETRY — the rect / squiggle builders that turn document + view state
/// into the instanced quads each draw layer uploads (selection / range / search
/// rects, the markdown rule quads, the spell squiggles, the IME preedit cells, the
/// search panel layout). Inherent methods ON [`TextPipeline`] reading its shaped
/// buffer / cursor / selection state, carved out verbatim. Byte-identical.
mod rects;

pub(crate) mod livingband;

/// INLINE IMAGES — the decode + GPU-upload cache (native-only, PNG). Keyed by
/// canonical path + mtime; decodes O(visible) and downscales to the display width.
#[cfg(not(target_arch = "wasm32"))]
mod image_cache;

/// PER-LAYER PREPARE ORCHESTRATION — the per-frame `prepare_*_layer` steps the
/// aggregating [`TextPipeline::prepare`] (still in `render.rs`) folds together:
/// background, document text, animated caret, selection/search, chrome, and spell
/// underlines. Inherent methods ON [`TextPipeline`] driving its GPU renderers /
/// pipelines, carved out verbatim. Byte-identical.
mod layers;

pub mod perfbench;

/// FRAME PROFILER — a hidden `--bench-frame` harness timing the EXACT live
/// redraw sequence (advance → each `prepare` sub-call in order → render encode
/// → submit+poll → atlas.trim) per stage over the real repo docs, at the
/// live-report canvas (2910x1720 @2x, debug panel hot). Also hosts the hidden
/// `--bench-theme-burst` THEME-BURST profiler: N successive font-changing theme
/// switches (the picker's live preview) timing `sync_theme` + the first frame
/// after each, cold/warm laps for atlas retention, plus an EAGER burst over the
/// same worlds (no debounce: every arrow step pays its own reshape) witnessing
/// the reshape count. A child of `render` for the same reason as [`perfbench`].
/// Dev-only; never on the render path.
pub mod framebench;

pub mod benchsuite;

pub mod caretbench;

/// The render-relevant editor SNAPSHOT — the [`ViewState`] struct + its canonical
/// [`ViewState::base`] default, carved out of `render.rs` VERBATIM into a physical
/// home (pure data, no `&self`, no GPU types — see the module doc). Re-exported
/// here so `crate::render::ViewState` resolves unchanged for every caller.
mod viewstate_def;
pub use viewstate_def::{DocSource, FoldTail, ViewState};

mod pipeline_band_epoch;
mod pipeline_draw;
mod pipeline_geometry;
mod pipeline_inverse;
mod pipeline_layers;
mod pipeline_overlay;
mod pipeline_prepare;
mod rotated_location;

pub const FONT_SIZE: f32 = 24.0;
pub const LINE_HEIGHT: f32 = 32.0;
pub const TEXT_LEFT: Logical = Logical(16.0);
pub const NONPAGE_INSET: Logical = Logical(32.0);
/// The page's inner text pad, in CHARACTER cells rather than pixels: the gap
/// between the page plate's edge and the writing column's own glyphs. Resolved
/// against `metrics.char_width`, which [`Metrics::with_dpi`] has already
/// multiplied by `zoom * dpi`, so it must never also pass [`Metrics::px`].
pub const PAGE_TEXT_PAD_CHARS: Chars = Chars(3.0);
pub const TEXT_TOP: Logical = Logical(16.0);
pub const PAGE_MIN_MARGIN_PX: Logical = Logical(64.0);
pub const PAGE_MIN_MARGIN_FRAC: f32 = 0.10;

pub fn page_min_margin(window_w: f32, dpi: f32) -> f32 {
    f32::max(PAGE_MIN_MARGIN_PX.px(dpi), window_w * PAGE_MIN_MARGIN_FRAC)
}
pub const CHAR_WIDTH: f32 = 14.4;
/// Caret cell metrics in pixels (at zoom 1.0). `CARET_W` is the default cell
/// advance used to place the glyph cell and as the MINIMUM block width at
/// end-of-line / empty lines. `CARET_H` is the glyph cell height (the box the
/// resting square covers, and that selection/preedit share).
pub const CARET_W: f32 = CHAR_WIDTH;
pub const CARET_H: f32 = 28.0;
pub const CARET_BLOCK_H: f32 = CARET_H * 0.80; // ~22.4 px
/// The caret ink box's own two pads, both resolved at the FULL `zoom * dpi`
/// factor: the caret sites read the STORED [`Metrics::scale`] and hand it to
/// [`Logical::px`], so these are logical lengths that already meet the DPI
/// multiply. Recovering the factor from an already-scaled length instead
/// (`caret_h / CARET_H`) is NOT bit-equal to it — multiplying by `s` and dividing
/// back is no `f32` round trip, and dpi 1 and 2, the only two any capture uses,
/// are exactly the factors where it looks like one (`caret_scale_law`).
pub const CARET_DESCENDER_PAD: Logical = Logical(1.5);
pub const CARET_INK_PAD: Logical = Logical(3.0);
/// The resting cell's HORIZONTAL counterpart to [`CARET_INK_PAD`]: grown
/// equally on both sides of the anchored glyph's own ink centre (never a
/// per-glyph raster read of its own — `caret_visual_body_dims` folds it in
/// alongside the width floor, so every anchor still comes from the ONE shared
/// body owner). Before this existed the resting body was the bare raster ink
/// width with no accent margin at all, which read as hugging the letter
/// rather than standing beside it — the vertical pad's own dead-space law
/// (`proportional_worlds_take_one_caret_top_at_every_letter`) already sits
/// within a fraction of a pixel of its ceiling on the roster's tightest face,
/// so this axis, not a taller pad, is where a modestly larger body has room.
pub const CARET_INK_PAD_W: Logical = Logical(1.0);
pub const CARET_STREAK_H: f32 = 2.8;
pub const CARET_STREAK_MIN_LEN: f32 = 10.0;
pub const CARET_STREAK_MAX_LEN: f32 = 64.0;
pub const CARET_STREAK_VEL_FULL: f32 = 2600.0;

pub const CARET_TRAIL_TEXT_CENTER_DROP: f32 = 3.0;

pub const CARET_SPACE_BAR_W: Logical = Logical(3.0);

pub const IBEAM_W: Logical = Logical(2.6);

pub const CARET_MORPH_SETTLE_SHOW: f32 = 0.65;

/// Hard, uniform dilation radius (LOGICAL px) applied to the MORPH glyph
/// silhouette so the caret reads a touch FATTER/bolder than the underlying
/// letter — but still SOLID in the accent (a morphological max-expansion of the
/// glyph's own crisp coverage, NOT a soft translucent glow or a tapered halo).
/// Think "the same letter, a bit bolder, one solid accent colour." Resolved through
/// [`Metrics::px`] on the CPU and passed per-instance to the shader.
pub const CARET_MORPH_DILATE_PX: Logical = Logical(2.0);

/// Zoom clamps and step. Effective metrics = base metric * zoom. 1.0 is the
/// default — but NOT, despite what this comment used to claim, the only zoom the
/// `--screenshot` path ever sees: `--zoom` sets it, and STICKY ZOOM folds
/// `config.zoom` in behind that flag for captures too (`main/args.rs`). A
/// capture-based test whose arithmetic uses BASE constants must still pin
/// `--zoom`; capture tests should instead read the sidecar's EFFECTIVE
/// `font.size` / `font.line_height`, which report this effective metric
/// scale. Believing this comment once made a personal `zoom = 1.5` turn a pixel
/// test red with no product change behind it.
///
/// The band/step/default no longer live here AT ALL. The former
/// `ZOOM_MIN`/`ZOOM_MAX`/`ZOOM_STEP` consts were deleted rather than re-pointed —
/// an alias is a drift risk, and there is now exactly ONE place zoom's authored
/// numbers exist: [`crate::range::ZOOM`] (`.min`/`.max`/`.step`/`.default`), the
/// same spec the Settings rail, the ⌘± keys, the ⌘-wheel, `--zoom` and a typed
/// `125%` all read. Every former reader was updated to read the spec.
/// Clamp + round a zoom factor to a sane stepped value. Rounding to the nearest
/// step keeps Cmd+= / Cmd+- / Ctrl+wheel landing on stable factors (so repeated
/// presses don't drift into ugly fractions) and keeps captures reproducible.
/// FINITE GUARD: NaN would sail straight through the step arithmetic AND
/// `f32::clamp` (clamp returns NaN for NaN) and poison every zoom-derived metric,
/// so it falls back to the 1.0 default; ±inf saturates through the normal clamp.
/// The result is always finite in `[ZOOM.min, ZOOM.max]`.
///
/// A ONE-LINE DELEGATE to [`crate::range::RangeSpec::quantize`]: this is
/// still the door every zoom caller knocks on, but the arithmetic behind it now
/// lives with the rest of the range rule (bit-identical to the formula that used
/// to sit here — `range::tests::quantize_reproduces_the_historical_zoom_clamp_formula`).
pub fn clamp_zoom(z: f32) -> f32 {
    crate::range::ZOOM.quantize(z)
}

/// A length authored in LOGICAL pixels — chrome's DEFAULT space, and the only
/// one a new chrome dimension should ever be written in.
///
/// awl draws in device pixels and nothing else; "logical" here means exactly
/// "multiplied by [`Metrics::scale`] on its way in", which is the same `s` the
/// text and caret families already pass through. The newtype IS the enrollment:
/// a `Logical` has no arithmetic of its own, so it cannot reach a draw call
/// without going through [`Metrics::px`], and a hand-authored constant cannot
/// silently stay physical by forgetting a multiply.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Logical(pub f32);

impl Logical {
    /// The one multiply. [`Metrics::px`] is the door that supplies `scale`;
    /// this exists for the geometry POLICIES that take a scale rather than a
    /// whole `Metrics` (they are pure and unit-tested without one).
    pub fn px(self, scale: f32) -> f32 {
        self.0 * scale
    }
}

impl LogicalGrowOnly {
    /// **THE FLOOR IS THE DISPLAY, NOT THE NUMBER ONE.** Grow-only is a ZOOM
    /// policy — a cap the user's type size may widen but never shrink below the
    /// value it was tuned at — while `scale` is `zoom * dpi`, so flooring it at
    /// a bare `1.0` makes the regime where the floor binds a property of the
    /// PANEL: the whole cap is held at `dpi 1` for every zoom under 1, and only
    /// below zoom 0.5 at `dpi 2`. Two readers at the same logical window and the
    /// same zoom then get different compositions, and the denser display gets
    /// the narrower card.
    ///
    /// `scale.max(dpi)` is the same rule stated in the space the cap is authored
    /// in — `dpi * zoom.max(1.0)` — so the resolved cap holds one LOGICAL width
    /// across densities and still only ever grows with zoom.
    pub fn px(self, scale: f32, dpi: f32) -> f32 {
        self.0 * scale.max(dpi)
    }
}

/// A length in DEVICE pixels that deliberately does NOT scale — the ANNOTATED
/// exception. Constructing one is the annotation; every declaration site states
/// what makes the device grid, rather than the reader's eye, the right
/// reference (a device resource bound, a quantity already derived from device
/// geometry, a rasterization feather).
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Physical(pub f32);

/// A logical length that only ever GROWS WITH ZOOM: held at its authored
/// logical value while `zoom <= 1`, proportional above it. The HONEST third
/// classification for a width CAP — recording one of these as plainly physical
/// reintroduces the zoom-blind collapse the grow-only form exists to fix, and
/// recording it as plainly logical shrinks it below the value it was tuned at.
/// It is LOGICAL in the density: the floor is the display's own ratio, never a
/// bare `1.0` (see [`LogicalGrowOnly::px`]). See [`Metrics::px_grow_only`].
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct LogicalGrowOnly(pub f32);

/// A length in multiples of the overlay CHARACTER cell. Already correct by
/// construction — the char width it resolves against is itself scaled — so it
/// must never also pass [`Metrics::px`], or it doubles.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Chars(pub f32);

/// A length in multiples of the overlay ROW pitch. Correct by construction for
/// the same reason [`Chars`] is, and double-scaled by the same mistake.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Rows(pub f32);

/// A DURATION in milliseconds — deliberately NOT a length, and the type says
/// so. An animation's span belongs to the clock, not to the pixel grid: it must
/// hold its value on a retina panel where every length doubles, so it has no
/// [`Metrics::px`] and cannot acquire one by accident.
///
/// The multiply a duration DOES want is the frame delta's, and
/// [`Self::progress_per`] is the one place it happens: live animation advances a
/// unit progress by `dt / span`, and reading the span out of any other
/// expression is how a raw `f32` ends up in a pixel multiply. (A test converting
/// a span to seconds for its own fixture clock reads `.0` directly; that is
/// arithmetic over TIME, which is exactly what this type permits and the length
/// families do not.)
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Millis(pub f32);

impl Millis {
    /// The progress this span advances over a frame delta of `dt` SECONDS —
    /// the only arithmetic an animation duration is asked for.
    pub fn progress_per(self, dt: f32) -> f32 {
        dt * 1000.0 / self.0
    }
}

/// Zoom-derived layout metrics. This is the SINGLE SOURCE OF TRUTH for every
/// pixel dimension that depends on zoom: the renderer, the caret quad, the
/// selection rectangles, and mouse hit-testing all read these, so a click lands
/// exactly where the glyph is drawn at any zoom.
#[derive(Clone, Copy, Debug)]
pub struct Metrics {
    pub zoom: f32,
    /// The display's own device ratio. Stored beside `scale` rather than
    /// recovered from it, because `scale / zoom` is a second derivation of a
    /// number the constructor already held — and because it is the FLOOR a
    /// [`LogicalGrowOnly`] cap resolves against, which is a different question
    /// from the multiply and must not be answered by dividing the multiply back
    /// out.
    pub dpi: f32,
    /// `zoom * dpi` — the ONE factor every enrolled quantity is multiplied by,
    /// stored rather than re-derived so a consumer cannot invent a second one.
    pub scale: f32,
    pub font_size: f32,
    pub line_height: f32,
    pub char_width: f32,
    pub caret_w: f32,
    pub caret_h: f32,
    pub caret_block_h: f32,
    pub caret_streak_h: f32,
    pub caret_streak_min_len: f32,
    pub caret_streak_max_len: f32,
    pub caret_streak_vel_full: f32,
    pub caret_streak_gap: f32,
    pub caret_trail_drop: f32,
    pub caret_held_len: f32,
}

impl Metrics {
    pub fn new(zoom: f32) -> Self {
        Self::with_dpi(zoom, 1.0)
    }

    pub fn with_dpi(zoom: f32, dpi: f32) -> Self {
        let zoom = clamp_zoom(zoom);
        let s = zoom * dpi;
        Self {
            zoom,
            dpi,
            scale: s,
            font_size: FONT_SIZE * s,
            line_height: LINE_HEIGHT * s,
            char_width: CHAR_WIDTH * s,
            caret_w: CARET_W * s,
            caret_h: CARET_H * s,
            caret_block_h: CARET_BLOCK_H * s,
            caret_streak_h: CARET_STREAK_H * s,
            caret_streak_min_len: CARET_STREAK_MIN_LEN * s,
            caret_streak_max_len: CARET_STREAK_MAX_LEN * s,
            caret_streak_vel_full: CARET_STREAK_VEL_FULL * s,
            caret_streak_gap: crate::caret::CARET_STREAK_GAP * s,
            caret_trail_drop: CARET_TRAIL_TEXT_CENTER_DROP * s,
            caret_held_len: crate::caret::HELD_STREAK_LEN * s,
        }
    }

    fn glyph_metrics(&self) -> GlyphMetrics {
        GlyphMetrics::new(self.font_size, self.line_height)
    }

    /// THE LOGICAL→DEVICE BOUNDARY, for chrome as for everything else. The same
    /// `s` that produced [`Self::font_size`] and [`Self::line_height`], so an
    /// enrolled length holds its ratio to the text at every zoom and DPI.
    ///
    /// Glyphs still rasterize at device resolution: this is the LAYOUT side of
    /// the seam only, and nothing here changes the sizes handed to the shaper.
    pub fn px(&self, l: Logical) -> f32 {
        l.px(self.scale)
    }

    /// Resolve a [`LogicalGrowOnly`] cap: `scale.max(dpi)`, so the authored
    /// value is a LOGICAL floor the cap only ever widens away from as the user
    /// zooms in — never a device-pixel floor that a denser panel walks under.
    pub fn px_grow_only(&self, l: LogicalGrowOnly) -> f32 {
        l.px(self.scale, self.dpi)
    }

    /// Resolve a [`Physical`] length. The identity — it exists so the annotated
    /// exception still passes the owner and is greppable as a choice.
    pub fn px_physical(&self, p: Physical) -> f32 {
        p.0
    }

    /// Length (px) of the fully-in-motion trailing streak for a given horizontal
    /// `speed` (px/s). Grows linearly from `caret_streak_min_len` at speed 0 up to
    /// `caret_streak_max_len` once `speed` reaches `caret_streak_vel_full`, and is
    /// clamped to the [min, max] band beyond that. Pure function of the metrics +
    /// speed, so the velocity→length mapping is unit-testable without a GPU.
    pub fn streak_len_for_speed(&self, speed: f32) -> f32 {
        let t = (speed.abs() / self.caret_streak_vel_full).clamp(0.0, 1.0);
        self.caret_streak_min_len + (self.caret_streak_max_len - self.caret_streak_min_len) * t
    }
}

/// Bundled DEFAULT/mono UI font (IBM Plex Mono, OFL). Embedding it makes
/// rendering identical on every platform and removes any dependency on system
/// font matching — the generic-monospace fallback is what rendered hyphens as
/// long en-dashes. It is also Tawny's (awl's original "home" world) display
/// face and the registered monospace family (so any glyph the theme face lacks
/// falls back to it, and the panel / fallback paths resolve here via
/// `Family::Monospace`).
pub const FONT_DATA: &[u8] = include_bytes!("../assets/fonts/IBMPlexMono-Light.ttf");

pub const FONT_DATA_PITCH: facepitch::Pitch = facepitch::Pitch::Mono;

/// Bundled SYMBOL / ORNAMENT face (a hand-merged subset built from CLEAN OFL
/// sources — the previous face's DejaVu/Bitstream-Vera dependency, the app's only
/// non-OFL asset, is gone). Decomposed glyph outlines were copied from four SIL
/// OFL faces into one UPM-1000 base: the macOS modifier glyphs + core ornaments +
/// reference marks (⌃ § † ‡ • ◦ and the fleurons ❧ ❦ ☙) from EB Garamond; the
/// remaining modifier glyphs + fleurons (⌘ ⌥ ⇧ ▪ ❡ ❥) from Noto Sans Symbols 2;
/// the key-hint keycaps (↵ Return, ⇥ Tab) from Iosevka; and the asterism ⁂ from
/// Junicode — all UPM 1000, so the merged metrics align. It carries the glyphs
/// awl's prose+chrome want but the mono/proportional display faces lack: the macOS
/// modifier glyphs (⌘ ⇧ ⌥ ⌃), the key-hint keycaps (↵ ⇥ ⌫), the fine-press ornaments
/// / fleurons (❧ ❦ ☙ ❡ ❥), the asterism (⁂), and the reference marks (§ † ‡). It
/// is NOT a display face — it is registered under the private family
/// [`SYMBOL_FAMILY`] and only ever named via per-run `AttrsList` family spans
/// ([`spans::add_symbol_spans`]) over the specific symbol codepoints, so every
/// theme's display face is untouched while those glyphs render (instead of falling
/// back to TOFU) in every world. The same family also shapes the command-palette
/// glyph chords and the markdown rule/end ornaments. Its cmap is a superset of the
/// retired `AwlSymbols.ttf` (parity confirmed — identical 18 codepoints).
pub const FONT_SYMBOLS: &[u8] = include_bytes!("../assets/fonts/AwlMarks.ttf");

/// The private family name [`FONT_SYMBOLS`] registers under (its `name` table
/// family ID, verified through fontdb). Named only via `AttrsList` family spans —
/// never as a `Theme::font` — so it overlays symbol glyphs without becoming any
/// world's display face.
pub const SYMBOL_FAMILY: &str = "Awl Marks";

/// Every per-theme display face, embedded so a theme switch reskins the glyph
/// SHAPES with zero runtime font discovery. Each is loaded into the glyphon
/// `FontSystem` at startup (see [`TextPipeline::new`]); a theme selects its face
/// by the exact registered family name recorded in `Theme::font`, shaped via
/// `Family::Name`. The registered family names (verified through fontdb) are, in
/// order: "IBM Plex Mono" (already FONT_DATA, the default), "Literata",
/// "Newsreader 16pt 16pt" (the static Newsreader master registers under this
/// optical-size name), "IBM Plex Sans", "Zilla Slab", "JetBrains Mono"
/// (Mangrove), "Figtree" (Galah), "iA Writer Quattro S" (now unassigned),
/// "Monaspace Xenon" (Potoroo), "Fraunces 9pt"
/// (Saltpan), and "EB Garamond" (Bombora) — eleven distinct faces.
///
/// Literata/Newsreader/Plex Sans/Zilla/Fraunces/EB Garamond are PROPORTIONAL and
/// iA Writer Quattro S / Monaspace Xenon are (duo/mono)spaced; cosmic-text shapes
/// them all with real per-glyph advances and awl's caret / hit-test / selection
/// ride those real advances (see [`Self::line_glyph_xs`]), so switching the
/// document family is all that is needed to make each world render and track
/// correctly. Every face here is a static Regular/400 (Monaspace Xenon was
/// instanced from its variable master at `wght=400`), so no `mono_safe_weight`
/// exception is needed beyond IBM Plex Mono's Light.
///
/// EACH FACE DECLARES ITS PITCH. The second tuple field is the
/// face's [`facepitch::Pitch`] — the tuple type is the point: a new
/// `include_bytes!` here CANNOT COMPILE without a conscious Mono/Proportional
/// call, which is what the caret's mono/proportional fork used to get wrong by
/// omission (a hardcoded three-name list in `caret::font_is_mono` silently missed
/// Iosevka, so Currawong and Cassowary lost the uniform caret grid). The
/// declaration does not DRIVE the caret — [`facepitch`] measures each face's own
/// advance widths and the caret rides the measurement — it exists so a wrong or
/// missing call FAILS `render::tests::facepitch` instead of quietly changing how
/// the caret looks.
pub const FONT_THEME_FACES: &[(&[u8], facepitch::Pitch)] = &[
    (
        include_bytes!("../assets/fonts/Literata-Regular.ttf"),
        facepitch::Pitch::Proportional,
    ),
    (
        include_bytes!("../assets/fonts/Newsreader-Regular.ttf"),
        facepitch::Pitch::Proportional,
    ),
    (
        include_bytes!("../assets/fonts/IBMPlexSans-Regular.ttf"),
        facepitch::Pitch::Proportional,
    ),
    (
        include_bytes!("../assets/fonts/ZillaSlab-Regular.ttf"),
        facepitch::Pitch::Proportional,
    ),
    (
        include_bytes!("../assets/fonts/JetBrainsMono.ttf"),
        facepitch::Pitch::Mono,
    ),
    (
        include_bytes!("../assets/fonts/Figtree-Regular.ttf"),
        facepitch::Pitch::Proportional,
    ),
    (
        include_bytes!("../assets/fonts/iAWriterQuattroS-Regular.ttf"),
        facepitch::Pitch::Proportional,
    ),
    (
        include_bytes!("../assets/fonts/MonaspaceXenon-Regular.ttf"),
        facepitch::Pitch::Mono,
    ),
    (
        include_bytes!("../assets/fonts/Fraunces9pt-Regular.ttf"),
        facepitch::Pitch::Proportional,
    ),
    (
        include_bytes!("../assets/fonts/EBGaramond-Regular.ttf"),
        facepitch::Pitch::Proportional,
    ),
    (
        include_bytes!("../assets/fonts/FiraSans-Regular.ttf"),
        facepitch::Pitch::Proportional,
    ),
    (
        include_bytes!("../assets/fonts/Iosevka-Regular.ttf"),
        facepitch::Pitch::Mono,
    ),
    (
        include_bytes!("../assets/fonts/Bitter-Regular.ttf"),
        facepitch::Pitch::Proportional,
    ),
    (
        include_bytes!("../assets/fonts/SourGummy-Regular.ttf"),
        facepitch::Pitch::Proportional,
    ),
];

/// THE ONE ROSTER of bundled DISPLAY faces + their declared pitch: [`FONT_DATA`]
/// (loaded separately, as the `Family::Monospace` fallback) spliced in front of
/// [`FONT_THEME_FACES`]. Every consumer that wants "the faces a `Theme::font` can
/// name" reads this rather than remembering that the default face lives in its own
/// const — the exact shape of forgetting this round is fixing.
pub fn bundled_display_faces() -> impl Iterator<Item = (&'static [u8], facepitch::Pitch)> {
    std::iter::once((FONT_DATA, FONT_DATA_PITCH)).chain(FONT_THEME_FACES.iter().copied())
}

/// BUNDLED BOLD (700) display faces — the WYSIWYG-pivot bold round. awl's bundled
/// display faces were Regular-only, so `**bold**` (whose `MdKind::Bold` arm in
/// `render/spans.rs` requests `Weight::BOLD`) fell into cosmic-text's
/// `weight_diff == 0` fallback trap: with only the 400 Regular present,
/// `|400-700| = 300` drops it during fallback filtering and the request lands in
/// the ugly MONO fallback (bold-as-monospace). Registering a real 700 face under
/// the SAME family name each Regular uses gives `weight_diff == 0` for the BOLD
/// request, so it survives name-matching and resolves to the bold FILE — no new
/// family, no wiring beyond this list (the `MdKind::Bold` arm is unchanged).
///
/// EVERY bundled display face now gets a bold — the 10 PROPORTIONAL faces plus,
/// as of the mono-bolds round, the 4 MONOSPACE display faces (IBM Plex Mono,
/// JetBrains Mono, Monaspace Xenon, Iosevka). The monos were the last Regular-only
/// families, so a `**bold**` span in the five mono-display worlds (Tawny = Plex
/// Mono, Mangrove = JetBrains, Firetail/Potoroo = Monaspace Xenon, Currawong =
/// Iosevka) tripped the SAME trap and fell into a FOREIGN proportional sans (the
/// user's "weird fi-ligature" report) — worse than the proportional case. A real
/// 700 mono keeps the fixed grid (same advance) AND gives true emphasis. Each face
/// is sourced exactly like the bundled CJK faces: a static upstream Bold where one
/// ships (Fira Sans, IBM Plex Sans, Zilla Slab, iA Writer Quattro S, IBM Plex Mono,
/// Iosevka), else instanced from the OFL variable source at `wght=700`
/// (`fonttools varLib.instancer`, pinning the Regular's optical size — Literata
/// `opsz=12`, Newsreader `opsz=16`, Fraunces `opsz=9` — and, for Monaspace Xenon,
/// its width/slant axes to the Regular's `wdth=100 slnt=0`; JetBrains Mono has a
/// lone `wght` axis), then name-fixed so family(1) EXACTLY matches the Regular's
/// registered family and subset to that Regular's own code-point set. All OFL 1.1
/// (see `assets/fonts/LICENSES.md`).
///
/// IBM Plex Mono is the one weight-asymmetric pair: awl ships its Regular as the
/// Light/300 weight (`mono_safe_weight` — the documented Plex-Light trap), but its
/// Bold is the genuine upstream 700. The `MdKind::Bold` arm requests a plain
/// `Weight::BOLD` (700), NOT the mono-safe weight, so it resolves to this 700 file
/// with `weight_diff == 0` and a bold span visibly jumps Light→Bold. A code buffer
/// still requests `mono_safe_weight` (300) and matches the Light face exactly (the
/// 700 is farther, never wins the 300 request), so code shaping is untouched.
///
/// DOCUMENTED GAP: `Fraunces9pt-Bold.ttf` covers 624 of the Regular's 637
/// code-points — 13 rare transliteration/combining marks (Ṅ Ṡ Ṧ Ṩ Ẏ + combining
/// hook/ring-above, dot-below) are absent from the upstream Fraunces VARIABLE
/// source itself (the shipped Regular was built from a fuller source), so no
/// `wght=700` instance can carry them; a bold occurrence of one of those 13
/// characters falls back like any missing glyph. Every other bold (including all
/// four monos) matches its Regular's coverage exactly.
pub const FONT_THEME_BOLD_FACES: &[&[u8]] = &[
    include_bytes!("../assets/fonts/Literata-Bold.ttf"),
    include_bytes!("../assets/fonts/Newsreader-Bold.ttf"),
    include_bytes!("../assets/fonts/IBMPlexSans-Bold.ttf"),
    include_bytes!("../assets/fonts/ZillaSlab-Bold.ttf"),
    include_bytes!("../assets/fonts/Figtree-Bold.ttf"),
    include_bytes!("../assets/fonts/iAWriterQuattroS-Bold.ttf"),
    include_bytes!("../assets/fonts/Fraunces9pt-Bold.ttf"),
    include_bytes!("../assets/fonts/EBGaramond-Bold.ttf"),
    include_bytes!("../assets/fonts/FiraSans-Bold.ttf"),
    include_bytes!("../assets/fonts/Bitter-Bold.ttf"),
    include_bytes!("../assets/fonts/SourGummy-Bold.ttf"),
    // Mono display faces — the mono-bolds round. Same-family 700 companions so a
    // `**bold**` span in a mono-display world keeps its grid instead of falling
    // into a foreign proportional sans (see the module doc above).
    include_bytes!("../assets/fonts/IBMPlexMono-Bold.ttf"),
    include_bytes!("../assets/fonts/JetBrainsMono-Bold.ttf"),
    include_bytes!("../assets/fonts/MonaspaceXenon-Bold.ttf"),
    include_bytes!("../assets/fonts/Iosevka-Bold.ttf"),
];

/// BUNDLED ORNAMENT faces — tiny ornament-only subsets registered under their
/// authentic family names for honest attribution. Assigned per world via
/// [`crate::theme::Theme::ornament_face`] and named only through the per-run
/// `AttrsList` family span on the section-break fleuron / About end-mark (never a
/// `Theme::font`), so no world's display shaping is touched.
///  - Junicode ornaments (fleurons ☙ ❦ ❧, asterisms ⁂ ⁑, + Caslon PUA fleuron
///    clusters). SIL OFL, github.com/psb1558/Junicode-font. The antique/slab
///    worlds' ornament face ([`crate::theme::ORNAMENT_JUNICODE`]).
///
/// The other two ornament faces are registered ELSEWHERE, not here: EB Garamond
/// ([`crate::theme::ORNAMENT_GARAMOND`], the literary worlds' fleurons) is already
/// a display face in `FONT_THEME_FACES` (Bombora's), and the geometric worlds'
/// [`crate::theme::ORNAMENT_MARKS`] IS the merged `SYMBOL_FAMILY` face. (The dud
/// `Vollkorn-Ornaments.ttf` — it ships NO classic fleurons, only ¶ ‸ ‽ … — was
/// dropped: no world could use it for a section break.)
pub const FONT_ORNAMENT_FACES: &[&[u8]] =
    &[include_bytes!("../assets/fonts/Junicode-Ornaments.ttf")];

pub const FONT_CHROME_FACES: &[&[u8]] = &[
    include_bytes!("../assets/fonts/ArchivoBlack-Regular.ttf"),
    include_bytes!("../assets/fonts/AbrilFatface-Regular.ttf"),
];

/// BUNDLED HEAVY-WEIGHT CANDIDATE — Sour Gummy at `wght=900`
/// ("Black"), registered under the SAME family "Sour Gummy" as the Regular
/// ([`FONT_THEME_FACES`]) and the real Bold companion
/// ([`FONT_THEME_BOLD_FACES`]'s `SourGummy-Bold.ttf`, subfamily "Bold",
/// `usWeightClass 700`) — both a
/// 700-weight AND a 900-weight real instance (not relabelled weight
/// metadata) so a human taste pass can pick the heavy companion, rather than
/// silently deciding in code. Both files share the Regular's exact 335-glyph
/// coverage (see `assets/fonts/LICENSES.md`).
///
/// NORMAL operation (no env set): a plain `Weight::BOLD` (700) request
/// (`**bold**` / Quokka's `heading_bold`) resolves to the 700 file
/// (`weight_diff == 0` beats this 900 file's `weight_diff == 200` — the
/// SAME nearest-weight fallback rule every other bundled Bold companion
/// relies on) — this face stays bundled + addressable, never selected by
/// default. `AWL_SOURGUMMY_HEAVY_FORCE=900` (dev-only, mirrors
/// [`awl_cjk_force`]'s "total no-op unless set" contract — no config key, no
/// CLI flag) prunes the 700 file from the font DB after load
/// ([`apply_sourgummy_heavy_force`]), so the SAME `Weight::BOLD` request
/// falls through to THIS file instead — a true in-app A/B capture of the
/// heavy candidate, not a synthetic side-by-side image.
pub const FONT_SOURGUMMY_HEAVY_CANDIDATE: &[u8] =
    include_bytes!("../assets/fonts/SourGummy-Black.ttf");

/// BUNDLED per-script JAPANESE faces — the "Japanese bundle round" (TASTE-GATED,
/// see `theme::CJK_MINCHO`/`CJK_GOTHIC`): Noto Serif JP + Noto Sans JP, the
/// Google-Fonts JP-scoped builds (OFL, github.com/google/fonts, ofl/notoserifjp
/// + ofl/notosansjp), each instanced from the upstream variable font at wght=400
///   then subset to JIS X 0208 (levels 1+2 — kana + the ~6,355 Jōyō/JIS kanji +
///   JP punctuation, ~6,879 codepoints) via `fonttools`/`pyftsubset`. Subsetting
///   keeps the bundle honest with `PHILOSOPHY.md`'s "every MB earns its place":
///   unsubset the pair is ~7.7 MB + ~5.5 MB (~13.2 MB); the JIS subset is ~3.5 MB
/// + ~2.5 MB (~6.0 MB) — see CLAUDE.md's Japanese-bundle-round report for the
///   exact built-binary delta. Registered under their own family names ("Noto
///   Serif JP" / "Noto Sans JP", verified through fontdb) exactly like
///   `FONT_THEME_FACES`, but named ONLY via the CJK per-run `AttrsList` spans
///   (`spans::add_cjk_spans`) — never a `Theme::font` — so no world's Latin
///   display face is touched. `theme::CJK_MINCHO`/`CJK_GOTHIC` list these FIRST,
///   ahead of the system Hiragino/Noto-CJK candidates, so a Japanese run resolves
///   to the bundled face on every machine (no system-font dependency); the
///   Hiragino/system entries stay as trailing candidates until the user's
///   gallery/jp-compare eyeball-call — see the seam comment on those lists for
///   the follow-up (bundled-only + `resolve_cjk` simplification).
pub const FONT_CJK_FACES: &[&[u8]] = &[
    include_bytes!("../assets/fonts/NotoSerifJP-Regular.ttf"),
    // Noto Sans JP — gothic companion for the sans/mono worlds (registers as
    // "Noto Sans JP"). OFL, github.com/google/fonts/tree/main/ofl/notosansjp.
    include_bytes!("../assets/fonts/NotoSansJP-Regular.ttf"),
];

/// BUNDLED per-WORLD JAPANESE VARIETY faces — the "JP face variety" round
/// (Phase 2, TASTE-GATED). The user's note: "with kana we probably want a
/// couple more — they don't really change much across themes." Latin varies
/// per world; Japanese used to resolve to just Noto Serif JP (serif worlds) or
/// Noto Sans JP (sans/mono), barely varying. This adds THREE distinct-character
/// OFL faces from Google Fonts, matched to worlds by taste (see THEMES.md's
/// assignment table + `theme::CJK_JA_SHIPPORI`/`CJK_JA_ZENMARU`/`CJK_JA_KLEE`),
/// each a STATIC Regular (400) — no `varLib.instancer` step needed, unlike the
/// Noto pairs — subset to the SAME JIS X 0208 set as the shipped Noto faces
/// (`pyftsubset`, ~7,040 codepoints; verified to cover EVERY Kana + Han char
/// the shipped Noto pair does, so a run's per-glyph fallback never tofus — the
/// ~0–193 chars any of them lacks are all Greek/Cyrillic/symbols that
/// `script::classify_char` returns `None` for and so route to the base Latin
/// face, never the JP span):
///  - Shippori Mincho (github.com/fontdasu/ShipporiMincho, ofl/shipporimincho)
///    — a warm, bookish LITERARY mincho, distinct from Noto Serif JP's neutral
///    modern one. For the warm book-serif worlds ([`theme::CJK_JA_SHIPPORI`]:
///    Gumtree, Bilby, Bombora). ~3.5 MB (vs the unsubset ~8.7 MB static).
///  - Zen Maru Gothic (github.com/googlefonts/zen-marugothic, ofl/zenmarugothic)
///    — a rounded "maru" gothic, warmer than Noto Sans JP's even geometric
///    gothic. For the two dedicated sans worlds ([`theme::CJK_JA_ZENMARU`]:
///    Galah, Bowerbird). ~3.5 MB (vs ~3.8 MB static).
///  - Klee One (github.com/fontworks-fonts/Klee, ofl/kleeone) — a kaisho
///    TEXTBOOK face with gentle brush entry strokes, the CHARACTERFUL override
///    for the two Klee-derived worlds ([`theme::CJK_JA_KLEE`]: Mopoke, Quokka)
///    so their JA now shares the brush character of their ZH (LXGW WenKai, a
///    Klee One-derived Chinese face — the pairing the Chinese round's
///    `CJK_ZH_HANS_KLEE` doc anticipated). ~4.7 MB (vs ~8.7 MB static — a
///    brush face with denser outlines, the heaviest of the three).
///
/// Registered under their own family names ("Shippori Mincho" / "Zen Maru
/// Gothic" / "Klee One", verified through fontdb — see
/// `render::tests::cjk::ja_variety_faces_register_under_their_expected_family_names`)
/// exactly like `FONT_CJK_FACES`, and listed in [`theme::EMBEDDED_CJK_FAMILIES`]
/// (the "is this bundled" table) + [`CHARACTERFUL_CJK_FAMILIES`] (so the
/// `AWL_CJK_FORCE=floor` A/B knob prunes them down to the Noto floor in their
/// ladder for the before/after `gallery/jp-worlds/` captures). Named ONLY via
/// the per-run CJK `AttrsList` spans — never a `Theme::font` — so no world's
/// Latin display face is touched.
pub const FONT_JA_VARIETY_FACES: &[&[u8]] = &[
    include_bytes!("../assets/fonts/ShipporiMincho-Regular.ttf"),
    include_bytes!("../assets/fonts/ZenMaruGothic-Regular.ttf"),
    include_bytes!("../assets/fonts/KleeOne-Regular.ttf"),
];

/// BUNDLED per-script SIMPLIFIED-CHINESE + KOREAN faces — the "Chinese round"
/// (the user + his boyfriend's own font picks: 思源宋体/思源黑体, "Source Han",
/// is Adobe/Google's shared design for the Noto Serif/Sans SC family; 京华
/// 老宋体/KingHwa OldSong was INVESTIGATED and DECLINED — see the license note
/// below). Four faces, all Google-Fonts/community OFL builds, each instanced
/// from its upstream variable font at wght=400 (`fonttools varLib.instancer
/// --update-name-table … wght=400`, matching the JP round's exact recipe) then
/// subset via `fonttools`/`pyftsubset`:
///  - Noto Serif SC (github.com/google/fonts, ofl/notoserifsc) — the zh-Hans
///    MINCHO companion ([`theme::CJK_ZH_HANS_SERIF`]), subset to GB 2312
///    (levels 1+2, ~6,763 hanzi + CJK punctuation + fullwidth forms — 7,445
///    codepoints total, built programmatically from Python's `gb2312` codec
///    exactly the way the JIS X 0208 list was built for the JP round). ~3.37 MB
///    (vs the unsubset instance's ~14.9 MB).
///  - Noto Sans SC (ofl/notosanssc) — the zh-Hans GOTHIC companion
///    ([`theme::CJK_ZH_HANS_SANS`]), same GB 2312 subset. ~2.43 MB (vs ~10.6 MB).
///  - Noto Sans KR (ofl/notosanskr) — the Korean "rider" ([`theme::CJK_KO`]),
///    ONE face (no serif/sans split this round), subset to KS X 1001's 2,350
///    modern Hangul syllables (built from Python's `euc_kr` codec, filtered to
///    the Hangul Syllables block) + the Hangul Jamo/compat/extended-A/B blocks
///    (mirroring `script::classify_char`'s own Hangul ranges) + minimal CJK
///    punctuation/fullwidth forms. ~0.84 MB (vs ~6.2 MB unsubset) — smaller than
///    the ~1.5–2 MB estimate, since the subset skips Hanja entirely (Han runs
///    resolve through the zh/ja ladders, never `Theme::ko`).
///  - LXGW WenKai (霞鹜文楷, github.com/lxgw/LxgwWenKai) — a CHARACTERFUL
///    Klee One-derived Chinese face, layered ABOVE the Noto SC floor for the
///    two Klee-derived worlds ([`theme::CJK_ZH_HANS_KLEE`]: Mopoke, Quokka), so
///    ja and zh-Hans share the same brush character there. Same GB 2312 subset.
///    ~3.66 MB (vs the shipped static Regular's ~24.4 MB — LXGW ships static
///    weights, not a variable font, so no instancing step was needed).
///
/// **KingHwa OldSong (京华老宋体) — INVESTIGATED, DECLINED (no official OFL
/// repo, and its actual license explicitly forbids the pipeline this bundling
/// requires):** it is distributed only via WeChat/Zhihu announcements and
/// third-party Chinese font-aggregator mirror sites (shejidt.com, doany.cn,
/// fontke.com, …) — no canonical GitHub repo with a LICENSE file. Its stated
/// terms (a custom "free for commercial use within the declared scope"
/// license, quoted/logged in CLAUDE.md's Chinese-round report) explicitly
/// include "禁止修改字库或字库的任何部分" (modifying the font, in whole or
/// part, is forbidden) and "禁止对字库或字库的任何部分创作衍生作品" (no
/// derivative works) — subsetting a font IS a modification/derivative work,
/// so bundling a subset copy in this repo would violate its own stated terms
/// even before reaching the "is it actually OFL-equivalent" question. Per the
/// task's own instruction ("unclear → skip + log"), it is SKIPPED; the
/// "bookish serif worlds' ZhHans" pairing this round's spec proposed for it
/// has no candidate face in v1 (those worlds keep the plain [`theme::
/// CJK_ZH_HANS_SERIF`] Noto Serif SC floor, no characterful override).
pub const FONT_ZH_KO_FACES: &[&[u8]] = &[
    include_bytes!("../assets/fonts/NotoSerifSC-Regular.ttf"),
    include_bytes!("../assets/fonts/NotoSansSC-Regular.ttf"),
    include_bytes!("../assets/fonts/NotoSansKR-Regular.ttf"),
    include_bytes!("../assets/fonts/LXGWWenKai-Regular.ttf"),
];

/// BUNDLED per-script CJK COMPANION faces — the "CJK companions" round (the user
/// + his boyfriend's picks; the OFL pool for zh/ko outside the Noto floor is
///   thin, so this round adds the one worthwhile KO companion and DECLINES the
///   proposed ZH one). ONE face landed:
///  - Gowun Batang (github.com/yangheeryu/Gowun-Batang, Google Fonts) — a
///    genuinely lovely Korean BATANG (serif / 明朝-equivalent), OFL 1.1. It
///    closes the i18n/Chinese round's LOGGED v1 gap ("no comparable bundled
///    serif Korean companion yet"): the SERIF worlds' ko ladder
///    ([`theme::CJK_KO_SERIF`]: Gumtree, Bilby, Bombora, Saltpan, Mulga,
///    Magpie) now names Gowun Batang FIRST, above the neutral Noto Sans KR
///    floor, mirroring the ja serif/sans split (`CJK_JA_SHIPPORI` sits above the
///    Noto Serif JP floor) — sans/mono worlds keep the plain Noto Sans KR floor
///    ([`theme::CJK_KO`]). Ships as a STATIC Regular (400) — no `varLib.
///    instancer` step — subset (`pyftsubset`) to the SAME KS X 1001 code-point
///    set the bundled Noto Sans KR floor uses (2,563 code-points: ALL 2,350
///    modern Hangul syllables + ALL 94 compatibility jamo — the whole
///    modern-text set — plus the punctuation + conjoining jamo it carries).
///    ~1.43 MB (vs the unsubset static ~8.4 MB — a dense batang serif, so larger
///    per-glyph than the Noto Sans KR floor's ~0.84 MB, in line with Shippori
///    Mincho's own serif-JP ~3.5 MB). The ~357 archaic conjoining jamo
///    (U+1100–11FF / Jamo Ext-A/B) it lacks are Middle Korean only — modern
///    Korean is written entirely in precomposed syllables + compatibility jamo,
///    both FULLY covered — and any that appear fall back per-glyph to the
///    still-bundled Noto Sans KR floor (registered, full coverage): never tofu,
///    never machine-dependent.
///
/// **GenSenRounded (源泉圓體, github.com/ButTaiwan/gensen-font) — INVESTIGATED,
/// DECLINED (license is CLEAN, but there is no Simplified variant to serve the
/// intended zh-Hans goal):** the round proposed it as the ONE zh-Hans add — a
/// rounded/warm Source-Han-derived companion for the rounded worlds (Galah/
/// Bowerbird, whose ja is the rounded Zen Maru Gothic). Its license IS a proper
/// SIL OFL 1.1 (`SIL_Open_Font_License_1.1.txt` ships in the repo), so — unlike
/// KingHwa OldSong — this is NOT a license decline. But the repo (and every
/// release, v2.100 down) provides ONLY the TRADITIONAL-Chinese TW (月, Taiwan
/// common forms + HKSCS 2021) and TC (丹, print forms) variants plus JP/PJP
/// Japanese variants — there is **no Simplified (SC/CN) build at all**. A
/// Traditional font cannot serve the zh-HANS ladder: it renders Traditional-
/// convention glyph shapes for Simplified code-points (exactly the wrong-
/// regionalization problem THEMES.md's Han-unification note exists to avoid),
/// and lacks the Simplified-only forms outright. Per the round's own decision
/// rule ("if only TW exists, it belongs to the zh-Hant ladder instead — decide
/// by what the font actually provides"), a TW-only font is a Traditional face —
/// so it would belong to zh-Hant. But zh-Hant needs Big5-class coverage (~13k
/// chars), which this round AND the codebase EXPLICITLY BANK (see `CJK_ZH_HANT`);
/// and a single rounded Traditional floor imposed across every world would
/// break the per-world character-matching the design is built on (a serif world
/// wants a mincho-style Traditional face, not a rounded one), while a per-world
/// zh-Hant split is itself out of scope. So — mirroring the KingHwa OldSong
/// decline exactly ("unclear/wrong-fit → skip + log, don't force it") —
/// GenSenRounded is NOT bundled this round: the rounded worlds keep the plain
/// [`theme::CJK_ZH_HANS_SANS`] Noto Sans SC zh-Hans floor. Bundling it for a
/// FUTURE rounded-zh-Hant round (a Big5 subset + a per-world zh-Hant split) is
/// BANKED, not attempted here.
pub const FONT_CJK_COMPANION_FACES: &[&[u8]] =
    &[include_bytes!("../assets/fonts/GowunBatang-Regular.ttf")];
/// Thickness (LOGICAL px) of the underline drawn beneath an active IME
/// preedit (composition) string. The underline reuses the selection quad
/// pipeline (same translucent-rect look) but is a thin bar at the glyph baseline
/// rather than a full cell, so the composing text reads as distinct/provisional.
pub const PREEDIT_UNDERLINE_H: Logical = Logical(2.5);

pub const SPELL_AMP: Logical = Logical(2.72);
pub const SPELL_PERIOD: Logical = Logical(10.2);
pub const SPELL_THICKNESS: Logical = Logical(3.06);

pub const NIT_THICKNESS: Logical = Logical(1.3);

/// How far below the glyph cell the writing-nit's straight band hangs. The spell
/// squiggle's own gap is per-world THEME DATA
/// (`theme::RenderCaps::spell_underline_gap`) and this is deliberately NOT that
/// dial: the dial is scoped to the spell band, and routing the nit through it
/// would move one world's nits as a side effect of a unit repair.
pub const NIT_UNDERLINE_GAP: Logical = Logical(1.0);

/// The narrowest a decoration quad is drawn when the run it marks has collapsed
/// to nothing — the ONE owner for a floor three builders had each spelled as a
/// bare `2.0 * metrics.zoom`.
pub const DECOR_MIN_W: Logical = Logical(2.0);

/// WYSIWYG inline-code PILL inset (LOGICAL px): a minimal overhang beyond
/// the span's own glyph box so the value-step background reads as a small pill
/// rather than a bare selection-shaped rect. Taste default — flagged for live
/// review (`code_pill_pipeline` in `render.rs`, geometry in
/// `rects::code_pill_rects`).
pub const CODE_PILL_INSET_X: Logical = Logical(3.0);
pub const CODE_PILL_INSET_Y: Logical = Logical(1.0);

pub const FENCE_PANEL_INSET_X: Logical = Logical(8.0);

pub const TABLE_CELL_PAD_X: Logical = Logical(8.0);

pub const TABLE_COL_GAP: Logical = Logical(12.0);

pub const TABLE_RULE_THICKNESS: Logical = Logical(1.0);

pub const TABLE_PAN_BAR_THICKNESS: Logical = Logical(2.0);

/// COPY PULSE (the M-w/Cmd-C in-world confirmation — "obvious and understated"):
/// how much the selection quad's own tint LIFTS on a successful copy, expressed
/// as an HSL LIGHTNESS delta added to `theme::selection_document()`'s own lightness — same
/// hue, same saturation, never a new color (DESIGN §3 — amber stays the
/// caret's). TASTE TUNABLE, flagged for live review.
pub const COPY_PULSE_LIFT_L: f32 = 0.18;
/// The matching ALPHA lift (0..255 scale, added to `theme::selection_document()`'s own
/// alpha and clamped) — the pulse also nudges the wash a touch more opaque,
/// decaying alongside the lightness. TASTE TUNABLE.
pub const COPY_PULSE_LIFT_ALPHA: f32 = 55.0;
pub const COPY_PULSE_MS: Millis = Millis(220.0);

pub const OVERLAY_ENTRANCE_MS: Millis = Millis(200.0);
pub const OVERLAY_ENTRANCE_DROP_PX: Logical = Logical(14.0);
pub const OVERLAY_BAND_SLIDE_MS: Millis = Millis(110.0);

/// The copy-pulse's eased SETTLE fraction at progress `t` ∈ `[0, 1]` (0 = just
/// kicked / full brighten, 1 = fully settled / no boost) — a smoothstep ease,
/// mirroring [`crate::caret::CaretAnim::pop_scale`]'s own easing curve exactly.
/// Pure (no GPU/clock), so it is unit-testable directly: monotonic, `f(0) == 0`,
/// `f(1) == 1`, symmetric about `t = 0.5`. Out-of-range `t` clamps first.
pub(crate) fn copy_pulse_ease(t: f32) -> f32 {
    crate::ease::smoothstep(t)
}

/// The COPY-PULSE peak tint: the active theme's own `selection()` wash lifted
/// ONE brighten-step within its OWN hue + saturation family (never a new hue,
/// never amber) plus a touch more opacity — [`COPY_PULSE_LIFT_L`] /
/// [`COPY_PULSE_LIFT_ALPHA`]. Mirrors the free `*_srgba` theme-derivation helpers
/// above (`float_shadow_srgba`, `nit_underline_srgba`): reads the active theme,
/// so `new` + a live theme switch agree without extra bookkeeping. At `settle ==
/// 1.0` (settled/off) [`TextPipeline::prepare_selection_layer`] never reaches
/// this value at all — see [`selection::SelectionPipeline::prepare_pulsed`].
fn copy_pulse_peak_srgba() -> [u8; 4] {
    let base = theme::selection_document();
    let (h, s, l) = base.to_hsl();
    let lifted = theme::Srgb::from_hsl(h, s, (l + COPY_PULSE_LIFT_L).min(1.0));
    let a = (base.a as f32 + COPY_PULSE_LIFT_ALPHA).min(255.0) as u8;
    theme::Srgb::rgba(lifted.r, lifted.g, lifted.b, a).rgba_bytes()
}

pub const HELLO_TEXT: &str = "awl - hello";

/// One rendered GFM table's deterministic geometry, stashed by
/// [`TextPipeline::prepare_table_grid`] and surfaced in the capture `tables`
/// sidecar block — so a headless assertion can read the grid's shape (row/col
/// counts, measured column widths, reveal state) without eyeballing pixels.
/// `col_widths` are the laid-out (post-clamp) column box widths in px; `revealed`
/// is true when the caret is inside the table OR the active selection touches it
/// (grid stays drawn, each caret-or-selection-touched row's raw source floats
/// instead — see [`XrayRow`]).
#[derive(Clone, Debug)]
pub struct TableReport {
    pub range: (usize, usize),
    pub rows: usize,
    pub cols: usize,
    pub col_widths: Vec<f32>,
    pub revealed: bool,
}

/// THE X-RAY (the user's canonized metaphor: the caret is an x-ray into the
/// standing structure). When the caret sits on a GFM table ROW — or the active
/// selection touches one — the table's drawn GRID stays put (the source rows
/// stay concealed → the document NEVER reflows during a keyboard walk or a
/// selection drag) and that row's RAW SOURCE floats as ONE NON-WRAPPING line
/// over the dimmed grid cells; the CARET's own row additionally pans
/// horizontally to keep the caret column visible (the find-field single-line
/// pan model) — a row revealed only by selection has no caret to pan toward and
/// always floats at `pan = 0` (flush-left). `line` is this row's document line;
/// `glyph_xs` are the source glyphs' left-x's (`char_count + 1` entries, 0-based
/// from the row's left, the last = the line's end x) used BOTH to place the
/// float and — for the caret's OWN entry — to REDIRECT `col_x_and_advance` onto
/// the floated glyphs (the concealed doc row has zero-width advances, so the
/// caret must ride the float); `pan` is the clamped horizontal offset. Stashed
/// as a `Vec` (one entry per revealed row, across every table) by
/// [`TextPipeline::prepare_table_xray`] (before the caret layer, so the redirect
/// is ready) and consumed by the grid draw + the caret geometry. Empty whenever
/// no row is caret- or selection-revealed (every default capture, so the frame
/// stays byte-identical).
#[derive(Clone, Debug)]
pub(crate) struct XrayRow {
    pub line: usize,
    pub source: String,
    pub glyph_xs: Vec<f32>,
    pub top: f32,
    pub height: f32,
    pub pan: f32,
}

/// One inline IMAGE's deterministic layout, stashed by
/// [`TextPipeline::rebuild_image_rows`] and surfaced in the capture `images`
/// sidecar block (+ consumed by the next-phase GPU draw). Pure layout facts — the
/// source byte `range`, the logical `line` the ref sits on, the resolved `path`
/// (as written in the doc, relative or absolute), the parsed `width_hint`, the
/// fit-to-column `display_w`/`display_h` in px (the row's reserved height), and
/// `missing` (true when the file's header couldn't be read — a placeholder
/// height is reserved and the placeholder glyph is the next phase). `revealed`
/// is true when the caret is on the image's line — the source shows at body size
/// CENTRED OVER the still-drawn, DIMMED image (the caption model: the reserved ROW
/// stays exactly the image height, so nothing reflows on reveal).
#[derive(Clone, Debug)]
pub struct ImageReport {
    pub range: (usize, usize),
    pub line: usize,
    pub path: String,
    #[cfg(not(target_arch = "wasm32"))]
    pub alt: String,
    pub width_hint: Option<u32>,
    pub display_w: f32,
    pub display_h: f32,
    pub missing: bool,
    pub revealed: bool,
}

pub const OVERSCROLL_KEEP_ROWS: usize = 1;

/// The glyphon `Attrs` for the SUMMONED overlays / search panel / gutter —
/// the SAME active-world display family the DOCUMENT uses (see
/// [`TextPipeline::doc_attrs`]). This makes a serif/sans world render the command
/// palette, theme picker, go-to list, search field, and gutter label in that world's
/// FACE instead of always-mono, so the picker matches the page. Monospace stays the
/// GLYPH fallback automatically — it is the registered global fallback face under
/// `Shaping::Advanced`, so any glyph the theme face lacks (and the whole UI on a mono
/// world) still resolves to IBM Plex Mono. Ligatures are disabled to match the
/// document (1 char = 1 advance), keeping the panels' fixed-pitch caret/column math
/// honest. The panel buffers are re-shaped every frame, so a live theme switch picks
/// up the new family on the next `prepare` with no extra reshape bookkeeping.
/// RETIRED (dark-depth Option C, 2026-07-22) — was the FLOATING PANEL
/// PRIMITIVE's drop-shadow tone: the active world's INK (`base_content`) at a
/// low alpha. That is exactly the measured bug: `base_content` is near-WHITE
/// on a dark world, so the "shadow" quad BRIGHTENED the ground it sat on into
/// a pale slab (+0.12..0.25 luminance on Currawong's card) instead of
/// receding it. `render::chrome::set_float_quads` no longer uploads a shadow
/// quad for ANY [`chrome::FloatElevation`], on any world — the raised
/// border's own `surface_selected` value step + the card's `base_300` step
/// over `base_100` carry the depth instead (DESIGN §5: "a thin value step
/// does the work", not a cast shadow). This fn is consequently DEAD CODE in
/// practice — every one of its six call sites (`sync_theme_colors`) colors a
/// `_shadow` pipeline that `set_float_quads` now unconditionally parks at 0
/// instances — kept only because the `_shadow` `SelectionPipeline` fields
/// themselves aren't deleted this round (a further cleanup, logged, not
/// blocking). Left computing a real per-world tone rather than a bare
/// `[0, 0, 0, 0]` so a future full removal of the shadow plumbing has nothing
/// surprising to untangle.
fn float_shadow_srgba() -> [u8; 4] {
    if theme::active().render_caps.decorative_wash == theme::DecorativeWash::Off {
        return [0, 0, 0, 0];
    }
    let c = theme::base_content();
    theme::Srgb::rgba(c.r, c.g, c.b, 0x26).rgba_bytes()
}

fn nit_underline_srgba() -> [u8; 4] {
    if theme::active().render_caps.decorative_wash == theme::DecorativeWash::Off {
        return [0, 0, 0, 0];
    }
    let c = theme::muted();
    theme::Srgb::rgba(c.r, c.g, c.b, 0xC0).rgba_bytes()
}

/// Whether CODE-buffer PROGRAMMING ligatures (the arrow / `!=` / `=>` / `::`
/// glyphs the pitch-safe monos ship, riding `calt`) are active. DEFAULT ON — a
/// code buffer on JetBrains Mono / Iosevka renders its programming ligatures;
/// OFF renders code ligature-free (the pre-split behaviour). Read each reshape by
/// [`text::font_features`] (via `doc_attrs` / `panel_attrs`), set once at launch
/// from the config sticky pref (`config/`) and live by the settings menu. Gates
/// ONLY code — PROSE standard fi/fl ligatures are uncontroversial and always on
/// (see [`text::font_features`]).
/// This flag's fresh-install value — the ONE owner, read by the static below
/// and by the generated reference (`settings::toggle_default`).
pub(crate) const CODE_LIGATURES_DEFAULT: bool = true;
static CODE_LIGATURES_ON: crate::toggle::Toggle =
    crate::toggle::Toggle::new(CODE_LIGATURES_DEFAULT);

pub(crate) fn code_ligatures_on() -> bool {
    CODE_LIGATURES_ON.on()
}

pub(crate) fn set_code_ligatures_on(on: bool) {
    CODE_LIGATURES_ON.set(on);
}

fn panel_attrs() -> Attrs<'static> {
    // Route through the ONE font-feature owner (see [`text::font_features`]) so the
    // panels' ligatures can never drift from the document's. Panels shape the active
    // world's DISPLAY face (never a code buffer), so they take the PROSE set —
    // matching the document body, which now renders standard fi/fl too. On a mono
    // world the display face is IBM Plex Mono (no ligatures), so panels stay
    // fixed-pitch there exactly as before.
    let ff = text::font_features(false, theme::active().font, code_ligatures_on());
    Attrs::new()
        .family(Family::Name(theme::active().font))
        .weight(mono_safe_weight(theme::active().font))
        .font_features(ff)
}

fn chrome_attrs() -> Attrs<'static> {
    match effective_chrome_face() {
        theme::ChromeFace::Body => panel_attrs(),
        theme::ChromeFace::Named(family) => {
            let ff = text::font_features(false, family, code_ligatures_on());
            Attrs::new()
                .family(Family::Name(family))
                .weight(mono_safe_weight(family))
                .font_features(ff)
        }
    }
}

use plan::CornerAnchor;

/// The shaping WEIGHT to request for a world's display family. Almost every
/// bundled face is Regular (Weight 400), so the default is `Weight::NORMAL`. The
/// exception is IBM Plex Mono: the bundled `IBMPlexMono-Light.ttf` registers
/// (correctly) under the family name "IBM Plex Mono" but at Weight 300 (Light).
/// cosmic-text's fallback keeps only faces whose `font_weight_diff == 0` before
/// matching the family name, so a default-400 request DROPS the Light face,
/// abandons the requested family, and lands on macOS's PROPORTIONAL `.SF NS`
/// (i ~5px / m ~19px) — the mono worlds (Tawny, Potoroo) then render in a
/// proportional system font. Requesting Weight 300 makes `weight_diff == 0`, so
/// the bundled Plex face matches and the mono worlds shape in TRUE monospace
/// (uniform ~14.4px pitch). This is the same "match the real registered
/// metadata" pattern Bilby uses for Newsreader's optical-size family name.
fn mono_safe_weight(font: &str) -> glyphon::Weight {
    if font == "IBM Plex Mono" {
        glyphon::Weight(300) // Light — matches the bundled IBMPlexMono-Light face.
    } else {
        glyphon::Weight::NORMAL
    }
}

/// Family names of non-scalable / advance-breaking fallback faces to drop from
/// the font DB before shaping. These bitmap CJK faces (present in the macOS
/// system font set) return `inf` glyph advances under cosmic-text 0.18 + harfrust,
/// which breaks full-width CJK layout (every kanji forced onto its own line). With
/// them removed, fallback resolves CJK to a proper outline face. Match is
/// case-insensitive on the family name.
const BAD_FALLBACK_FAMILIES: &[&str] = &["GB18030 Bitmap"];

fn awl_font_override() -> &'static Option<std::path::PathBuf> {
    static ONCE: std::sync::OnceLock<Option<std::path::PathBuf>> = std::sync::OnceLock::new();
    ONCE.get_or_init(|| std::env::var_os("AWL_FONT").map(std::path::PathBuf::from))
}

/// Build the shaping font system: register the MONO/default UI face (AWL_FONT
/// override or bundled), every per-theme display face, then prune the bad
/// fallback faces — the one-time font setup behind [`TextPipeline::new`].
fn build_font_system() -> FontSystem {
    let mut font_system = FontSystem::new();
    // Choose the MONO/default UI font: AWL_FONT=/path/to/font.ttf overrides the
    // bundled default at runtime (handy for trying fonts). Whatever loads becomes
    // the monospace family, so the panel + the mono worlds (and any glyph a
    // proportional theme face lacks) resolve to it via Family::Monospace.
    let font_bytes: Vec<u8> = match awl_font_override() {
        Some(path) => crate::fs::active()
            .read(path.as_path())
            .unwrap_or_else(|e| {
                eprintln!("AWL_FONT {path:?}: {e}; falling back to bundled font");
                FONT_DATA.to_vec()
            }),
        None => FONT_DATA.to_vec(),
    };
    let face_ids =
        font_system
            .db_mut()
            .load_font_source(glyphon::cosmic_text::fontdb::Source::Binary(
                std::sync::Arc::new(font_bytes),
            ));
    if let Some(family) = face_ids
        .first()
        .and_then(|id| font_system.db().face(*id))
        .and_then(|f| f.families.first().map(|(name, _)| name.clone()))
    {
        font_system.db_mut().set_monospace_family(family);
    }

    // Load every per-theme display face so a live theme switch (or a headless
    // `--theme NAME` capture) can shape the document in that world's family via
    // `Family::Name` with no runtime font discovery. Each registers under the
    // exact family name recorded on its `Theme::font`; verified through fontdb
    // (see FONT_THEME_FACES). The mono default above stays the registered
    // monospace family, so it remains the fallback for any glyph a proportional
    // face is missing, and the panel/UI text keeps its mono look.
    for &(face_bytes, _pitch) in FONT_THEME_FACES {
        font_system
            .db_mut()
            .load_font_source(glyphon::cosmic_text::fontdb::Source::Binary(
                std::sync::Arc::new(face_bytes.to_vec()),
            ));
    }

    // DEV-ONLY (FIRETAIL-MAXIMALIST-SHOWCASE round): `AWL_CHROME_FACE_FILE`
    // registers UNCOMMITTED audition font files (colon-separated paths) so the
    // chrome-face gallery can shoot candidate faces that are deliberately NOT
    // in the tree (candidate files stay out of the repo until a flip round
    // bundles the winner — the board's own rule). Pairs with
    // `AWL_CHROME_FACE_FORCE=<family>` to select one. Total no-op unset; a
    // missing/unreadable file prints a note and is skipped (never a crash).
    if let Ok(paths) = std::env::var("AWL_CHROME_FACE_FILE") {
        for path in paths.split(':').filter(|p| !p.trim().is_empty()) {
            match std::fs::read(path.trim()) {
                Ok(bytes) => {
                    font_system.db_mut().load_font_source(
                        glyphon::cosmic_text::fontdb::Source::Binary(std::sync::Arc::new(bytes)),
                    );
                }
                Err(e) => eprintln!("AWL_CHROME_FACE_FILE {path:?}: {e}; skipped"),
            }
        }
    }

    // Register the bundled BOLD (700) display faces (see FONT_THEME_BOLD_FACES).
    // Each registers under the IDENTICAL family name its Regular uses, so a
    // `Weight::BOLD` request (the `**bold**` / `MdKind::Bold` arm) resolves to the
    // bold FILE instead of tripping cosmic-text's `weight_diff == 0` fallback trap
    // (which otherwise drops the Regular and lands in the mono fallback). No new
    // family and no other wiring — the bold arm is unchanged.
    for &face_bytes in FONT_THEME_BOLD_FACES {
        font_system
            .db_mut()
            .load_font_source(glyphon::cosmic_text::fontdb::Source::Binary(
                std::sync::Arc::new(face_bytes.to_vec()),
            ));
    }

    // Register the bundled JAPANESE faces (Noto Serif/Sans JP — see
    // FONT_CJK_FACES) so `resolve_cjk` finds "Noto Serif JP"/"Noto Sans JP" in
    // the font DB on every machine, with no dependency on a system CJK face.
    // Named only via per-run CJK `AttrsList` spans (never a `Theme::font`), so
    // this changes zero Latin display shaping.
    for &face_bytes in FONT_CJK_FACES {
        font_system
            .db_mut()
            .load_font_source(glyphon::cosmic_text::fontdb::Source::Binary(
                std::sync::Arc::new(face_bytes.to_vec()),
            ));
    }

    // Register the bundled per-WORLD JAPANESE VARIETY faces (Shippori Mincho,
    // Zen Maru Gothic, Klee One — see FONT_JA_VARIETY_FACES) so `resolve_font_id`
    // finds them for the worlds whose `Theme::cjk` ladder names them first, with
    // no dependency on a system CJK face. Named only via per-run CJK `AttrsList`
    // spans (never a `Theme::font`), so this changes zero Latin display shaping.
    for &face_bytes in FONT_JA_VARIETY_FACES {
        font_system
            .db_mut()
            .load_font_source(glyphon::cosmic_text::fontdb::Source::Binary(
                std::sync::Arc::new(face_bytes.to_vec()),
            ));
    }

    // Register the bundled ZH-HANS + KOREAN faces (Noto Serif/Sans SC, Noto
    // Sans KR, LXGW WenKai — see FONT_ZH_KO_FACES) so `resolve_font_id` finds
    // them in the font DB on every machine, with no dependency on a system
    // PingFang/Apple SD Gothic Neo/Noto-CJK face. Named only via per-run CJK
    // `AttrsList` spans (never a `Theme::font`), so this changes zero Latin
    // display shaping — mirrors the JP faces' registration exactly.
    for &face_bytes in FONT_ZH_KO_FACES {
        font_system
            .db_mut()
            .load_font_source(glyphon::cosmic_text::fontdb::Source::Binary(
                std::sync::Arc::new(face_bytes.to_vec()),
            ));
    }

    // Register the bundled CJK COMPANION faces (Gowun Batang — the serif worlds'
    // characterful Korean batang; see FONT_CJK_COMPANION_FACES) so `resolve_font_id`
    // finds it in the font DB on every machine, above the Noto Sans KR floor.
    // Named only via per-run CJK `AttrsList` spans (never a `Theme::font`), so
    // this changes zero Latin display shaping — mirrors the JP/ZH faces exactly.
    for &face_bytes in FONT_CJK_COMPANION_FACES {
        font_system
            .db_mut()
            .load_font_source(glyphon::cosmic_text::fontdb::Source::Binary(
                std::sync::Arc::new(face_bytes.to_vec()),
            ));
    }

    for &face_bytes in FONT_ORNAMENT_FACES {
        font_system
            .db_mut()
            .load_font_source(glyphon::cosmic_text::fontdb::Source::Binary(
                std::sync::Arc::new(face_bytes.to_vec()),
            ));
    }

    // Register the bundled CHROME-VOICE faces (Archivo Black, Abril Fatface — see
    // FONT_CHROME_FACES) so `chrome_attrs`'s `Family::Name` request resolves them
    // on every machine when a world's `render_caps.chrome_face` names one. Named
    // ONLY through the chrome span (placard wordmark / title prefix / lens-strip
    // label — never a `Theme::font`), so this changes zero document display
    // shaping — a world with `ChromeFace::Body` (all but Firetail) is untouched.
    for &face_bytes in FONT_CHROME_FACES {
        font_system
            .db_mut()
            .load_font_source(glyphon::cosmic_text::fontdb::Source::Binary(
                std::sync::Arc::new(face_bytes.to_vec()),
            ));
    }

    font_system
        .db_mut()
        .load_font_source(glyphon::cosmic_text::fontdb::Source::Binary(
            std::sync::Arc::new(FONT_SOURGUMMY_HEAVY_CANDIDATE.to_vec()),
        ));

    font_system
        .db_mut()
        .load_font_source(glyphon::cosmic_text::fontdb::Source::Binary(
            std::sync::Arc::new(FONT_SYMBOLS.to_vec()),
        ));

    // Drop non-scalable / advance-breaking fallback faces before any shaping.
    // On macOS the system font DB includes bitmap CJK faces (e.g. "GB18030
    // Bitmap") that cosmic-text's fallback may pick FIRST for kanji; their
    // glyph advances come back as `inf`, which forces every kanji onto its own
    // wrapped line and drops the visual layout. Removing them lets fallback
    // resolve kanji to a proper outline JP face (e.g. Hiragino / BIZ UDGothic),
    // so full-width CJK shapes inline with finite advances. Latin is untouched.
    prune_bad_fallback_faces(&mut font_system);
    apply_cjk_force(&mut font_system);
    apply_sourgummy_heavy_force(&mut font_system);
    font_system
}

use theme::EMBEDDED_CJK_FAMILIES as BUNDLED_CJK_FAMILIES;

const SYSTEM_CJK_FAMILIES: &[&str] = &[
    "Hiragino Mincho ProN",
    "Hiragino Kaku Gothic ProN",
    "Noto Serif CJK JP",
    "Noto Sans CJK JP",
    "PingFang SC",
    "PingFang TC",
    "Noto Sans CJK SC",
    "Noto Sans CJK TC",
    "Apple SD Gothic Neo",
    "Noto Sans CJK KR",
];

const CHARACTERFUL_CJK_FAMILIES: &[&str] = &[
    "LXGW WenKai",
    "Shippori Mincho",
    "Zen Maru Gothic",
    "Klee One",
    "Gowun Batang",
];

fn awl_cjk_force() -> &'static Option<String> {
    static ONCE: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    ONCE.get_or_init(|| std::env::var("AWL_CJK_FORCE").ok())
}

/// DEV-ONLY escape hatch for the Japanese-bundle-round + Chinese-round
/// TASTE-GATE captures (`gallery/jp-compare/`, `gallery/zh-worlds/`):
/// `AWL_CJK_FORCE=bundled` prunes the SYSTEM families from the font DB so
/// [`TextPipeline::resolve_font_id`] can only land on a bundled face;
/// `AWL_CJK_FORCE=system` prunes ALL bundled families instead, so resolution
/// falls through to whichever system CJK face is installed (Hiragino/PingFang/
/// Apple SD Gothic Neo on macOS); `AWL_CJK_FORCE=floor` prunes ONLY the
/// [`CHARACTERFUL_CJK_FAMILIES`] (LXGW WenKai / the JP-variety picks / Gowun
/// Batang), forcing every world that names a characterful override down to its
/// plain Noto floor (Klee worlds → Noto Sans SC zh-Hans; serif worlds → Noto
/// Sans KR ko; etc.) while leaving every other bundled floor face untouched.
/// Unset (the
/// default, every normal run) prunes nothing — every candidate stays
/// registered and each `Theme::candidates` ladder's priority order decides
/// (bundled/characterful first). This exists ONLY to produce the A/B(/C)
/// captures for the user's eyeball-call; it is not a product feature (no
/// config key, no CLI flag, undocumented in CAPTURE.md) and is a total no-op
/// unless the env var is set, so it changes nothing about normal/headless
/// determinism.
fn apply_cjk_force(font_system: &mut FontSystem) {
    let drop: &[&str] = match awl_cjk_force().as_deref() {
        Some("bundled") => SYSTEM_CJK_FAMILIES,
        Some("system") => BUNDLED_CJK_FAMILIES,
        Some("floor") => CHARACTERFUL_CJK_FAMILIES,
        _ => return,
    };
    let bad_ids: Vec<_> = font_system
        .db()
        .faces()
        .filter(|f| {
            f.families
                .iter()
                .any(|(name, _)| drop.iter().any(|d| name.eq_ignore_ascii_case(d)))
        })
        .map(|f| f.id)
        .collect();
    let db = font_system.db_mut();
    for id in bad_ids {
        db.remove_face(id);
    }
}

fn awl_sourgummy_heavy_force() -> &'static Option<String> {
    static ONCE: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    ONCE.get_or_init(|| std::env::var("AWL_SOURGUMMY_HEAVY_FORCE").ok())
}

/// DEV-ONLY escape hatch for the 700-vs-900 heavy-candidate gallery
/// (mirrors [`apply_cjk_force`]'s shape exactly): unset (every normal run,
/// every default capture) prunes nothing — `Weight::BOLD` resolves to the
/// 700 [`FONT_THEME_BOLD_FACES`] file by nearest-weight, and the bundled 900
/// [`FONT_SOURGUMMY_HEAVY_CANDIDATE`] just sits addressable-but-unselected.
/// `AWL_SOURGUMMY_HEAVY_FORCE=900` removes the 700 "Sour Gummy" face (by
/// family name + exact weight, so the Regular/400 face is untouched) from the
/// font DB, so the SAME `Weight::BOLD` request falls through to the 900 file
/// instead — a real in-app A/B, not a synthetic side-by-side. Not a product
/// feature (no config key, no CLI flag, undocumented in CAPTURE.md); a total
/// no-op unless the env var is set.
fn apply_sourgummy_heavy_force(font_system: &mut FontSystem) {
    if awl_sourgummy_heavy_force().as_deref() != Some("900") {
        return;
    }
    let bold_weight = glyphon::cosmic_text::fontdb::Weight(700);
    let bad_ids: Vec<_> = font_system
        .db()
        .faces()
        .filter(|f| {
            f.weight == bold_weight
                && f.families
                    .iter()
                    .any(|(name, _)| name.eq_ignore_ascii_case("Sour Gummy"))
        })
        .map(|f| f.id)
        .collect();
    let db = font_system.db_mut();
    for id in bad_ids {
        db.remove_face(id);
    }
}

fn parse_page_frame_force(s: &str) -> Option<theme::PageFrame> {
    let s = s.trim();
    if s.eq_ignore_ascii_case("none") {
        return Some(theme::PageFrame::None);
    }
    let w: f32 = s.parse().ok()?;
    if w > 0.0 && w.is_finite() {
        Some(theme::PageFrame::Line {
            weight_px: Logical(w),
        })
    } else {
        None
    }
}

fn awl_page_frame_force() -> &'static Option<theme::PageFrame> {
    static ONCE: std::sync::OnceLock<Option<theme::PageFrame>> = std::sync::OnceLock::new();
    ONCE.get_or_init(|| {
        std::env::var("AWL_PAGE_FRAME_FORCE")
            .ok()
            .and_then(|s| parse_page_frame_force(&s))
    })
}

pub(crate) fn effective_page_frame() -> theme::PageFrame {
    match awl_page_frame_force() {
        Some(frame) => *frame,
        None => theme::active().render_caps.page_frame,
    }
}

pub(crate) fn frost_seed_radius(row_h: f32, feather_px: f32, zoom: f32, dpi: f32) -> f32 {
    row_h * crate::lava::FROST_SEED_RADIUS_FRAC + crate::lava::frost_px(feather_px, zoom, dpi)
}

pub(crate) fn frost_run_radius(r_row: f32, run_ink_w: f32, skirt: f32) -> f32 {
    let ink_bound = run_ink_w * crate::lava::FROST_RUN_INK_RADIUS_FRAC + skirt;
    let end_cap = skirt * crate::lava::FROST_END_RADIUS_SKIRTS;
    r_row.min(ink_bound).min(end_cap)
}

pub(crate) fn push_text_seeds(
    seeds: &mut Vec<[f32; 4]>,
    left: f32,
    width: f32,
    yc: f32,
    r_row: f32,
    skirt: f32,
    label: &str,
) {
    let chars: Vec<char> = label.chars().collect();
    let n = chars.len();
    if n == 0 || width <= 0.0 {
        return;
    }
    let cw = width / n as f32;
    if crate::lava::FROST_SEED_PER_GLYPH {
        for (i, &c) in chars.iter().enumerate() {
            if c.is_whitespace() {
                continue; // a space seeds no halo — the ink's gaps stay open
            }
            let cx = left + (i as f32 + 0.5) * cw;
            let r = frost_run_radius(r_row, cw, skirt);
            seeds.push([cx, cx, yc, r]);
        }
    } else {
        let mut i = 0usize;
        while i < n {
            if chars[i].is_whitespace() {
                i += 1;
                continue;
            }
            let start = i;
            while i < n && !chars[i].is_whitespace() {
                i += 1;
            }
            let run_ink_w = (i - start) as f32 * cw;
            let r = frost_run_radius(r_row, run_ink_w, skirt);
            seeds.push([left + start as f32 * cw, left + i as f32 * cw, yc, r]);
        }
    }
}

pub(crate) fn effective_title_style() -> theme::TitleStyle {
    match overrides::current().title_style {
        Some(style) => style,
        None => theme::active().render_caps.title_style,
    }
}

pub(crate) fn derived_placard_corner(
    corner: theme::PlacardCorner,
    anchor: theme::CardAnchor,
) -> theme::PlacardCorner {
    use theme::{CardAnchor, PlacardCorner};
    if corner != PlacardCorner::Auto {
        return corner;
    }
    match anchor {
        CardAnchor::TopLeft => PlacardCorner::BR,
        CardAnchor::TopRight => PlacardCorner::BL,
        CardAnchor::TopCenter => PlacardCorner::BR,
        CardAnchor::Inset { x_frac } => {
            if x_frac >= 0.5 {
                PlacardCorner::BL
            } else {
                PlacardCorner::BR
            }
        }
    }
}

pub(crate) fn effective_card_anchor() -> theme::CardAnchor {
    match overrides::current().card_anchor {
        Some(anchor) => anchor,
        None => theme::active().render_caps.card_anchor,
    }
}

pub(crate) fn resolve_overlay_anchor(frozen: Option<theme::CardAnchor>) -> theme::CardAnchor {
    frozen.unwrap_or_else(effective_card_anchor)
}

fn parse_overlay_elevation_force(s: &str) -> Option<theme::Elevation> {
    match s.trim().to_ascii_lowercase().as_str() {
        "bordered" | "border" | "on" => Some(theme::Elevation::Bordered),
        "flat" | "off" => Some(theme::Elevation::Flat),
        _ => None,
    }
}

fn awl_overlay_elevation_force() -> &'static Option<theme::Elevation> {
    static ONCE: std::sync::OnceLock<Option<theme::Elevation>> = std::sync::OnceLock::new();
    ONCE.get_or_init(|| {
        std::env::var("AWL_OVERLAY_ELEVATION_FORCE")
            .ok()
            .and_then(|s| parse_overlay_elevation_force(&s))
    })
}

pub(crate) fn effective_card_elevation() -> theme::Elevation {
    #[cfg(test)]
    if let Some(elevation) = tests::potoroo_pane::elevation_override() {
        return elevation;
    }
    match awl_overlay_elevation_force() {
        Some(e) => *e,
        None => theme::active().render_caps.elevation,
    }
}

fn parse_overlay_selrow_force(s: &str) -> Option<bool> {
    match s.trim().to_ascii_lowercase().as_str() {
        "new" | "strong" | "on" => Some(true),
        "old" | "weak" | "off" => Some(false),
        _ => None,
    }
}

fn awl_overlay_selrow_force() -> &'static Option<bool> {
    static ONCE: std::sync::OnceLock<Option<bool>> = std::sync::OnceLock::new();
    ONCE.get_or_init(|| {
        std::env::var("AWL_OVERLAY_SELROW_FORCE")
            .ok()
            .and_then(|s| parse_overlay_selrow_force(&s))
    })
}

pub(crate) fn effective_overlay_selrow_band() -> theme::Srgb {
    match awl_overlay_selrow_force() {
        Some(false) => theme::surface_selected(),
        _ => theme::selection_ui(),
    }
}

pub(crate) fn effective_chrome_face() -> theme::ChromeFace {
    match overrides::current().chrome_face {
        Some(f) => f,
        None => theme::active().render_caps.chrome_face,
    }
}

pub(crate) fn effective_motion_juice() -> theme::MotionJuice {
    match overrides::current().motion_juice {
        Some(m) => m,
        None => theme::MotionJuice::CALM,
    }
}

pub(crate) fn overlay_slant() -> Option<SlantProbe> {
    overrides::current().slant
}

pub(crate) fn slant_offset(slant: &SlantProbe, row: usize) -> f32 {
    slant.px_per_row * row as f32
}

/// The deepest row's TOTAL inward step, as a magnitude — used to tax the width
/// elision budgets against, never as a signed position. The probe's
/// step is signed (which edge moves), but a mirrored (right-moving) stagger
/// eats just as much usable width as a straight (left-moving) one, so the tax
/// itself is always `>= 0`, `.abs()` of the signed step. Byte-identical to the
/// former formula for every existing (positive-only) probe value.
pub(crate) fn slant_max_offset(slant: &SlantProbe, n_rows: usize) -> f32 {
    slant.px_per_row.abs() * n_rows.saturating_sub(1) as f32
}

pub(crate) const BAR_OUTLINE_STROKE: Logical = Logical(1.5);

pub(crate) fn effective_list_style() -> theme::ListStyle {
    match overrides::current().list_style {
        Some(s) => s,
        None => theme::active().render_caps.list_style,
    }
}

/// The `Bars` layout dials — never a per-`Theme` value (`ListStyle::Bars`
/// carries none), so there is no world default to fall through to.
/// [`theme::BarConfig::SHIPPED`] is the one owner of what every `Bars` world
/// has ever rendered; a forced knob can still replace it for exploration.
pub(crate) fn effective_bar_config() -> theme::BarConfig {
    overrides::current()
        .bar_config
        .unwrap_or(theme::BarConfig::SHIPPED)
}

pub(crate) fn effective_facet_style() -> theme::FacetStyle {
    match overrides::current().facet_style {
        Some(s) => s,
        None => theme::active().render_caps.facet_style,
    }
}

pub(crate) fn effective_pane_split() -> theme::PaneSplit {
    match overrides::current().pane_split {
        Some(s) => s,
        None => theme::PaneSplit::Split,
    }
}

pub(crate) fn effective_overlay_density() -> TypeDensity {
    match overrides::current().density {
        Some(d) => d,
        None => TypeDensity::shipped(),
    }
}

pub(crate) fn effective_overlay_scale() -> f32 {
    effective_overlay_density().scale
}

pub(crate) fn effective_overlay_leading() -> f32 {
    effective_overlay_density().leading
}

pub(crate) fn overlay_motion_probe() -> Option<OverlayMotionProbe> {
    overrides::current().overlay_motion
}

/// Remove [`BAD_FALLBACK_FAMILIES`] from the font system's database so cosmic-text
/// never selects them during fallback. Safe no-op if none are present (e.g. on
/// non-macOS, or if the system set changes). Only affects fallback for glyphs the
/// bundled mono font lacks (CJK); Latin still resolves to the bundled monospace.
fn prune_bad_fallback_faces(font_system: &mut FontSystem) {
    let bad_ids: Vec<_> = font_system
        .db()
        .faces()
        .filter(|f| {
            f.families.iter().any(|(name, _)| {
                BAD_FALLBACK_FAMILIES
                    .iter()
                    .any(|bad| name.eq_ignore_ascii_case(bad))
            })
        })
        .map(|f| f.id)
        .collect();
    let db = font_system.db_mut();
    for id in bad_ids {
        db.remove_face(id);
    }
}

fn line_col_to_char_index(text: &str, line: usize, col: usize) -> usize {
    let mut cur_line = 0usize;
    let mut col_in_line = 0usize;
    let mut idx = 0usize;
    for c in text.chars() {
        if cur_line == line && col_in_line == col {
            return idx;
        }
        if c == '\n' {
            if cur_line == line {
                return idx;
            }
            cur_line += 1;
            col_in_line = 0;
        } else {
            col_in_line += 1;
        }
        idx += 1;
    }
    idx
}

fn lerp_srgb(a: theme::Srgb, b: theme::Srgb, t: f32) -> theme::Srgb {
    let t = t.clamp(0.0, 1.0);
    let mix = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    theme::Srgb::rgb(mix(a.r, b.r), mix(a.g, b.g), mix(a.b, b.b))
}

/// One visual row (wrapped sub-line) of a logical line. Built by
/// [`TextPipeline::visual_rows`]; carries the wrap-aware top y plus this row's
/// char/byte span and per-char x boundaries so overlays can land on the right
/// row both vertically (via `line_top`) and horizontally (via `xs`).
///
/// `Clone` so [`rowgeom::RowGeom`] can memoize the cursor line's rows across the
/// ~4 per-frame caret-geometry reads (see the single-slot row memo there); a hit
/// hands back a clone rather than re-walking every shaped run of the document.
struct VisualRow {
    line_top: f32,
    /// This row's HEIGHT in px (cosmic-text `run.line_height`). Uniform for body
    /// text, LARGER for a heading row. Caret / selection / squiggle centering use
    /// it so overlays grow with a heading instead of floating in a base-height cell.
    line_height: f32,
    start_col: usize,
    end_col: usize,
    xs: Vec<f32>,
}

impl Clone for VisualRow {
    fn clone(&self) -> Self {
        #[cfg(test)]
        rowgeom::note_visual_row_clone();
        Self {
            line_top: self.line_top,
            line_height: self.line_height,
            start_col: self.start_col,
            end_col: self.end_col,
            xs: self.xs.clone(),
        }
    }
}

/// One row in the exact shaped frame partition. Keeping the logical-line
/// identity beside the row lets reports walk draw order without reconstructing
/// a line-prefix table.
struct FrameVisualRow {
    logical_line: usize,
    row: VisualRow,
}

fn byte_col(text: &str, byte: usize) -> usize {
    if byte >= text.len() {
        return text.chars().count();
    }
    text.char_indices().take_while(|(b, _)| *b < byte).count()
}

mod scroll;
pub use scroll::ScrollPos;

mod shape_reach;
pub use shape_reach::{OFFSCREEN_CULL_MARGIN_ROWS, ShapeReach};
pub struct TextPipeline {
    pub font_system: FontSystem,
    pub swash_cache: SwashCache,
    pub viewport: Viewport,
    pub atlas: TextAtlas,
    pub renderer: TextRenderer,
    pub buffer: GlyphBuffer,
    document_active: bool,
    /// The GPU quad pipeline that draws the caret underline/dot (no glow/trail).
    /// This is the classic BLOCK caret; left untouched by the Morph work.
    pub caret_pipeline: CaretPipeline,
    /// The GPU quad pipeline that draws the COSMETIC | TRAIL: a fading accent streak
    /// from the OLD caret position to the NEW on a qualifying navigation move, layered
    /// OVER the instantly-snapped caret. Decoupled from position (the caret stays
    /// pinned to target); driven by the spring's `trail_*` state via `caret_geometry`'s
    /// sibling `caret_trail_geometry`. Same amber accent as the caret, drawn at a
    /// fading alpha. One extra instanced quad; empty when no streak is active.
    pub caret_trail_pipeline: CaretPipeline,
    /// The GPU pipeline that draws the MORPH caret: the INHABITED glyph's
    /// silhouette — the char BEFORE the insertion point, the letter just typed
    /// (see [`TextPipeline::caret_anchor_col`]) — filled SOLID in the accent
    /// (hard-dilated a touch fatter, no soft glow/halo), drawn OVER the text so it
    /// recolours the letter, cross-fading between glyphs as it glides. Only active
    /// in [`CaretMode::Morph`].
    pub caret_glyph_pipeline: CaretGlyphPipeline,
    /// Cached rasterized mask of the glyph the caret is ARRIVING at (the newly
    /// INHABITED glyph at the anchor column), keyed by its `CacheKey` so it is
    /// only re-rasterized when the glyph / font / zoom (hence the key) changes.
    caret_mask_to: Option<GlyphMask>,
    caret_mask_from: Option<GlyphMask>,
    caret_from_key: Option<CacheKey>,
    caret_look: CaretMode,
    pub background_pipeline: BackgroundPipeline,
    /// THE LAVA-LAMP GROUND ([`Background::Lava`]): a slow 2D metaball field
    /// painted MARGINS-ONLY, drawn right AFTER `background_pipeline` and BEFORE the
    /// washes/selection/text. INACTIVE (draws nothing) for every non-lava world,
    /// so all fifteen shipped worlds stay byte-identical. The animation PHASE is
    /// driven by the live App's slow ~10 fps tick (never `advance()`'s hot loop);
    /// [`Self::lava_phase`] holds it. See [`crate::lava`].
    pub lava_pipeline: crate::lava::LavaPipeline,
    lava_phase: f32,
    /// WARPED-GRID travel phase in seconds.
    warp_phase: f32,
    lava_field_viewport: [f32; 2],
    /// THE ORGANIC FROST SEED FIELD (proto-cache): the visible margin glyphs' halo
    /// seeds `[x0, x1, yc, r]` (the outline entries + the gutter), summed by the
    /// lava shader into one continuous frosted field. Rebuilt only when
    /// [`Self::frost_seed_key`] misses — warm steady frames reuse it (zero
    /// rebuilds); a margin-text / zoom / resize change rebuilds once. EMPTY in every
    /// non-frost frame. See [`Self::prepare_lava_layer`].
    frost_seeds: Vec<[f32; 4]>,
    /// The frost seed-field cache key: viewport, zoom×DPI, the column, the active
    /// face, and the drawn outline/gutter text (see [`Self::frost_seed_key`]).
    /// `None` clears the cache (a non-frost frame).
    frost_seed_key: Option<u64>,
    pub frost_seed_rebuilds: u64,
    // (frost seed count is exposed via `TextPipeline::frost_seed_count`.)
    /// TWINKLING STARS (`theme::AmbientStyle::Stars`, the TWINKLING-STARS
    /// round): tiny individually-phased breathing points in the page-mode
    /// MARGINS — Currawong's ambient differentiator. A reused
    /// `SelectionPipeline` (fully-rounded tiny quads via `set_corner`,
    /// per-star alpha via `prepare_multicolor` — the writing-streaks
    /// per-instance-color path; no new shader, nothing new for WebGL2).
    /// ZERO instances for every `AmbientStyle::None` world (fifteen of
    /// sixteen — byte-identical), and for page-off (no margins → no stars).
    /// The twinkle rides [`Self::lava_phase`] — ONE ambient clock, two
    /// consumers. See [`Self::prepare_stars_layer`] + [`crate::stars`].
    pub stars_pipeline: SelectionPipeline,
    /// The star-field PROTOS (the proto-cache shape): the scattered layout,
    /// built once per (viewport size, star params) by [`crate::stars::layout`]
    /// and re-culled/re-tinted per frame against the LIVE column geometry +
    /// twinkle phase. Rebuilt only when [`Self::stars_proto_key`] misses.
    stars_protos: Vec<crate::stars::Star>,
    /// The proto cache key: `(width, height, cell_px bits, density bits)` —
    /// a resize or a theme switch onto different star data rebuilds the
    /// layout; everything else is pure per-frame arithmetic over the protos.
    stars_proto_key: Option<(u32, u32, u32, u32)>,
    /// THE PAGE FRAME (`theme::PageFrame`, the personality-assignment round's
    /// graduated capability — subsumes the never-shipped `AWL_PAGE_BORDER`
    /// gallery probe): four thin quads framing the writing column over the
    /// document's vertical extent, drawn right after the lava ground and
    /// before the washes/text. Ink = `theme::page_frame_ink()` (the world's
    /// own `base_content`, ONE owner); weight = the capability's
    /// `weight_px`. Zero instances for every `PageFrame::None` world (all
    /// but Wagtail), so those stay byte-identical. Drawn HARD-EDGED via the
    /// shader's dither branch at density 1.0 (`bayer_threshold01` < 1.0 at
    /// every pixel — a full fill with a crisp per-pixel edge instead of the
    /// ordinary ~1px antialiased rim): the one shipping frame world is
    /// 1-bit, where a fractional-alpha edge would paint a forbidden grey
    /// line down the whole column. See [`Self::prepare_page_frame`].
    pub page_frame_pipeline: SelectionPipeline,
    /// SYNTAX WASHES: the low-alpha tinted quads drawn BEHIND prose-comment spans
    /// (all worlds) — the warm band that carries comment identity now that prose
    /// comments render at FULL ink (the tonsky inversion). A reused
    /// `SelectionPipeline` (the rule/ornament pattern) with a fixed per-world tint
    /// from [`role_style_for`], re-tinted in `sync_theme_colors` so the theme
    /// picker's O(1) preview recolors it for free. Geometry from the
    /// [`rects::WashCache`] protos; empty for prose / comment-less buffers
    /// (byte-identical).
    pub wash_comment_pipeline: SelectionPipeline,
    /// SYNTAX WASHES: the green band behind STRING spans on the DARK worlds
    /// (wash-first on dark; light worlds carry string identity in the fg tint and
    /// upload zero instances here). Sibling of `wash_comment_pipeline`.
    pub wash_string_pipeline: SelectionPipeline,
    /// MARKDOWN `==highlight==` WASH: the DEDICATED violet band behind every
    /// `MdKind::Highlight` span, DECOUPLED from the warm comment wash so a
    /// highlighter POPS ("look here") instead of reading as muddy warm cream on
    /// the cool pale light grounds. Its own [`highlight_wash`] tint (a deliberate,
    /// narrow break of the "one warm-wash owner" — a highlighter and a comment
    /// wash are different intents); another instance of the SAME
    /// `SelectionPipeline` shader (no new pipeline class), re-tinted in
    /// `sync_theme_colors`. Every world carries it (no opt-out); empty for prose /
    /// non-highlight buffers (byte-identical).
    pub wash_highlight_pipeline: SelectionPipeline,
    pub fence_panel_pipeline: SelectionPipeline,
    pub code_pill_pipeline: SelectionPipeline,
    /// The GPU quad pipeline that draws translucent selection highlights.
    pub selection_pipeline: SelectionPipeline,
    /// The GPU quad pipeline that draws translucent search-match highlights
    /// (same SELECTION color; the current match is shown by the amber caret).
    pub match_pipeline: SelectionPipeline,
    /// Selection's arbitrary two-colour palette-role swap. It is drawn after
    /// document text so the resolved ground and ink exchange roles. Idle when
    /// [`theme::SelectionStyle::Fill`] is active.
    pub selection_invert: SelectionPipeline,
    /// Block caret's independently authored two-colour palette-role swap,
    /// using the same pipeline mechanism as [`Self::selection_invert`] and
    /// drawn after text. It carries the BLOCK caret's own current
    /// ANIMATED rect (position + scale from the spring/juice geometry;
    /// rotation is dropped — `fs_two_colour` has no axis field, and the caret's
    /// diagonal travel streak is rare + still legible axis-aligned) instead
    /// of a selection range. Fixes the "white block over a white glyph
    /// erases the glyph" bug (a caret parked on a heading's `#` used to make
    /// the `#` vanish): drawing the block BEFORE text in the ordinary amber
    /// pipeline painted an opaque quad the SAME pure-white ink as the text,
    /// so the glyph on top composited into uniform white with no visible
    /// seam. `prepare_caret_block` routes the caret's rect here (and leaves
    /// `caret_pipeline` empty for that frame) ONLY on a one-bit world, so a
    /// non-Wagtail capture is byte-identical (`caret_invert` stays parked at
    /// zero instances everywhere else). MORPH degrades to this same path on
    /// a one-bit world (see `prepare_caret_layer`'s mode override) — a
    /// glyph-shaped invert mask would be real new pipeline work for a mode
    /// whose whole point (a colored accent letter) doesn't exist in a
    /// two-value world. Ibeam is UNCHANGED (its thin bar sits BETWEEN glyph
    /// cells, never over one, so it never needed inverting). KEEPS ITS
    /// ROUNDED SILHOUETTE: every `prepare_caret_block` call also uploads the
    /// frame's already zoom/settle/squash-animated corner radius via
    /// `SelectionPipeline::set_corner`, so `fs_two_colour`'s hard-discard SDF
    /// (`shaders/selection.wgsl`) still traces the same rounded shape
    /// `caret_pipeline` draws on an ordinary world — aliased at the corners
    /// (the role-swap silhouette is hard-edged), never a hard square.
    /// `selection_invert` never calls `set_corner` and so stays a plain
    /// rectangle, exactly right for a selection range.
    pub caret_invert: SelectionPipeline,
    pub ornament_renderer: TextRenderer,
    /// THE FOLD CHEVRON — `›`/`⌄`, two rotated-quad arms meeting at a vertex
    /// ([`crate::selection::chevron_arms`]), drawn OUTSIDE the glyphon
    /// text pipeline (glyphon 0.11 carries no transform, so a shaped run cannot
    /// rotate — `docs/render.md`'s "Rotated labels" section). Built + uploaded by
    /// `TextPipeline::prepare_fold_chevron_marks`, drawn in `draw_document_content`
    /// alongside `ornament_renderer` (the mark's other document-margin siblings —
    /// rule glyphs, bullets, the fold TAIL — stay glyphon; only the chevron, which
    /// must turn, left that pipeline).
    pub fold_chevron_pipeline: SelectionPipeline,
    pub table_renderer: TextRenderer,
    pub table_rule_pipeline: SelectionPipeline,
    pub panel_card: SelectionPipeline,
    pub panel_shadow: SelectionPipeline,
    pub panel_border: SelectionPipeline,
    pub blur: blur::BlurBackdrop,
    blur_recompute: bool,
    /// The signature the cached blur was built for (`None` = no cache). Compared in
    /// `prepare` against the live doc/size/theme signature to decide `blur_recompute`.
    blur_sig: Option<u64>,
    /// Second text renderer for the search panel text (composited OVER the
    /// document text). Shares this struct's atlas + viewport.
    pub panel_renderer: TextRenderer,
    /// DESIGNER PIXEL-PASS FIX (2026-07-16) — a DEDICATED renderer for the
    /// placard wordmark under [`theme::ListStyle::Bars`], so the watermark can be
    /// drawn UNDER the bar quads (`draw_overlay_card` runs it between the room
    /// veil and `overlay_bars`). The placard is glyphon text and the bars are
    /// quads in a separate pipeline, so the only way to get placard-BEHIND-bars
    /// is a distinct glyphon pass that renders before the bar quads: `panel_
    /// renderer` runs AFTER them (row text must sit on top), so it can never hold
    /// a behind-the-bars placard. Under `Pane` the placard stays first-in-batch
    /// in `panel_renderer` exactly as before (this renderer parks empty), so
    /// every non-Bars world is byte-identical. Shares the atlas + viewport.
    pub placard_renderer: TextRenderer,
    pub panel_buffer: GlyphBuffer,
    pub panel_bind_buffer: GlyphBuffer,
    pub placard_buffer: GlyphBuffer,
    pub panel_caret: CaretPipeline,
    pub caret_preview_pipeline: CaretPipeline,
    pub caret_preview_glyph_pipeline: CaretGlyphPipeline,
    pub float_shadow: SelectionPipeline,
    pub float_border: SelectionPipeline,
    pub float_card: SelectionPipeline,
    pub(in crate::render) float_panel_model: Option<chrome::FloatPanelModel>,
    pub preview_renderer: TextRenderer,
    pub preview_buffer: GlyphBuffer,
    /// The GPU quad pipeline that draws the wavy spell-check underlines.
    pub spell_pipeline: SpellUnderlinePipeline,
    /// The GPU quad pipeline that draws the STRAIGHT muted WRITING-NIT underlines.
    /// It reuses the spell squiggle pipeline (amplitude 0 → a flat line) tinted the
    /// muted neutral ink, so a nit reads as a calm hint distinct from the wavy
    /// error-red spell squiggle. Gated per-frame on [`crate::nits::nits_on`].
    pub nit_pipeline: SpellUnderlinePipeline,
    /// The GPU quad pipeline that draws the markdown `~~strikethrough~~` STRIKE
    /// LINES — the same flat-line trick as `nit_pipeline` (amplitude 0), tinted
    /// THE strike ink (`spans::strike_srgba_bytes`, the one owner the struck
    /// TEXT's muted transform and the popover's `S` demo share). Geometry from
    /// [`rects`]' strike bucket (`strike_lines`); empty for a strike-less buffer.
    pub strike_pipeline: SpellUnderlinePipeline,
    /// The GPU quad pipeline that draws the quiet markdown LINK UNDERLINE — the
    /// same flat-line trick as `strike_pipeline`, just a different vertical band
    /// (`spans::link_underline_band`) and its own instance (mirrors `nit_pipeline`
    /// / `strike_pipeline` / the popover's demo pipeline all sharing this ONE
    /// pipeline TYPE), tinted THE link-underline ink (`spans::
    /// link_underline_srgba_bytes`, the SAME muted rung the strike shares — the
    /// link TEXT itself stays full content ink, see `md_attrs`'s `LinkText` arm).
    /// Geometry from [`rects`]' link-underline bucket (`link_underlines`); empty
    /// for a link-less buffer.
    pub link_underline_pipeline: SpellUnderlinePipeline,
    pub caret: CaretAnim,
    cursor_line: usize,
    cursor_col: usize,
    /// The caret's wrap AFFINITY latched from the last `set_view` — the caret's own
    /// row/x placement reads it (via the `_aff` geometry seams) to disambiguate a
    /// shared soft-wrap boundary (see [`crate::caret::Affinity`]). `Downstream` for
    /// any caret not parked at a visual-row end, so ordinary placement is unchanged.
    caret_affinity: crate::caret::Affinity,
    scroll: ScrollPos,
    metrics: Metrics,
    dpi: f32,
    /// Last window/canvas WIDTH in physical pixels (from `set_size`). PAGE MODE
    /// centers the column within this, so the column left/width are derived from
    /// it rather than from the buffer's (column-derived) wrap width.
    window_w: f32,
    window_h: f32,
    /// Active selection endpoints (ordered), or `None`.
    selection: Option<((usize, usize), (usize, usize))>,
    fold_tails: Vec<FoldTail>,
    /// The FILTERED-line-space lines of every currently folded heading — a mirror
    /// of [`crate::render::ViewState::folded_headings`] (see its own doc for why
    /// this is not merely `fold_tails`' key set: an empty-section fold has no
    /// tail). The fold chevron's ONE source of direction: `›` (collapsed) while a
    /// heading's line is in here, `⌄` (expanded) otherwise.
    folded_headings: Vec<usize>,
    /// Per-heading fold-chevron TURN progress, keyed on the heading's CURRENT
    /// filtered line: `0.0` draws fully `›` (pointing right, collapsed), `1.0`
    /// draws fully `⌄` (pointing down, expanded) — see `fold_chevron::fold_chevron_turn_deg`.
    /// A heading with no entry yet (freshly scrolled into view, or seen for the
    /// first time this session) paints at its settled target directly with no
    /// glide — see the read-only accessor `fold_chevron_turn_fraction`, which
    /// `prepare_fold_chevron_marks` and every capture path use. Only
    /// [`Self::advance`]'s per-frame `step_fold_chevrons` mutates this map, so a
    /// headless capture (which never calls `advance`) always renders the settled
    /// state — the determinism law every other live-only animator in this file
    /// already honours (the caret spring, the copy pulse). Live-only: the
    /// quarter-turn GLIDE itself needs a real clock to witness.
    fold_chevron_turn: std::collections::HashMap<usize, f32>,
    hover_line: Option<usize>,
    preedit: String,
    misspelled: Vec<Misspelling>,
    /// Version counter for [`Self::misspelled`]: bumped by `sync_view_fields`
    /// whenever the incoming spell list actually DIFFERS from the mirrored one.
    /// Half of the squiggle proto cache's key (the other half is the row-geometry
    /// generation), so a spell rescan invalidates the cached squiggle geometry
    /// while every other event leaves it warm. See [`rects::UnderlineCache`].
    spell_gen: u64,
    /// The COMPOSED text (document + any spliced preedit) that is currently shaped
    /// into `buffer`. `set_view` reshapes ONLY when the newly-composed text or the
    /// zoom changes; a cursor move / scroll / selection / spell change leaves this
    /// untouched, so no reshape happens. `None` until the first shape. This is the
    /// key lever that makes every non-typing event free.
    shaped_key: Option<String>,
    /// The display FAMILY name the document buffer is currently shaped with (the
    /// active theme's `font` at the last shape). A live theme switch may change the
    /// world's font WITHOUT changing the text or zoom, which would otherwise leave
    /// the buffer shaped in the old face; [`Self::sync_theme`] compares against this
    /// and forces a whole-document reshape in the new family when it differs.
    shaped_font: &'static str,
    /// The theme INDEX ([`theme::active_index`]) whose palette the document's
    /// per-span text colors (syntax / markdown / focus) were last BAKED under.
    /// Those colors are frozen into the buffer `AttrsList` at shape time
    /// (`syn_attrs`/`md_attrs` call `role_style_for(&theme::active(), ..)`), so a
    /// theme switch that keeps the SAME effective face (e.g. Magpie -> Bombora, both
    /// Monaspace Xenon, on a code buffer) would leave those spans colored for the OLD
    /// world's light/dark ink derivation on the NEW ground. [`Self::sync_theme_font`]
    /// compares against this alongside `shaped_font` and re-bakes (`restyle_all_lines`)
    /// when EITHER differs — the font tracker alone can't see a same-face recolor.
    shaped_theme: usize,
    /// The cursor line the markdown rule/bullet CONCEAL was last refreshed for (see
    /// [`Self::refresh_rule_conceal`]). The reveal-on-cursor conceal toggles ONLY when
    /// the caret's LINE changes, so a pure scroll / same-line move / idle redraw can
    /// skip the O(lines × md_spans) rescan entirely by comparing against this. `None`
    /// forces the next refresh (the initial state, and after every reshape/edit, which
    /// pass `force`), so a stale cached value can never suppress a needed re-conceal.
    last_conceal_cursor_line: Option<usize>,
    /// The active SELECTION [`Self::refresh_rule_conceal`] was last refreshed for —
    /// the selection-reveal companion to `last_conceal_cursor_line` (2026-07-22,
    /// "selection reveals raw markdown"): a selection change can widen or shrink the
    /// touched-line set WITHOUT moving the caret's own line (e.g. a Shift-click that
    /// keeps the caret's line but moves the anchor, or a C-g that clears the mark),
    /// so the gate compares this TOO, never just the cursor line. `None` forces the
    /// next refresh (the initial state, and after every reshape/edit).
    last_conceal_selection: Option<((usize, usize), (usize, usize))>,
    /// VARIABLE-ROW-HEIGHT geometry cache + the lazily-cached total visual-row
    /// count, owned as a cohesive sub-struct (see [`rowgeom::RowGeom`]). With
    /// heading lines the visual rows are no longer a uniform `line_height` tall, so
    /// the scroll<->pixel conversion can no longer use `row_index * line_height`;
    /// `RowGeom` holds, per visual row in document order (as `layout_runs()` yields
    /// them — ascending `line_top`), the row's top y + height plus the document's
    /// total pixel height, built lazily from the shaped runs and invalidated whenever
    /// the buffer is reshaped or its metrics change. Counting rows walks every shaped
    /// run, so caching keeps the per-frame / per-keystroke `app.rs` reads free. The
    /// pipeline's `row_top_px` / `row_height_px` / `total_doc_height` /
    /// `total_visual_rows` delegate here.
    row_geom: rowgeom::RowGeom,
    /// TARGET-LINE-LOCAL caret glyph record — the cursor line's shaped
    /// glyph clusters `(start_byte, end_byte, CacheKey)`, read from that line's OWN
    /// `layout_opt()` rather than by filtering the whole document's `layout_runs()`.
    /// A SINGLE slot (the caret is only ever on one line), rebuilt when the cursor
    /// crosses to a new line or the shaped geometry changes (a newer `row_geom`
    /// generation). Shared by the block ink box / morph masks / descender / cluster
    /// span so their per-frame glyph lookups cost O(the cursor line's glyphs) rather
    /// than O(the whole prefix before the caret). Interior-mutable so the `&self`
    /// lookups (`cursor_glyph_key_at`) can lazily fill it. See [`caret::CaretLineGlyphs`].
    caret_line_glyphs: std::cell::RefCell<Option<caret::CaretLineGlyphs>>,
    ornament_cache: rects::OrnamentCache,
    table_report: std::cell::RefCell<Vec<TableReport>>,
    /// LIVE-ONLY horizontal table PAN (the reading gesture the user asked for after
    /// revising the no-scroll call): `(block start byte, pan offset px)` for the
    /// table currently being panned, or `None` (the default — every capture) when
    /// no table is panned. A too-wide grid grows into the margins and then pans;
    /// `prepare_table_grid` shifts the matching table's columns left by the offset,
    /// draws a thin bottom-edge indicator bar, and writes the CLAMPED offset back
    /// (so a stale value self-corrects when the grid narrows / a theme reshape
    /// changes widths). Fed by [`Self::try_table_pan`] on a horizontal wheel; NEVER
    /// set on the headless path, so a default `--screenshot` stays byte-identical.
    table_pan: Option<(usize, f32)>,
    /// THE X-RAY: every table ROW currently floating its raw source non-wrapping
    /// over the grid — the caret's OWN row (as before), PLUS every OTHER row the
    /// active selection touches (2026-07-22, "selection reveals raw markdown" — a
    /// selected table shows its raw `|` source, not just the caret's one row). See
    /// [`XrayRow`]. Filled by [`Self::prepare_table_xray`] BEFORE the caret layer
    /// (the caret's `col_x_and_advance` redirects onto the entry whose `line`
    /// matches), drawn by `prepare_table_grid` (one float per entry), and read by
    /// `caret_band_scale` (a table row sizes the caret to the SOURCE band, like an
    /// image line). Empty whenever no table row is caret- or selection-revealed —
    /// every default capture — so the frame stays byte-identical. `xray_report`
    /// (the sidecar) still surfaces only the CARET's own entry (schema-unchanged);
    /// the selection-only entries are render-only, verified by pixel/instance-count
    /// arithmetic rather than the sidecar (the state-vs-appearance tripwire).
    xray: Vec<XrayRow>,
    image_base_dir: Option<std::path::PathBuf>,
    /// Per LOGICAL LINE, the display HEIGHT (px) to RESERVE a tall row on that
    /// line, or `None` for an ordinary line. Two producers share this slot (a line
    /// is never both): an INLINE IMAGE's fit-to-column height (`compute_image_layout`
    /// from the `ConcealMarkup(Image)` md_spans + header dims) AND a WRAPPED GFM
    /// TABLE row's height (`compute_table_layout` — a too-wide table wraps its cells
    /// and grows the row). Read by [`build_line_attrs`] (all three call sites) to
    /// give the line a TALL row (normal font, tall line-height) via the same
    /// variable-row-height machinery headings use. Empty when neither feature
    /// applies (off / no images-or-tables / non-markdown) → byte-identical.
    image_heights: Vec<Option<f32>>,
    /// INLINE IMAGES: per LOGICAL LINE, `Some((dh, target_advance_px))` when
    /// that line is a MIXED image line (`- caption text ![alt](p)`) currently
    /// OFF-CURSOR — `None` for every other line (bare image lines, revealed mixed
    /// lines, non-image lines). Unlike `image_heights` this NEVER inflates the
    /// line's own shaped row (cosmic-text centers a row's content around its own
    /// glyph height unconditionally — inflating the CAPTION's row strands the
    /// marker from the caption); instead
    /// [`add_wysiwyg_conceal_spans`] gives the concealed image markup's SECOND
    /// byte (the `[` of `![alt](p)` — NEVER the leading `!`, see its doc comment
    /// for the UAX14 LB13 tripwire that rules the `!` out) a large `letter_spacing`
    /// (a pure position offset, never touching glyph rasterization — safe from
    /// atlas blow-up, unlike a huge font-size) sized to `target_advance_px`,
    /// forcing cosmic-text's own `Wrap::WordOrGlyph` engine to push it (and the
    /// rest of the concealed markup, which trivially fits alongside it) onto a
    /// GENUINE new visual row of THIS SAME logical line, with `dh` as that row's
    /// `line_height_opt`. Because this is real cosmic-text layout (not a side
    /// table), `RowGeom`/`hit_test`/`visual_rows` need no changes — they already
    /// read whatever cosmic-text actually laid out. `target_advance_px` is
    /// computed once per reshape by [`Self::measure_last_row_width`] (marker+
    /// caption's own LAST wrapped row width at the real wrap width, so a caption
    /// that already wraps on its own is handled too) plus a small safety margin,
    /// so the forcing glyph overflows the caption's row but still fits — with
    /// room for the near-zero-width remainder — on a fresh one.
    /// [`Self::image_draw_top`] reads this table (via [`Self::visual_rows`]'s
    /// LAST row) to place/hit-test the image quad directly below the caption,
    /// never at the row top. Empty when the feature is off / non-markdown / on
    /// wasm, matching `image_heights`.
    image_force: Vec<Option<(f32, f32)>>,
    /// INLINE IMAGES: the deterministic per-image layout the LAST
    /// [`Self::rebuild_image_rows`] produced — the source for the capture
    /// `images` sidecar block and the GPU draw. Interior-mutable so the reshape
    /// fills it and the read-only sidecar reads it back.
    image_report: std::cell::RefCell<Vec<ImageReport>>,
    image_preview: Option<(usize, usize, f32)>,
    image_preview_dirty: bool,
    /// INLINE IMAGES: the textured-quad pipeline that draws each visible, off-cursor
    /// image (one instanced quad per image) fit-to-column in its reserved tall row,
    /// after the washes + before selection. Empty (nothing drawn) when the feature
    /// is off / no visible images / on wasm, so a default capture is byte-identical.
    pub image_pipeline: crate::image_pipeline::ImageQuadPipeline,
    pub image_placeholder_pipeline: SelectionPipeline,
    pub image_scrim_pipeline: SelectionPipeline,
    pub image_placeholder_renderer: TextRenderer,
    /// INLINE IMAGES: the decode + GPU-upload cache (native-only), keyed by canonical
    /// path + mtime. Decodes O(visible) and downscales to the display width; pruned
    /// to the open doc's images each reshape ([`image_cache::ImageCache::retain_paths`]).
    #[cfg(not(target_arch = "wasm32"))]
    image_cache: image_cache::ImageCache,
    /// CACHED SPELL-SQUIGGLE PROTOS — the scroll-independent geometry of every
    /// misspelling's underline band, keyed on (row-geometry generation, spell list
    /// generation) so the per-frame squiggle pass is O(misspellings) arithmetic
    /// instead of a whole-doc `layout_runs()` walk PER misspelling PER frame (the
    /// measured 22 ms of a squiggle-dense doc's 28 ms frame). See
    /// [`rects::UnderlineCache`].
    squiggle_cache: rects::UnderlineCache,
    nit_cache: rects::UnderlineCache,
    /// CACHED SYNTAX-WASH PROTOS — the scroll-independent comment/string wash
    /// quads, keyed on (row-geometry generation, reshape count) exactly like the
    /// nit cache (the span lists re-lex per reshape). Cursor moves and scrolls
    /// keep it warm; the per-frame wash pass is O(visible) offset + cull. See
    /// [`rects::WashCache`].
    wash_cache: rects::WashCache,
    fence_panel_cache: rects::FencePanelCache,
    /// CACHED SHAPED TABLE-GRID GEOMETRY — the ONE shape site
    /// ([`layers::TableGridCache`]) both [`Self::compute_table_layout`] (which
    /// WRITES it at reshape time, the row-height reservation's own source) and
    /// [`layers::TextPipeline::prepare_table_grid`] (which only ever READS it, never
    /// reshapes) share, so a wrapped table's reserved document row and its drawn
    /// grid can never disagree — see the cache's own doc comment for the
    /// `sync_wrap_width`-without-a-full-reshape divergence this closes.
    table_grid_cache: layers::TableGridCache,
    /// TEST-ONLY: every table CELL's document line pushed as a `TextArea` by the
    /// LAST [`Self::prepare_table_grid`] call — exposes the "the caret's revealed
    /// row uploads zero grid cells" swap law at the purest reachable seam (a real
    /// draw call, not a GPU pixel diff). Cleared at the top of every
    /// `prepare_table_grid`, appended to alongside every cell `TextArea` push (both
    /// the revealed and the plain draw path). `cfg(test)` only — the release
    /// binary never carries this bookkeeping.
    #[cfg(test)]
    last_table_cell_lines: std::cell::RefCell<Vec<usize>>,
    /// Number of times the document text has actually been (re)shaped. A pure
    /// instrumentation counter (cursor-only / scroll-only / selection-only updates
    /// do NOT increment it); used by tests to prove non-typing events don't reshape.
    pub reshape_count: u64,
    /// `Some` while a [`ShapeReach::Presentable`] reshape owes an off-screen tail;
    /// the value is the last settled whole-document height. A preview burst keeps
    /// it stable while the live row table is intentionally truncated, so each
    /// superseding preview makes the same reach decision. The quiet settle and
    /// commit/revert paths clear it after paying only the latest world's debt.
    shape_tail_settled_height: Option<f32>,
    search_active: bool,
    search_matches: Vec<((usize, usize), (usize, usize))>,
    search_query: String,
    search_current: Option<usize>,
    search_case_sensitive: bool,
    search_replace_active: bool,
    search_replacement: String,
    search_editing_replacement: bool,
    search_query_caret: usize,
    search_replacement_caret: usize,
    /// The selected-ROW highlight quad behind the overlay's chosen candidate
    /// (same rounded SelectionPipeline primitive as match/selection). The band
    /// COLOR comes from the ONE `highlight_treatment` owner: the muted selection
    /// token on an ordinary (`Fill`) world so amber stays reserved for the
    /// caret, or solid `base_content` (white) on a true 1-bit world, where the
    /// the shaper (`selected_ink`) so the pair reads as crisp black-on-white.
    /// That solid-fill + recolor SUPERSEDED an earlier framebuffer invert of the
    /// row (retired), whose gamma-limited flip of the antialiased row text read
    /// as a faint grey — see [`theme::HighlightTreatment::InverseFill`].
    pub overlay_rows: SelectionPipeline,
    pub overlay_bars: SelectionPipeline,
    /// The `Bars` FOOTER PLATE's rim — that plate's own rect grown one pixel on
    /// every side and drawn under it, the same mechanism the calm notice's own
    /// rim uses for the identical failure mode: `overlay_bar_unselected`'s FILL
    /// alone measured ΔE 1.91 from Cassowary's own page, under the ≈2.3 JND.
    /// Colour resolved fresh every `overlay_prepare_selection` from
    /// `theme::overlay_footer_plate_rim`, so it carries no `sync_theme_colors`
    /// entry — same reasoning as `notice_rim`. Empty on every frame whose card
    /// draws no footer plate (Pane, Diagonal, Rules, or a `Bars` card with no
    /// footer row).
    pub footer_plate_rim: SelectionPipeline,
    pub overlay_spine: SelectionPipeline,
    pub overlay_spine_selected: SelectionPipeline,
    pub overlay_lens_underline: SelectionPipeline,
    /// V6 P5 round — the faceted strip's INACTIVE ghost pills under
    /// [`theme::FacetStyle::Chips`]: one hairline STROKE pill per non-active
    /// facet label (the active label rides `overlay_lens_underline` as a FILLED
    /// pill). Drawn via the selection pipeline's `stroke` uniform in the same
    /// under-the-text z-slot; parked empty for `Text`/`Band` and every non-theme
    /// card, so those render byte-identically.
    pub overlay_facet_ghost: SelectionPipeline,
    pub overlay_cross: SelectionPipeline,
    pub overlay_range_track: SelectionPipeline,
    pub overlay_range_thumb: SelectionPipeline,
    /// THE STIPPLE PLACARD (`theme::PlacardInk::Stipple`): the corner wordmark
    /// rendered as a Bayer-matrix stipple of individual full-ink pixels
    /// instead of ordinary antialiased glyphs. The SHAPING half is shared
    /// verbatim with the text placard (`overlay_shape_placard` — same buffer,
    /// same corner math, same reveal rules); this pipeline then draws the
    /// shaped glyphs' COVERAGE RUNS (CPU-rasterized off the same swash cache
    /// glyphon uses — see [`Self::placard_stipple_rects`]) through the
    /// selection shader's EXISTING dither branch at
    /// `theme::placard_stipple_density()` — the same matrix, the same
    /// mechanism, as Wagtail's highlight stipple (one pattern language, per
    /// the round's own rule). Ink = `theme::placard_ink(Stipple)` =
    /// `base_content` (the ladder's full-ink rung; the DENSITY carries the
    /// perceived Faint-tone quietness). Drawn in `draw_overlay_card` right
    /// before the overlay text (the same "behind the rows" slot the text
    /// placard's first-in-batch upload gives it); parked empty on every
    /// non-stipple world and whenever no overlay is up.
    pub placard_stipple: SelectionPipeline,
    /// THE ROTATED SECONDARY-LOCATION HEADING: a `RenderCaps::location_style
    /// == LocationStyle::RotatedRail(_)` world's active facet locator, turned
    /// 90° in the ROOM's own outer margin — the one its wordmark placard keeps.
    /// Its face, relative scale, palette ink, tracking and locator grammar are
    /// theme data. Reuses the
    /// world-neutral rotated-label capability
    /// wholesale — this pipeline draws nothing of its own shape, only a
    /// composed glyph mask rotated onto an axis. Parked (`clear()`) for
    /// every world that keeps the default `Inline` treatment, so those stay
    /// byte-identical.
    pub rotated_label_pipeline: crate::rotated_label::RotatedLabelPipeline,
    /// The rotated cue's own compose-once cache, keyed by
    /// [`crate::rotated_label::mask::LabelMask::matches`] against this
    /// frame's shaped run — so an unchanged facet name (the common case: the
    /// same lens held across many frames) re-uploads no texture.
    rotated_location_mask: Option<crate::rotated_label::mask::LabelMask>,
    overlay_theme_underline: Option<[f32; 4]>,
    /// V6 P5 round — the INACTIVE ghost-pill rects `[x, y, w, h]` recorded during
    /// theme-strip shaping under [`theme::FacetStyle::Chips`] (one per non-active
    /// facet label, from the SAME shaped glyphs the active pill reads, so the
    /// skin can't disagree with the hit-test). Consumed by `overlay_draw_card`
    /// into `overlay_facet_ghost`. EMPTY under `Text`/`Band` and off the theme
    /// picker, so they render byte-identically.
    overlay_theme_facet_ghosts: Vec<[f32; 4]>,
    overlay_strip_tab_plates: Vec<[f32; 4]>,
    overlay_right_shown: bool,
    diagonal_cluster: Option<chrome::diagonal::DiagonalClusterRail>,
    pub wordcount_renderer: TextRenderer,
    pub wordcount_buffer: GlyphBuffer,
    pub notice_renderer: TextRenderer,
    pub notice_buffer: GlyphBuffer,
    /// THE CALM NOTICE's PLATE — one value-stepped quad under the notice's own
    /// line, so a sentence of chrome seated inside the writing column reads as
    /// chrome instead of colliding with the prose it covers. Empty (nothing
    /// drawn) on every frame without a notice.
    ///
    /// Its colour is set in `prepare_notice` from the live theme every frame, and
    /// it therefore has NO entry in `sync_theme_colors` — a plane that depends on
    /// the notice's KIND cannot be carried by one baked seed, and resolving it at
    /// prepare time is what lets a headless capture (which never runs that sync)
    /// and a live world switch agree.
    pub notice_plate: SelectionPipeline,
    /// THE CALM NOTICE's RIM — the plate's rect grown one pixel on every side and
    /// drawn under it, so only a hairline shows. It carries the kind on the ink
    /// ladder (`muted` for a toast, `base_content` for a held sticky) and it is
    /// what makes the notice's boundary visible on a world whose surface ramp
    /// collapses; see `notice_plate_inks` for the measurements behind it.
    pub notice_rim: SelectionPipeline,
    pub page_drag_renderer: TextRenderer,
    pub page_drag_buffer: GlyphBuffer,
    pub zoom_readout_renderer: TextRenderer,
    pub zoom_readout_buffer: GlyphBuffer,
    pub debug_renderer: TextRenderer,
    pub debug_buffer: GlyphBuffer,
    pub gutter_renderer: TextRenderer,
    pub gutter_buffer: GlyphBuffer,
    pub outline_renderer: TextRenderer,
    pub outline_buffer: GlyphBuffer,
    /// WEB/LINUX MENU BAR (`menubar.rs` + `render/chrome/menubar.rs`): the slim
    /// awl-rendered strip of menu titles across the top of the canvas, shown when
    /// `crate::menubar::menu_bar_on()` (default on web/Linux, off on macOS — the
    /// native NSMenu bar is the door there). All parked off-screen / empty when the
    /// bar is off, so a default (macOS) capture stays byte-identical.
    ///   * `menubar_bg` — the bar's ground strip (a value step off the room, `base_200`).
    ///   * `menubar_hi` — the OPEN title's highlight band (never amber). Its band
    ///     COLOR comes from the ONE `highlight_treatment` owner: the muted
    ///     `selection_document` token on a `Fill` world — the DOCUMENT wash, NOT
    ///     the picker row's `selection_ui` step; the two tokens make that
    ///     difference visible — or solid `base_content` (white) on a TRUE 1-BIT
    ///     world, where the open title's own glyphs are recolored to solid
    ///     `base_300` (black) so black text lands crisp on the white band — the
    ///     SAME solid-fill + recolor answer the picker's selected row uses (see
    ///     [`theme::HighlightTreatment::InverseFill`]).
    ///   * `menubar_renderer`/`_buffer` — the title glyphs (LABEL size, faint / the
    ///     open one muted), laid out as ONE shaped line and read back for hit-testing.
    pub menubar_bg: SelectionPipeline,
    pub menubar_hi: SelectionPipeline,
    pub menubar_renderer: TextRenderer,
    pub menubar_buffer: GlyphBuffer,
    pub menu_drop_shadow: SelectionPipeline,
    pub menu_drop_border: SelectionPipeline,
    pub menu_drop_card: SelectionPipeline,
    pub menu_drop_sep: SelectionPipeline,
    pub menu_drop_renderer: TextRenderer,
    pub menu_drop_buffer: GlyphBuffer,
    pub menu_chord_renderer: TextRenderer,
    pub menu_chord_buffer: GlyphBuffer,
    /// MENU BAR hit-test geometry, recomputed every `prepare_menubar` from the SHAPED
    /// title glyphs + the open dropdown's layout, and read back by
    /// `menubar_title_at` / `menubar_item_at` (the click + cursor-shape hit-tests), so
    /// the drawn pixels and the hit-test can never drift. All empty / `None` when the
    /// bar is off or the dropdown is closed.
    pub menubar_boxes: Vec<crate::menubar::TitleBox>,
    pub menubar_bar_h: f32,
    pub menu_drop_rect: Option<[f32; 4]>,
    pub menu_drop_rows: Vec<crate::menubar::DropRow>,
    /// Which roster menu the stored `menu_drop_rect`/`menu_drop_rows` belong to, so a
    /// stale frame's geometry can't be attributed to the wrong menu. `None` closed.
    pub menu_drop_menu: Option<usize>,
    pub hud_shadow: SelectionPipeline,
    pub hud_border: SelectionPipeline,
    pub hud_card: SelectionPipeline,
    pub streak_cells: SelectionPipeline,
    pub hud_renderer: TextRenderer,
    pub hud_buffer: GlyphBuffer,
    hud: HudDefaults,
    streaks_view: Option<crate::streaks::StreaksView>,
    peek_rows: Vec<crate::peek::PeekRow>,
    keybindings_tips: Vec<String>,
    pub wk_shadow: SelectionPipeline,
    pub wk_border: SelectionPipeline,
    pub wk_card: SelectionPipeline,
    pub wk_renderer: TextRenderer,
    pub wk_buffer: GlyphBuffer,
    wk: WhichKeyDefaults,
    pub popover_wash: SelectionPipeline,
    pub popover_hl_wash: SelectionPipeline,
    pub popover_strike: SpellUnderlinePipeline,
    pub popover_renderer: TextRenderer,
    pub popover_buffer: GlyphBuffer,
    popover_model: Option<crate::popover::PopoverModel>,
    /// The popover's laid-out geometry (card rect + per-button pixel spans),
    /// computed in `prepare_popover` and read by the pure `&self` hit-test
    /// [`Self::popover_hit`] + the sidecar — the SAME geometry the buttons draw
    /// from, so a click can never disagree with where a button is painted. `None`
    /// when the popover is down.
    popover_geom: Option<crate::render::chrome::PopoverGeom>,
    notice: String,
    /// The notice text `prepare_notice` last SHAPED — the sentence after any
    /// elision to the column's budget, i.e. exactly what the PNG shows. The
    /// sidecar reports this rather than the intended `notice` above, on the same
    /// "as drawn" convention the page-mode gutter's own block already keeps: a
    /// block that reported the pre-elision sentence would be an artifact stating
    /// something its own pixels do not.
    notice_drawn: String,
    /// The kind of the notice in `notice` — a lifetime, not a severity (see
    /// [`crate::actions::NoticeKind`]). Read by the notice chrome so a HELD
    /// notice (one the writer must act on) can be treated differently from a
    /// self-clearing acknowledgement without either one growing its own path.
    notice_kind: crate::actions::NoticeKind,
    /// MOTION-JUICE ARMING (the FIRETAIL-MAXIMALIST-SHOWCASE round's
    /// determinism gate): `false` by default and in EVERY headless capture /
    /// bench / test pipeline — only the live App's GPU init calls
    /// [`Self::arm_live_juice`]. Every motion-juice kick checks this first,
    /// so the capture path is STRUCTURALLY animation-free (the settled state
    /// is the only state it can ever render), regardless of the dev-only
    /// motion override.
    juice_live: bool,
    overlay_enter_t: f32,
    /// Selection-BAND slide state: the row-top the band is easing FROM and
    /// the ease progress (`1.0` = settled on target). `band_last` memoizes
    /// the last TARGET row-top so a selection move is detected at the draw
    /// seam ([`Self::overlay_band_drawn`]); `None` when no overlay is open.
    overlay_band_from: f32,
    overlay_band_t: f32,
    overlay_band_last: Option<f32>,
    /// Live theme-picker timing. The movement epoch is stamped before the
    /// synchronous preview reshape, then sampled at the redraw's injected
    /// `now`; headless pipelines never stamp it and therefore never read a
    /// clock. `pending_from` is the old pose sampled at input time, before a
    /// rapid retarget applies the latest-selection-wins snap policy.
    overlay_band_started_at: Option<crate::clock::Instant>,
    overlay_band_frame_now: Option<crate::clock::Instant>,
    overlay_band_pending_at: Option<crate::clock::Instant>,
    overlay_band_pending_from: f32,
    overlay_band_pending_snap: bool,
    page_drag_readout: Option<(f32, f32, usize)>,
    zoom_readout: Option<(f32, f32, f32)>,
    debug: DebugDefaults,
    debug_still: bool,
    /// Latest queried GPU memory (bytes) the live loop feeds in for the debug panel's
    /// `gpu <n> MB` line, or `None` when there is no query (non-macOS backend, or the
    /// clockless headless capture) — both render the fixed `gpu —` placeholder.
    overlay_active: bool,
    overlay_align: Option<theme::CardAnchor>,
    overlay_crisp: bool,
    overlay_query: String,
    overlay_query_caret: usize,
    overlay_title: &'static str,
    overlay_row_path_splits: bool,
    overlay_items: Vec<String>,
    overlay_empty: Option<String>,
    overlay_bindings: Vec<String>,
    /// Mirror of [`ViewState::overlay_ranges`]: the per-row RAIL FRACTION
    /// (parallel to `overlay_items`), `None` for a row with no rail and EMPTY for
    /// every non-Settings card. Read by the ONE rail owner
    /// (`chrome::TextPipeline::overlay_rails`), which both the draw path and the
    /// pointer hit-test go through.
    overlay_ranges: Vec<Option<f32>>,
    overlay_times: Vec<String>,
    overlay_git: Vec<String>,
    overlay_selected: usize,
    overlay_scroll: usize,
    overlay_window_rows: usize,
    overlay_hint: String,
    overlay_lens: Vec<(String, bool)>,
    overlay_sections: Vec<String>,
    /// Mirror of [`ViewState::overlay_location`] — the summoned
    /// picker's SECONDARY location, `None` at the All home. The display plan
    /// consumes it; no render path re-derives it from `overlay_lens`.
    overlay_location: Option<String>,
    overlay_spell: Option<(usize, usize, usize)>,
    overlay_context_anchor: Option<(f32, f32)>,
    overlay_detail_focus: bool,
    /// Whether the summoned card is drawn as a workspace (mirror of
    /// [`ViewState::overlay_workspace`]). The one input that routes
    /// `overlay_geometry` to its third family.
    overlay_workspace: bool,
    /// Mirror of [`ViewState::overlay_rows_primary`] — within a
    /// workspace, does the primary column carry rows rather than labels?
    /// `false` for Settings; `true` for the History timeline.
    overlay_rows_primary: bool,
    /// Mirror of [`ViewState::overlay_comparison`] — the second half of the
    /// relocation gate: the shape says there IS a comparison region, this says
    /// there is something in it.
    overlay_comparison: bool,
    /// Workspace PRIMARY width (device px): category labels or timeline versions,
    /// measured at `set_view` like `overlay_content_w`. `0.0` off a workspace.
    workspace_primary_w: f32,
    workspace_rail_buffer: GlyphBuffer,
    /// Footer-fitting scratch, separate from both final rendered columns.
    workspace_hint_measure_buffer: GlyphBuffer,
    /// EVERY rail entry's rect for this frame, tagged with whether it is the
    /// ACTIVE one — recorded by the rail shaper and consumed by the shared
    /// facet-mark owner. Empty when no rail is drawn, so the marks park with the
    /// rail. The whole list rather than the active entry alone because a rail IS
    /// a list, and a composition that arranges rows by the boundaries between
    /// them needs every neighbour to know where those boundaries fall.
    workspace_rail_rows: Vec<([f32; 4], bool)>,
    /// Where the shaped rail buffer is placed (`(left, top)`), or `None` when no
    /// rail is drawn this frame.
    workspace_rail_placement: Option<(f32, f32)>,
    overlay_spell_w: f32,
    overlay_content_w: f32,
    /// PROTO-CACHE for the roster-width measurements, one slot per question.
    roster_memo: [Option<(u64, f32)>; chrome::roster::ROSTER_SLOTS],
    caret_preview: Option<CaretMode>,
    caret_demo: crate::caret::CaretDemo,
    caret_preview_mask_to: Option<GlyphMask>,
    caret_preview_mask_from: Option<GlyphMask>,
    caret_preview_from_key: Option<CacheKey>,
    gutter_name: String,
    gutter_project: String,
    gutter_changed: bool,
    /// The margin working set's rows, or empty for the single-file margin — see
    /// [`ViewState::gutter_files`].
    gutter_files: Vec<crate::workingset::StackRow>,
    /// The working-set row/zone under the live pointer. `None` on every
    /// headless frame (no pointer driver) and visually inert unless the explicit
    /// affordance prototype environment switch is armed.
    gutter_stack_hover: Option<chrome::GutterStackHit>,
    /// The soft row plate under the working set's ACTIVE row. Holds no instances
    /// at all unless the stack is drawn, so a single-file frame issues no extra
    /// draw (`SelectionPipeline::draw` returns early at zero instances).
    gutter_stack_plate: crate::selection::SelectionPipeline,
    /// Mirror of [`ViewState::config_keys`] — the user's `[keys]` overrides, the
    /// SAME slice `overlay::BuildCtx::config_keys` hands the palette. Read by the
    /// awl-drawn menu bar's chord column (`chrome::menubar::dropdown`) so a
    /// rebind updates that label exactly like it already updates the palette's.
    config_keys: Vec<(String, Vec<String>)>,
    /// Mirror of [`ViewState::config_linux_keep`] — `Config::effective_linux_keep()`,
    /// composed once per sync from the `keymap` flavor + `linux_keep_emacs`. Empty
    /// (never `linux_keeps_chord`-true) on every non-Linux capture, so this is
    /// inert everywhere the menu bar's own convention isn't `Convention::Linux`.
    config_linux_keep: Vec<String>,
    md_enabled: bool,
    /// WYSIWYG / INLINE-IMAGES LATCH: the last-shaped value of the two rendering
    /// process-globals (`markdown::wysiwyg_on()` / `inline_images_on()`), so
    /// [`Self::set_view`] can force a full restyle when either FLIPS on UNCHANGED
    /// text — exactly like the `md_enabled` / `syn_lang` gates beside it. The
    /// conceal geometry (zero-width metrics) and image row heights are baked into
    /// each line's attrs at shape time, so a settings-menu toggle with no text edit
    /// would otherwise leave them stale until the next edit; this is the live-apply
    /// path that gap needed. A no-op on every ordinary frame (the value is unchanged).
    wysiwyg_latched: bool,
    inline_images_latched: bool,
    md_spans: Vec<(std::ops::Range<usize>, crate::markdown::MdKind)>,
    outline_headings: Vec<crate::markdown::Heading>,
    last_outline_current: Option<usize>,
    syn_lang: Option<crate::syntax::Lang>,
    syn_spans: Vec<(std::ops::Range<usize>, crate::syntax::SynKind)>,
    doc_lang: Option<crate::frontmatter::Lang>,
    script_fonts: text::ScriptFonts,
    /// Mirrored from [`ViewState::doc_source`]; read only by `figure_source`.
    doc_source: Option<DocSource>,
    cjk_priority: Vec<crate::frontmatter::Lang>,
    eol: crate::buffer::Eol,
    /// COPY PULSE: progress of the selection-tint brighten/decay pulse played on a
    /// successful M-w/Cmd-C copy — `1.0` = settled/off (no boost, the selection
    /// quad draws its plain theme tint), `0.0` = just kicked (full brighten).
    /// Eases back to `1.0` over [`COPY_PULSE_MS`] on the LIVE clock via
    /// [`Self::step_copy_pulse`], OR-folded into [`Self::advance`]. Starts (and
    /// idles) at `1.0`, so a default headless capture never carries a boost — the
    /// field is only ever written by [`Self::copy_pulse`], which nothing in the
    /// headless `--keys` replay path calls (see `main/run.rs`'s `Effect::CopyPulse`
    /// no-op arm).
    copy_pulse_t: f32,
}

#[derive(Default)]
struct HudDefaults {
    stats: Option<crate::hud::HudStats>,
    saved: Option<crate::hud::HudSaved>,
    update_checked: Option<crate::updates::UpdateChecked>,
    pending_crash: bool,
}

#[derive(Default)]
struct WhichKeyDefaults {
    rows: Option<Vec<(String, String)>>,
}

#[derive(Default)]
struct DebugDefaults {
    frame_cost: Option<(f32, f32)>,
    latency_ms: Option<f32>,
    redraws: Option<u64>,
    budget_ms: Option<f32>,
    gpu_bytes: Option<u64>,
    autosave: Option<crate::debug::AutosaveState>,
    theme_settle: Option<crate::themeswitch::SwitchReport>,
}

/// Flatten the ACTIVE world's [`crate::theme::Background`] into the host-side
/// [`BgDesc`] the margin pipeline uploads — gradient endpoints + direction, the
/// ground discriminant, and the mark/band tint plus its per-ground params (the
/// Dots proximity flag / the Stripes angle). Read at construction AND on every
/// live theme switch so both paths agree.
/// Convert an 8-bit sRGB RGBA quad to LINEAR-light rgb (alpha dropped), for the
/// frosted-blur composite's dim-toward-base_100 (the blur targets are sRGB, so the
/// shader's `mix` must happen in linear space). Same curve the selection /
/// background pipelines use — all route through `theme`'s one `f32`-width sRGB
/// EOTF.
fn srgb_u8_to_linear3(c: [u8; 4]) -> [f32; 3] {
    let ch = theme::srgb_channel_to_linear_f32;
    [ch(c[0]), ch(c[1]), ch(c[2])]
}

fn background_desc() -> BgDesc {
    // Lava's gallery override wins; otherwise the authored ground, verbatim.
    let bg = crate::lava::env_override().unwrap_or_else(theme::background);
    BgDesc {
        from: bg.from().rgba_bytes(),
        to: bg.to().rgba_bytes(),
        dir: bg.dir(),
        shader: bg.shader_id(),
        tint: bg.tint().rgb_bytes(),
        edge: bg.edge(),
        angle: bg.angle(),
        period_px: bg.period_px(),
        amplitude_px: bg.amplitude_px(),
        density: bg.density(),
        banded: bg.zigzag_banded(),
        profile: bg.profile_mode(),
        tunnel: bg.tunnel_mode(),
    }
}
/// The visual-line motion LAYOUT ORACLE, implemented on the GPU pipeline because
/// it owns the SHAPED text (and hence the wrap geometry). Every query is answered
/// from the same [`TextPipeline::visual_rows`] / [`pick_row`] / per-char `xs` the
/// caret + hit-test already use, so live motion and the visual placement of the
/// caret can't disagree. `apply_transition` reaches these through the renderer-agnostic
/// [`crate::actions::LayoutOracle`] trait, keeping the motion logic itself free of
/// any GPU type. Columns are CHAR columns; `goal_x` and the returned x are pixels
/// relative to `text_left()` (the space `xs` lives in).
///
/// These ARE the live/headless visual-line motions (the flat default): the live
/// window borrows the GPU pipeline as the oracle, the headless `--keys` replay an
/// offscreen-shaped twin, so the two flows step the same wrapped rows.
///
/// Land the caret under `goal_x` on `rows[target]` and GUARANTEE the returned
/// column actually RENDERS on that row — never on a neighbour. [`col_in_row`]'s
/// past-content default is the row's `end_col`; at a SHARED wrap boundary (a wrap
/// with NO dropped whitespace — e.g. mid-word or inside a long `|`-delimited table
/// row) `end_col` EQUALS the next row's `start_col`, and [`pick_row_index`] gives
/// that shared column to the LOWER row. So a large `goal_x` would leave the caret
/// on the SAME visual row it started from — a vertical-motion FIXED POINT ("moving
/// straight up/down gets stuck"). When the naive landing escapes to a neighbour we
/// pull it back to the last column this row itself owns, so every step lands on the
/// intended adjacent row. Boundaries with a dropped space (a 1-col gap, the common
/// prose case) and every small-`goal_x` landing already resolve to `target`, so
/// this is a no-op there — the caret placement for ordinary wraps is unchanged.
fn col_on_row(rows: &[VisualRow], target: usize, goal_x: f32) -> usize {
    let row = &rows[target];
    let nc = TextPipeline::col_in_row(row, goal_x);
    if pick_row_index(rows, nc) == target {
        return nc;
    }
    row.end_col.saturating_sub(1).max(row.start_col)
}

impl crate::actions::LayoutOracle for TextPipeline {
    fn visual_row_of(&self, line: usize, col: usize) -> usize {
        TextPipeline::visual_row_of(self, line, col)
    }

    fn visual_x_of(&self, line: usize, col: usize, affinity: crate::caret::Affinity) -> f32 {
        // Read this line's local rows; affinity resolves shared wrap boundaries.
        let rows = self.line_rows_local(line);
        let row = pick_row_aff(&rows, col, affinity);
        let c = col.min(row.xs.len().saturating_sub(1));
        row.xs[c]
    }

    fn visual_line_up(
        &self,
        line: usize,
        col: usize,
        goal_x: f32,
        affinity: crate::caret::Affinity,
    ) -> (usize, usize) {
        let rows = self.line_rows_local(line);
        let idx = pick_row_index_aff(&rows, col, affinity);
        if idx > 0 {
            // A wrapped continuation: step to the previous visual row of the SAME
            // logical line, landing under the goal-x (owned by that row — see
            // `col_on_row`, which keeps a large goal-x off the shared wrap boundary
            // so the step actually ascends instead of sticking).
            return (line, col_on_row(&rows, idx - 1, goal_x));
        }
        if line == 0 {
            return (line, col); // top visual row of the first line: nowhere up
        }
        let prev = self.line_rows_local(line - 1);
        (line - 1, col_on_row(&prev, prev.len() - 1, goal_x))
    }

    fn visual_line_down(
        &self,
        line: usize,
        col: usize,
        goal_x: f32,
        affinity: crate::caret::Affinity,
    ) -> (usize, usize) {
        let rows = self.line_rows_local(line);
        let idx = pick_row_index_aff(&rows, col, affinity);
        if idx + 1 < rows.len() {
            // A wrapped line with rows below: step to the next visual row of the
            // SAME logical line (owned by that row — `col_on_row` keeps a large
            // goal-x off the shared wrap boundary so the step lands on the
            // immediately-next row rather than skipping past it).
            return (line, col_on_row(&rows, idx + 1, goal_x));
        }
        let last_line = self.buffer.lines.len().saturating_sub(1);
        if line >= last_line {
            return (line, col); // bottom visual row of the last line: nowhere down
        }
        let next = self.line_rows_local(line + 1);
        (line + 1, col_on_row(&next, 0, goal_x))
    }

    fn visual_line_start(
        &self,
        line: usize,
        col: usize,
        affinity: crate::caret::Affinity,
    ) -> (usize, usize) {
        let rows = self.line_rows_local(line);
        (line, pick_row_aff(&rows, col, affinity).start_col)
    }

    fn visual_line_end(
        &self,
        line: usize,
        col: usize,
        affinity: crate::caret::Affinity,
    ) -> (usize, usize) {
        let rows = self.line_rows_local(line);
        (line, pick_row_aff(&rows, col, affinity).end_col)
    }
}
#[cfg(test)]
mod tests;
