//! The browser payload enters the shared action spine only while awl has focus.

use crate::app::*;

impl App {
    #[cfg(target_arch = "wasm32")]
    pub(in crate::app) fn sync_browser_paste_policy(&mut self) {
        let modifier = match crate::convention::Convention::current() {
            crate::convention::Convention::Mac => ModifiersState::SUPER,
            crate::convention::Convention::Linux => ModifiersState::CONTROL,
        };
        let allowed = self.input.keyboard.preedit.is_empty()
            && !self
                .workspace_state
                .overlay()
                .is_some_and(|card| card.capture.is_some())
            && self
                .input
                .keyboard
                .keymap
                .single_action(&Key::Character("v".into()), modifier)
                == Some(&Action::Yank);
        if let Some(clipboard) = &self.clipboard {
            clipboard.allow_gesture.set(allowed);
        }
    }

    pub(in crate::app) fn receive_browser_paste(
        &mut self,
        payload: Result<String, ()>,
        focused: bool,
        exit: &dyn schedule::Exit,
    ) {
        if !focused
            || self
                .workspace_state
                .overlay()
                .is_some_and(|card| card.capture.is_some())
        {
            return;
        }
        match payload {
            Ok(text) if !text.is_empty() => {
                self.feed_peek(crate::peek::PeekStimulus::KeyJoined);
                self.apply(
                    Action::PasteText(text),
                    false,
                    exit,
                    crate::stats::Door::Palette,
                );
            }
            Ok(_) => {}
            Err(()) => self.browser_paste_notice(),
        }
    }

