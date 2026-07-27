//! Hidden `--bench-frame` profiler. It replays the live redraw order headlessly,
//! using real document state and a fixed report canvas; `set_view` is timed separately.
//!
//! The replay includes every live preparation stage; `STAGE_NAMES` and `mark()` calls
//! must remain in lockstep.

use anyhow::Context as _;
use glyphon::{Cache, Resolution};
use std::path::Path;

use crate::buffer::Buffer;
use crate::capture::FORMAT;
use crate::clock::Instant;

use super::{TextPipeline, ViewState};

const WIDTH: u32 = 2910;
const HEIGHT: u32 = 1720;
const DPI: f32 = 2.0;
/// Untimed settle frames before sampling (atlas fills, caret spring settles).
const WARMUP: usize = 30;
const FRAMES: usize = 300;
const DT: f32 = 1.0 / 60.0;

/// The per-frame stages, in the EXACT order the `mark()` calls are taken in
/// [`profile_doc`] — i.e. the order [`TextPipeline::prepare`] makes its
/// sub-calls, then the encode/submit/trim tail `Gpu::redraw` runs. Keep this
/// list and the marks in lockstep (asserted per frame).
const STAGE_NAMES: [&str; 24] = [
    "advance (spring step)",
    "sync_wrap_width",
    "viewport.update (uniforms)",
    "background layer",
    "wash layer (cull + upload)",
    "text layer (glyphon prepare)",
    "caret layer (geom + upload)",
    "selection/search rects",
    "ornaments (rules + bullets)",
    "table grid (grid geometry)",
    "chrome: caret-preview panel",
    "chrome: overlay/panel park",
    "chrome: gutter",
    "chrome: debug panel",
    "chrome: stats HUD (parked)",
    "chrome: which-key (parked)",
    "spell: squiggle rect build",
    "spell: underline upload",
    "nits: rect build (line scan)",
    "nits: underline upload",
    "blur (inactive)",
    "render encode (all draws)",
    "queue.submit + device.poll",
    "atlas.trim",
];

struct Marks {
    t0: Instant,
    samples: Vec<Vec<u128>>,
    i: usize,
    timed: bool,
}

impl Marks {
    fn new(n: usize) -> Self {
        Self {
            t0: Instant::now(),
            samples: vec![Vec::new(); n],
            i: 0,
            timed: false,
        }
    }
    fn begin(&mut self, timed: bool) {
        self.i = 0;
        self.timed = timed;
        self.t0 = Instant::now();
    }
    fn mark(&mut self) {
        let ns = self.t0.elapsed().as_nanos();
        if self.timed {
            self.samples[self.i].push(ns);
        }
        self.i += 1;
        self.t0 = Instant::now();
    }
}

fn median(mut v: Vec<u128>) -> u128 {
    v.sort_unstable();
    v[v.len() / 2]
}

/// A `ViewState` built the way the LIVE `App::sync_view` builds one for a calm
/// open-file frame: cursor at the origin, no selection / search / overlay, and
/// — the load-bearing part — `misspelled` populated by the SAME
/// `SpellChecker::misspellings(&text)` scan the live app caches into
/// `spell_cache` (see `app/viewstate.rs`), so every squiggle the user sees is
/// present. Mirrors `perfbench::bench_view` otherwise.
fn live_view(buffer: &Buffer, misspelled: Vec<crate::spell::Misspelling>) -> ViewState {
    ViewState {
        text: buffer.text(),
        misspelled,
        gutter_name: buffer.display_name(),
        gutter_project: "awl-next".to_string(),
        is_markdown: buffer.is_markdown(),
        syn_lang: buffer.syntax_lang(),
        eol: buffer.eol(),
        ..ViewState::base()
    }
}

pub fn run() -> anyhow::Result<()> {
    pollster::block_on(run_async())
}

async fn run_async() -> anyhow::Result<()> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .map_err(|e| anyhow::anyhow!("no wgpu adapter for frame bench: {e:?}"))?;
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("awl frame bench device"),
            ..Default::default()
        })
        .await?;
    let cache = Cache::new(&device);

    crate::debug::set_debug_on(true);

    let spell = crate::spell::SpellChecker::new(crate::spell::DictVariant::EnUs)
        .map_err(|e| anyhow::anyhow!("spell checker failed to load: {e}"))?;

    println!(
        "frame profiler — {WIDTH}x{HEIGHT} @{DPI}x · debug panel ON · {WARMUP} warmup + {FRAMES} timed frames"
    );
    println!(
        "(headless: submit+poll SERIALIZES the GPU cost; the window overlaps it and adds present/acquire)"
    );
    for name in ["CAPTURE.md", "CLAUDE.md"] {
        profile_doc(&device, &queue, &cache, &spell, name)?;
    }
    Ok(())
}

