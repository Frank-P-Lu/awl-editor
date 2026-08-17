use super::*;

// A Goto picker over N synthetic rows (row0..rowN-1), empty query so items are in
// corpus order 1:1.
fn deep(n: usize) -> OverlayState {
    let corpus: Vec<String> = (0..n).map(|i| format!("row{i}")).collect();
    OverlayState::new(OverlayKind::Goto, corpus, vec![], vec![])
}

#[test]
fn hover_only_highlights_visible_rows_and_never_scrolls() {
    // 40 rows, window 12. Keyboard down to row 30 → the window scrolls so 30 is the
    // BOTTOM visible row (scroll = 30+1-12 = 19), showing items 19..=30.
    let mut ov = deep(40);
    ov.move_sel(30);
    assert_eq!(ov.selected, 30);
    assert_eq!(ov.scroll, 19);
    // Hovering a row INSIDE the visible band re-highlights it WITHOUT moving scroll.
    assert!(ov.hover_select(21));
    assert_eq!(ov.selected, 21);
    assert_eq!(ov.scroll, 19, "a hover must NOT move the scroll window");
    // Hovering the TOP visible row: still no scroll (the bug was this scrolling up).
    assert!(ov.hover_select(19));
    assert_eq!(ov.scroll, 19);
    // Hovering ABOVE the band (a row scrolled off the top) is REJECTED, no change.
    assert!(!ov.hover_select(5));
    assert_eq!(ov.selected, 19);
    assert_eq!(ov.scroll, 19);
    // Hovering BELOW the band (past the last visible row) is likewise rejected.
    assert!(!ov.hover_select(31));
    assert_eq!(ov.selected, 19);
    assert_eq!(ov.scroll, 19);
    // Re-hovering the SAME row is a no-op (returns false, nothing moved).
    assert!(!ov.hover_select(19));
}

/// THE REAL-MOTION GATE LAW: a world's re-layout under a STATIONARY
/// pointer must never synthesize a new hover selection. `hover_at` is the
/// production seam `app/input/mouse.rs::overlay_hover` calls on every
/// `CursorMoved` (real travel OR a platform-synthesized duplicate at the
/// identical coordinates); its ONLY input beyond the OverlayState itself is the
/// `(px, py, hit)` triple the caller resolved, so this test drives it exactly
/// like the live seam does — no GPU/window needed.
///
/// NON-VACUOUS: the test first shows the hazard is REAL — the SAME `hover_select`
/// this gate wraps WOULD flip the highlight if called directly with the
/// post-relayout hit (simulating the pre-item-85 code, which called it
/// unconditionally) — before showing the gated `hover_at` refuses to.
#[test]
fn hover_at_gates_on_real_pointer_motion_not_a_relayout_hit_test_change() {
    let mut ov = deep(10);
    assert_eq!(
        ov.last_hover_px, None,
        "a fresh summon has no hover memory yet"
    );

    // The pointer's FIRST real hover: it rests at (100.0, 200.0), which the
    // caller's hit-test resolves to row 3 (whatever picker layout was current).
    assert!(
        ov.hover_at(100.0, 200.0, Some(3)),
        "the first hover at a fresh position always re-hit-tests"
    );
    assert_eq!(ov.selected, 3);
    assert_eq!(ov.last_hover_px, Some((100.0, 200.0)));

    // A THEME-PICKER WORLD JUMP relayouts the card (reanchor / Pane<->Bars row
    // pitch / font-reshape settle) — simulated here as "the row now under THE
    // EXACT SAME PIXEL is different" (row 7, not row 3). The pointer itself never
    // moved a single pixel.
    let relayout_hit = Some(7usize);

    // PROVE THE HAZARD IS REAL (non-vacuous): calling `hover_select` directly with
    // the post-relayout hit — exactly what the pre-item-85 `overlay_hover` did
    // unconditionally — DOES cascade the highlight onto the new row.
    let mut naive = ov.clone();
    assert!(
        naive.hover_select(7),
        "an UNGATED re-hit-test really would flip the highlight — the bug is real"
    );
    assert_eq!(
        naive.selected, 7,
        "the naive path cascades onto whatever row the relayout put there"
    );

    // THE ACTUAL LAW: the SAME stationary pixel, run back through the gated
    // `hover_at`, must NOT move the highlight — the pointer didn't move, only the
    // content under it did.
    assert!(
        !ov.hover_at(100.0, 200.0, relayout_hit),
        "a relayout under a stationary pointer must not report a hover move"
    );
    assert_eq!(
        ov.selected, 3,
        "the highlighted world stays stable across its own re-layout"
    );
    assert_eq!(
        ov.last_hover_px,
        Some((100.0, 200.0)),
        "the stationary position is still recorded (idempotent, not a no-op on the memory)"
    );

    // A SECOND spurious duplicate at the identical position (e.g. another
    // synthesized CursorMoved before the user's hand ever moves) is likewise
    // inert — this isn't a one-shot debounce, it holds indefinitely under a
    // genuinely still pointer (no cascade across MULTIPLE spurious events either).
    assert!(!ov.hover_at(100.0, 200.0, Some(2)));
    assert_eq!(ov.selected, 3);

    // ORDINARY HARDWARE JITTER under a resting hand (a real, but
    // sub-slop, pixel of travel) must likewise stay inert: a stationary hand's
    // mouse routinely emits a physical pixel or two of noise, and that must
    // never read as "the user moved the pointer".
    assert!(
        !ov.hover_at(101.0, 200.0, Some(7)),
        "a 1px jitter — real travel, but under the slop — must not steal the highlight"
    );
    assert_eq!(
        ov.selected, 3,
        "jitter below the slop leaves the highlight exactly where it was"
    );
    // The anchor is STICKY below the slop (never re-based on a rejected check),
    // so it still reads the ORIGINAL (100,200) — not the jittered (101,200).
    assert_eq!(ov.last_hover_px, Some((100.0, 200.0)));

    // REAL MOTION, finally: the pointer genuinely moves PAST the slop. NOW the
    // gate re-hit-tests and the highlight follows — real travel is never
    // suppressed, only a stationary re-layout (or a resting hand's jitter) is.
    let past_slop = 100.0 + super::nav::HOVER_MOVE_SLOP_PX + 1.0;
    assert!(
        ov.hover_at(past_slop, 200.0, Some(7)),
        "a real move past the slop must re-hit-test normally, on the very first such event"
    );
    assert_eq!(ov.selected, 7);
    assert_eq!(ov.last_hover_px, Some((past_slop, 200.0)));
}

