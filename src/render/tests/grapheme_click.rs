//! probe

use super::super::*;
use super::{headless_pipeline, view};
use unicode_segmentation::UnicodeSegmentation;

fn boundaries(text: &str) -> Vec<usize> {
    std::iter::once(0)
        .chain(text.graphemes(true).scan(0usize, |a, g| {
            *a += g.chars().count();
            Some(*a)
        }))
        .collect()
}

#[test]
fn probe_glyphs() {
    let _t = crate::testlock::serial();
    let Some(mut p) = headless_pipeline() else {
        eprintln!("skipping: no wgpu adapter");
        return;
    };
    let corpus = [
        ("decomposed", "e\u{0301}X"),
        ("stacked", "a\u{0301}\u{0308}\u{0327}b"),
        ("zwj", "a\u{1f468}\u{200d}\u{1f469}\u{200d}\u{1f467}b"),
        ("flag", "a\u{1f1ef}\u{1f1f5}b"),
        ("two flags", "\u{1f1ef}\u{1f1f5}\u{1f1fa}\u{1f1f8}"),
        ("keycap", "a1\u{fe0f}\u{20e3}b"),
        ("skin", "a\u{1f44d}\u{1f3fd}b"),
        ("vs", "a\u{2764}\u{fe0f}b"),
        ("hangul", "\u{1100}\u{1161}\u{11a8}z"),
        ("indic", "a\u{0915}\u{094d}\u{0915}b"),
        ("indic ksha", "a\u{0915}\u{094d}\u{0937}b"),
        ("tamil", "a\u{0b95}\u{0bcd}\u{0b95}b"),
        ("thai", "a\u{0e01}\u{0e33}b"),
        ("tibetan", "a\u{0f40}\u{0fb5}b"),
        ("hebrew points", "a\u{05d0}\u{05b8}\u{05b0}b"),
        ("arabic harakat", "a\u{0628}\u{064e}\u{0651}b"),
        ("tag flag", "a\u{1f3f4}\u{e0077}\u{e0061}\u{e0061}\u{e007f}b"),
        ("nonsense zwj", "a\u{1f600}\u{200d}\u{1f600}b"),
        ("long stack", "a\u{0301}\u{0308}\u{0327}\u{0331}\u{0324}b"),
        ("odd ri", "a\u{1f1e6}\u{1f1e7}\u{1f1e8}b"),
    ];
    for t in theme::THEMES.iter() {
        theme::set_active_by_name(t.name);
        p.sync_theme();
        let mono = crate::render::facepitch::family_is_mono(theme::active().font);
        let mut bad = vec![];
        for (label, text) in corpus {
            let bs = boundaries(text);
            let doc = format!("{text}\n");
            let v = view(&doc, 1, 0);
            p.set_view(&v);
            let doc_top = p.doc_top();
            let text_left = p.text_left();
            let py = doc_top + p.metrics.line_height * 0.5;
            let mut seen = std::collections::BTreeSet::new();
            for i in 0..400 {
                let px = text_left + i as f32 * 0.5;
                let (_l, c) = p.hit_test_scroll(px, py, crate::render::ScrollPos::default());
                if !bs.contains(&c) {
                    seen.insert(c);
                }
            }
            if !seen.is_empty() {
                bad.push(format!("{label}: interior {seen:?} (boundaries {bs:?})"));
                for run in p.buffer.layout_runs() {
                    for g in run.glyphs.iter() {
                        bad.push(format!(
                            "      g start={} end={} x={:.2} w={:.2}",
                            g.start, g.end, g.x, g.w
                        ));
                    }
                }
                let mut prev = usize::MAX;
                for i in 0..400 {
                    let px = text_left + i as f32 * 0.5;
                    let (_l, c) = p.hit_test_scroll(px, py, crate::render::ScrollPos::default());
                    if c != prev {
                        bad.push(format!("      x=+{:.1} -> col {c}", i as f32 * 0.5));
                        prev = c;
                    }
                }
            }
        }
        eprintln!("--- {} mono={mono} font={}", t.name, theme::active().font);
        for b in bad {
            eprintln!("    {b}");
        }
    }
    theme::set_active(theme::DEFAULT_THEME);
    p.sync_theme();
}
