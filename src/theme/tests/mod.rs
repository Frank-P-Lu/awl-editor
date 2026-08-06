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
mod tokens;
mod world_pin_item94;