/// THE MOVEMENT-SLOP BOUNDARY LAW, in both directions: a real move
/// of exactly `HOVER_MOVE_SLOP_PX - 1` (still within the slop) must NOT take
/// over; `HOVER_MOVE_SLOP_PX + 1` (just past it) MUST — on that very event,
/// no added latency. Pure distance math (no clock, no pipeline), mirroring
/// `app::input::tests::exceeds_drag_slop_is_false_for_sub_slop_jitter`'s own
/// shape for `DRAG_ARM_SLOP_PX`.
#[test]
fn hover_at_movement_slop_boundary_law() {
    let slop = super::nav::HOVER_MOVE_SLOP_PX;
    assert!(
        slop > 1.0,
        "the boundary probes below assume at least 1px of headroom"
    );

    // Below the slop: no take-over.
    let mut under = deep(10);
    assert!(
        under.hover_at(0.0, 0.0, Some(1)),
        "cold-start anchor at the origin"
    );
    assert!(
        !under.hover_at(slop - 1.0, 0.0, Some(5)),
        "{}px (slop - 1) must NOT take over",
        slop - 1.0
    );
    assert_eq!(
        under.selected, 1,
        "the anchor's selection survives a sub-slop move"
    );

    // Exactly at the slop: the gate is a STRICT inequality (`>`), so a move of
    // EXACTLY the slop distance still does not take over (matches
    // `PointerInput::exceeds_drag_slop`'s own strict `>`).
    let mut at = deep(10);
    assert!(at.hover_at(0.0, 0.0, Some(1)));
    assert!(
        !at.hover_at(slop, 0.0, Some(5)),
        "a move of EXACTLY the slop must not take over (strict >)"
    );
    assert_eq!(at.selected, 1);

    // Past the slop: takes over immediately, first event.
    let mut over = deep(10);
    assert!(over.hover_at(0.0, 0.0, Some(1)));
    assert!(
        over.hover_at(slop + 1.0, 0.0, Some(5)),
        "{}px (slop + 1) MUST take over, on the first such event",
        slop + 1.0
    );
    assert_eq!(
        over.selected, 5,
        "the real move wins outright — no debounce, no dead zone"
    );

    // DIAGONAL travel: the gate is a squared-distance circle, not an x-axis-only
    // check — every case above moves along dy=0, which would pass even a buggy
    // per-axis (Chebyshev/Manhattan) gate instead of the intended Euclidean one.
    // dx = dy = slop/sqrt(2) sits exactly ON the circle (dx^2+dy^2 == slop^2),
    // so a hair under/over it must land on the same side as the axis-aligned
    // boundary above.
    let leg = slop / std::f32::consts::SQRT_2;
    let mut diag_under = deep(10);
    assert!(diag_under.hover_at(0.0, 0.0, Some(1)));
    assert!(
        !diag_under.hover_at(leg - 0.01, leg - 0.01, Some(5)),
        "a diagonal move a hair under the slop circle must NOT take over"
    );
    assert_eq!(diag_under.selected, 1);

    let mut diag_over = deep(10);
    assert!(diag_over.hover_at(0.0, 0.0, Some(1)));
    assert!(
        diag_over.hover_at(leg + 0.01, leg + 0.01, Some(5)),
        "a diagonal move a hair past the slop circle MUST take over"
    );
    assert_eq!(diag_over.selected, 5);
}

