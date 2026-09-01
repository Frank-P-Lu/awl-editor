//! THE ASSET CLEANER's live preview panel: decode-on-selection-only (never
//! once per frame), the panel's own narrow-canvas degrade, and the
//! contain-fit geometry its thumbnail draws through — split out of
//! `render/tests/images.rs` because this feature reuses that ONE decode
//! cache rather than owning a second one (`render/chrome/asset_preview.rs`'s
//! own module doc).

use super::{headless_dqp, view};

/// THE DECODE-ON-SELECTION-ONLY LAW. `prepare_asset_preview` reuses the ONE
/// inline-image decode cache (`image_cache::ImageCache`), keyed by canonical
/// path + mtime — so re-preparing the SAME selection across several frames
/// must be a cache HIT, never a fresh decode. This is the exact seam
/// `render/layers.rs`'s `prepare_images` almost broke: an orphan is by
/// definition unreferenced by any document, so it is never in that
/// function's own `keep` set unless the preview's path is folded in too —
/// without that fold, `retain_paths` evicts the just-decoded entry every
/// single frame and this law's second assertion goes red (a decode EVERY
/// frame the preview is open, forever, not once per selection).
#[test]
#[cfg(not(target_arch = "wasm32"))]
fn asset_preview_decodes_once_per_selection_never_once_per_frame() {
    let _g = crate::testlock::serial();
    let Some((device, queue, mut p)) = headless_dqp(1200.0, 800.0) else {
        eprintln!(
            "skipping asset_preview_decodes_once_per_selection_never_once_per_frame: no wgpu adapter"
        );
        return;
    };
    for f in ["samples/tiny.png", "samples/photo.png"] {
        if std::fs::metadata(f).is_err() {
            eprintln!("skipping: {f} fixture not present");
            return;
        }
    }
    let tiny = std::path::Path::new("samples/tiny.png")
        .canonicalize()
        .unwrap();
    let photo = std::path::Path::new("samples/photo.png")
        .canonicalize()
        .unwrap();

    let mut v = view("", 0, 0);
    v.overlay_active = true;
    v.overlay_asset_preview = Some(tiny.clone());
    p.set_view(&v);
    p.prepare(&device, &queue, 1200, 800).unwrap();
    assert_eq!(
        p.image_decode_count(),
        1,
        "the first frame decodes the selected orphan exactly once"
    );

    // A SECOND frame at the UNCHANGED selection: without folding the
    // preview's own path into `prepare_images`' retain set, this redecodes —
    // the exact defect this law exists to catch.
    p.set_view(&v);
    p.prepare(&device, &queue, 1200, 800).unwrap();
    assert_eq!(
        p.image_decode_count(),
        1,
        "an unchanged selection across frames is a cache hit, never a redecode"
    );

    // The selection moves to a DIFFERENT orphan: a genuinely new decode.
    v.overlay_asset_preview = Some(photo.clone());
    p.set_view(&v);
    p.prepare(&device, &queue, 1200, 800).unwrap();
    assert_eq!(
        p.image_decode_count(),
        2,
        "a changed selection decodes the newly-selected orphan"
    );

    // Back to the FIRST orphan: it fell out of `keep` while `photo` alone was
    // selected, so it was pruned and this is a genuine redecode — the other
    // half of the same seam (the cache stays bounded to the LIVE selection,
    // never growing to hold every orphan ever previewed this session).
    v.overlay_asset_preview = Some(tiny);
    p.set_view(&v);
    p.prepare(&device, &queue, 1200, 800).unwrap();
    assert_eq!(
        p.image_decode_count(),
        3,
        "returning to a deselected orphan redecodes -- it was pruned while off-selection"
    );
}

/// THE NARROW-CANVAS DEGRADE: below `asset_preview_rect`'s own room floor
/// the panel draws NOTHING, and the list's own card geometry is BYTE-IDENTICAL
/// to the no-preview baseline — the preview yields to the list, the list
/// never yields to the preview (the item's own "sized so the list remains
/// the primary surface" constraint, made a law rather than a taste claim).
#[test]
#[cfg(not(target_arch = "wasm32"))]
fn asset_preview_yields_entirely_on_a_narrow_canvas_leaving_the_card_untouched() {
    let _g = crate::testlock::serial();
    let Some(mut p) = super::headless_pipeline() else {
        eprintln!(
            "skipping asset_preview_yields_entirely_on_a_narrow_canvas_leaving_the_card_untouched: no wgpu adapter"
        );
        return;
    };
    let mut v = view("", 0, 0);
    v.overlay_active = true;
    v.overlay_items = vec!["orphan.png".to_string()];
    v.overlay_bindings = vec!["12.3 KB . assets".to_string()];
    v.overlay_asset_preview = Some(std::path::PathBuf::from("/no/such/awl-preview-fixture.png"));

    // WIDE: real room beside the card -> the panel claims a rect.
    p.set_size(1600.0, 800.0);
    p.set_view(&v);
    assert!(
        p.asset_preview_rect(1600).is_some(),
        "a wide canvas leaves genuine room beside the card"
    );

    // NARROW: the SAME overlay, a canvas with no room beside the card at all.
    p.set_size(420.0, 800.0);
    p.set_view(&v);
    let narrow_with_preview = p.overlay_geometry(420);
    assert!(
        p.asset_preview_rect(420).is_none(),
        "below the room floor the preview draws nothing"
    );

    // The NO-PREVIEW BASELINE at the identical narrow canvas: the same view,
    // minus the one field that gates the panel. `overlay_geometry` never
    // reads `overlay_asset_preview` at all (only `asset_preview_rect` does,
    // and only AFTER calling it) -- so this is a genuine non-vacuity check,
    // not a tautology: it would catch a future regression that let the
    // preview's presence leak into the row list's own width/text budget.
    v.overlay_asset_preview = None;
    p.set_view(&v);
    let narrow_without_preview = p.overlay_geometry(420);
    assert_eq!(
        (
            narrow_with_preview.text_left,
            narrow_with_preview.text_w,
            narrow_with_preview.card_h,
        ),
        (
            narrow_without_preview.text_left,
            narrow_without_preview.text_w,
            narrow_without_preview.card_h,
        ),
        "the list's own card geometry is byte-identical whether or not a \
         (room-less) preview is gated on -- the preview yields to the list, \
         never the other way round"
    );
}
