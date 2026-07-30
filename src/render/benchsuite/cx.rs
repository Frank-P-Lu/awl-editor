//! Per-tier bench CONTEXT for the `--bench-suite` scenarios: one pipeline +
//! offscreen target + the corpus text, shaped once and warmed, plus the two
//! per-step primitives every scenario shares — the live-shaped FRAME (the
//! exact `RedrawRequested` aggregate, GPU serialized by a blocking poll, the
//! same shape [`super::super::framebench`]'s zoom profiler replays) and the
//! pixel SNAPSHOT (the capture harness's own `read_frame` readback, feeding
//! the outcome witnesses). Split out of [`super::scenarios`] purely for the
//! ~500-line file ceiling; the seams are unchanged.

use anyhow::{Context as _, Result, ensure};

use crate::buffer::Buffer;
use crate::clock::Instant;
use crate::config::Config;

use super::corpus::{self, Tier};
use super::{DPI, DT, HEIGHT, WIDTH};
use crate::render::{TextPipeline, ViewState};

/// ITEM 174 — the SCENE-PLAN witness for a scenario that opens an overlay, so a
/// bench cannot "measure" a card while the planner does nothing (the theme bench
/// that once reported ~5 ms with zero reshapes). Marked before the timed samples
/// and settled after, against the two invariants the planner promises.
pub(super) struct PlanWitness {
    plans: u64,
    rows: u64,
}

impl PlanWitness {
    pub(super) fn mark() -> Self {
        let (plans, rows) = crate::render::plan::plan_witness();
        Self { plans, rows }
    }

    /// `(plans, mean planned rows per plan)` since the mark. EXACTLY one plan per
    /// timed frame — zero means nothing planned, more than one means a consumer
    /// grew its own plan instead of reading the frame's — and each plan bounded by
    /// the drawn window rather than the `items`-row corpus.
    pub(super) fn settle(&self, frames: u64, window_rows: usize, items: u64) -> Result<(u64, u64)> {
        let (plans_now, rows_now) = crate::render::plan::plan_witness();
        let plans = plans_now - self.plans;
        let rows = rows_now - self.rows;
        ensure!(
            plans == frames,
            "the scene planner must run exactly once per timed frame ({plans} plans over {frames})"
        );
        let mean = rows / plans.max(1);
        ensure!(
            mean > 0 && mean <= window_rows as u64,
            "each plan must hold 1..={window_rows} rows ({mean} per plan over {items} items) \
             — the per-frame plan is O(visible), never O(doc)"
        );
        Ok((plans / frames, mean))
    }
}

/// Per-tier bench context: one pipeline + offscreen target + the corpus text,
/// shaped once and warmed before the scenarios run.
pub(super) struct Cx<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub config: &'a Config,
    pub p: TextPipeline,
    pub texture: wgpu::Texture,
    pub tview: wgpu::TextureView,
    pub text: String,
    /// A `Buffer` over the same text — the char<->line/col oracle the search
    /// scenario needs (the same conversion the capture harness performs).
    pub buffer: Buffer,
    pub view: ViewState,
    pub lines: usize,
    pub words: u64,
    /// The canvas width currently fed to `prepare` (the resize scenario steps it).
    pub width: u32,
}

impl<'a> Cx<'a> {
    /// Push a built overlay into the view (the palette scenario's timed half) and
    /// tear it back down (its untimed half), so the scenario body stays readable.
    pub(super) fn open_overlay(&mut self, ov: &crate::overlay::OverlayState) {
        self.view.overlay_active = true;
        self.view.overlay_crisp = false;
        self.view.overlay_title = ov.kind.title();
        self.view.overlay_query = String::new();
        self.view.overlay_items = ov.item_strings();
        self.view.overlay_bindings = ov.item_bindings();
        self.view.overlay_git = ov.item_git_tags();
        self.view.overlay_empty = ov.empty_notice();
        self.view.overlay_selected = ov.selected;
        self.view.overlay_scroll = ov.scroll;
        self.view.overlay_window_rows = ov.window_rows();
        self.view.overlay_hint = ov.foot_hint();
    }

    pub(super) fn close_overlay(&mut self) {
        self.view.overlay_active = false;
        self.view.overlay_items = Vec::new();
        self.view.overlay_bindings = Vec::new();
        self.view.overlay_git = Vec::new();
        self.view.overlay_title = "";
        self.view.overlay_hint = String::new();
        self.view.overlay_empty = None;
    }

    pub(super) fn new(
        device: &'a wgpu::Device,
        queue: &'a wgpu::Queue,
        cache: &glyphon::Cache,
        config: &'a Config,
        tier: Tier,
        text: String,
        misspelled: Vec<crate::spell::Misspelling>,
    ) -> Result<Self> {
        let mut p = TextPipeline::new(device, queue, cache, crate::capture::FORMAT);
        p.set_size(WIDTH as f32, HEIGHT as f32);
        p.set_dpi(DPI);
        let buffer = Buffer::from_str(&text);
        let lines = text.lines().count();
        let words = corpus::count_words(&text);
        let view = ViewState {
            text: text.clone(),
            misspelled,
            gutter_name: tier.doc_name().to_string(),
            gutter_project: "bench-suite".to_string(),
            is_markdown: tier.is_markdown(),
            syn_lang: tier.syn_lang(),
            ..ViewState::base()
        };
        p.set_view(&view);
        let (texture, tview) = crate::capture::gpu::offscreen_target(device, WIDTH, HEIGHT);
        let mut cx = Cx {
            device,
            queue,
            config,
            p,
            texture,
            tview,
            text,
            buffer,
            view,
            lines,
            words,
            width: WIDTH,
        };
        // Warm the atlas + caches like an editor sitting on the open document.
        for _ in 0..3 {
            cx.frame()?;
        }
        Ok(cx)
    }

    /// One live-shaped frame at the CURRENT canvas width: the exact
    /// `RedrawRequested` aggregate, GPU serialized by the blocking poll.
    pub(super) fn frame(&mut self) -> Result<()> {
        self.p.advance(DT);
        self.p
            .prepare(self.device, self.queue, self.width, HEIGHT)?;
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("awl bench suite encoder"),
            });
        self.p.render(&mut encoder, &self.tview)?;
        self.queue.submit(Some(encoder.finish()));
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .context("device poll failed")?;
        self.p.atlas.trim();
        Ok(())
    }

    /// Push the working view and draw one frame (the per-step unit most
    /// scenarios time).
    pub(super) fn sync_frame(&mut self) -> Result<()> {
        self.p.set_view(&self.view);
        self.frame()
    }

    /// Read the offscreen target back for a pixel witness.
    pub(super) fn snapshot(&mut self) -> Result<image::RgbaImage> {
        crate::capture::gpu::read_frame(self.device, self.queue, &self.texture, WIDTH, HEIGHT)
    }
}

/// Count differing pixels between two equally-sized snapshots.
pub(super) fn differing_pixels(a: &image::RgbaImage, b: &image::RgbaImage) -> u64 {
    a.pixels().zip(b.pixels()).filter(|(x, y)| x != y).count() as u64
}

/// Elapsed milliseconds since `t0` (the per-sample unit).
pub(super) fn ms(t0: Instant) -> f64 {
    t0.elapsed().as_secs_f64() * 1e3
}