/// THE KEYBOARD-BASELINE STAMP LAW: a PURE keyboard session (the
/// pointer never explicitly hovered a row, so `last_hover_px` is `None`) must
/// not hand its selection to wherever a motionless pointer happens to rest.
/// This is a DIFFERENT hazard than the movement-slop distance gate proven
/// above: with no prior hover at all, `hover_at`'s own cold-start rule (a
/// `None` baseline is unconditional real motion — the same rule that lets a
/// picker's very first genuine hover work with no warm-up) would otherwise
/// treat the pointer's first incidental `CursorMoved`, however small or
/// irrelevant, as real travel. `arm_hover_baseline` (stamped by `App::apply`
/// / `ReplaySession::apply_chord` after every keyboard-driven action) closes
/// this by giving the gate a real anchor to measure from BEFORE that first
/// check ever happens.
///
/// NON-VACUOUS: the UNSTAMPED scenario is proven to really steal the
/// selection first, then the STAMPED one is proven not to.
#[test]
fn keyboard_baseline_stamp_protects_a_pointer_that_was_never_explicitly_hovered() {
    let mut ov = deep(30);
    assert_eq!(
        ov.last_hover_px, None,
        "a pure keyboard session never calls hover_at"
    );

    // A plain keyboard session: Down x5. The pointer has been resting at
    // (500, 500) the whole time — nobody ever hovered a row with it.
    ov.move_sel(5);
    assert_eq!(ov.selected, 5);

    // WITHOUT the baseline stamp: the pointer's incidental first hover check
    // (e.g. a platform `CursorMoved` fired for an unrelated reason, landing
    // exactly where the hand already rested) hits whatever row (500, 500)
    // resolves to — simulated as row 8, still inside the fresh window — and
    // with `last_hover_px` still `None`, `hover_at`'s cold-start rule treats
    // this as real motion and steals the keyboard's selection outright.
    // Proven on a CLONE so it doesn't corrupt the state the actual law
    // asserts against below.
    let mut unstamped = ov.clone();
    assert!(
        unstamped.hover_at(500.0, 500.0, Some(8)),
        "UNSTAMPED: a None baseline really does treat the pointer's first \
         incidental check as unconditional motion — the bug is real"
    );
    assert_eq!(
        unstamped.selected, 8,
        "and it really does steal the keyboard's selection"
    );

    // THE ACTUAL LAW: `App::apply`'s stamp ran after every one of those five
    // keyboard presses, so the baseline is already (500, 500) by the time
    // this SAME incidental check fires.
    ov.arm_hover_baseline(500.0, 500.0);
    assert!(
        !ov.hover_at(500.0, 500.0, Some(8)),
        "STAMPED: the SAME incidental check, now measured from the keyboard's \
         own baseline, must not steal the selection"
    );
    assert_eq!(ov.selected, 5, "the keyboard's selection survives");
}

/// A cold-start regression guard: with NO keyboard action ever having run (so
/// `arm_hover_baseline` never fired either), a picker's very first genuine
/// hover must still select immediately — the slop/baseline widening
/// must never turn into added latency or a dead zone for ordinary mouse-only
/// use.
#[test]
fn hover_still_works_normally_from_a_cold_start_with_no_prior_keyboard_action() {
    let mut ov = deep(10);
    assert_eq!(ov.last_hover_px, None);
    assert!(
        ov.hover_at(42.0, 42.0, Some(6)),
        "a cold-start hover, with no keyboard action ever run, selects on the first check"
    );
    assert_eq!(ov.selected, 6);
}