fn run_one_frame(
    p: &mut TextPipeline,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    target_view: &wgpu::TextureView,
    marks: &mut Marks,
    frame: usize,
    ema: Option<f32>,
) -> anyhow::Result<u128> {
    let ft0 = Instant::now();

    p.advance(DT);
    p.set_debug_perf(
        ema.map(|e| (e, e)),
        None,
        Some(frame as u64),
        false,
        Some(1000.0 / 60.0),
    );
    marks.mark();

    // ---- TextPipeline::prepare, sub-call by sub-call (same order) -----
    p.sync_wrap_width();
    marks.mark();
    p.viewport.update(
        queue,
        Resolution {
            width: WIDTH,
            height: HEIGHT,
        },
    );
    marks.mark();
    p.prepare_background_layer(queue, WIDTH, HEIGHT);
    marks.mark();
    p.prepare_wash_layer(device, queue, WIDTH, HEIGHT);
    marks.mark();
    p.prepare_text_layer(device, queue, WIDTH, HEIGHT)?;
    marks.mark();
    p.prepare_caret_layer(device, queue, WIDTH, HEIGHT);
    marks.mark();
    p.prepare_selection_layer(device, queue, WIDTH, HEIGHT);
    marks.mark();
    p.prepare_ornaments(device, queue, WIDTH, HEIGHT)?;
    marks.mark();
    p.prepare_table_grid(device, queue, WIDTH, HEIGHT)?;
    marks.mark();
    p.begin_float_panel_frame();
    p.prepare_caret_preview_panel(device, queue, WIDTH, HEIGHT)?;
    marks.mark();
    p.panel_card.prepare(device, queue, WIDTH, HEIGHT, &[]);
    p.panel_shadow.prepare(device, queue, WIDTH, HEIGHT, &[]);
    p.panel_border.prepare(device, queue, WIDTH, HEIGHT, &[]);
    p.overlay_rows.prepare(device, queue, WIDTH, HEIGHT, &[]);
    marks.mark();
    p.prepare_gutter(device, queue, WIDTH, HEIGHT)?;
    marks.mark();
    p.prepare_debug(device, queue, WIDTH, HEIGHT)?;
    marks.mark();
    p.prepare_hud(device, queue, WIDTH, HEIGHT)?;
    p.flush_float_panel(device, queue, WIDTH, HEIGHT);
    marks.mark();
    p.prepare_whichkey(device, queue, WIDTH, HEIGHT)?;
    marks.mark();
    // prepare_spell_layer, split: rect building vs GPU upload
    let squiggles = p.spell_squiggles();
    marks.mark();
    p.spell_pipeline
        .prepare(device, queue, WIDTH, HEIGHT, &squiggles);
    marks.mark();
    let nits = p.nit_underlines();
    marks.mark();
    p.nit_pipeline.prepare(device, queue, WIDTH, HEIGHT, &nits);
    marks.mark();
    p.prepare_blur(device, queue, WIDTH, HEIGHT);
    marks.mark();

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("awl frame bench encoder"),
    });
    p.render(&mut encoder, target_view)?;
    let cmd = encoder.finish();
    marks.mark();
    queue.submit(Some(cmd));
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .context("device poll failed")?;
    marks.mark();
    p.atlas.trim();
    marks.mark();

    assert_eq!(marks.i, STAGE_NAMES.len(), "stage marks out of lockstep");
    Ok(ft0.elapsed().as_nanos())
}

