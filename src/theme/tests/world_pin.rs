use super::super::*;

/// THE WORLD PIN, at its own seam: [`WorldPin`] is the EXPLICIT tool a
/// test reaches for when it renders a specific world — it snapshots the active
/// index on construction and stores it back on DROP, however the global moved in
/// between (one swap, five swaps, a `cycle`). It is deliberately opt-in: the
/// active world is a process GLOBAL that PRODUCTION code writes too (the theme
/// picker's live preview), so nothing ambient may restore it behind an action's
/// back — a test that swaps worlds says so by holding a pin.
///
/// Run while holding the standing serialization guard, so no other thread can
/// move the global under the assertions.
#[test]
fn a_world_pin_restores_the_active_world_however_it_moved() {
    let _g = crate::testlock::serial();
    let before = active_index();
    {
        let _pin = WorldPin::snapshot();
        set_active_by_name("Tawny");
        cycle(1);
        // A world that is never `before`, whatever `before` is (the roster is
        // longer than 5).
        set_active(before + 5);
        assert_ne!(
            active_index(),
            before,
            "the pin's window really did move the global (a vacuous law otherwise)"
        );
    }
    assert_eq!(
        active_index(),
        before,
        "a WorldPin must put the world it snapshotted back when it drops"
    );
    // The pinning CONSTRUCTOR: pin + switch in one move, same restore.
    {
        let pin = WorldPin::world("Bombora").expect("Bombora is a world");
        assert_eq!(pin.restores_to(), before);
        assert_eq!(active().name, "Bombora");
    }
    assert_eq!(active_index(), before, "…including WorldPin::world");
    assert!(
        WorldPin::world("Not A World").is_none(),
        "an unknown world changes nothing and yields no pin"
    );
    assert_eq!(active_index(), before);

    // AND ON THE UNWIND PATH: a law that fails mid-sweep (a roster-wide rail
    // sweep is exactly this shape) must still hand the next test a clean world —
    // the restore rides `Drop`, so a panic through the pin's scope restores it.
    let quiet = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {})); // the deliberate panic prints nothing
    let out = std::panic::catch_unwind(|| {
        let _pin = WorldPin::world("Bombora").expect("Bombora is a world");
        panic!("a law failing inside the pin's window");
    });
    std::panic::set_hook(quiet);
    assert!(out.is_err(), "the panic really happened (non-vacuous)");
    assert_eq!(
        active_index(),
        before,
        "the pin restores while unwinding, not just on a clean exit"
    );
}

/// The process-global writer itself is the enforcement seam. A helper cannot
/// hide an unguarded write behind a caller that merely happens to contain a
/// lock acquisition in its source: the write is rejected on the thread that
/// actually performs it, before ACTIVE changes.
#[test]
fn an_unguarded_world_leak_fails_at_the_runtime_choke_point_before_it_writes() {
    // Every ACTIVE observation owns the same serialization window as its
    // writers. The writer below is deliberately on a fresh, unguarded thread;
    // it receives the already-observed index rather than reading ACTIVE itself.
    let before = {
        let _g = crate::testlock::serial();
        active_index()
    };
    let leaked = std::thread::spawn(move || {
        assert!(!crate::testlock::currently_held());
        std::panic::catch_unwind(|| set_active(before + 1)).is_err()
    })
    .join()
    .unwrap();

    assert!(leaked, "a deliberately unguarded writer must fail");
    let _g = crate::testlock::serial();
    assert_eq!(
        active_index(),
        before,
        "the choke point rejects the leak before mutating the world"
    );
}

/// The choke-point probe keeps every ACTIVE read inside `serial()`, even while
/// its writer thread remains deliberately unguarded. A compliant writer changes
/// and restores the world in its own window before the probe observes the
/// rejected write's result; the probe therefore cannot mistake that in-flight
/// change for corruption caused by its rejected call.
#[test]
fn unguarded_choke_point_probe_serializes_its_active_reads() {
    let before = {
        let _g = crate::testlock::serial();
        active_index()
    };
    let (read_tx, read_rx) = std::sync::mpsc::channel();
    let (changed_tx, changed_rx) = std::sync::mpsc::channel();

    let probe = std::thread::spawn(move || {
        assert!(!crate::testlock::currently_held());
        let probe_before = {
            let _g = crate::testlock::serial();
            active_index()
        };
        read_tx.send(probe_before).unwrap();
        changed_rx.recv().unwrap();
        let rejected = std::panic::catch_unwind(|| set_active(probe_before + 2));
        let observed = {
            let _g = crate::testlock::serial();
            active_index()
        };
        (rejected.is_err(), observed)
    });

    assert_eq!(read_rx.recv().unwrap(), before);
    {
        let _writer_window = crate::testlock::serial();
        set_active(before + 1);
        set_active(before);
    }
    changed_tx.send(()).unwrap();
    let (rejected, observed) = probe.join().unwrap();

    assert!(rejected, "the deliberately unguarded write is rejected");
    assert_eq!(
        observed, before,
        "a probe whose observations serialize with writers sees no mutation from its rejected call"
    );
}

/// Production's own guarded action window is not a restore boundary. The theme
/// picker preview is a real product write and must remain active after
/// `apply_transition` returns even when the caller did not already hold the test lock.
#[test]
fn a_world_a_production_action_sets_survives_that_action() {
    use crate::actions::ActionCtx;
    use crate::keymap::Action;
    use crate::overlay::{OverlayKind, OverlayState};

    // Keep the ONE mutex held across arrange / action / observation / cleanup,
    // while `apply_transition` requests its own product guard reentrantly. This avoids
    // dropping the lock around the observation, which would briefly expose this
    // deliberately changed world to compliant sibling tests.
    let _probe = crate::testlock::product();
    let original = active_index();
    set_active(0);
    let product_requests_before = crate::testlock::product_requests();

    let names: Vec<String> = world_names().iter().map(|n| n.to_string()).collect();
    let mut overlay = crate::overlay::Journey::seeded(Some(OverlayState::new_theme(names, 0)));
    let mut buffer = crate::buffer::Buffer::scratch();
    let mut shift = false;
    let mut zoom = 1.0f32;
    let mut search = None;
    let mut make_overlay = |_k: OverlayKind| -> Option<OverlayState> { None };
    let mut browse_to = |_k: OverlayKind, _r: Option<String>| -> Option<OverlayState> { None };
    let mut ctx = ActionCtx {
        buffer: &mut buffer,
        shift_selecting: &mut shift,
        zoom: &mut zoom,
        search: &mut search,
        scroll_page_lines: 1,
        journey: &mut overlay,
        make_overlay: &mut make_overlay,
        browse_to: &mut browse_to,
        oracle: None,
    };

    crate::actions::apply_transition(&mut ctx, &Action::LineEnd, false).primary();
    assert_eq!(
        crate::testlock::product_requests(),
        product_requests_before + 1,
        "apply_transition must request the product door, not the checked test door"
    );
    let previewed = overlay
        .card()
        .and_then(|ov| ov.selected_value())
        .expect("the theme picker has a selected world")
        .to_string();
    assert_ne!(
        previewed, THEMES[0].name,
        "the action genuinely previews another world"
    );

    let observed = active().name.to_string();
    set_active(original);
    assert_eq!(
        observed, previewed,
        "the world the product action set must survive that action"
    );
}
