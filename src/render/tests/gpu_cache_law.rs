//! THE PROGRAM-CACHE LAWS (the hosted-macOS `mac (build + test)` wedge).
//!
//! `TextPipeline::new` stands up 8 shader modules and ~33 render pipelines. The
//! live app pays that ONCE PER LAUNCH; the test suite paid it 795 times per
//! process, against ONE shared wgpu device — 1675 translations of a 1346-line
//! `background.wgsl` and roughly 26 000 render pipelines, 39.9 s of a single
//! `cargo test --bin awl` run on the dev host and, on a three-vCPU hosted
//! runner with a virtualised Metal stack, the churn the mac CI job wedged in.
//! [`crate::gpu_cache`] holds those objects for the process; this file is what
//! stops that from silently coming undone, and what proves it changed nothing a
//! test can see.
//!
//! Three laws, because the defect has three ways back in:
//!
//! 1. the cache stops being consulted (a helper reverts to
//!    `shared_device_queue`, or a new pipeline family is built raw);
//! 2. a new shader is compiled outside the one owner;
//! 3. the cache "works" by handing every world the same program state — which
//!    would trade a CI hang for a cross-test state leak.
//!
//! Law 3 is the one worth the most: it does not check that the cache is FAST,
//! it checks that a world rendered through the shared programs is BYTE-IDENTICAL
//! to the same world rendered through freshly built ones, and it sweeps the
//! WHOLE `THEMES` roster rather than a world someone imagined.

use super::super::*;
use super::{dither, headless_dqp};

/// Law 1 — the amortisation itself. A second `TextPipeline` on the shared
/// device must build ZERO new GPU programs.
///
/// This reads `gpu_cache::builds()`, which counts every object the module
/// actually built, cache misses and uncached pass-throughs alike — so a helper
/// that stops routing through the cache shows up here as a non-zero delta, not
/// as a silent slowdown.
#[test]
fn a_second_headless_pipeline_builds_no_new_gpu_programs() {
    let _g = crate::testlock::serial();
    let Some((_d, _q, _first)) = headless_dqp(400.0, 200.0) else {
        eprintln!(
            "skipping a_second_headless_pipeline_builds_no_new_gpu_programs: no wgpu adapter"
        );
        return;
    };
    // NON-VACUOUS: the FIRST pipeline in this process may or may not have been
    // the one that filled the cache (another test may have run first), so the
    // measurement is taken across the second and third — by which point every
    // program this path needs is certainly present.
    let before = crate::gpu_cache::builds();
    let Some((_d2, _q2, _second)) = headless_dqp(400.0, 200.0) else {
        return;
    };
    let after = crate::gpu_cache::builds();
    assert_eq!(
        after,
        before,
        "a second TextPipeline on the shared device built {} new GPU programs — the test \
         helpers must go through `test_gpu::with_shared_programs`, which is what keeps the \
         suite from compiling `background.wgsl` 1675 times against one device",
        after - before
    );
}

/// Law 2 — one owner for every shader compile. `create_shader_module` may
/// appear in `src/` ONLY inside `gpu_cache.rs`, so a new pipeline family cannot
/// quietly add a ninth per-instance translation.
///
/// A grep-law in the shape `float_surface_law` already established, including
/// its self-exemption: this file necessarily names the pattern it bans.
#[test]
fn create_shader_module_has_exactly_one_owner() {
    const OWNER: &str = "gpu_cache.rs";
    const PATTERN: &str = "create_shader_module(";
    let src = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut hits = Vec::new();
    let mut owner_hits = 0usize;
    scan(&src, &src, &mut |rel, text| {
        let file = rel.rsplit('/').next().unwrap_or(rel);
        if file == "gpu_cache_law.rs" {
            return;
        }
        for (i, line) in text.lines().enumerate() {
            // Doc comments NAME the call without being one — `selection.rs` and
            // `webgl_shader_validation.rs` both discuss it.
            if !line.contains(PATTERN) || line.trim_start().starts_with("//") {
                continue;
            }
            if file == OWNER {
                owner_hits += 1;
            } else {
                hits.push(format!("  {rel}:{}", i + 1));
            }
        }
    });
    assert!(
        hits.is_empty(),
        "every WGSL translation must go through `gpu_cache::shader`, which is what makes it \
         once-per-process instead of once-per-pipeline — offending lines:\n{}",
        hits.join("\n")
    );
    // NON-VACUOUS: the owner really does carry the call. If `compile` were ever
    // deleted or renamed, the ban above would go quiet without ever having been
    // exercised.
    assert_eq!(
        owner_hits, 1,
        "expected exactly one `create_shader_module` call, in gpu_cache.rs's `compile`; found \
         {owner_hits}"
    );
}

/// Law 2b — the same rule for render pipelines, which is where a Metal backend
/// actually compiles. The descriptors are too varied to move into one module,
/// so the law is structural instead: a file that calls `create_render_pipeline`
/// must call `gpu_cache::render_pipeline` exactly as many times, i.e. every raw
/// construction sits inside a cached build closure.
#[test]
fn every_render_pipeline_is_built_inside_the_cache() {
    let src = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut bad = Vec::new();
    let mut cached_files = 0usize;
    scan(&src, &src, &mut |rel, text| {
        let file = rel.rsplit('/').next().unwrap_or(rel);
        if file == "gpu_cache_law.rs" || file == "gpu_cache.rs" {
            return;
        }
        let code = |pat: &str| {
            text.lines()
                .filter(|l| l.contains(pat) && !l.trim_start().starts_with("//"))
                .count()
        };
        let raw = code("create_render_pipeline(&");
        if raw == 0 {
            return;
        }
        let wrapped = code("gpu_cache::render_pipeline(");
        if raw != wrapped {
            bad.push(format!("  {rel}: {raw} raw vs {wrapped} cached"));
        } else {
            cached_files += 1;
        }
    });
    assert!(
        bad.is_empty(),
        "every `create_render_pipeline` must be the body of a `gpu_cache::render_pipeline` \
         build closure — otherwise the suite compiles that program once per pipeline INSTANCE \
         rather than once per process:\n{}",
        bad.join("\n")
    );
    // NON-VACUOUS: the nine pipeline families really are there to be checked.
    assert_eq!(
        cached_files, 9,
        "expected the nine pipeline families (background, blur, caret, caret_glyph, image, \
         lava, rotated_label, selection, spellunderline) to build through the cache; found \
         {cached_files}"
    );
}