/// KEYBOARD NAV SURVIVES A POINTER PARKED ANYWHERE: sweeps the
/// pointer's parked position across four representative cases relative to
/// where the keyboard ends up — the row ABOVE it, the row BELOW it, a FAR row
/// (the opposite end of the list), and the row the keyboard landed ON itself
/// (parking exactly on the destination is the one case where a "steal" would
/// be invisible in the sidecar — `selected` already reads right — so it is
/// swept precisely because a naive test would miss it).
#[test]
fn keyboard_nav_survives_a_pointer_parked_over_any_row_relative_to_the_destination() {
    const LANDING: usize = 20;
    let scenarios: [(&str, usize); 4] = [
        ("row above the landing", LANDING - 1),
        ("row below the landing", LANDING + 1),
        ("a far row", 0),
        ("the landing row itself", LANDING),
    ];
    for (label, parked_row) in scenarios {
        let mut ov = deep(30);
        // The pointer rests over `parked_row` — whatever row a hit-test would
        // currently report at some ordinary fixed pixel P. Established with
        // `arm_hover_baseline` (not `hover_at`) so this doesn't itself force a
        // real hover-select — the pointer is simply PARKED there, exactly as
        // it would be if the user had never touched the mouse since opening
        // the picker but the OS still reports SOME resting coordinate.
        let (px, py) = (77.0, 88.0);
        ov.arm_hover_baseline(px, py);

        // A real keyboard session drives the selection to LANDING; the
        // pointer physically never moves. `App::apply` stamps the baseline
        // again after this action too (idempotent: same px, py).
        ov.move_sel(LANDING as isize);
        assert_eq!(
            ov.selected, LANDING,
            "{label}: keyboard reaches the landing row"
        );
        ov.arm_hover_baseline(px, py);

        // A stray check at the SAME parked pixel — jitter, a relayout, or a
        // scrolled window's row identity shifting under it, whatever `hit`
        // the pipeline would now report there — must not move the selection
        // off LANDING, in every one of the four parked positions.
        assert!(
            !ov.hover_at(px, py, Some(parked_row)),
            "{label}: a stationary parked pointer must not override the keyboard"
        );
        assert_eq!(
            ov.selected, LANDING,
            "{label}: keyboard selection survives a stray re-check"
        );
    }
}

/// NO-WILDCARD SURFACE SWEEP: `hover_at`/`arm_hover_baseline`
/// never branch on `self.kind` at all — the `match` below is a compile-time
/// EXHAUSTIVE (no `_` arm) check over every `OverlayKind` variant, so a new
/// variant fails THIS FILE to compile until a match arm is added for it,
/// forcing a developer to consciously touch this law rather than have a new
/// kind silently inherit an untested code path. The loop drives
/// [`OverlayKind::ALL`], generated from the same declaration as the enum, so
/// the exhaustive match and runtime sweep grow together.
#[test]
fn hover_movement_slop_gate_holds_across_every_overlay_kind_no_wildcard() {
    fn sweep_this_kind(kind: OverlayKind) {
        // Exhaustive on purpose (no `_` arm): the real no-wildcard guard.
        match kind {
            OverlayKind::Goto
            | OverlayKind::Project
            | OverlayKind::ProjectBrowse
            | OverlayKind::Browse
            | OverlayKind::Theme
            | OverlayKind::Caret
            | OverlayKind::Dictionary
            | OverlayKind::CjkLang
            | OverlayKind::Date
            | OverlayKind::Keymap
            | OverlayKind::MoveDest
            | OverlayKind::ExportDest
            | OverlayKind::Command
            | OverlayKind::Spell
            | OverlayKind::Keybindings
            | OverlayKind::History
            | OverlayKind::Conflict
            | OverlayKind::Settings
            | OverlayKind::Assets
            | OverlayKind::Rename
            | OverlayKind::InsertLink
            | OverlayKind::KeepName
            | OverlayKind::Context => {}
        }
        let ctx = format!("kind={kind:?}");
        let corpus: Vec<String> = (0..30).map(|i| format!("row{i}")).collect();
        let mut ov = OverlayState::new(kind, corpus, vec![], vec![]);

        // A real, earlier hover establishes the anchor on row 2 (inside the
        // fresh window, so the hover genuinely lands).
        assert!(ov.hover_at(20.0, 20.0, Some(2)), "{ctx}");
        assert_eq!(ov.selected, 2, "{ctx}");

        // A real keyboard session scrolls the window far past it.
        ov.move_sel(23);
        assert_eq!(ov.selected, 25, "{ctx}");
        ov.arm_hover_baseline(20.0, 20.0); // App::apply's stamp

        // A REAL 1px jitter off the parked pixel — not an exact repeat (item
        // 85's own exact-equality gate already refused a bare duplicate; the
        // regression this law names needs genuine, if tiny, travel) — now
        // hit-testing to a DIFFERENT row because the window scrolled under
        // it, must not steal the keyboard's selection, for every kind
        // without exception. Row 22: safely INSIDE the post-scroll visible
        // band for every kind's own `window_rows` (Spell's smallest, 6, still
        // covers [20, 26) here) — this must fail on the GATE, never on
        // `hover_select`'s own separate visible-band rejection, or the
        // assertion would pass for the wrong reason regardless of the gate.
        assert!(
            !ov.hover_at(21.0, 20.0, Some(22)),
            "{ctx}: a 1px jitter off a stationary pointer must not steal the keyboard's selection"
        );
        assert_eq!(ov.selected, 25, "{ctx}: keyboard selection survives");
    }

    for kind in OverlayKind::ALL {
        sweep_this_kind(kind);
    }
}

