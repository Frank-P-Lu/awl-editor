use super::*;

mod font_symbols;
pub(super) use font_symbols::*;

/// Split by subject into this directory: [`attrs`] builds `Attrs` from a
/// markdown/syntax span kind and lays spans onto an `AttrsList`; [`conceal`]
/// is the WYSIWYG reveal/conceal mechanism (zero-width markup hiding, the
/// caret/selection reveal rule); [`colors`] derives every syntax-role,
/// highlight-wash, and strike/underline ink from the active theme;
/// [`layout`] holds the per-line size scale and `build_line_attrs`, the
/// final assembly that layers every pass in the canonical order.
mod attrs;
mod colors;
mod conceal;
mod layout;

pub(super) use attrs::*;
pub(super) use colors::*;
// `wysiwyg_reveals` alone is named rather than globbed: it is `pub(crate)` in
// `conceal` (the capture sidecar's `wysiwyg_report` reads it from outside
// `render`), and `render.rs` re-exports it again under `cfg(test)` — which
// this module's own top `use super::*;` glob then re-imports right back,
// circularly. A second glob source for the same name is an ambiguity Rust
// only warns on today; naming it explicitly here sidesteps that entirely.
pub(crate) use conceal::wysiwyg_reveals;
pub(super) use conceal::{
    IMAGE_MAX_VIEWPORT_FRAC, add_bullet_conceal_span, add_list_indent_span, add_rule_conceal_span,
    add_wysiwyg_conceal_spans, cell_inline_attrs, image_line_has_other_content, line_has_code_span,
    line_has_image_span, line_has_rule_span, selection_touch_bytes, selection_touches,
};
#[cfg(not(target_arch = "wasm32"))]
pub(super) use conceal::{IMAGE_MISSING_ROW_LINES, image_display_size};
pub(super) use layout::*;

const QUOTE_TEXT_DIM: bool = true;

fn quote_text_dim() -> bool {
    static V: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *V.get_or_init(|| QUOTE_TEXT_DIM && std::env::var_os("AWL_QUOTE_FULL_INK").is_none())
}