/// Law 3 — THE CONSTRAINT. Sharing a `wgpu::RenderPipeline` must not share a
/// world. For EVERY shipped world, the ground rendered through the shared
/// cached programs is BYTE-IDENTICAL to the same world rendered through freshly
/// built ones — swept over the whole `THEMES` roster, not a world someone
/// picked.
///
/// **And the cached instances are built and prepared INTERLEAVED, not one at a
/// time.** That is the axis the churn hypothesis did not think of, and the only
/// one that can see the failure worth fearing here: a cache that shared a
/// uniform buffer or a bind group as well as a program would still render every
/// world correctly when each is drawn alone, because each `prepare` would
/// overwrite the last in time. Standing all twenty up FIRST, preparing all
/// twenty, and only then drawing them is what makes one world's state visible
/// in another's picture — a cross-test state leak, which is not a trade worth
/// making to fix a CI hang.
#[test]
fn every_world_renders_identically_through_cached_and_fresh_programs() {
    let _g = crate::testlock::serial();
    let (w, h) = (72u32, 96u32);
    let Some((device, queue)) = crate::test_gpu::shared_device_queue() else {
        eprintln!(
            "skipping every_world_renders_identically_through_cached_and_fresh_programs: \
             no wgpu adapter"
        );
        return;
    };

    let draw = |bg: &crate::background::BackgroundPipeline| -> Vec<[u8; 4]> {
        let (texture, tview) = dither::offscreen(&device, w, h);
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("awl gpu-cache-law encoder"),
        });
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("awl gpu-cache-law pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &tview,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            bg.draw(&mut pass);
        }
        queue.submit(Some(enc.finish()));
        dither::read_pixels(&device, &queue, &texture, w, h)
    };

    let descs: Vec<_> = theme::THEMES
        .iter()
        .map(|t| super::backgrounds_item69::bg_desc_for(t.background))
        .collect();

    // FRESH: nothing armed, one world at a time — every program built from
    // scratch, exactly what the suite did before the cache existed.
    let fresh: Vec<Vec<[u8; 4]>> = descs
        .iter()
        .map(|d| {
            let mut bg = crate::background::BackgroundPipeline::new(&device, dither::FMT, *d);
            bg.prepare(&queue, w, h, 0.0, 0.0, Default::default(), 1.0);
            draw(&bg)
        })
        .collect();

    // CACHED and INTERLEAVED: every world's pipeline stands up and uploads
    // BEFORE any of them draws.
    let cached: Vec<Vec<[u8; 4]>> = crate::test_gpu::with_shared_programs(|device, queue| {
        let pipes: Vec<_> = descs
            .iter()
            .map(|d| {
                let mut bg = crate::background::BackgroundPipeline::new(device, dither::FMT, *d);
                bg.prepare(queue, w, h, 0.0, 0.0, Default::default(), 1.0);
                bg
            })
            .collect();
        pipes.iter().map(&draw).collect()
    })
    .expect("the shared device was already proven present above");

    let mut distinct = std::collections::HashSet::new();
    for (i, t) in theme::THEMES.iter().enumerate() {
        // A compact verdict, not two 6912-pixel vectors: the count of differing
        // pixels and the first one, which is what a reader actually needs.
        let first = cached[i]
            .iter()
            .zip(fresh[i].iter())
            .position(|(a, b)| a != b);
        let differing = cached[i]
            .iter()
            .zip(fresh[i].iter())
            .filter(|(a, b)| a != b)
            .count();
        assert!(
            first.is_none(),
            "{}: the ground rendered through the shared program cache — with every other \
             world's pipeline standing up and uploading first — differs from the same ground \
             rendered alone through freshly built programs, at {differing}/{} pixels (first at \
             index {:?}: cached {:?} vs fresh {:?}). A cached PROGRAM must carry no world \
             state, and no instance's uniform buffer or bind group may be shared.",
            t.name,
            fresh[i].len(),
            first,
            first.map(|k| cached[i][k]),
            first.map(|k| fresh[i][k]),
        );
        distinct.insert(cached[i].clone());
    }

    // NON-VACUOUS: the roster really does produce different grounds, so the
    // equality above is comparing something. If every world rendered the same
    // picture, the identity law would hold for the wrong reason.
    assert!(
        distinct.len() >= theme::THEMES.len() / 2,
        "only {} distinct grounds across {} worlds — the roster should not be near-uniform, and \
         if it is, the identity law above is close to vacuous",
        distinct.len(),
        theme::THEMES.len()
    );
}

/// Walk `dir` recursively, handing every `.rs` file's repo-relative path and
/// contents to `f`.
fn scan(base: &std::path::Path, dir: &std::path::Path, f: &mut impl FnMut(&str, &str)) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan(base, &path, f);
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let rel = path
            .strip_prefix(base)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        if let Ok(text) = std::fs::read_to_string(&path) {
            f(&rel, &text);
        }
    }
}
