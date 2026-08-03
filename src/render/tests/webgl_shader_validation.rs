//! OFFLINE WGSL -> GLSL ES 300 (WebGL2) shader validation. `cargo build
//! --target wasm32-unknown-unknown` only proves the RUST side compiles —
//! WGSL itself is parsed/validated by `wgpu`/`naga` at RUNTIME
//! (`create_shader_module`), which needs a real GPU/browser context this
//! sandbox cannot open. This file closes that gap OFFLINE: it runs the exact
//! same `naga` (pinned to the identical `=29.0.3` version `wgpu` uses) parse
//! -> validate -> GLSL-backend pipeline wgpu's own GL/WebGL backend runs
//! internally, without needing a device at all — dev-only (`naga` is a
//! `[dev-dependencies]`-only addition, never a runtime dependency of the
//! shipped binary).
//!
//! ITEM 240: this used to be a hand-kept list of `#[test]` functions, one
//! per shader the author remembered to add — 4 of 9 shaders in `shaders/`
//! ended up covered, and the other 5 (including `blur.wgsl`, whose fragment
//! stage has THREE entry points) were validated at native runtime only,
//! never against the WebGL2 fallback awl actually ships. A hand-kept list is
//! the defect, not a fix (CLAUDE.md: item 97's hardcoded mono-face list is
//! the same shape). So this sweeps `shaders/` ITSELF at test time: every
//! `.wgsl` file, every `@vertex`/`@fragment` entry point found in it — a
//! tenth shader, or an eleventh entry point on an existing one, is picked up
//! with no list to remember to edit, and a construct the GLSL-ES backend
//! rejects fails this test BY NAME (the panic names the file and entry
//! point).

use std::fs;
use std::path::Path;

fn validate_and_glsl(source: &str, stage: naga::ShaderStage, file: &str, entry_point: &str) {
    let module = naga::front::wgsl::parse_str(source)
        .unwrap_or_else(|e| panic!("WGSL parse failed for {file}::{entry_point}: {e}"));

    let info = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::default(),
    )
    .validate(&module)
    .unwrap_or_else(|e| panic!("naga validation failed for {file}::{entry_point}: {e}"));

    let options = naga::back::glsl::Options {
        version: naga::back::glsl::Version::Embedded {
            version: 300,
            is_webgl: true,
        },
        writer_flags: naga::back::glsl::WriterFlags::empty(),
        binding_map: Default::default(),
        zero_initialize_workgroup_memory: true,
    };
    let pipeline_options = naga::back::glsl::PipelineOptions {
        shader_stage: stage,
        entry_point: entry_point.to_string(),
        multiview: None,
    };
    let mut out = String::new();
    let mut writer = naga::back::glsl::Writer::new(
        &mut out,
        &module,
        &info,
        &options,
        &pipeline_options,
        naga::proc::BoundsCheckPolicies::default(),
    )
    .unwrap_or_else(|e| {
        panic!("GLSL ES 300 (WebGL2) writer construction failed for {file}::{entry_point}: {e}")
    });
    writer.write().unwrap_or_else(|e| {
        panic!("GLSL ES 300 (WebGL2) translation failed for {file}::{entry_point}: {e}")
    });
    assert!(
        out.contains("void main"),
        "GLSL output for {file}::{entry_point} looks empty/malformed: {out:?}"
    );
}

/// Scans WGSL source for `@vertex`/`@fragment` entry points in file order.
/// This is the directory-driven part: it reads the ATTRIBUTE the shader
/// author already has to write to make an entry point real, rather than a
/// second, separate list a human must remember to keep in sync with it.
fn entry_points(source: &str) -> Vec<(naga::ShaderStage, String)> {
    let mut out = Vec::new();
    let mut lines = source.lines().peekable();
    while let Some(line) = lines.next() {
        let stage = match line.trim() {
            "@vertex" => naga::ShaderStage::Vertex,
            "@fragment" => naga::ShaderStage::Fragment,
            _ => continue,
        };
        // Skip any further stacked attribute lines to reach the `fn` line.
        while lines
            .peek()
            .is_some_and(|next| next.trim().starts_with('@'))
        {
            lines.next();
        }
        let fn_line = lines
            .next()
            .unwrap_or_else(|| panic!("entry-point attribute with no following `fn` line"));
        let rest = fn_line.trim().strip_prefix("fn ").unwrap_or_else(|| {
            panic!("expected `fn` after entry-point attribute, found: {fn_line:?}")
        });
        let name = rest
            .split(['(', '<'])
            .next()
            .unwrap_or(rest)
            .trim()
            .to_string();
        out.push((stage, name));
    }
    out
}

#[test]
fn every_shader_under_shaders_dir_targets_webgl2() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("shaders");
    let mut files: Vec<_> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("cannot read shaders dir {}: {e}", dir.display()))
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("wgsl"))
        .collect();
    files.sort();
    assert!(
        !files.is_empty(),
        "no .wgsl files found under {}",
        dir.display()
    );

    for path in &files {
        let file = path.file_name().unwrap().to_string_lossy().to_string();
        let source = fs::read_to_string(path).unwrap_or_else(|e| panic!("cannot read {file}: {e}"));
        let points = entry_points(&source);
        assert!(
            !points.is_empty(),
            "{file} has no @vertex/@fragment entry point -- the sweep found nothing to validate, \
             which means it would silently skip this shader rather than covering it"
        );
        for (stage, entry_point) in &points {
            validate_and_glsl(&source, *stage, &file, entry_point);
        }
    }
}
