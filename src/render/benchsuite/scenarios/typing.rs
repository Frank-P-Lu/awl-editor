//! Production preparation and offscreen rendering typing workload for typing.

use super::super::cx::FramePhases;
use super::*;

const KEYS: usize = 30;

struct LiveRun {
    spell: crate::spell::SpellChecker,
    spell_projection: crate::spell::SpellProjection,
    buffer: crate::buffer::Buffer,
    spell_cache: Vec<crate::spell::SpellVerdict>,
    sync_text_cache: String,
    timings: LiveTimings,
}

impl LiveRun {
    fn prime(cx: &mut Cx) -> Result<Self> {
        let spell = crate::spell::SpellChecker::new(crate::spell::DictVariant::EnUs)
            .map_err(|e| anyhow::anyhow!("spell checker failed to load: {e}"))?;
        let mut spell_projection = crate::spell::SpellProjection::default();
        let mut buffer = crate::buffer::Buffer::from_str(&cx.text);
        buffer.set_path(std::path::PathBuf::from(cx.view.gutter_name.clone()));
        cx.view.scroll = crate::render::ScrollPos::default();
        cx.view.cursor_line = 0;
        cx.view.is_edit_move = true;

        // Prime the same complete operation untimed so samples measure steady
        // state. Retain both caches exactly as the live document session does.
        buffer.insert_char('z');
        let spell_cache = spell_projection.refresh(&buffer, &spell);
        let text = buffer.text();
        let sync_text_cache = text.clone();
        std::hint::black_box(&sync_text_cache);
        cx.view.cursor_col = 1;
        cx.view.text = text;
        cx.view.misspelled = crate::spell::visible(&spell_cache, &cx.view.text);
        cx.sync_frame()?;
        Ok(Self {
            spell,
            spell_projection,
            buffer,
            spell_cache,
            sync_text_cache,
            timings: LiveTimings::new(),
        })
    }

    fn type_char(&mut self, cx: &mut Cx, ch: char) -> Result<()> {
        let total_at = Instant::now();
        let edit_at = Instant::now();
        self.buffer.insert_char(ch);
        self.timings.edit.push(ms(edit_at));

        // Include replacement and destruction of the prior cache inside the
        // spelling stage, matching the live assignment.
        let spell_at = Instant::now();
        self.spell_cache = self.spell_projection.refresh(&self.buffer, &self.spell);
        self.timings
            .add_spell(ms(spell_at), self.spell_projection.work());

        // Mirror `DocumentSession::sync_text` on an edited version: materialize
        // the rope a second time and retain one owned cached copy.
        let text_at = Instant::now();
        let text = self.buffer.text();
        self.sync_text_cache = text.clone();
        std::hint::black_box(&self.sync_text_cache);
        self.timings.text.push(ms(text_at));

        let visible_at = Instant::now();
        let misspelled = crate::spell::visible(&self.spell_cache, &text);
        self.timings.visible.push(ms(visible_at));
        let (line, col) = self.buffer.cursor_line_col();
        cx.view.cursor_line = line;
        cx.view.cursor_col = col;
        cx.view.text = text;
        cx.view.misspelled = misspelled;
        let frame = cx.sync_frame_phases()?;
        self.timings.add_frame(frame);
        self.timings.total.push(ms(total_at));
        Ok(())
    }
}

