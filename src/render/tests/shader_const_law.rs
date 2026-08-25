//! THE CONSTANT-TWIN PIN — every Rust value that a WGSL shader also declares
//! as a literal, asserted equal by reading naga's PARSED AST rather than
//! scanning the source text. Per the shader-dedup round's law policy: a pin
//! ships only if it parses (this file) or if the fix is structural (folding
//! a literal into a preamble every consumer shares removes the duplication
//! rather than policing it — see `shaders/common/dither.wgsl`, which is what
//! collapsed the three-file WGSL-side copy of `BAYER8` this file's own
//! Rust<->WGSL half still can't be folded away, because a compile-time WGSL
//! array literal and a compile-time Rust array literal have no runtime
//! crossing to route through instead).
//!
//! Two twins, because both are dense literals with no test that already
//! reads the parsed shader for them (`render::tests::backgrounds_item89`,
//! `bowerbird_finds`, `deckle_ground` and `warped_grid` already pin their own
//! constants by TEXT scan — pre-existing, out of scope here, and untouched):
//!
//! 1. `BAYER8` — the 8x8 ordered-dither matrix, declared once in
//!    `shaders/common/dither.wgsl` and mirrored by hand in
//!    `render::dither::BAYER8` (`src/render/dither.rs`'s own module doc names
//!    the duplication as deliberate: WGSL has no runtime array upload for a
//!    `var<private>` this small, so there is nothing to single-source
//!    through).
//! 2. `ORGANIC_LOOP_CYCLES` — `shaders/background.wgsl`'s own copy of
//!    `crate::lava::LAVA_LOOP_CYCLES`, needed as a `const` inside a WGSL
//!    expression where a uniform upload cannot reach (it sizes a compile-time
//!    trig period, not a per-frame value).

use super::super::*;

fn parsed_background() -> naga::Module {
    let src = crate::gpu_cache::source_for_file("background.wgsl")
        .expect("gpu_cache::Shader::Background owns background.wgsl");
    naga::front::wgsl::parse_str(src)
        .unwrap_or_else(|e| panic!("background.wgsl (assembled) failed to parse: {e}"))
}

/// Resolve a constant expression to a flat list of literal f64s (integers and
/// floats alike travel as f64 here — every value this file compares fits
/// exactly, and a single numeric type keeps the walk one function instead of
/// one per WGSL scalar kind).
fn literals(module: &naga::Module, expr: naga::Handle<naga::Expression>) -> Vec<f64> {
    match &module.global_expressions[expr] {
        naga::Expression::Literal(lit) => vec![match *lit {
            naga::Literal::F64(v) => v,
            naga::Literal::F32(v) => v as f64,
            naga::Literal::F16(v) => f32::from(v) as f64,
            naga::Literal::U32(v) => v as f64,
            naga::Literal::I32(v) => v as f64,
            naga::Literal::U64(v) => v as f64,
            naga::Literal::I64(v) => v as f64,
            naga::Literal::Bool(v) => v as u32 as f64,
            naga::Literal::AbstractInt(v) => v as f64,
            naga::Literal::AbstractFloat(v) => v,
        }],
        naga::Expression::Compose { components, .. } => components
            .iter()
            .flat_map(|&c| literals(module, c))
            .collect(),
        other => panic!("expected a literal or a composite of literals, found {other:?}"),
    }
}

fn global_var_literals(module: &naga::Module, name: &str) -> Vec<f64> {
    let var = module
        .global_variables
        .iter()
        .map(|(_, v)| v)
        .find(|v| v.name.as_deref() == Some(name))
        .unwrap_or_else(|| panic!("no `var<private> {name}` found in the parsed module"));
    let init = var
        .init
        .unwrap_or_else(|| panic!("`{name}` has no initializer to read"));
    literals(module, init)
}

fn const_literal(module: &naga::Module, name: &str) -> f64 {
    let c = module
        .constants
        .iter()
        .map(|(_, c)| c)
        .find(|c| c.name.as_deref() == Some(name))
        .unwrap_or_else(|| panic!("no `const {name}` found in the parsed module"));
    let vs = literals(module, c.init);
    assert_eq!(vs.len(), 1, "`{name}` is not a scalar constant: {vs:?}");
    vs[0]
}

/// Twin 1 — `BAYER8`. The WGSL side is a SINGLE array now (folded into
/// `shaders/common/dither.wgsl` by the structural dedup), so this reads it
/// once and compares all 64 entries against `render::dither::BAYER8` in one
/// pass, rather than once per consuming shader.
#[test]
fn bayer8_matches_the_rust_mirror() {
    let module = parsed_background();
    let wgsl: Vec<f64> = global_var_literals(&module, "BAYER8");
    let rust: Vec<f64> = dither::BAYER8.iter().map(|&b| b as f64).collect();
    assert_eq!(
        wgsl, rust,
        "shaders/common/dither.wgsl's BAYER8 has drifted from render::dither::BAYER8 — the \
         two are a deliberate cross-language duplication (see dither.rs's module doc), kept \
         equal only by this pin"
    );
}

/// Twin 2 — `ORGANIC_LOOP_CYCLES`, background.wgsl's own copy of
/// `crate::lava::LAVA_LOOP_CYCLES`. Unlike the FINDS/DECKLE/warp-grid
/// constants (already pinned by an existing text-scan test each), no test
/// read this one before — a shader retune could have moved it silently.
#[test]
fn organic_loop_cycles_matches_lava_loop_cycles() {
    let module = parsed_background();
    let wgsl = const_literal(&module, "ORGANIC_LOOP_CYCLES");
    assert_eq!(
        wgsl,
        crate::lava::LAVA_LOOP_CYCLES as f64,
        "shaders/background.wgsl's ORGANIC_LOOP_CYCLES no longer byte-matches \
         crate::lava::LAVA_LOOP_CYCLES — the companion breathe reads the shared ambient clock \
         under the assumption that they agree"
    );
}
