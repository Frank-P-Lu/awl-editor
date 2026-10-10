//! Capture assembly keeps document composition and its effective caret together.
use super::super::*;
use crate::buffer::Buffer;
use crate::render::{TextPipeline, ViewState};

#[test]
fn document_preedit_capture_matches_the_composed_caret_geometry() {
    if !crate::test_gpu::adapter_present() {
        eprintln!("skipping document IME capture projection: no wgpu adapter");
        return;
    }
    let _g = crate::testlock::serial();
    let mut pipeline = crate::test_gpu::with_shared_programs(|device, queue| {
        let cache = glyphon::Cache::new(device);
        let mut p = TextPipeline::new(device, queue, &cache, FORMAT);
        p.set_size(1200.0, 800.0);
        p
    })
    .unwrap();
    let mut buffer = Buffer::from_str("ab suffix");
    buffer.set_cursor(2);
    for text in ["abc", "にほん", "a\u{301}b"] {
        for cursor in [Some(0), Some(1), Some(99), None] {
            let opts = CaptureOpts {
                preedit: Some(text.into()),
                preedit_cursor: cursor,
                ..Default::default()
            };
            let actual = modes::settled_viewstate(&mut pipeline, &buffer, &opts, 800);
            assert_eq!(actual.preedit_cursor, cursor);
            let rect = pipeline.caret_pixel_rect();
            let offset = cursor
                .unwrap_or(text.chars().count())
                .min(text.chars().count());
            let mut expected = ViewState {
                text: format!("ab{text} suffix"),
                cursor_col: 2 + offset,
                preedit: String::new(),
                preedit_cursor: None,
                ..actual
            };
            pipeline.set_view(&expected);
            assert_eq!(rect, pipeline.caret_pixel_rect(), "{text}: {cursor:?}");
            expected.preedit_cursor = Some(99);
            pipeline.set_view(&expected);
            assert_eq!(
                rect,
                pipeline.caret_pixel_rect(),
                "empty preedit ignores cursor"
            );
        }
    }
}

#[test]
fn document_preedit_caret_visual_smoke() {
    if !crate::test_gpu::adapter_present() {
        eprintln!("skipping document IME visual smoke: no wgpu adapter");
        return;
    }
    let _g = crate::testlock::serial();
    let scratch = crate::testscratch::ScratchDir::new(
        std::env::temp_dir().join(format!("awl-ime-document-{}", std::process::id())),
    );
    let out = std::env::var_os("AWL_IME_DOCUMENT_CAPTURE_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| scratch.to_path_buf());
    std::fs::create_dir_all(&out).unwrap();
    let mut buffer = Buffer::from_str(
        "Writing 日本語 calmly.\nTransient composition keeps the document unchanged.\n",
    );
    buffer.set_cursor(8);
    let opts = CaptureOpts {
        preedit: Some("にほん".into()),
        preedit_cursor: Some(1),
        canvas: Some((1200, 800)),
        ..Default::default()
    };
    for world in ["Paperbark", "Bowerbird", "Gumtree", "Saltpan", "Wagtail"] {
        let _world = crate::theme::WorldPin::world(world).unwrap();
        let path = out.join(format!("{world}.png"));
        capture_with(&path, &buffer, &opts).unwrap();
        let image = image::open(&path).unwrap();
        assert_eq!((image.width(), image.height()), (1200, 800));
    }
}
