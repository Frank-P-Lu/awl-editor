//! Recording stays observational when the last document is closed.

use super::*;

#[test]
fn input_receipts_cover_empty_document_rejection_and_start_actions() {
    let _guard = crate::testlock::serial();
    for action in [
        Action::DeleteBackward,
        Action::OpenGoto,
        Action::NewDocument,
    ] {
        let name = format!("{action:?}");
        let lines = traced_with(|app| {
            assert!(crate::probe::recording());
            app.close_active_buffer();
            assert!(!app.document.has_active());
            app.apply(
                action.clone(),
                false,
                &schedule::RecordingExit::new(),
                crate::stats::Door::Menu,
            );
            match action {
                Action::DeleteBackward => assert!(!app.document.has_active()),
                Action::OpenGoto => assert!(app.workspace_state.overlay_open()),
                Action::NewDocument => assert!(app.document.has_active()),
                _ => unreachable!(),
            }
        });
        let before = lines
            .iter()
            .find(|line| line.contains(&format!("input-state before door=Menu action={name}")))
            .expect("the action records its empty-document input state");
        assert!(before.contains("doc_version=None doc_cursor=None doc_selection=None"));
        let phase = if action == Action::DeleteBackward {
            "rejected-no-document"
        } else {
            "after"
        };
        assert!(lines.iter().any(|line| {
            line.contains(&format!("input-state {phase} door=Menu action={name}"))
        }));
    }
}
