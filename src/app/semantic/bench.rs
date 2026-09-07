//! `--bench-a11y` — the latency and allocation witness for the accessibility
//! projection.
//!
//! Two paths over the same documents, in one run so the numbers are comparable:
//!
//! * **monolithic** — what every redraw once did while an assistive technology
//!   was attached: build a whole `SemanticSnapshot` (read every line out of the
//!   rope, segment all of it under UAX #29, allocate every node) and project a
//!   whole `TreeUpdate` with its `Tree` metadata.
//! * **incremental** — what a redraw does now: refresh the retained projection,
//!   which re-reads only the lines the run table marks, and publish the changed
//!   nodes.
//!
//! It prints the COUNTS beside the milliseconds. A duration on its own cannot
//! tell "one line was re-read" from "nothing was re-read" — CLAUDE.md records a
//! theme bench that measured 5 ms while nothing reshaped — so the counters are
//! the load-bearing half and the clock is the consequence.
//!
//! Opens no window and touches no GPU. Judge it in `--release`: dev frames are
//! 10–20x slower and a dev number here is not honest.

use super::*;

/// Sizes a prose user actually reaches, ending at a book.
const SIZES: [usize; 4] = [100, 1_000, 10_000, 50_000];
/// Keystrokes per arm. Enough that a median means something and short enough
/// that the monolithic arm at 50 000 lines still finishes.
const KEYSTROKES: usize = 40;

fn document(lines: usize) -> String {
    (0..lines)
        .map(|n| format!("line {n:06} of some ordinary prose in a paragraph"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn app_with(text: &str) -> App {
    let mut app = App::new_headless_capture(
        None,
        std::env::temp_dir(),
        None,
        crate::config::Config::empty(),
    );
    // A NAMED EXEMPTION from the insertion-door census
    // (`app/input/text_door.rs`): this `App` has no window, no overlay and no
    // user, so there is no reading surface for it to write behind.
    app.write_document_text(
        crate::app::TextDoor::AccessibilityBench,
        crate::app::TextEdit::Whole(text),
    );
    app
}

fn median(mut samples: Vec<f64>) -> f64 {
    samples.sort_by(|a, b| a.partial_cmp(b).unwrap());
    samples[samples.len() / 2]
}

pub(crate) fn run() -> anyhow::Result<()> {
    println!(
        "a11y projection witness — per KEYSTROKE, with an assistive technology attached.\n\
         monolithic = the retired path (whole snapshot + whole TreeUpdate every redraw).\n\
         incremental = the retained projection (only the lines an edit touched).\n\
         Counts are the witness; the clock is the consequence. Release numbers only.\n"
    );
    println!(
        "{:>7} | {:>12} | {:>12} | {:>10} | {:>7} | {:>11} | {:>10}",
        "lines", "monolithic", "incremental", "speed-up", "runs", "bytes read", "nodes"
    );
    println!("{}", "-".repeat(88));

    for lines in SIZES {
        let text = document(lines);
        let middle_line = lines / 2;

        // --- monolithic ---------------------------------------------------
        let mut app = app_with(&text);
        let caret = app.document.buffer().line_col_to_char(middle_line, 0);
        app.document.set_cursor(caret);
        let mut mono = Vec::with_capacity(KEYSTROKES);
        for _ in 0..KEYSTROKES {
            app.write_document_text(
                crate::app::TextDoor::AccessibilityBench,
                crate::app::TextEdit::Char('x'),
            );
            let start = std::time::Instant::now();
            let snapshot = app.semantic_snapshot();
            let update = crate::semantic::native::tree_update(&snapshot);
            mono.push(start.elapsed().as_secs_f64() * 1000.0);
            std::hint::black_box(update.nodes.len());
        }
        let mono_nodes = app.semantic_snapshot().nodes.len();

        // --- incremental ----------------------------------------------------
        let mut app = app_with(&text);
        let caret = app.document.buffer().line_col_to_char(middle_line, 0);
        app.document.set_cursor(caret);
        let mut projection = SemanticProjection::new();
        let mut projector = crate::semantic::native::TreeProjector::default();
        projection.refresh(&app.semantic_view());
        std::hint::black_box(projector.full(projection.snapshot(), projection.shape_rev()));
        projection.note_full_tree();
        let base = projection.stats();
        let mut inc = Vec::with_capacity(KEYSTROKES);
        let mut refresh_only = Vec::with_capacity(KEYSTROKES);
        for _ in 0..KEYSTROKES {
            app.write_document_text(
                crate::app::TextDoor::AccessibilityBench,
                crate::app::TextEdit::Char('x'),
            );
            let start = std::time::Instant::now();
            projection.refresh(&app.semantic_view());
            let split = start.elapsed().as_secs_f64() * 1000.0;
            let update = projector.incremental(
                projection.snapshot(),
                projection.changed(),
                projection.shape_rev(),
            );
            let published = update.nodes.len();
            inc.push(start.elapsed().as_secs_f64() * 1000.0);
            refresh_only.push(split);
            std::hint::black_box(published);
            projection.note_incremental(published);
        }
        let stats = projection.stats();

        let mono_ms = median(mono);
        let inc_ms = median(inc);
        println!(
            "{lines:>7} | {mono_ms:>9.3} ms | {inc_ms:>9.3} ms | \
             {:>9.0}x | {:>7} | {:>11} | {:>10}",
            mono_ms / inc_ms.max(f64::MIN_POSITIVE),
            (stats.runs_rebuilt - base.runs_rebuilt) as f64 / KEYSTROKES as f64,
            (stats.bytes_read - base.bytes_read) / KEYSTROKES as u64,
            (stats.nodes_published - base.nodes_published) / KEYSTROKES as u64,
        );
        println!("        | refresh alone: {:.3} ms", median(refresh_only));
        println!(
            "        | whole tree: {mono_nodes} nodes, {} bytes of document re-read and \
             re-segmented per keystroke",
            text.len(),
        );
    }
    println!("\nThe monolithic column is what a VoiceOver user paid on every frame while typing.");
    Ok(())
}
