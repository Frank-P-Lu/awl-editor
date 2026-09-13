use super::*;

#[test]
fn browser_paste_bulk_work_is_constant_across_all_fields_and_payload_lengths() {
    let _guard = crate::testlock::serial();
    let _fs = crate::fs::FsGuard::install(Arc::new(seeded()));
    for field in TextField::ALL {
        for n in [1, 16_384] {
            let mut app = app();
            summon(&mut app, field);
            crate::textbox::work::take();
            app.receive_browser_paste(Ok("4".repeat(n)), true, &schedule::RecordingExit::new());
            let counts = crate::textbox::work::take();
            let want = match field {
                TextField::PickerQuery => [1, 1, 0, 1, 0],
                TextField::FindQuery => [1, 0, 1, 0, 0],
                TextField::ReplaceText => [1, 0, 0, 0, 0],
                TextField::Rename
                | TextField::InsertLink
                | TextField::KeepVersion
                | TextField::SettingsValue => [1, 0, 0, 0, 1],
            };
            assert_eq!(counts, want, "{field:?}, payload={n}");
            assert!(field_text(&app, field).unwrap().contains(&"4".repeat(n)));
            assert_eq!(app.document.buffer().text(), DOC);
        }
    }
}

#[test]
fn browser_paste_bulk_matches_typed_filters_with_unicode_and_selected_text() {
    let _guard = crate::testlock::serial();
    let _fs = crate::fs::FsGuard::install(Arc::new(seeded()));
    for field in TextField::ALL {
        for payload in ["日本語 👩‍💻 e\u{301}/42.5%", ".%.%1.%.%", "\r\n\t\0"] {
            let mut bulk = app();
            let mut typed = app();
            summon(&mut bulk, field);
            summon(&mut typed, field);
            let select = |app: &mut App| {
                if let Some(card) = app.workspace_state.overlay_mut() {
                    match field {
                        TextField::PickerQuery => card.query.select_all(),
                        TextField::Rename => card.rename_edit.as_mut().unwrap().input.select_all(),
                        TextField::InsertLink => {
                            card.link_edit.as_mut().unwrap().input.select_all()
                        }
                        TextField::KeepVersion => {
                            card.keep_edit.as_mut().unwrap().input.select_all()
                        }
                        TextField::SettingsValue => {
                            card.value_edit.as_mut().unwrap().input.select_all()
                        }
                        TextField::FindQuery | TextField::ReplaceText => unreachable!(),
                    }
                } else {
                    app.apply(
                        Action::SelectAll,
                        false,
                        &schedule::RecordingExit::new(),
                        crate::stats::Door::Menu,
                    );
                }
            };
            select(&mut bulk);
            select(&mut typed);
            crate::textbox::work::take();
            bulk.receive_browser_paste(Ok(payload.into()), true, &schedule::RecordingExit::new());
            if payload.chars().all(char::is_control) {
                assert_eq!(
                    crate::textbox::work::take(),
                    [0; 5],
                    "{field:?}: filtered paste does no field work"
                );
            }
            for c in payload.chars().filter(|c| !c.is_control()) {
                if matches!(field, TextField::FindQuery | TextField::ReplaceText) {
                    typed.receive_browser_paste(
                        Ok(c.to_string()),
                        true,
                        &schedule::RecordingExit::new(),
                    );
                } else {
                    typed.apply(
                        Action::InsertChar(c),
                        false,
                        &schedule::RecordingExit::new(),
                        crate::stats::Door::Chord,
                    );
                }
            }
            assert_eq!(
                field_text(&bulk, field),
                field_text(&typed, field),
                "{field:?}: {payload:?}"
            );
            assert_eq!(bulk.document.buffer().text(), DOC);
        }
    }
}

#[test]
#[ignore = "release-only observational measurement; not a timing assertion"]
fn browser_paste_bulk_release_measurement() {
    let _guard = crate::testlock::serial();
    let _fs = crate::fs::FsGuard::install(Arc::new(seeded()));
    for field in TextField::ALL {
        let mut app = app();
        summon(&mut app, field);
        let payload = "4日本語👩‍💻 ".repeat(2_048);
        let bytes = payload.len();
        let start = std::time::Instant::now();
        app.receive_browser_paste(Ok(payload), true, &schedule::RecordingExit::new());
        println!(
            "paste {field:?}: bytes={bytes} elapsed_us={} output_chars={}",
            start.elapsed().as_micros(),
            field_text(&app, field).unwrap().chars().count()
        );
        assert_eq!(app.document.buffer().text(), DOC);
    }
}
