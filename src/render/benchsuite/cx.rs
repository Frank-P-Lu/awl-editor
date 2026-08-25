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

/// THE NAMED PLANNING PASSES one timed overlay frame is entitled to.
///
/// Deliberately NOT a number. "Some number of plans is fine" protects nothing;
/// every pass is named, carries its owner, and is decided by an oracle derived
/// from that pass's own OUTCOME rather than from the predicate the pipeline
/// branches on. A pass nobody named is a third plan, and a third plan is the
/// defect this witness exists to catch.
pub(in crate::render) struct FramePasses {
    /// A RIGHT-ANCHORED card HUGS its content, so before the drawn geometry can
    /// be resolved the pipeline shapes the candidate rows against a PROVISIONAL
    /// wide-cap geometry and measures their widest ink
    /// (`measure_overlay_content_w`, inside `set_view`). That measurement plans
    /// its own provisional card — a genuinely different card from the one this
    /// frame draws, and it cannot be the frame's plan, because the frame's
    /// geometry is downstream of its result.
    ///
    /// The oracle is the measurement's own PRODUCT — the hug width it shaped and
    /// cached — not `overlay_right_anchored()`, the predicate the pipeline itself
    /// branches on. An oracle reading the same switch as the code agrees with it
    /// by construction: the planner's own device law was once satisfied because
    /// the sidecar and the plan both read `selected_display()`, so a planner that
    /// forgot grouped headers kept them in perfect agreement while pointing at
    /// the wrong row.
    pub content_hug: bool,
}

impl FramePasses {
    /// Read on a frame with the card OPEN — the hug width is cleared the moment
    /// the overlay closes.
    pub(in crate::render) fn observe(p: &TextPipeline) -> Self {
        Self {
            content_hug: p.overlay_content_w > 0.0,
        }
    }

    /// The frame's OWN drawn plan, always, plus whichever named pass this world
    /// really ran.
    fn per_frame(&self) -> u64 {
        1 + u64::from(self.content_hug)
    }

    fn names(&self) -> String {
        let mut named = vec!["the frame's own drawn plan"];
        if self.content_hug {
            named.push("the right-anchored content-hug measurement (`measure_overlay_content_w`)");
        }
        named.join(" + ")
    }
}

/// The SCENE-PLAN witness for a scenario that opens an overlay, so a bench cannot
/// "measure" a card while the planner does nothing (the theme bench that once
/// reported ~5 ms with zero reshapes). Marked before the timed samples and
/// settled after, against the two invariants the planner promises.
pub(in crate::render) struct PlanWitness {
    plans: u64,
    rows: u64,
}

impl PlanWitness {
    pub(in crate::render) fn mark() -> Self {
        let (plans, rows) = crate::render::plan::plan_witness();
        Self { plans, rows }
    }

    /// `(plans per frame, mean planned rows per plan)` since the mark. Exactly
    /// the NAMED passes per timed frame — zero means nothing planned, an
    /// unnamed extra means a consumer grew its own plan instead of reading the
    /// frame's — and each plan bounded by the drawn window rather than the
    /// `items`-row corpus.
    pub(in crate::render) fn settle(
        &self,
        frames: u64,
        passes: &FramePasses,
        window_rows: usize,
        items: u64,
    ) -> Result<(u64, u64)> {
        let (plans_now, rows_now) = crate::render::plan::plan_witness();
        let plans = plans_now - self.plans;
        let rows = rows_now - self.rows;
        let per_frame = passes.per_frame();
        ensure!(
            plans == frames * per_frame,
            "the scene planner must run exactly once per timed frame per NAMED pass \
             ({plans} plans over {frames} frames, expected {per_frame} each: {}) — \
             an extra plan is a consumer that grew its own instead of reading the frame's",
            passes.names()
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
    /// THE POSTURE WORLD this context was built under — the suite's one world for
    /// the whole matrix. A scenario that moves the global theme restores THIS,
    /// never a constant, or every cell after it in the tier would silently
    /// measure the default world instead of the posed one.
    pub world: usize,
}

impl<'a> Cx<'a> {
    /// Push a built overlay into the view (the palette scenario's timed half) and
    /// tear it back down (its untimed half), so the scenario body stays readable.
    pub(super) fn open_overlay(&mut self, ov: &crate::overlay::OverlayState) {
        self.view.overlay_active = true;
        self.view.overlay_crisp = false;
        self.view.overlay_title = ov.title();
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
        self.view.overlay_title = String::new();
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
            world: crate::theme::active_index(),
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

/// THE SELECTED ROW'S OWN SURFACE, uploaded — whichever pipeline this world's
/// list style draws it with. A no-wildcard match, so a new `ListStyle` cannot
/// quietly fall through to a pipeline it never fills.
///
/// The suite ran only the default world until the posture world became settable,
/// so this used to read `overlay_rows` outright. That is the flat/poster
/// families' band quad; a DIAGONAL world deliberately draws no row fill at all
/// (`overlay_prepare_selection`'s `Diagonal` arm) and carries focus on the bright
/// local spine segment instead — so on a diagonal world the old read was
/// structurally zero and the cell failed on a witness looking at the wrong
/// surface.
pub(super) fn selected_row_surface_instances(cx: &Cx) -> u64 {
    match crate::render::effective_list_style() {
        // `Ruled` joins them rather than `Diagonal`: its selection mark is a
        // quad on the SAME pipeline, only never a row-tall one, so this witness
        // is looking at the right surface without any new plumbing.
        crate::theme::ListStyle::Pane
        | crate::theme::ListStyle::Bars
        | crate::theme::ListStyle::Ruled(_) => cx.p.overlay_rows.instance_count() as u64,
        crate::theme::ListStyle::Diagonal(_) => cx.p.overlay_spine_selected.instance_count() as u64,
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