fn profile_doc(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    cache: &Cache,
    spell: &crate::spell::SpellChecker,
    name: &str,
) -> anyhow::Result<()> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(name);
    let buffer = Buffer::from_file(&path);
    let text = buffer.text();
    let misspelled = spell.misspellings_for(&text, buffer.syntax_lang());
    let lines = text.lines().count();

    let mut p = TextPipeline::new(device, queue, cache, FORMAT);
    // Mirror the live App's wiring order: the surface size first (physical
    // pixels, `Gpu::new`), then the display scale factor (`App::resumed`),
    // then the first view sync.
    p.set_size(WIDTH as f32, HEIGHT as f32);
    p.set_dpi(DPI);
    let view = live_view(&buffer, misspelled.clone());
    p.set_view(&view);

    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("awl frame bench target"),
        size: wgpu::Extent3d {
            width: WIDTH,
            height: HEIGHT,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());

    let mut marks = Marks::new(STAGE_NAMES.len());
    let mut totals: Vec<u128> = Vec::with_capacity(FRAMES);
    let mut ema: Option<f32> = None;

    for frame in 0..(WARMUP + FRAMES) {
        let timed = frame >= WARMUP;
        marks.begin(timed);
        let ns = run_one_frame(&mut p, device, queue, &target_view, &mut marks, frame, ema)?;
        if timed {
            totals.push(ns);
        }
        let ms = ns as f32 / 1.0e6;
        ema = Some(ema.map_or(ms, |e| e * 0.9 + ms * 0.1));
    }

    let total_med = median(totals.clone());
    println!();
    println!(
        "==== {name}: {lines} lines · {} misspellings (live SpellChecker scan) ====",
        misspelled.len()
    );
    if let Some((words, mins)) = p.readout_report() {
        println!("     ({words} words · {mins} min read)");
    }
    println!(
        "{:>29} | {:>10} | {:>10}",
        "stage", "median ms", "% of total"
    );
    println!("{:->29}-+-{:->10}-+-{:->10}", "", "", "");
    let mut sum_med: u128 = 0;
    for (i, stage) in STAGE_NAMES.iter().enumerate() {
        let med = median(marks.samples[i].clone());
        sum_med += med;
        println!(
            "{:>29} | {:>10.3} | {:>9.1}%",
            stage,
            med as f64 / 1.0e6,
            med as f64 / total_med as f64 * 100.0
        );
    }
    println!("{:->29}-+-{:->10}-+-{:->10}", "", "", "");
    println!(
        "{:>29} | {:>10.3} | {:>9.1}%",
        "TOTAL (median frame)",
        total_med as f64 / 1.0e6,
        100.0
    );
    let gap = total_med as i128 - sum_med as i128;
    println!(
        "{:>29} | {:>10.3} | gap {:+.3} ms ({:+.1}% of total)",
        "sum of stage medians",
        sum_med as f64 / 1.0e6,
        gap as f64 / 1.0e6,
        gap as f64 / total_med as f64 * 100.0
    );

    let mut sv = Vec::with_capacity(41);
    for _ in 0..41 {
        let t0 = Instant::now();
        p.set_view(&view);
        sv.push(t0.elapsed().as_nanos());
    }
    println!(
        "  set_view (per input EVENT — sync_view; NOT per frame): median {:.3} ms over {} calls",
        median(sv.clone()) as f64 / 1.0e6,
        sv.len()
    );
    // The markdown word-count readout scan: the persistent readout moved into
    // the held HUD, so this O(doc) scan runs only while the HUD is HELD (and
    // for the capture sidecar) — never in the hot loop. Timed to close it out.
    let mut ro = Vec::with_capacity(41);
    for _ in 0..41 {
        let t0 = Instant::now();
        std::hint::black_box(p.readout_report());
        ro.push(t0.elapsed().as_nanos());
    }
    println!(
        "  readout_report word-count scan (HUD-held/sidecar only — NOT per frame): median {:.3} ms",
        median(ro.clone()) as f64 / 1.0e6
    );
    Ok(())
}

const BURST_WIDTH: u32 = 5120;
const BURST_HEIGHT: u32 = 2756;
const BURST_ZOOM: f32 = 1.1;

const ZOOM_BURST_WIDTH: u32 = 3538;
const ZOOM_BURST_HEIGHT: u32 = 2610;
const ZOOM_BURST_START: f32 = 0.6;
const ZOOM_BURST_LEVELS: [f32; 5] = [0.7, 0.8, 0.7, 0.6, 0.7];
const ZOOM_BURST_SAMPLES: usize = 7;

/// The burst route: every hop lands on a world with a DIFFERENT display face
/// than the previous one (see `theme/worlds.rs` FONT_THEME_FACES), so each switch takes
/// `sync_theme`'s font-reshape branch — exactly what arrowing through the
/// faceted picker does. Starts from Mangrove (JetBrains Mono, the user's world)
/// and returns to it, so lap 2 replays the identical face sequence.
const BURST_WORLDS: [&str; 10] = [
    "Gumtree",  // Literata
    "Bilby",    // Newsreader 16pt 16pt
    "Saltpan",  // Fraunces 9pt
    "Quokka",   // IBM Plex Sans
    "Bombora",  // EB Garamond
    "Mulga",    // Zilla Slab
    "Tawny",    // IBM Plex Mono
    "Mopoke",   // Bitter
    "Galah",    // Figtree
    "Mangrove", // JetBrains Mono (back to the start face)
];

/// Run the THEME-BURST profiler: N successive font-changing theme switches over
/// CLAUDE.md (real spell load) at the user geometry, timing `sync_theme` (the
/// reshape) AND the first full frame after EACH switch (where glyphon
/// rasterizes the new face's visible glyphs into the atlas), split per stage.
/// Two laps over the same worlds: lap 1 rasterizes every face COLD; lap 2
/// re-visits them, showing whether the atlas retained the faces (`atlas.trim`
/// only clears the per-frame in-use set — eviction is LRU under allocation
/// pressure — so a big enough atlas keeps them hot).
pub fn run_theme_burst() -> anyhow::Result<()> {
    pollster::block_on(theme_burst_async())
}

async fn theme_burst_async() -> anyhow::Result<()> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .map_err(|e| anyhow::anyhow!("no wgpu adapter for theme-burst bench: {e:?}"))?;
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("awl theme burst device"),
            ..Default::default()
        })
        .await?;
    let cache = Cache::new(&device);

    crate::debug::set_debug_on(true);
    crate::page::set_page_on(true);
    crate::theme::set_active_by_name("Mangrove");

    let spell = crate::spell::SpellChecker::new(crate::spell::DictVariant::EnUs)
        .map_err(|e| anyhow::anyhow!("spell checker failed to load: {e}"))?;
    println!(
        "theme-burst profiler — {BURST_WIDTH}x{BURST_HEIGHT} @{DPI}x · zoom {BURST_ZOOM} · page ON · debug ON"
    );
    println!(
        "per switch: sync_theme (color retint + font reshape) | first frame after, split into\n\
         text prepare (glyphon shape walk + NEW-FACE RASTERIZATION into the atlas) |\n\
         squiggle/nit proto rebuild | rest of prepare | encode+submit+poll | total; then frame 2 (settled)."
    );
    for doc in ["CLAUDE.md", "benches/fixtures/long_bullets.md"] {
        burst_doc(&device, &queue, &cache, &spell, doc)?;
    }
    Ok(())
}

