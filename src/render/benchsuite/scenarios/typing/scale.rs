//! Release-only mixed Markdown editing scale probe.

use super::*;

const MIXED_SECTION: &str = concat!(
    "# Field notes / 観察記録\n\n",
    "The little river bends around the garden and carries a low yellow light across the window. ",
    "この段落には日本語と英語が混じっていて、長い行の折り返しを確認できる。 ",
    "On the far side, the old lantern is still bright.\n\n",
    "> 余白と文字の形を観察する。\n\n",
    "```rust\nlet lantern = 2;\n```\n\n",
);

fn mixed_fixture(lines: usize) -> String {
    // The last two lines form an identical edit target across all sizes.
    let block = MIXED_SECTION.lines().count();
    let complete = (lines - 2) / block;
    let padding = (lines - 2) % block;
    let mut text = MIXED_SECTION
        .lines()
        .cycle()
        .take(complete * block)
        .chain(std::iter::repeat_n("観察記録 / field note.", padding))
        .collect::<Vec<_>>()
        .join("\n");
    text.push_str("\nField notes continue here. 観察を続ける。\nEnd of notebook.");
    text
}

fn image_hash(image: &image::RgbaImage) -> u64 {
    image.as_raw().iter().fold(0xcbf29ce484222325u64, |h, b| {
        (h ^ u64::from(*b)).wrapping_mul(0x100000001b3)
    })
}

pub(in crate::render::benchsuite) fn scale(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    cache: &glyphon::Cache,
    config: &crate::config::Config,
) -> Result<()> {
    println!(
        "typing-scale: release headless edits + serialized GPU completion; not display latency"
    );
    let spell = crate::spell::SpellChecker::new(crate::spell::DictVariant::EnUs)
        .map_err(|e| anyhow::anyhow!("spell checker failed to load: {e}"))?;
    for lines in [100, 1_000, 10_000] {
        let text = mixed_fixture(lines);
        ensure!(text.lines().count() == lines, "wrong mixed fixture size");
        let misspelled = spell.misspellings_for(&text, None);
        let mut cx = Cx::new(device, queue, cache, config, Tier::M, text, misspelled)?;
        cx.view.gutter_name = format!("mixed-{lines}.md");
        let mut run = LiveRun::prime(&mut cx, true)?;
        let before_text = run.buffer.text();
        let edit_at = run.buffer.cursor_byte();
        let start = run.buffer.cursor_line_col();
        ensure!(start == (lines - 2, 1), "wrong edit target: {start:?}");
        let before = cx.snapshot()?;
        let reshapes = cx.p.reshape_count;
        let typed: String = (0..KEYS).map(|k| (b'a' + (k % 26) as u8) as char).collect();
        for ch in typed.chars() {
            run.type_char(&mut cx, ch)?;
        }
        let after = cx.snapshot()?;
        let changed = differing_pixels(&before, &after);
        run.timings
            .validate(&cx, &run.buffer, cx.p.reshape_count - reshapes, changed)?;
        let mut expected = before_text;
        expected.insert_str(edit_at, &typed);
        let end = run.buffer.cursor_line_col();
        ensure!(
            end == (start.0, start.1 + KEYS),
            "caret did not follow the edits"
        );
        ensure!(
            cx.view.text == expected && run.buffer.text() == expected,
            "edited text mismatched"
        );
        run.timings.report(&cx.view.gutter_name);
        let samples = &mut run.timings.total;
        samples.sort_by(f64::total_cmp);
        let p50 = (samples[KEYS / 2 - 1] + samples[KEYS / 2]) / 2.0;
        let p95 = samples[(KEYS * 95).div_ceil(100) - 1];
        println!(
            "TYPING-SCALE lines={lines} min_ms={:.3} p50_ms={p50:.3} p95_ms={p95:.3} \
             caret={}:{} text={:016x} before={:016x} after={:016x} changed={changed} parsed_bytes={}",
            samples[0],
            end.0,
            end.1,
            corpus::fingerprint(&expected),
            image_hash(&before),
            image_hash(&after),
            run.timings.spans_scan_bytes,
        );
    }
    Ok(())
}
