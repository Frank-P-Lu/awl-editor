use super::*;

#[test]
fn keybindings_capture_key_mode_finishes_instantly() {
    // SUMMON: the rebind menu lists the catalog with its effective chords.
    let names = crate::commands::visible_names();
    let binds = crate::commands::visible_effective_bindings(&[], &[]);
    let mut ov = OverlayState::new_keybindings(names.clone(), binds);
    assert_eq!(ov.kind.as_str(), "keybindings");
    assert_eq!(ov.item_strings(), names);
    assert!(ov.capture.is_none());
    // NAVIGATE: filter to "Undo" so the selected command is deterministic.
    for c in "undo".chars() {
        ov.push(c);
    }
    assert_eq!(ov.selected_value(), Some("Undo"));
    // ENTER → ChooseMode; default selection is KEY.
    ov.start_capture();
    let cap = ov.capture.as_ref().unwrap();
    assert_eq!(cap.stage, CaptureStage::ChooseMode);
    assert_eq!(cap.cmd_name, "Undo");
    assert!(!cap.chord_mode);
    // Choose KEY, begin recording, then ONE combo finishes instantly.
    ov.capture_move_mode(-1); // KEY row
    ov.capture_begin_recording();
    assert_eq!(ov.capture.as_ref().unwrap().stage, CaptureStage::Recording);
    let done = ov.capture_record("C-j".to_string());
    assert!(done, "KEY mode finishes on the first combo");
    assert_eq!(
        ov.capture_target(),
        Some(("undo".to_string(), "C-j".to_string()))
    );
}

#[test]
fn keybindings_capture_chord_mode_collects_then_finishes() {
    let mut ov = OverlayState::new_keybindings(
        crate::commands::visible_names(),
        crate::commands::visible_effective_bindings(&[], &[]),
    );
    for c in "save".chars() {
        ov.push(c);
    }
    assert_eq!(ov.selected_value(), Some("Save a Copy…"));
    ov.start_capture();
    ov.capture_move_mode(1); // CHORD row
    ov.capture_begin_recording();
    assert!(ov.capture.as_ref().unwrap().chord_mode);
    // First combo does NOT finish a chord; the 2-deep cap does.
    assert!(!ov.capture_record("C-x".to_string()));
    assert!(ov.capture_record("C-s".to_string()));
    // A THIRD combo is dropped (capped at 2).
    assert!(ov.capture_record("C-q".to_string()));
    assert_eq!(
        ov.capture_target(),
        Some(("save_a_copy".to_string(), "C-x C-s".to_string()))
    );
}

#[test]
fn keybindings_confirm_and_reset_helpers() {
    let mut ov = OverlayState::new_keybindings(
        crate::commands::visible_names(),
        crate::commands::visible_effective_bindings(&[], &[]),
    );
    // RESET targets the highlighted command's slug.
    for c in "redo".chars() {
        ov.push(c);
    }
    assert_eq!(ov.selected_command_slug().as_deref(), Some("redo"));
    // CONFLICT: a finished capture can be pushed into the Confirm phase, which the
    // prompt reflects (naming the clashing command). Esc-equivalent aborts it.
    ov.start_capture();
    ov.capture_begin_recording();
    ov.capture_record("C-s".to_string());
    ov.capture_into_confirm("Search forward".to_string());
    let cap = ov.capture.as_ref().unwrap();
    assert_eq!(cap.stage, CaptureStage::Confirm);
    assert!(cap.prompt().contains("Search forward"));
    ov.capture_abort();
    assert!(ov.capture.is_none());
}