fn burst_doc(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    cache: &Cache,
    spell: &crate::spell::SpellChecker,
    doc: &str,
) -> anyhow::Result<()> {
    crate::theme::set_active_by_name("Mangrove");
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(doc);
    let buffer = Buffer::from_file(&path);
    let text = buffer.text();
    let misspelled = spell.misspellings_for(&text, buffer.syntax_lang());
    let lines = text.lines().count();

    let mut p = TextPipeline::new(device, queue, cache, FORMAT);
    p.set_size(BURST_WIDTH as f32, BURST_HEIGHT as f32);
    p.set_dpi(DPI);
    let mut view = live_view(&buffer, misspelled.clone());
    view.zoom = BURST_ZOOM;
    p.set_view(&view);

    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("awl theme burst target"),
        size: wgpu::Extent3d {
            width: BURST_WIDTH,
            height: BURST_HEIGHT,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());

    println!();
    println!(
        "==== {doc}: {lines} lines · {} misspellings · start world Mangrove ====",
        misspelled.len()
    );

    // Settle: warm the Mangrove atlas exactly like a live editor sitting idle.
    for _ in 0..10 {
        burst_frame(&mut p, device, queue, &target_view, false)?;
    }

    for lap in 1..=2usize {
        let label = if lap == 1 {
            "cold (each face's first-ever rasterization)"
        } else {
            "warm (same faces revisited — atlas retention)"
        };
        println!();
        println!("---- lap {lap}: {label} ----");
        println!(
            "{:>10} | {:>21} | {:>9} | {:>9} | {:>9} | {:>9} | {:>9} | {:>9} | {:>9}",
            "world",
            "face",
            "sync_thm",
            "text prep",
            "spell/nit",
            "rest prep",
            "gpu",
            "frame1",
            "frame2"
        );
        for name in BURST_WORLDS {
            crate::theme::set_active_by_name(name);
            let face = crate::theme::active().font;

            // The live apply path: post_apply_effects -> sync_theme (this is where
            // the font-branch reshape — restyle_all_lines over every line — runs).
            let t0 = Instant::now();
            p.sync_theme();
            let sync_ms = t0.elapsed().as_secs_f64() * 1e3;

            p.set_view(&view);

            let s1 = burst_frame(&mut p, device, queue, &target_view, true)?;
            let s2 = burst_frame(&mut p, device, queue, &target_view, true)?;

            println!(
                "{:>10} | {:>21} | {:>8.1}ms | {:>8.1}ms | {:>8.1}ms | {:>8.1}ms | {:>8.1}ms | {:>8.1}ms | {:>8.1}ms",
                name, face, sync_ms, s1.text, s1.proto, s1.rest, s1.gpu, s1.total, s2.total
            );
        }
    }

    println!();
    println!("---- debounced preview (colors per arrow, ONE deferred reshape at settle) ----");
    println!(
        "{:>10} | {:>21} | {:>10} | {:>9}",
        "world", "face", "colors", "frame"
    );
    let mut worst_arrow: f64 = 0.0;
    for &name in &BURST_WORLDS[..BURST_WORLDS.len() - 1] {
        crate::theme::set_active_by_name(name);
        let face = crate::theme::active().font;
        let t0 = Instant::now();
        p.sync_theme_colors();
        let colors_ms = t0.elapsed().as_secs_f64() * 1e3;
        p.set_view(&view);
        let s = burst_frame(&mut p, device, queue, &target_view, true)?;
        worst_arrow = worst_arrow.max(colors_ms + s.total);
        println!(
            "{:>10} | {:>21} | {:>8.2}ms | {:>7.1}ms",
            name, face, colors_ms, s.total
        );
    }
    let t0 = Instant::now();
    p.sync_theme_font();
    let settle_ms = t0.elapsed().as_secs_f64() * 1e3;
    p.set_view(&view);
    let s = burst_frame(&mut p, device, queue, &target_view, true)?;
    println!(
        "  settle: sync_theme_font {settle_ms:.2}ms + first frame {:.1}ms (worst arrow step {worst_arrow:.1}ms)",
        s.total
    );

    // Suspect #3: per-switch font resolution (resolve_cjk queries the font DB per
    // restyle; a slow system-font query would tax every switch). Timed standalone.
    let mut cj = Vec::with_capacity(41);
    for _ in 0..41 {
        let t0 = Instant::now();
        std::hint::black_box(p.resolve_cjk());
        cj.push(t0.elapsed().as_nanos());
    }
    println!();
    println!(
        "  resolve_cjk (font-DB walk, runs inside each restyle): median {:.3} ms",
        median(cj.clone()) as f64 / 1.0e6
    );
    Ok(())
}