struct LiveTimings {
    total: Vec<f64>,
    edit: Vec<f64>,
    spell: Vec<f64>,
    text: Vec<f64>,
    visible: Vec<f64>,
    set_view: Vec<f64>,
    prepare: Vec<f64>,
    render: Vec<f64>,
    context: Vec<f64>,
    spans: Vec<f64>,
    lines: Vec<f64>,
    embeds: Vec<f64>,
    splice: Vec<f64>,
    shape: Vec<f64>,
    geometry: Vec<f64>,
    conceal: Vec<f64>,
    caret: Vec<f64>,
    spell_work: crate::spell::SpellRefreshWork,
    geometry_patch_hits: u64,
    geometry_lines_patched: u64,
    geometry_rows_patched: u64,
    geometry_index_probes: u64,
    // Frame-preparation owner witnesses. `_ms` fields are
    // wall-clock SUMS across every key (not medians — a per-owner miss is rare
    // and lumpy, so a sum states the real cumulative cost over the run instead
    // of hiding it in a mostly-zero median). The `_lines`/`_spans`/`_calls`
    // counters are also sums, so "every key rescans the whole document" shows
    // up as `lines_total == doc_lines * KEYS`, not as a single-key sample.
    nit_scan_ms: f64,
    nit_scan_lines: u64,
    nit_scan_misses: u64,
    ornament_scan_ms: f64,
    ornament_scan_lines: u64,
    ornament_scan_spans: u64,
    ornament_scan_misses: u64,
    destination_join_ms: f64,
    destination_join_calls: u64,
    destination_join_bytes: u64,
    squiggle_scan_ms: f64,
    squiggle_scan_misspellings: u64,
    squiggle_scan_misses: u64,
    /// Lines the retained `SquiggleProjection` actually re-looked-up in
    /// `RowGeom` this key (a SUM, same convention as `nit_scan_lines` — the
    /// win is `lines_total == KEYS` on a document with no structural edits,
    /// not `KEYS * lines_with_a_misspelling`).
    squiggle_lines_rebuilt: u64,
    // The two document-context owners: CJK-evidence retention (a SUM, same
    // convention as `nit_scan_lines` — the win is `lines_total == KEYS`, not
    // `KEYS * doc_lines`) and the still-unretained markdown/syntax span parse
    // (a size, reported as the LAST call's byte/line count like
    // `destination_join_bytes`, proving the full document is what it sees on
    // every single key).
    evidence_scan_ms: f64,
    evidence_scan_lines: u64,
    spans_scan_bytes: u64,
    spans_scan_lines: u64,
}

impl LiveTimings {
    fn new() -> Self {
        let samples = || Vec::with_capacity(KEYS);
        Self {
            total: samples(),
            edit: samples(),
            spell: samples(),
            text: samples(),
            visible: samples(),
            set_view: samples(),
            prepare: samples(),
            render: samples(),
            context: samples(),
            spans: samples(),
            lines: samples(),
            embeds: samples(),
            splice: samples(),
            shape: samples(),
            geometry: samples(),
            conceal: samples(),
            caret: samples(),
            spell_work: crate::spell::SpellRefreshWork::default(),
            geometry_patch_hits: 0,
            geometry_lines_patched: 0,
            geometry_rows_patched: 0,
            geometry_index_probes: 0,
            nit_scan_ms: 0.0,
            nit_scan_lines: 0,
            nit_scan_misses: 0,
            ornament_scan_ms: 0.0,
            ornament_scan_lines: 0,
            ornament_scan_spans: 0,
            ornament_scan_misses: 0,
            destination_join_ms: 0.0,
            destination_join_calls: 0,
            destination_join_bytes: 0,
            squiggle_scan_ms: 0.0,
            squiggle_scan_misspellings: 0,
            squiggle_scan_misses: 0,
            squiggle_lines_rebuilt: 0,
            evidence_scan_ms: 0.0,
            evidence_scan_lines: 0,
            spans_scan_bytes: 0,
            spans_scan_lines: 0,
        }
    }

    fn add_spell(&mut self, elapsed: f64, work: crate::spell::SpellRefreshWork) {
        self.spell.push(elapsed);
        self.spell_work.line_keys_scanned += work.line_keys_scanned;
        self.spell_work.lines_tokenized += work.lines_tokenized;
        self.spell_work.bytes_tokenized += work.bytes_tokenized;
        self.spell_work.full_scans += work.full_scans;
    }

    fn add_frame(&mut self, frame: FramePhases) {
        self.set_view.push(frame.set_view_ms);
        self.prepare.push(frame.prepare_ms);
        self.render.push(frame.render_ms);
        self.context.push(frame.text.context_ms);
        self.spans.push(frame.text.spans_ms);
        self.lines.push(frame.text.lines_ms);
        self.embeds.push(frame.text.embeds_ms);
        self.splice.push(frame.text.splice_ms);
        self.shape.push(frame.text.shape_ms);
        self.geometry.push(frame.text.geometry_ms);
        self.geometry_patch_hits += frame.text.geometry_patch_hits;
        self.geometry_lines_patched += frame.text.geometry_lines_patched;
        self.geometry_rows_patched += frame.text.geometry_rows_patched;
        self.geometry_index_probes += frame.text.geometry_index_probes;
        self.conceal.push(frame.conceal_ms);
        self.caret.push(frame.caret_ms);
        let o = frame.owner_scan;
        self.nit_scan_ms += o.nit_scan_ms;
        self.nit_scan_lines += o.nit_scan_lines;
        self.nit_scan_misses += u64::from(o.nit_scan_lines > 0);
        self.ornament_scan_ms += o.ornament_scan_ms;
        self.ornament_scan_lines += o.ornament_scan_lines;
        self.ornament_scan_spans = self.ornament_scan_spans.max(o.ornament_scan_spans);
        self.ornament_scan_misses += u64::from(o.ornament_scan_lines > 0);
        self.destination_join_ms += o.destination_join_ms;
        self.destination_join_calls += o.destination_join_calls;
        self.destination_join_bytes = self.destination_join_bytes.max(o.destination_join_bytes);
        self.squiggle_scan_ms += o.squiggle_scan_ms;
        self.squiggle_scan_misspellings += o.squiggle_scan_misspellings;
        self.squiggle_scan_misses +=
            u64::from(o.squiggle_scan_misspellings > 0 || o.squiggle_scan_ms > 0.0);
        self.squiggle_lines_rebuilt += o.squiggle_lines_rebuilt;
        self.evidence_scan_ms += o.evidence_scan_ms;
        self.evidence_scan_lines += o.evidence_scan_lines;
        self.spans_scan_bytes = self.spans_scan_bytes.max(o.spans_scan_bytes);
        self.spans_scan_lines = self.spans_scan_lines.max(o.spans_scan_lines);
    }

