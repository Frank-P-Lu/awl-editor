//! THE DOCK NEVER CHURNS ON A HOVER.
//!
//! The per-world app icon follows the ACTIVE world, which raises the obvious
//! hazard: the theme picker previews live, so arrowing (or sweeping the mouse)
//! down nineteen worlds would restamp the Dock tile nineteen times if the
//! adoption hung off the preview. It does not, and the reason is structural
//! rather than careful — `crate::app_icon::adopt` is reachable from exactly two
//! places, both settled: startup after the sticky theme is restored, and the
//! `theme_committed` arm of [`App::post_apply_effects`], the same guard that
//! decides whether the sticky preference is written at all. The preview path
//! ([`App::retint_theme_preview`]) re-tints pipelines and defers a reshape; it
//! has no route to `app_icon`.
//!
//! These tests drive the real seams and count adoptions, so the property is
//! observable rather than asserted by reading the code. The AppKit call itself
//! is live-only (a test process has no Dock tile) — `adopt` records the
//! adoption either way, which is what makes the count testable at all.

use super::*;

/// A FULL PREVIEW SWEEP — every world, through both preview doors (the direct
/// re-tint and `post_apply_effects`'s preview arm) — moves the Dock ZERO times.
#[test]
fn a_theme_preview_sweep_never_adopts_a_dock_icon() {
    let _g = crate::testlock::serial();
    let restore = crate::theme::active_index();
    let mut app = App::new_hermetic(None, PathBuf::from("/tmp"), Config::empty());
    crate::app_icon::reset_adoptions_for_test();

    for i in 0..crate::theme::THEMES.len() {
        let before = crate::theme::active();
        crate::theme::set_active(i);
        // (a) the seam every input kind (arrow, hover, wheel) funnels through
        app.retint_theme_preview(before);
        // (b) the post-apply arm a preview keypress actually lands in: the
        // picker is still OPEN (`theme_overlay_before`) and nothing committed.
        app.post_apply_effects(&Action::NextLine, true, false, before);
    }

    assert_eq!(
        crate::app_icon::adoptions(),
        0,
        "previewing all {} worlds must not touch the Dock once",
        crate::theme::THEMES.len()
    );
    assert_eq!(
        crate::app_icon::adopted(),
        None,
        "no world was adopted by a preview"
    );
    crate::theme::set_active(restore);
}

/// A COMMIT adopts, exactly once, and it adopts the world that actually won.
/// Same seam, same call, one flag different — which is the whole point: the
/// Dock rides the settled choice, like the sticky preference beside it.
#[test]
fn a_theme_commit_adopts_the_settled_world_once() {
    let _g = crate::testlock::serial();
    let restore = crate::theme::active_index();
    let mut app = App::new_hermetic(None, PathBuf::from("/tmp"), Config::empty());
    crate::app_icon::reset_adoptions_for_test();

    // Preview a couple of worlds first (still no adoption), then commit a third.
    let start = crate::theme::active();
    crate::theme::set_active_by_name("Mangrove").unwrap();
    app.post_apply_effects(&Action::NextLine, true, false, start);
    crate::theme::set_active_by_name("Galah").unwrap();
    app.post_apply_effects(&Action::NextLine, true, false, start);
    assert_eq!(crate::app_icon::adoptions(), 0, "still only previews");

    let before = crate::theme::active();
    crate::theme::set_active_by_name("Wagtail").unwrap();
    app.post_apply_effects(&Action::Newline, true, true, before);
    assert_eq!(crate::app_icon::adoptions(), 1, "the commit adopts, once");
    assert_eq!(
        crate::app_icon::adopted(),
        Some("Wagtail"),
        "the world that won the picker is the one in the Dock"
    );

    // A SECOND commit re-adopts — the Dock follows every settled change, not
    // just the first one.
    let before = crate::theme::active();
    crate::theme::set_active_by_name("Tawny").unwrap();
    app.post_apply_effects(&Action::Newline, true, true, before);
    assert_eq!(crate::app_icon::adoptions(), 2);
    assert_eq!(crate::app_icon::adopted(), Some("Tawny"));
    crate::theme::set_active(restore);
}

/// Whatever world is adopted, the bytes handed to AppKit are THAT world's
/// committed icon — the tie between the picker's choice and the artwork on
/// disk. macOS-only, because only that build embeds the bytes.
#[test]
#[cfg(target_os = "macos")]
fn the_adopted_world_hands_over_its_own_committed_icon() {
    let _g = crate::testlock::serial();
    let restore = crate::theme::active_index();
    for t in crate::theme::THEMES.iter() {
        crate::app_icon::reset_adoptions_for_test();
        crate::app_icon::adopt(t);
        assert_eq!(crate::app_icon::adopted(), Some(t.name));
        let embedded = crate::app_icon::icns_for(t.name)
            .unwrap_or_else(|| panic!("{} embeds an icon", t.name));
        let on_disk = std::fs::read(
            PathBuf::from(crate::app_icon::WORLD_ICON_DIR).join(format!("{}.icns", t.name)),
        )
        .unwrap_or_else(|e| panic!("{}: {e}", t.name));
        assert_eq!(
            embedded,
            on_disk.as_slice(),
            "{} adopts its own file",
            t.name
        );
    }
    crate::theme::set_active(restore);
}

/// APPKIT ITSELF ACCEPTS THE CONTAINER. `set_dock_icon` hands raw `.icns` bytes
/// to `NSImage`, so the one thing a hand-written packer must not get wrong is
/// whether AppKit can read what it wrote — a container macOS silently refuses
/// would leave the Dock showing the generic application icon with nothing in
/// any log. Decoding here (not on the main thread, which `NSImage(data:)` does
/// not require) proves the bytes are a real icon and that every representation
/// arrived: 7 distinct pixel sizes, up to the 1024 master.
///
/// This is the closest a test can get to the live Dock; the swap ITSELF — the
/// tile actually changing after a commit — stays a human confirmation.
#[test]
#[cfg(target_os = "macos")]
fn appkit_decodes_every_committed_icon_with_its_full_size_ladder() {
    use objc2::AnyThread;
    use objc2_app_kit::NSImage;
    use objc2_foundation::NSData;

    let _g = crate::testlock::serial();
    for t in crate::theme::THEMES.iter() {
        let bytes = crate::app_icon::icns_for(t.name)
            .unwrap_or_else(|| panic!("{} embeds an icon", t.name));
        let data = NSData::with_bytes(bytes);
        let image = NSImage::initWithData(NSImage::alloc(), &data)
            .unwrap_or_else(|| panic!("{}: AppKit refused the .icns", t.name));
        let mut sizes: Vec<i64> = image
            .representations()
            .iter()
            .map(|r| r.pixelsWide() as i64)
            .collect();
        sizes.sort_unstable();
        sizes.dedup();
        assert_eq!(
            sizes,
            vec![16, 32, 64, 128, 256, 512, 1024],
            "{}: AppKit sees the wrong size ladder",
            t.name
        );
    }
}