pub fn run_zoom_burst() -> anyhow::Result<()> {
    pollster::block_on(zoom_burst_async())
}

async fn zoom_burst_async() -> anyhow::Result<()> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .map_err(|e| anyhow::anyhow!("no wgpu adapter for zoom-burst bench: {e:?}"))?;
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("awl zoom burst device"),
            ..Default::default()
        })
        .await?;
    let cache = Cache::new(&device);

    crate::debug::set_debug_on(true);
    crate::page::set_page_on(true);
    crate::theme::set_active_by_name("Firetail");
    let spell = crate::spell::SpellChecker::new(crate::spell::DictVariant::EnUs)
        .map_err(|e| anyhow::anyhow!("spell checker failed to load: {e}"))?;
    println!(
        "zoom-burst profiler — {ZOOM_BURST_WIDTH}x{ZOOM_BURST_HEIGHT} @{DPI}x · Firetail · page ON · debug ON"
    );
    println!(
        "burst: 60% -> 70 -> 80 -> 70 -> 60 -> 70; eager = five input-side reflows, coalesced = final level once before present"
    );
    for doc in ["CLAUDE.md", "benches/fixtures/long_bullets.md"] {
        zoom_burst_doc(&device, &queue, &cache, &spell, doc)?;
    }
    Ok(())
}

fn zoom_burst_doc(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    cache: &Cache,
    spell: &crate::spell::SpellChecker,
    doc: &str,
) -> anyhow::Result<()> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(doc);
    let buffer = Buffer::from_file(&path);
    let text = buffer.text();
    let misspelled = spell.misspellings_for(&text, buffer.syntax_lang());
    let lines = text.lines().count();
    let mut view = live_view(&buffer, misspelled);
    view.zoom = ZOOM_BURST_START;

    let mut p = TextPipeline::new(device, queue, cache, FORMAT);
    p.set_size(ZOOM_BURST_WIDTH as f32, ZOOM_BURST_HEIGHT as f32);
    p.set_dpi(DPI);
    p.set_view(&view);
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("awl zoom burst target"),
        size: wgpu::Extent3d {
            width: ZOOM_BURST_WIDTH,
            height: ZOOM_BURST_HEIGHT,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());
    for _ in 0..5 {
        zoom_frame(&mut p, device, queue, &target_view)?;
    }

    let mut eager_layout = Vec::with_capacity(ZOOM_BURST_SAMPLES);
    let mut eager_total = Vec::with_capacity(ZOOM_BURST_SAMPLES);
    let mut eager_reshapes = Vec::with_capacity(ZOOM_BURST_SAMPLES);
    let mut coalesced_layout = Vec::with_capacity(ZOOM_BURST_SAMPLES);
    let mut coalesced_total = Vec::with_capacity(ZOOM_BURST_SAMPLES);
    let mut coalesced_reshapes = Vec::with_capacity(ZOOM_BURST_SAMPLES);

    for _ in 0..ZOOM_BURST_SAMPLES {
        view.zoom = ZOOM_BURST_START;
        p.set_view(&view);
        let before = p.reshape_count;
        let total_start = Instant::now();
        let layout_start = Instant::now();
        for zoom in ZOOM_BURST_LEVELS {
            view.zoom = zoom;
            p.set_view(&view);
        }
        eager_layout.push(layout_start.elapsed().as_nanos());
        eager_reshapes.push(p.reshape_count - before);
        zoom_frame(&mut p, device, queue, &target_view)?;
        eager_total.push(total_start.elapsed().as_nanos());

        view.zoom = ZOOM_BURST_START;
        p.set_view(&view);
        let before = p.reshape_count;
        let total_start = Instant::now();
        let layout_start = Instant::now();
        view.zoom = *ZOOM_BURST_LEVELS.last().unwrap();
        p.set_view(&view);
        coalesced_layout.push(layout_start.elapsed().as_nanos());
        coalesced_reshapes.push(p.reshape_count - before);
        zoom_frame(&mut p, device, queue, &target_view)?;
        coalesced_total.push(total_start.elapsed().as_nanos());
    }

    anyhow::ensure!(
        eager_reshapes
            .iter()
            .all(|&n| n == ZOOM_BURST_LEVELS.len() as u64),
        "zoom eager replay did not reshape once per requested level: {eager_reshapes:?}"
    );
    anyhow::ensure!(
        coalesced_reshapes.iter().all(|&n| n == 1),
        "zoom coalesced replay did not reshape exactly once: {coalesced_reshapes:?}"
    );
    let eager_layout_ms = median(eager_layout) as f64 / 1.0e6;
    let eager_total_ms = median(eager_total) as f64 / 1.0e6;
    let coalesced_layout_ms = median(coalesced_layout) as f64 / 1.0e6;
    let coalesced_total_ms = median(coalesced_total) as f64 / 1.0e6;
    println!();
    println!("==== {doc}: {lines} lines ====");
    println!(
        "{:>11} | {:>8} | {:>12} | {:>18}",
        "route", "reflows", "layout", "layout+first frame"
    );
    println!(
        "{:>11} | {:>8} | {:>10.1} ms | {:>16.1} ms",
        "eager",
        ZOOM_BURST_LEVELS.len(),
        eager_layout_ms,
        eager_total_ms
    );
    println!(
        "{:>11} | {:>8} | {:>10.1} ms | {:>16.1} ms",
        "coalesced", 1, coalesced_layout_ms, coalesced_total_ms
    );
    println!(
        "  saved: {:.1} ms median ({:.1}x end-to-end)",
        eager_total_ms - coalesced_total_ms,
        eager_total_ms / coalesced_total_ms.max(0.001)
    );
    Ok(())
}