    pub(in crate::app) fn browser_paste_notice(&mut self) {
        self.set_sticky_notice(
            "Copy plain text, then use the browser’s Edit → Paste command \
             or its standard Paste shortcut."
                .to_string(),
        );
        self.sync_view(false);
        self.request_frame();
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    fn app() -> App {
        let mut app = App::new_hermetic(None, PathBuf::from("/tmp"), Config::empty());
        app.document.set_text("before selected after");
        app.document.set_kill("STALE INTERNAL TEXT");
        app
    }

    #[test]
    fn browser_paste_payload_replaces_unicode_selection_once_and_one_undo_restores() {
        let _guard = crate::testlock::serial();
        let mut app = app();
        let exit = schedule::RecordingExit::new();
        app.document.select_range(7, 15);
        let payload = "日本語 👩‍💻 e\u{301}\r\nsecond line";
        app.receive_browser_paste(Ok(payload.into()), true, &exit);
        assert_eq!(
            app.document.buffer().text(),
            "before 日本語 👩‍💻 e\u{301}\nsecond line after"
        );
        assert_eq!(
            app.document.buffer().kill_buffer(),
            payload.replace("\r\n", "\n")
        );
        app.apply(Action::Undo, false, &exit, crate::stats::Door::Chord);
        assert_eq!(app.document.buffer().text(), "before selected after");
        app.receive_browser_paste(Ok("再".into()), true, &exit);
        app.receive_browser_paste(Ok("再".into()), true, &exit);
        assert_eq!(app.document.buffer().text().matches('再').count(), 2);
    }

    #[test]
    fn browser_paste_empty_unsupported_and_unfocused_never_yank_stale_text() {
        let _guard = crate::testlock::serial();
        for (payload, focused) in [
            (Ok(String::new()), true),
            (Err(()), true),
            (Ok("external".into()), false),
        ] {
            let mut app = app();
            app.document.select_range(7, 15);
            let before = app.document.buffer().version();
            let selection = app.document.buffer().selection_range();
            app.receive_browser_paste(payload, focused, &schedule::RecordingExit::new());
            assert_eq!(app.document.buffer().text(), "before selected after");
            assert_eq!(app.document.buffer().version(), before);
            assert_eq!(app.document.buffer().selection_range(), selection);
        }
    }

    #[test]
    fn browser_paste_gesture_respects_custom_binding_and_pending_prefix() {
        let _guard = crate::testlock::serial();
        for convention in [
            crate::convention::Convention::Mac,
            crate::convention::Convention::Linux,
        ] {
            let modifier = match convention {
                crate::convention::Convention::Mac => ModifiersState::SUPER,
                crate::convention::Convention::Linux => ModifiersState::CONTROL,
            };
            let key = Key::Character("v".into());
            let mut map = crate::keymap::KeymapState::new_with_convention(convention);
            assert_eq!(map.single_action(&key, modifier), Some(&Action::Yank));
            let chord = if convention == crate::convention::Convention::Mac {
                "s-v"
            } else {
                "C-v"
            };
            map.apply_overrides(&[("undo".into(), vec![chord.into()])]);
            assert_eq!(map.single_action(&key, modifier), Some(&Action::Undo));
            map.apply_overrides(&[("paste".into(), vec!["C-x v".into()])]);
            map.apply_linux_keep(&["C-x".into()]);
            map.resolve(&Key::Character("x".into()), &ModifiersState::CONTROL.into());
            assert_eq!(map.single_action(&key, modifier), None);
        }
    }

    #[test]
    fn browser_paste_leaves_every_capture_stage_and_existing_binding_untouched() {
        let _guard = crate::testlock::serial();
        for stage in crate::overlay::CaptureStage::ALL {
            for chord_mode in [false, true] {
                for recorded in [
                    vec![],
                    vec!["C-x".to_string()],
                    vec!["C-x".into(), "v".into()],
                ] {
                    let mut app = app();
                    app.document.select_range(7, 15);
                    let mut card = crate::overlay::OverlayState::new_keybindings(
                        crate::commands::visible_names(),
                        crate::commands::visible_effective_bindings(
                            &[],
                            &[],
                            crate::keymap::KeymapFlavor::Native,
                        ),
                    );
                    card.set_query_text("undo");
                    card.query.select_all();
                    card.start_capture();
                    let capture = card.capture.as_mut().unwrap();
                    capture.stage = stage;
                    capture.chord_mode = chord_mode;
                    capture.captured = recorded;
                    capture.conflict = Some("Save".into());
                    let before_capture = card.capture.clone();
                    let before_query = card.query.clone();
                    let before_bindings = format!("{:?}", card.rows);
                    let before_config = app.config.keys.clone();
                    let before_version = app.document.buffer().version();
                    app.workspace_state.install_overlay_for_test(card);
                    for payload in [Ok("s-\n日本語".into()), Ok(String::new()), Err(())] {
                        app.receive_browser_paste(payload, true, &schedule::RecordingExit::new());
                        let card = app.workspace_state.overlay().unwrap();
                        assert_eq!(card.capture, before_capture, "{stage:?} chord={chord_mode}");
                        assert_eq!(card.query, before_query);
                        assert_eq!(format!("{:?}", card.rows), before_bindings);
                        assert_eq!(app.config.keys, before_config);
                        assert_eq!(app.document.buffer().text(), "before selected after");
                        assert_eq!(app.document.buffer().version(), before_version);
                        assert_eq!(app.document.buffer().selection_range(), Some((7, 15)));
                    }
                }
            }
        }
    }

    #[test]
    fn browser_paste_queued_before_composition_cannot_edit_after_composition_starts_or_ends() {
        let _guard = crate::testlock::serial();
        use crate::app::browser_paste_events::{Paste, PasteEvents};
        let mut events = PasteEvents::default();
        let Paste::Read(epoch) = events.paste(true, true) else {
            panic!()
        };
        let mut app = app();
        app.document.select_range(7, 15);
        for composing in [true, false] {
            events.context_changed(composing);
            app.receive_browser_paste(
                Ok("stale queued text".into()),
                events.accepts_delivery(epoch),
                &schedule::RecordingExit::new(),
            );
            assert_eq!(app.document.buffer().text(), "before selected after");
            assert_eq!(app.document.buffer().selection_range(), Some((7, 15)));
        }
        let Paste::Read(epoch) = events.paste(true, true) else {
            panic!()
        };
        app.receive_browser_paste(
            Ok("新".into()),
            events.accepts_delivery(epoch),
            &schedule::RecordingExit::new(),
        );
        assert_eq!(app.document.buffer().text(), "before 新 after");
    }
}
