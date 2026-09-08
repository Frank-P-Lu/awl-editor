//! Production preparation and offscreen rendering typing workload for typing.

use super::*;

/// TYPING LIVE PREPARATION: insert one character through the real `Buffer`,
/// rebuild the live spell cache and view-facing result through their production
/// seams, then pay `set_view` + one serialized offscreen frame. This is a stable
/// preparation + render measurement; the window event loop and compositor are
/// outside its boundary. The separate stage line reports medians but is not a
/// baseline witness because timings naturally vary between runs.
pub(super) fn live(cx: &mut Cx) -> Result<CellOut> {
    const KEYS: usize = 30;
    let pristine_misspelled = cx.view.misspelled.clone();
    let spell = crate::spell::SpellChecker::new(crate::spell::DictVariant::EnUs)
        .map_err(|e| anyhow::anyhow!("spell checker failed to load: {e}"))?;
    let mut spell_projection = crate::spell::SpellProjection::default();
    let mut buffer = crate::buffer::Buffer::from_str(&cx.text);
    buffer.set_path(std::path::PathBuf::from(cx.view.gutter_name.clone()));
    cx.view.scroll = crate::render::ScrollPos::default();
    cx.view.cursor_line = 0;
    cx.view.is_edit_move = true;

    // Prime the same complete operation untimed so the samples measure steady
    // state. Retain both caches exactly as the live document session does.
    buffer.insert_char('z');
    let mut spell_cache = spell_projection.refresh(&buffer, &spell);
    let text = buffer.text();
    let mut sync_text_cache = text.clone();
    std::hint::black_box(&sync_text_cache);
    cx.view.cursor_col = 1;
    cx.view.text = text;
    cx.view.misspelled = crate::spell::visible(&spell_cache, &cx.view.text);
    cx.sync_frame()?;
    let first = cx.snapshot()?;

    let before = cx.p.reshape_count;
    let mut samples = Vec::with_capacity(KEYS);
    let mut edit_samples = Vec::with_capacity(KEYS);
    let mut spell_samples = Vec::with_capacity(KEYS);
    let mut text_samples = Vec::with_capacity(KEYS);
    let mut visible_samples = Vec::with_capacity(KEYS);
    let mut set_view_samples = Vec::with_capacity(KEYS);
    let mut prepare_samples = Vec::with_capacity(KEYS);
    let mut render_samples = Vec::with_capacity(KEYS);
    let mut context_samples = Vec::with_capacity(KEYS);
    let mut spans_samples = Vec::with_capacity(KEYS);
    let mut lines_samples = Vec::with_capacity(KEYS);
    let mut embeds_samples = Vec::with_capacity(KEYS);
    let mut splice_samples = Vec::with_capacity(KEYS);
    let mut shape_samples = Vec::with_capacity(KEYS);
    let mut geometry_samples = Vec::with_capacity(KEYS);
    let mut conceal_samples = Vec::with_capacity(KEYS);
    let mut caret_samples = Vec::with_capacity(KEYS);
    let mut spell_work = crate::spell::SpellRefreshWork::default();
    let mut geometry_patch_hits = 0u64;
    let mut geometry_lines_patched = 0u64;
    let mut geometry_rows_patched = 0u64;
    let mut geometry_index_probes = 0u64;
    for k in 0..KEYS {
        let ch = (b'a' + (k % 26) as u8) as char;
        let t0 = Instant::now();

        let edit_at = Instant::now();
        buffer.insert_char(ch);
        edit_samples.push(ms(edit_at));

        // Include replacement and destruction of the prior cache inside the
        // spelling stage, matching the live assignment.
        let spell_at = Instant::now();
        spell_cache = spell_projection.refresh(&buffer, &spell);
        let work = spell_projection.work();
        spell_work.line_keys_scanned += work.line_keys_scanned;
        spell_work.lines_tokenized += work.lines_tokenized;
        spell_work.bytes_tokenized += work.bytes_tokenized;
        spell_work.full_scans += work.full_scans;
        spell_samples.push(ms(spell_at));

        // Mirror `DocumentSession::sync_text` on an edited version: materialize
        // the rope a second time and retain one owned cached copy.
        let text_at = Instant::now();
        let text = buffer.text();
        sync_text_cache = text.clone();
        std::hint::black_box(&sync_text_cache);
        text_samples.push(ms(text_at));

        let visible_at = Instant::now();
        let misspelled = crate::spell::visible(&spell_cache, &text);
        visible_samples.push(ms(visible_at));

        let (line, col) = buffer.cursor_line_col();
        cx.view.cursor_line = line;
        cx.view.cursor_col = col;
        cx.view.text = text;
        cx.view.misspelled = misspelled;
        let frame = cx.sync_frame_phases()?;
        set_view_samples.push(frame.set_view_ms);
        prepare_samples.push(frame.prepare_ms);
        render_samples.push(frame.render_ms);
        context_samples.push(frame.text.context_ms);
        spans_samples.push(frame.text.spans_ms);
        lines_samples.push(frame.text.lines_ms);
        embeds_samples.push(frame.text.embeds_ms);
        splice_samples.push(frame.text.splice_ms);
        shape_samples.push(frame.text.shape_ms);
        geometry_samples.push(frame.text.geometry_ms);
        geometry_patch_hits += frame.text.geometry_patch_hits;
        geometry_lines_patched += frame.text.geometry_lines_patched;
        geometry_rows_patched += frame.text.geometry_rows_patched;
        geometry_index_probes += frame.text.geometry_index_probes;
        conceal_samples.push(frame.conceal_ms);
        caret_samples.push(frame.caret_ms);
        samples.push(ms(t0));
    }
    let reshapes = cx.p.reshape_count - before;
    ensure!(
        reshapes == KEYS as u64,
        "typing_live must reshape exactly once per keystroke, got {reshapes} over {KEYS}"
    );
    let last = cx.snapshot()?;
    let changed = differing_pixels(&first, &last);
    ensure!(
        changed > 0,
        "typing_live characters must change the rendered frame"
    );
    if buffer.syntax_lang().is_none() {
        ensure!(
            spell_work.full_scans == 0,
            "typing_live ordinary prose must not take a full spell scan"
        );
        ensure!(
            spell_work.lines_tokenized == KEYS as u64,
            "typing_live must tokenize exactly one edited line per key"
        );
    }
    if cx.view.gutter_name == "novel.md" {
        ensure!(
            geometry_patch_hits == KEYS as u64,
            "typing_live prose must retain row geometry once per key, got {geometry_patch_hits}"
        );
        ensure!(
            geometry_lines_patched == KEYS as u64,
            "typing_live prose must patch exactly one logical line per key"
        );
        ensure!(
            geometry_index_probes <= (KEYS * 64) as u64,
            "typing_live row lookup must remain logarithmic, got {geometry_index_probes} probes"
        );
        if cx.view.gutter_name == "novel.md" {
            ensure!(
                geometry_rows_patched == KEYS as u64,
                "typing_live novel edits must replace one visual row per key"
            );
        }
    }

    println!(
        "BENCH-STAGES typing_live {} edit={:.3}ms spell={:.3}ms text={:.3}ms visible={:.3}ms set_view={:.3}ms prepare={:.3}ms render={:.3}ms",
        cx.view.gutter_name,
        median_ms(&edit_samples),
        median_ms(&spell_samples),
        median_ms(&text_samples),
        median_ms(&visible_samples),
        median_ms(&set_view_samples),
        median_ms(&prepare_samples),
        median_ms(&render_samples),
    );
    println!(
        "BENCH-SET-VIEW {} context={:.3}ms spans={:.3}ms lines={:.3}ms embeds={:.3}ms splice={:.3}ms shape={:.3}ms geometry={:.3}ms conceal={:.3}ms caret={:.3}ms",
        cx.view.gutter_name,
        median_ms(&context_samples),
        median_ms(&spans_samples),
        median_ms(&lines_samples),
        median_ms(&embeds_samples),
        median_ms(&splice_samples),
        median_ms(&shape_samples),
        median_ms(&geometry_samples),
        median_ms(&conceal_samples),
        median_ms(&caret_samples),
    );

    // Restore every view field changed by the workload before the next cell.
    cx.view.text.clone_from(&cx.text);
    cx.view.misspelled = pristine_misspelled;
    cx.view.is_edit_move = false;
    cx.view.cursor_line = 0;
    cx.view.cursor_col = 0;
    cx.view.scroll = crate::render::ScrollPos::default();
    cx.sync_frame()?;
    Ok(CellOut {
        samples_ms: samples,
        witness: vec![
            ("reshapes", reshapes),
            ("pixels_changed", changed),
            ("words", cx.words),
            ("corpus_fnv", corpus::fingerprint(&cx.text)),
            ("spell_line_keys", spell_work.line_keys_scanned),
            ("spell_lines_tokenized", spell_work.lines_tokenized),
            ("spell_bytes_tokenized", spell_work.bytes_tokenized),
            ("spell_full_scans", spell_work.full_scans),
            ("geometry_patch_hits", geometry_patch_hits),
            ("geometry_lines_patched", geometry_lines_patched),
            ("geometry_rows_patched", geometry_rows_patched),
            ("geometry_index_probes", geometry_index_probes),
        ],
    })
}

fn median_ms(samples_ms: &[f64]) -> f64 {
    let mut samples = samples_ms.to_vec();
    samples.sort_by(|a, b| a.partial_cmp(b).expect("typing phase samples are finite"));
    samples[samples.len() / 2]
}