/// One complete first frame at the reported zoom geometry. `prepare` is the
/// live pipeline's real aggregate, and the blocking poll serializes submitted
/// GPU work into the measurement just like the other frame profilers here.
fn zoom_frame(
    p: &mut TextPipeline,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    target_view: &wgpu::TextureView,
) -> anyhow::Result<()> {
    p.advance(DT);
    p.set_debug_perf(None, None, Some(1), false, Some(1000.0 / 60.0));
    p.prepare(device, queue, ZOOM_BURST_WIDTH, ZOOM_BURST_HEIGHT)?;
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("awl zoom burst encoder"),
    });
    p.render(&mut encoder, target_view)?;
    queue.submit(Some(encoder.finish()));
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .context("device poll failed")?;
    p.atlas.trim();
    Ok(())
}

const FROST_DOC: &str = "CLAUDE.md";

pub fn run_frost() -> anyhow::Result<()> {
    pollster::block_on(frost_async())
}

async fn frost_async() -> anyhow::Result<()> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await
        .map_err(|e| anyhow::anyhow!("no wgpu adapter for frost bench: {e:?}"))?;
    let (device, queue) = adapter
        .request_device(&wgpu::DeviceDescriptor {
            label: Some("awl frost bench device"),
            ..Default::default()
        })
        .await?;
    let cache = Cache::new(&device);

    crate::debug::set_debug_on(true);
    crate::page::set_page_on(true);
    crate::page::set_measure(72);
    crate::outline::set_outline_on(true);
    let spell = crate::spell::SpellChecker::new(crate::spell::DictVariant::EnUs)
        .map_err(|e| anyhow::anyhow!("spell checker failed to load: {e}"))?;

    let per_glyph = crate::lava::FROST_SEED_PER_GLYPH;
    println!(
        "frost profiler — {WIDTH}x{HEIGHT} @{DPI}x · page ON · outline ON · debug ON · \
         seed mode: {} · {WARMUP} warmup + {FRAMES} timed frames",
        if per_glyph {
            "PER-GLYPH"
        } else {
            "WORD-RUN (degradation arm)"
        }
    );
    println!(
        "(headless: submit+poll SERIALIZES the GPU cost; the window overlaps it and adds present/acquire)"
    );
    for world in ["Mangrove", "Firetail"] {
        frost_world(&device, &queue, &cache, &spell, world)?;
    }
    crate::page::set_page_on(false);
    crate::page::set_measure(80);
    crate::outline::set_outline_on(false);
    crate::theme::set_active(crate::theme::DEFAULT_THEME);
    Ok(())
}

fn frost_world(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    cache: &Cache,
    spell: &crate::spell::SpellChecker,
    world: &str,
) -> anyhow::Result<()> {
    crate::theme::set_active_by_name(world);
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(FROST_DOC);
    let buffer = Buffer::from_file(&path);
    let text = buffer.text();
    let misspelled = spell.misspellings_for(&text, buffer.syntax_lang());
    let mut view = live_view(&buffer, misspelled);
    view.zoom = 1.0;

    let mut p = TextPipeline::new(device, queue, cache, FORMAT);
    p.set_size(WIDTH as f32, HEIGHT as f32);
    p.set_dpi(DPI);
    p.set_view(&view);
    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("awl frost bench target"),
        size: wgpu::Extent3d {
            width: WIDTH,
            height: HEIGHT,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());

    // Warm the atlas + settle the caret spring, then WITNESS the seed field.
    for f in 0..WARMUP {
        frost_frame(&mut p, device, queue, &target_view, f)?;
    }
    let seed_count = p.frost_seed_count();
    if crate::lava::frost_on() {
        anyhow::ensure!(
            seed_count > 0,
            "{world}: the frost seed field is EMPTY after warmup — the bench is not \
             measuring the organic frost (no outline/gutter ink seeded?)"
        );
    }

    let rebuilds_before = p.frost_seed_rebuilds;
    let mut totals = Vec::with_capacity(FRAMES);
    for f in 0..FRAMES {
        totals.push(frost_frame(
            &mut p,
            device,
            queue,
            &target_view,
            WARMUP + f,
        )?);
    }
    let steady_rebuilds = p.frost_seed_rebuilds - rebuilds_before;
    anyhow::ensure!(
        steady_rebuilds == 0,
        "{world}: {steady_rebuilds} frost seed rebuilds across {FRAMES} warm steady \
         frames (expected 0 — the cache is churning)"
    );
    let med_ms = median(totals) as f64 / 1.0e6;

    if crate::lava::frost_on() {
        let before = p.frost_seed_rebuilds;
        view.zoom = 1.25;
        p.set_view(&view);
        frost_frame(&mut p, device, queue, &target_view, 0)?;
        let zoom_rebuilds = p.frost_seed_rebuilds - before;
        anyhow::ensure!(
            zoom_rebuilds == 1,
            "{world}: a zoom step rebuilt the frost seed field {zoom_rebuilds} times (expected exactly 1)"
        );

        let before = p.frost_seed_rebuilds;
        view.gutter_name = "renamed-fixture.md".to_string();
        p.set_view(&view);
        frost_frame(&mut p, device, queue, &target_view, 0)?;
        let text_rebuilds = p.frost_seed_rebuilds - before;
        anyhow::ensure!(
            text_rebuilds == 1,
            "{world}: a margin-text change rebuilt the frost seed field {text_rebuilds} times (expected exactly 1)"
        );
    }

    let witness = if crate::lava::frost_on() {
        "steady rebuilds 0 | +1 on zoom | +1 on margin-text"
    } else {
        "FLOOR (AWL_LAVA_FROST=off — raw lamp, no seed field)"
    };
    println!("  {world:<9} | median frame {med_ms:>7.3} ms | seeds {seed_count:>4} | {witness}");
    Ok(())
}

