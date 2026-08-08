//! Tests for the `theme` module (the sixteen worlds + their derivation laws)
//! -- split by SUBJECT (the 2026-08 code-organization pass, following
//! `render/tests/`'s established shape) out of one 4256-line `theme::tests`
//! file into this `theme/tests/` directory -- every test's NAME is
//! unchanged, only its module path grew one segment
//! (`theme::tests::foo` -> `theme::tests::<subject>::foo`). `use super::*;`
//! here still resolves to the `theme` root exactly as before the split; each
//! child module re-derives theme access directly via its own
//! `use super::super::*;` plus a targeted `use super::super::derive::{..};`
//! for whichever of `derive`'s constants it actually calls.

use super::*;

mod ambient;
/// The page's CLEAR colour: the sRGB→linear decode `LoadOp::Clear` needs, and
/// the one transfer function the whole tree shares.
mod clear;
mod distinctness;
mod firetail;
mod fonts;
mod frost;
mod ground;
mod heatmap;
mod lava;
mod ornament;
mod page_frame;
mod personality;
mod placard;
mod roster;
mod selection_ui;
/// THEMES.md is `include_str!`'d under `not(wasm32)` (its reader is a native
/// law and the wasm test binary has no business embedding the world-laws
/// document), so this module carries the same gate.
#[cfg(not(target_arch = "wasm32"))]
mod themes_md;
mod tokens;
mod world_pin_item94;
