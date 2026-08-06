#![allow(dead_code)]
// Some tokens (BASE_200, PRIMARY_CONTENT) and converters are
// not consumed by every surface yet — reserved for the
// upcoming minibuffer/panel surfaces. The per-theme `font`
// field is now LIVE: it drives the glyphon `Family::Name`
// used to shape/render the document (see render.rs).

//! src/theme/ — the palette model, split by natural seam (2026-07
//! code-organization pass) out of the former `theme.rs` monolith:
//! [`color`] (the [`Srgb`] primitive), [`model`] (the [`Theme`]/[`Background`]/
//! [`Lens`] data model), [`ornament`] (the section-break + list-bullet trios),
//! [`cjk`] (the per-script fallback ladders + [`FontId`]), [`worlds`] (the
//! shipped [`Theme`] literals), and [`derive`] (the active-theme
//! index + every derived-from-active-theme accessor). Every external path
//! (`theme::Theme`, `theme::THEMES`, `theme::CJK_MINCHO`, …) is unchanged —
//! this file only re-exports.
//!
//! [`THEMES`] is the SINGLE ENROLLMENT DOOR, and that is a load-bearing fact
//! rather than a description: nothing else constructs a world, so an authored
//! [`Theme`] absent from it cannot reach the picker, a capture, the icon roster
//! or a law sweep by accident. It is what lets `worlds.rs` be a roster a reader
//! can count instead of a pile a reader must filter.
//!
//! Naming follows DaisyUI: base-100/200/300 are the base planes (100 = the
//! canvas; on a dark world that is the deepest plane, on a light world the
//! lightest), `*-content` is the ink that sits on a given surface, `primary` is
//! the one organic accent (the caret), `error` is the signal color, and
//! `selection` is a custom token (DaisyUI has no selection role).
//!
//! There are twenty [`Theme`]s ("worlds"), eleven dark and nine light. Three are
//! DESIGN.md §3 statement worlds: Wagtail (awl's first true MONOCHROME/1-bit
//! world — zero saturation everywhere, the caret included), Firetail (awl's
//! first LAVA-LAMP world — a slow metaball ground whose living warmth IS the
//! statement; Mangrove folds the cool second lava ground) and Kite (the light
//! counterpart, travelling through a warped-grid tunnel). See their own doc
//! comments in `worlds.rs` and THEMES.md's logged DESIGN.md §3 amendments. One is the
//! ACTIVE theme at any moment (an index into [`THEMES`]); the windowed app can
//! cycle it live (`C-x t` / `C-x T`) and the headless `--theme NAME` flag pins
//! it before a capture. Every color call site reads the active theme rather than
//! a fixed const, so a theme switch reskins the whole UI. Each world also names a
//! display `font`; that family is loaded at startup and selected per-frame, so a
//! theme switch reskins the GLYPH SHAPES too (mono / serif / slab / sans).

mod cjk;
mod color;
mod derive;
mod diagonal;
mod ground;
mod ground_space;
mod icon_ground;
mod model;
mod ornament;
mod worlds;