/// One complete steady frame (the live `RedrawRequested` body: advance → prepare
/// aggregate → encode → submit+poll → trim), returning its total nanoseconds. The
/// blocking poll serializes submitted GPU work into the number like the sibling
/// profilers.
fn frost_frame(
    p: &mut TextPipeline,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    target_view: &wgpu::TextureView,
    frame: usize,
) -> anyhow::Result<u128> {
    let t0 = Instant::now();
    p.advance(DT);
    p.set_debug_perf(None, None, Some(frame as u64), false, Some(1000.0 / 60.0));
    p.prepare(device, queue, WIDTH, HEIGHT)?;
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("awl frost bench encoder"),
    });
    p.render(&mut encoder, target_view)?;
    queue.submit(Some(encoder.finish()));
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .context("device poll failed")?;
    p.atlas.trim();
    Ok(t0.elapsed().as_nanos())
}

/// One frame's coarse stage split (ms): the glyphon text prepare (shape walk +
/// atlas rasterization), the squiggle+nit proto rebuild + upload, everything
/// else in `prepare`, the encode+submit+poll GPU tail, and the total.
struct BurstSplit {
    text: f64,
    proto: f64,
    rest: f64,
    gpu: f64,
    total: f64,
}

/// Run ONE live-shaped frame (the exact `RedrawRequested` body the frame
/// profiler above replays: advance → prepare sub-calls in order → encode →
/// submit+poll → trim) against the burst target, returning the coarse split.
fn burst_frame(
    p: &mut TextPipeline,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    target_view: &wgpu::TextureView,
    timed: bool,
) -> anyhow::Result<BurstSplit> {
    let (w, h) = (BURST_WIDTH, BURST_HEIGHT);
    let ft0 = Instant::now();
    p.advance(DT);
    p.set_debug_perf(None, None, Some(1), false, Some(1000.0 / 60.0));
    p.sync_wrap_width();
    p.viewport.update(
        queue,
        Resolution {
            width: w,
            height: h,
        },
    );
    p.prepare_background_layer(queue, w, h);

    let t_text = Instant::now();
    p.prepare_text_layer(device, queue, w, h)?;
    let text_ms = t_text.elapsed().as_secs_f64() * 1e3;

    let t_rest = Instant::now();
    p.prepare_caret_layer(device, queue, w, h);
    p.prepare_selection_layer(device, queue, w, h);
    p.prepare_ornaments(device, queue, w, h)?;
    p.prepare_table_grid(device, queue, w, h)?;
    p.prepare_caret_preview_panel(device, queue, w, h)?;
    p.panel_card.prepare(device, queue, w, h, &[]);
    p.panel_shadow.prepare(device, queue, w, h, &[]);
    p.panel_border.prepare(device, queue, w, h, &[]);
    p.overlay_rows.prepare(device, queue, w, h, &[]);
    p.prepare_gutter(device, queue, w, h)?;
    p.prepare_debug(device, queue, w, h)?;
    p.prepare_hud(device, queue, w, h)?;
    p.prepare_whichkey(device, queue, w, h)?;
    let rest_ms = t_rest.elapsed().as_secs_f64() * 1e3;

    // The proto-cache rebuild (suspect #4): the RowGeom generation bump after a
    // reshape forces the squiggle + nit rect rebuilds here.
    let t_proto = Instant::now();
    let squiggles = p.spell_squiggles();
    p.spell_pipeline.prepare(device, queue, w, h, &squiggles);
    let nits = p.nit_underlines();
    p.nit_pipeline.prepare(device, queue, w, h, &nits);
    let proto_ms = t_proto.elapsed().as_secs_f64() * 1e3;

    p.prepare_blur(device, queue, w, h);

    let t_gpu = Instant::now();
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("awl theme burst encoder"),
    });
    p.render(&mut encoder, target_view)?;
    queue.submit(Some(encoder.finish()));
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .context("device poll failed")?;
    let gpu_ms = t_gpu.elapsed().as_secs_f64() * 1e3;
    p.atlas.trim();

    let total_ms = ft0.elapsed().as_secs_f64() * 1e3;
    let _ = timed;
    Ok(BurstSplit {
        text: text_ms,
        proto: proto_ms,
        rest: rest_ms,
        gpu: gpu_ms,
        total: total_ms,
    })
}