    fn validate(
        &self,
        cx: &Cx,
        buffer: &crate::buffer::Buffer,
        reshapes: u64,
        changed: u64,
    ) -> Result<()> {
        ensure!(
            reshapes == KEYS as u64,
            "typing_live must reshape exactly once per keystroke, got {reshapes} over {KEYS}"
        );
        ensure!(
            changed > 0,
            "typing_live characters must change the rendered frame"
        );
        if buffer.syntax_lang().is_none() {
            ensure!(
                self.spell_work.full_scans == 0,
                "typing_live ordinary prose must not take a full spell scan"
            );
            ensure!(
                self.spell_work.lines_tokenized == KEYS as u64,
                "typing_live must tokenize exactly one edited line per key"
            );
        }
        if cx.view.gutter_name == "novel.md" {
            ensure!(
                self.geometry_patch_hits == KEYS as u64,
                "typing_live prose must retain row geometry once per key, got {}",
                self.geometry_patch_hits
            );
            ensure!(
                self.geometry_lines_patched == KEYS as u64,
                "typing_live prose must patch exactly one logical line per key"
            );
            ensure!(
                self.geometry_index_probes <= (KEYS * 64) as u64,
                "typing_live row lookup must remain logarithmic, got {} probes",
                self.geometry_index_probes
            );
            ensure!(
                self.geometry_rows_patched == KEYS as u64,
                "typing_live novel edits must replace one visual row per key"
            );
            ensure!(
                self.squiggle_lines_rebuilt <= KEYS as u64,
                "typing_live prose must retain squiggle geometry outside the \
                 edited line, got {} lines rebuilt over {KEYS} keys — the \
                 retained SquiggleProjection did not engage (a full-document \
                 reseed would report far more)",
                self.squiggle_lines_rebuilt
            );
        }
        Ok(())
    }

    fn report(&self, name: &str) {
        println!(
            "BENCH-STAGES typing_live {name} edit={:.3}ms spell={:.3}ms text={:.3}ms \
             visible={:.3}ms set_view={:.3}ms prepare={:.3}ms render={:.3}ms",
            median_ms(&self.edit),
            median_ms(&self.spell),
            median_ms(&self.text),
            median_ms(&self.visible),
            median_ms(&self.set_view),
            median_ms(&self.prepare),
            median_ms(&self.render),
        );
        println!(
            "BENCH-SET-VIEW {name} context={:.3}ms spans={:.3}ms lines={:.3}ms \
             embeds={:.3}ms splice={:.3}ms shape={:.3}ms geometry={:.3}ms conceal={:.3}ms \
             caret={:.3}ms",
            median_ms(&self.context),
            median_ms(&self.spans),
            median_ms(&self.lines),
            median_ms(&self.embeds),
            median_ms(&self.splice),
            median_ms(&self.shape),
            median_ms(&self.geometry),
            median_ms(&self.conceal),
            median_ms(&self.caret),
        );
        // `destination_join_bytes` (and `spans_scan_bytes`/`spans_scan_lines`
        // below) are sizes, not per-key rates, so they are reported once via
        // `witnesses()` instead of on this per-key line.
        println!(
            "BENCH-OWNERS typing_live {name} nit_scan_ms={:.3}ms nit_scan_lines={} \
             nit_scan_misses={} ornament_scan_ms={:.3}ms ornament_scan_lines={} \
             ornament_scan_spans={} ornament_scan_misses={} destination_join_ms={:.3}ms \
             destination_join_calls={} squiggle_scan_ms={:.3}ms squiggle_scan_misspellings={} \
             squiggle_scan_misses={} squiggle_lines_rebuilt={} evidence_scan_ms={:.3}ms \
             evidence_scan_lines={}",
            self.nit_scan_ms,
            self.nit_scan_lines,
            self.nit_scan_misses,
            self.ornament_scan_ms,
            self.ornament_scan_lines,
            self.ornament_scan_spans,
            self.ornament_scan_misses,
            self.destination_join_ms,
            self.destination_join_calls,
            self.squiggle_scan_ms,
            self.squiggle_scan_misspellings,
            self.squiggle_scan_misses,
            self.squiggle_lines_rebuilt,
            self.evidence_scan_ms,
            self.evidence_scan_lines,
        );
    }