pub(crate) use cjk::EMBEDDED_CJK_FAMILIES;
pub use cjk::FontId;
#[allow(unused_imports)] // per-world CJK ladders: public API surface consumed by
// `theme::worlds` internally + named in doc comments crate-wide; no NON-TEST
// in-crate caller reaches them through this re-export path today.
pub use cjk::{
    ALL_FONT_IDS, CJK_GOTHIC, CJK_JA_KLEE, CJK_JA_SHIPPORI, CJK_JA_ZENMARU, CJK_KO, CJK_KO_SERIF,
    CJK_MINCHO, CJK_ZH_HANS_KLEE, CJK_ZH_HANS_SANS, CJK_ZH_HANS_SERIF, CJK_ZH_HANT,
};
pub use color::Srgb;
#[allow(unused_imports)] // cycle/overlay_scrim/primary_content/tag_for/WorldPin:
// public API surface, no NON-TEST in-crate caller today (tag_for's real callers
// all live under `#[cfg(test)]`; `WorldPin` is the explicit world restore a test
// that renders a NAMED world holds — deliberately never taken by product code).
pub use derive::{WorldPin, cycle, overlay_scrim, primary_content, tag_for};
pub use derive::{
    active, active_index, background, base_100, base_200, base_300, base_content, card_texture_ink,
    error, faint, fold_afford_chevron_ink, fold_afford_tail_ink, heatmap_colors,
    image_reveal_scrim, muted, overlay_band_overlap, overlay_bar_unselected, overlay_bars_scrim,
    page_frame_ink, pane_surface, placard_ink, placard_stipple_density, primary, selected_row_ink,
    selected_row_secondary_ink, selection_document, selection_ui, set_active, set_active_by_name,
    surface_selected,
};
// `DiagonalMark` is authored in `worlds.rs` and reached by the renderer through
// its `DiagonalSpine`, so the NAME is read only by the laws that assert the two
// diagonal worlds author different marks — the whole point of the split.
#[allow(unused_imports)]
pub use diagonal::{DiagonalDirection, DiagonalMark, DiagonalSpine};
pub use ground::Background;
// ITEM 186 — the coordinate-space vocabulary every authored ground quantity
// is classified in (`ground_space` holds the table; `Background`'s own
// accessors are inherent, so they need no import).
#[allow(unused_imports)] // GroundQuantity/GroundSpace: read by the item-186
// laws and by anyone authoring a new ground; product code reaches the table
// through `Background::authored_quantities` itself.
pub use ground_space::{GroundQuantity, GroundSpace};
pub use model::{Theme, WashOverride};
// ITEM 89's ZIGZAG geometry mirror — `cfg(test)` at the source (see their own
// docs: the GPU is the only runtime consumer; the host reads them ONLY to state
// the field's laws), so the re-export is gated identically rather than carrying
// an `allow(dead_code)` a future genuinely-dead constant could hide behind.
#[cfg(test)]
pub use ground::{DECKLE_MAX_PERIOD_PX, DECKLE_MID, DECKLE_MIN_PERIOD_PX, DECKLE_SPREAD_GAIN};
#[cfg(test)]
pub use ground::{
    ORGANIC_FINDS_ACCENT_HI, ORGANIC_FINDS_ANCHOR_HI, ORGANIC_FINDS_ANCHOR_LO,
    ORGANIC_FINDS_COMPANION_HI, ORGANIC_FINDS_COMPANION_LO, ORGANIC_FINDS_DROPOUT,
    ORGANIC_FINDS_MIN_SCALE_PX,
};
#[allow(unused_imports)] // Weave/Lens/RoleOverrides/ThemeTags: public API
// surface, no NON-TEST in-crate caller today (the world literals reach `Weave`
// through `super::ground` directly).
pub use ground::{Tunnel, Weave};
#[cfg(test)]
pub use ground::{ZIGZAG_MAX_ROW_PITCH_PX, ZIGZAG_MIN_STROKE_PX, ZIGZAG_STROKE_FRAC};
#[allow(unused_imports)]
pub use model::{Lens, RoleOverrides, ThemeTags};
// THEME CAPABILITIES AS DATA: the declarative render-behavior bundle every
// per-theme render decision reads instead of an ad hoc `is_one_bit()` branch.
// See `model::RenderCaps`'s own module doc.
#[allow(unused_imports)] // public API surface; every in-crate caller today
// reaches it through `Theme::icon_ground` / `icon_ground_color()`, and on
// wasm (no native icon exporter) nothing outside `theme` names it directly.
pub use icon_ground::IconGround;
#[allow(unused_imports)] // RenderCaps/ImageReveal: public API surface (the full
// bundle type + one field's enum); every non-test in-crate caller today reaches
// them through `Theme::render_caps.<field>` rather than this bare re-export.
pub use model::{
    AmbientStyle, Backdrop, BandResponse, BarConfig, BarCoverage, BarExtent, CardAnchor, CardShape,
    CardTexture, CaretBlockStyle, ChipVariant, ChromeFace, DecorativeWash, Elevation, FacetStyle,
    FoldAfford, Frost, HighlightTexture, HighlightTreatment, IconCursor, ImageReveal, ListBacking,
    ListStyle, LocationStyle, MotionJuice, OverlayEntrance, PageFrame, PaneSplit, PlacardCorner,
    PlacardInk, RenderCaps, RuleSelection, SelectionStyle, TitleStyle,
};
#[allow(unused_imports)] // the per-world ornament/bullet data: public API
// surface, no NON-TEST in-crate caller today.
pub use ornament::{
    BULLET_SCALE_GARAMOND, BULLET_SCALE_ORNAMENT, BULLET_SCALE_PLAIN, BULLETS_PLAIN,
    LIST_INDENT_SCALE_PLAIN, LIST_INDENT_SCALE_WIDE, ORNAMENT_GARAMOND, ORNAMENT_JUNICODE,
    ORNAMENT_MARKS, ORNAMENT_SCALE_FLEURON, ORNAMENT_SCALE_GEOMETRIC, ORNAMENT_SCALE_ORNATE,
    ORNAMENTS_DEFAULT, Ornaments,
};
#[allow(unused_imports)] // the individually named world consts: public
// API surface (each usable individually, e.g. `theme::TAWNY.mono`); non-test code
// always reaches them through the `THEMES` array instead (Cassowary among them).
pub use worlds::{
    BILBY, BOMBORA, BOWERBIRD, BROLGA, CURRAWONG, FIRETAIL, GALAH, GUMTREE, KITE, MAGPIE, MANGROVE,
    MOPOKE, MULGA, PAPERBARK, POTOROO, QUOKKA, SALTPAN, TAWNY, WAGTAIL,
};
pub use worlds::{DEFAULT_THEME, THEMES, world_names};

#[cfg(test)]
mod tests;
