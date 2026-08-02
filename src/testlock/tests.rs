use super::*;

#[test]
fn serial_is_reentrant_and_the_outermost_guard_owns_the_release() {
    let g1 = serial();
    assert!(currently_held(), "the outer guard sets the thread flag");
    let g2 = serial(); // nested acquire on the SAME thread: must not deadlock
    drop(g2);
    assert!(
        currently_held(),
        "dropping the inner (no-op) guard must NOT release the outer hold"
    );
    drop(g1);
    assert!(!currently_held(), "the outermost drop clears the flag");
}

#[test]
fn nested_guards_from_many_former_lock_sites_share_one_underlying_lock() {
    // The collapse's core promise: what used to be a theme lock + a page
    // lock + a caret lock (three DIFFERENT mutexes, taken in a fixed order)
    // is now ONE reentrant guard. Acquiring it three deep on one thread must
    // never deadlock and the outermost must own the release.
    let a = serial();
    let b = serial();
    let c = serial();
    assert!(currently_held());
    drop(c);
    drop(b);
    assert!(currently_held(), "still held while the outermost lives");
    drop(a);
    assert!(!currently_held(), "released once the outermost drops");
}

#[test]
fn a_writer_thread_blocks_until_the_guard_is_released() {
    // THE mutual-exclusion law (the visual_* / wash-cache flake fix rests on
    // it): a thread that does not hold the guard cannot proceed past its own
    // acquire while another thread holds it.
    let g = serial();
    let done = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let writer = {
        let done = done.clone();
        std::thread::spawn(move || {
            let _held = serial();
            done.store(true, std::sync::atomic::Ordering::SeqCst);
        })
    };
    std::thread::sleep(std::time::Duration::from_millis(50));
    assert!(
        !done.load(std::sync::atomic::Ordering::SeqCst),
        "the other thread must still be blocked while we hold the guard"
    );
    drop(g);
    writer.join().unwrap();
    assert!(
        done.load(std::sync::atomic::Ordering::SeqCst),
        "the blocked thread proceeds once the guard is released"
    );
}

#[test]
fn a_global_writer_nested_under_the_guard_never_self_deadlocks() {
    // A page-global WRITER (`set_measure`) acquires the guard INTERNALLY under
    // `cfg(test)`. A test that already holds the guard and then drives such a
    // writer must nest for free (not self-deadlock), and the write must land.
    let _g = serial();
    crate::page::set_measure(33);
    assert_eq!(
        crate::page::measure(),
        33,
        "the nested writer's write lands"
    );
    crate::page::set_measure(crate::page::DEFAULT_MEASURE); // leave as found
}

#[test]
fn a_deliberately_leaking_test_window_fails_and_cleans_before_unlocking() {
    let before = {
        let _g = serial();
        crate::theme::active_index()
    };
    let leaked = std::panic::catch_unwind(|| {
        let _g = serial();
        crate::theme::set_active(before + 1);
    });
    assert!(
        leaked.is_err(),
        "a checked test window must reject a dirty exit"
    );
    let _g = serial();
    assert_eq!(
        crate::theme::active_index(),
        before,
        "the failing window cleans the global before another test can acquire it"
    );
}

#[test]
fn a_product_request_cannot_bypass_an_outer_test_window() {
    let before = {
        let _g = serial();
        crate::theme::active_index()
    };
    let nested_leak = std::panic::catch_unwind(|| {
        let _test = serial();
        let _product = product();
        crate::theme::set_active(before + 1);
    });
    assert!(
        nested_leak.is_err(),
        "a nested production acquire cannot punch through the outer test check"
    );
    let _g = serial();
    assert_eq!(crate::theme::active_index(), before);
}

#[test]
fn page_signature_distinguishes_the_retired_eighty_measure_from_prose_default() {
    let _g = serial();
    let _page = crate::page::PagePin::snapshot();
    crate::page::set_page_on(true);
    crate::page::set_measure(crate::page::DEFAULT_MEASURE);
    let prose_default = (crate::page::page_on(), crate::page::measure());
    crate::page::set_measure(80);
    let retired_eighty = (crate::page::page_on(), crate::page::measure());
    assert_eq!(prose_default, (true, crate::page::DEFAULT_MEASURE));
    assert_eq!(retired_eighty, (true, 80));
    assert_ne!(
        prose_default, retired_eighty,
        "the victim must distinguish the predecessor's 80 from the prose default"
    );
}

#[test]
fn a_deliberately_dirty_page_window_fails_and_restores_before_the_next_reader() {
    let before = {
        let _g = serial();
        (crate::page::page_on(), crate::page::measure())
    };
    let leaked = std::panic::catch_unwind(|| {
        let _g = serial();
        crate::page::set_page_on(!before.0);
        crate::page::set_measure(before.1 + 1);
    });
    assert!(
        leaked.is_err(),
        "a checked window must reject dirty page inputs"
    );
    let _g = serial();
    assert_eq!(
        (crate::page::page_on(), crate::page::measure()),
        before,
        "the failed page window restores before its next reader enters"
    );
}