    fn witnesses(&self, cx: &Cx, reshapes: u64, changed: u64) -> Vec<(&'static str, u64)> {
        vec![
            ("reshapes", reshapes),
            ("pixels_changed", changed),
            ("words", cx.words),
            ("corpus_fnv", corpus::fingerprint(&cx.text)),
            ("spell_line_keys", self.spell_work.line_keys_scanned),
            ("spell_lines_tokenized", self.spell_work.lines_tokenized),
            ("spell_bytes_tokenized", self.spell_work.bytes_tokenized),
            ("spell_full_scans", self.spell_work.full_scans),
            ("geometry_patch_hits", self.geometry_patch_hits),
            ("geometry_lines_patched", self.geometry_lines_patched),
            ("geometry_rows_patched", self.geometry_rows_patched),
            ("geometry_index_probes", self.geometry_index_probes),
            ("nit_scan_lines", self.nit_scan_lines),
            ("nit_scan_misses", self.nit_scan_misses),
            ("ornament_scan_lines", self.ornament_scan_lines),
            ("ornament_scan_spans", self.ornament_scan_spans),
            ("ornament_scan_misses", self.ornament_scan_misses),
            ("destination_join_calls", self.destination_join_calls),
            ("destination_join_bytes", self.destination_join_bytes),
            (
                "squiggle_scan_misspellings",
                self.squiggle_scan_misspellings,
            ),
            ("squiggle_scan_misses", self.squiggle_scan_misses),
            ("squiggle_lines_rebuilt", self.squiggle_lines_rebuilt),
            ("evidence_scan_lines_total", self.evidence_scan_lines),
            ("spans_scan_bytes", self.spans_scan_bytes),
            ("spans_scan_lines", self.spans_scan_lines),
        ]
    }
}

/// TYPING LIVE PREPARATION: insert one character through the real `Buffer`,
/// rebuild the live spell cache and view-facing result through their production
/// seams, then pay `set_view` + one serialized offscreen frame. This is a stable
/// preparation + render measurement; the window event loop and compositor are
/// outside its boundary. The separate stage line reports medians but is not a
/// baseline witness because timings naturally vary between runs.
pub(super) fn live(cx: &mut Cx) -> Result<CellOut> {
    let pristine_misspelled = cx.view.misspelled.clone();
    let mut run = LiveRun::prime(cx)?;
    let first = cx.snapshot()?;
    let before = cx.p.reshape_count;
    for k in 0..KEYS {
        run.type_char(cx, (b'a' + (k % 26) as u8) as char)?;
    }
    let reshapes = cx.p.reshape_count - before;
    let changed = differing_pixels(&first, &cx.snapshot()?);
    run.timings.validate(cx, &run.buffer, reshapes, changed)?;
    run.timings.report(&cx.view.gutter_name);
    let witness = run.timings.witnesses(cx, reshapes, changed);

    // Restore every view field changed by the workload before the next cell.
    cx.view.text.clone_from(&cx.text);
    cx.view.misspelled = pristine_misspelled;
    cx.view.is_edit_move = false;
    cx.view.cursor_line = 0;
    cx.view.cursor_col = 0;
    cx.view.scroll = crate::render::ScrollPos::default();
    cx.sync_frame()?;
    Ok(CellOut {
        samples_ms: run.timings.total,
        witness,
    })
}

fn median_ms(samples_ms: &[f64]) -> f64 {
    let mut samples = samples_ms.to_vec();
    samples.sort_by(|a, b| a.partial_cmp(b).expect("typing phase samples are finite"));
    samples[samples.len() / 2]
}