// ============================================================================
// BENCH-MUST-WITNESS-THE-WORK (CLAUDE.md's own rule): the "wash layer (cull +
// upload)" stage added this round used to be entirely UNCALLED in this bench's
// replayed sequence — not folded into a neighbor's number, just skipped, so a
// reader of the printed table would never know the cost was missing. This
// confirms, on the same class of content the real fixtures (`CAPTURE.md` /
// `CLAUDE.md`) carry — a fenced code block with a prose comment + a string
// literal, inheriting the wash through the markdown seam (see
// `render::tests::washes::markdown_fence_inherits_washes`) — that the stage now
// does REAL, nonzero cull+upload work rather than timing an empty prepare call.
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn headless_dqp() -> Option<(wgpu::Device, wgpu::Queue, TextPipeline)> {
        pollster::block_on(async {
            let instance =
                wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions::default())
                .await
                .ok()?;
            let (device, queue) = adapter
                .request_device(&wgpu::DeviceDescriptor {
                    label: Some("awl framebench test device"),
                    ..Default::default()
                })
                .await
                .ok()?;
            let cache = Cache::new(&device);
            let mut p = TextPipeline::new(&device, &queue, &cache, FORMAT);
            p.set_size(WIDTH as f32, HEIGHT as f32);
            p.set_dpi(DPI);
            Some((device, queue, p))
        })
    }

    /// A cheap, no-GPU-needed sanity check: both stages this rescue round named
    /// (the new wash stage, and the pre-existing table-grid stage that turned
    /// out to have NO name at all — see the module doc above) are present.
    /// Holds even on a machine with no wgpu adapter, where the GPU-backed test
    /// below skips.
    #[test]
    fn stage_names_include_wash_and_table_grid() {
        assert!(
            STAGE_NAMES.contains(&"wash layer (cull + upload)"),
            "the wash-layer stage must be named in STAGE_NAMES: {STAGE_NAMES:?}"
        );
        assert!(
            STAGE_NAMES.contains(&"table grid (grid geometry)"),
            "the table-grid stage must be named in STAGE_NAMES: {STAGE_NAMES:?}"
        );
    }

    #[test]
    fn wash_layer_and_table_grid_stages_stay_in_lockstep() {
        let _g = crate::testlock::serial();
        let _world = crate::theme::WorldPin::snapshot();
        let Some((device, queue, mut p)) = headless_dqp() else {
            eprintln!(
                "skipping wash_layer_and_table_grid_stages_stay_in_lockstep: no wgpu adapter"
            );
            return;
        };
        // Pin a DARK world explicitly — the STRING wash bucket only uploads on
        // dark worlds (`role_style_for`'s documented rule; light worlds carry
        // string identity in the fg tint alone), and the process-global active
        // theme's own default (`theme::DEFAULT_THEME` = Saltpan, a LIGHT world)
        // would otherwise make this test's outcome depend on whichever OTHER
        // test happened to run first in the process and leave a dark world
        // active — exactly the kind of order-dependent flake this codebase's
        // `testlock::serial()` discipline exists to rule out.
        crate::theme::set_active_by_name("Tawny").unwrap();
        let text = "prose before\n```sh\n# a comment\nexport PATH=\"/usr/bin\"\n```\nprose after\n";
        let view = ViewState {
            text: text.to_string(),
            is_markdown: true,
            ..ViewState::base()
        };
        p.set_view(&view);

        let (comments, strings, _highlights) = p.wash_rects();
        assert!(
            !comments.is_empty(),
            "the fenced comment must produce wash geometry: {comments:?}"
        );
        assert!(
            !strings.is_empty(),
            "the fenced string literal must produce wash geometry: {strings:?}"
        );

        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("awl framebench test target"),
            size: wgpu::Extent3d {
                width: WIDTH,
                height: HEIGHT,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let target_view = target.create_view(&wgpu::TextureViewDescriptor::default());

        let mut marks = Marks::new(STAGE_NAMES.len());
        marks.begin(true);
        run_one_frame(&mut p, &device, &queue, &target_view, &mut marks, 0, None)
            .expect("one bench frame must run cleanly");
        assert_eq!(
            marks.i,
            STAGE_NAMES.len(),
            "stage marks must stay in lockstep with STAGE_NAMES"
        );

        assert!(
            p.wash_comment_pipeline.instance_count() > 0,
            "prepare_wash_layer must upload the comment wash instances it built"
        );
        assert!(
            p.wash_string_pipeline.instance_count() > 0,
            "prepare_wash_layer must upload the string wash instances it built"
        );
    }
}