#[test]
fn a_deliberately_dirty_spellcheck_window_fails_and_restores_before_the_next_reader() {
    let before = {
        let _g = serial();
        crate::spell::spellcheck_on()
    };
    let leaked = std::panic::catch_unwind(|| {
        let _g = serial();
        crate::spell::set_spellcheck_on(!before);
    });
    assert!(
        leaked.is_err(),
        "a checked window must reject dirty spellcheck state"
    );
    let _g = serial();
    assert_eq!(
        crate::spell::spellcheck_on(),
        before,
        "the failed spellcheck window restores before its next reader enters"
    );
}

/// The exact forced value that once escaped a test window and made an unrelated
/// jump-hint law in another file report a clip that was not one.
fn item_233_poison() -> crate::theme::ListStyle {
    crate::theme::ListStyle::Bars {
        radius: 6.0,
        gap: 8.0,
        grow_px: 24.0,
        extent: crate::theme::BarExtent::FullWidth,
        coverage: crate::theme::BarCoverage::All,
    }
}

#[test]
fn a_deliberately_dirty_render_override_window_fails_and_restores_before_the_next_reader() {
    let before = {
        let _g = serial();
        crate::render::overrides::pins()
    };
    let leaked = std::panic::catch_unwind(|| {
        let _g = serial();
        crate::render::set_list_style_test_override(Some(item_233_poison()));
    });
    assert!(
        leaked.is_err(),
        "a checked window must reject a forced render knob it never reset"
    );
    let _g = serial();
    assert_eq!(
        crate::render::overrides::pins(),
        before,
        "the failed override window restores before its next reader enters"
    );
}

#[test]
fn a_window_that_forces_a_knob_and_then_panics_cannot_poison_the_next_test() {
    // THE headline case, and the one only a Drop-based restore buys: the
    // fixture never reaches its own reset, because it dies first. The victim
    // read `effective_list_style()` — assert on that OUTCOME, not on the
    // private snapshot, since that is what a poisoned test actually measures.
    let world_own_style = {
        let _g = serial();
        crate::render::effective_list_style()
    };
    let died = std::panic::catch_unwind(|| {
        let _g = serial();
        crate::render::set_list_style_test_override(Some(item_233_poison()));
        assert_eq!(
            crate::render::effective_list_style(),
            item_233_poison(),
            "the fixture's own window really does see the forced knob"
        );
        panic!("fixture dies before it can reset its override");
    });
    assert!(died.is_err(), "the fixture must have unwound");
    let _g = serial();
    assert_eq!(
        crate::render::effective_list_style(),
        world_own_style,
        "the next test reads the world's own list style, not the dead fixture's"
    );
    assert!(
        crate::render::overrides::leaked_knobs(
            &crate::render::overrides::pins(),
            &crate::render::overrides::OverridePins::none()
        )
        .is_empty(),
        "no forced knob at all survives an unwinding window"
    );
}

#[test]
fn every_forced_knob_is_restored_not_just_the_one_that_bit_us() {
    // The axis the reported leak did not name: it was `list_style`, but the
    // guard must sweep the WHOLE roster — a knob left out of the restore is the
    // next expensive discovery. Force all eleven, die, require a pristine exit.
    let before = {
        let _g = serial();
        crate::render::overrides::pins()
    };
    let died = std::panic::catch_unwind(|| {
        let _g = serial();
        crate::render::set_test_override(crate::render::RenderOverrides {
            title_style: Some(crate::theme::TitleStyle::InlinePrefix),
            card_anchor: Some(crate::theme::CardAnchor::TopLeft),
            chrome_face: Some(crate::theme::ChromeFace::Named("IBM Plex Mono")),
            motion_juice: Some(crate::theme::MotionJuice {
                entrance: crate::theme::OverlayEntrance::SpringIn,
                band: crate::theme::BandResponse::Slide,
            }),
            slant: Some(crate::render::SlantProbe {
                px_per_row: 3.0,
                italic: true,
            }),
            list_style: Some(item_233_poison()),
            facet_style: Some(crate::theme::FacetStyle::Band),
            pane_split: Some(crate::theme::PaneSplit::Split),
            density: Some(crate::render::TypeDensity {
                scale: 1.25,
                leading: 1.5,
            }),
            overlay_motion: Some(crate::render::OverlayMotionProbe {
                enter: 0.5,
                band: 0.25,
            }),
        });
        crate::render::livingband::set_motion_test_override(Some(
            crate::render::livingband::MotionForce {
                choreo: crate::render::livingband::Choreo::Morph,
                phase: Some(0.5),
            },
        ));
        panic!("a fixture that forced everything and died");
    });
    assert!(died.is_err());
    let _g = serial();
    let after = crate::render::overrides::pins();
    assert_eq!(
        crate::render::overrides::leaked_knobs(&before, &after),
        Vec::<String>::new(),
        "every forced knob is restored, including the living-band probe that \
         is not a RenderOverrides field"
    );
}

#[test]
fn an_inner_guard_drop_never_releases_the_outer_hold_for_a_following_writer() {
    // Models `apply_transition` (and any writer): while a test holds the guard,
    // a nested acquire+drop must NOT release the test's outer hold, so a
    // FOLLOWING nested writer still serializes under the same outer window.
    let outer = serial();
    {
        let _inner = serial();
    }
    assert!(
        currently_held(),
        "the outer hold survives an inner acquire+drop"
    );
    crate::page::set_measure(44);
    assert_eq!(crate::page::measure(), 44);
    crate::page::set_measure(crate::page::DEFAULT_MEASURE);
    drop(outer);
    assert!(!currently_held());
}