/// A hover that lands OFF every row (`hit: None`) — e.g. the query line, a foot
/// hint, an inter-row gap — still records the pointer's position (so leaving then
/// returning to a row tracks correctly) but never touches `selected`, matching
/// `hover_select`'s own off-row behavior.
#[test]
fn hover_at_off_a_row_records_position_without_selecting() {
    let mut ov = deep(10);
    ov.move_sel(2); // selected = 2, established by keyboard (not hover)
    assert!(
        !ov.hover_at(50.0, 50.0, None),
        "off a row: no selection move"
    );
    assert_eq!(ov.selected, 2, "the keyboard-set selection is untouched");
    assert_eq!(
        ov.last_hover_px,
        Some((50.0, 50.0)),
        "position still recorded"
    );
}

/// The keyboard half of the law: ↓/↑ (`move_sel(±1)`, what
/// `Action::NextLine`/`PreviousLine` drive) advances EXACTLY one visible row per
/// press, from wherever the selection actually sits — including right after a
/// mouse hover moved it, proving the two input kinds compose cleanly (a hover's
/// own pixel bookkeeping in `last_hover_px` never leaks into how far a keypress
/// travels).
#[test]
fn keyboard_advances_exactly_one_visible_row_per_press_even_after_a_hover() {
    let mut ov = deep(20);
    assert!(
        ov.hover_at(10.0, 10.0, Some(4)),
        "a mouse hover selects row 4"
    );
    assert_eq!(ov.selected, 4);
    ov.move_sel(1);
    assert_eq!(
        ov.selected, 5,
        "one keyboard press = exactly one row of movement"
    );
    ov.move_sel(1);
    assert_eq!(ov.selected, 6);
    ov.move_sel(-1);
    assert_eq!(
        ov.selected, 5,
        "PreviousLine likewise moves exactly one row back"
    );
}

#[test]
fn keyboard_move_keeps_selection_in_the_window() {
    let mut ov = deep(40);
    // Down a page-ish: selection tracks, window scrolls the minimum to keep it shown.
    ov.move_sel(15);
    assert_eq!(ov.selected, 15);
    assert_eq!(ov.scroll, 4); // 15+1-12
    assert!(ov.selected >= ov.scroll && ov.selected < ov.scroll + ov.window_rows());
    // Back up above the window → scroll follows up (never leaves selection off-screen).
    ov.move_sel(-14);
    assert_eq!(ov.selected, 1);
    assert_eq!(ov.scroll, 1);
    // A short list never scrolls.
    let mut small = deep(5);
    small.move_sel(100);
    assert_eq!(small.selected, 4);
    assert_eq!(small.scroll, 0);
}

#[test]
fn query_edit_resets_scroll_to_top() {
    let mut ov = deep(40);
    ov.move_sel(30);
    assert_eq!(ov.scroll, 19);
    ov.push('r'); // matches every "rowN" → list stays long, but selection resets top
    assert_eq!(ov.selected, 0);
    assert_eq!(ov.scroll, 0);
}
