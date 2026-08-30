use super::*;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::app) struct TestExtraProjection(BufferExtra);

impl DocumentSession {
    fn test_active(&self) -> &crate::buffers::Entry<BufferExtra> {
        self.active
            .as_ref()
            .expect("test fixture has an active document")
    }

    fn test_active_mut(&mut self) -> &mut crate::buffers::Entry<BufferExtra> {
        self.active
            .as_mut()
            .expect("test fixture has an active document")
    }

    pub(in crate::app) fn contains_background(&self, key: &crate::buffers::BufferKey) -> bool {
        self.registry.contains(key)
    }

    pub(in crate::app) fn replace_buffer(&mut self, buffer: Buffer) {
        let active = self.test_active_mut();
        active.buffer = buffer;
        active.extra = BufferExtra::default();
        active.extra.caret_synced_version = active.buffer.version();
    }

    pub(in crate::app) fn undo(&mut self) {
        self.test_active_mut().buffer.undo();
    }

    pub(in crate::app) fn set_mark(&mut self) {
        self.test_active_mut().buffer.set_mark();
    }

    pub(in crate::app) fn toggle_fold_at_cursor(&mut self) {
        self.test_active_mut().buffer.toggle_fold_at_cursor();
    }

    pub(in crate::app) fn mark_list_continuation_generated(&mut self) {
        self.test_active_mut()
            .buffer
            .mark_list_continuation_generated();
    }

    pub(in crate::app) fn take_list_continuation_generated(&mut self) -> bool {
        self.test_active_mut()
            .buffer
            .take_list_continuation_generated()
    }

    pub(in crate::app) fn start_fresh_for_test(&mut self, root: PathBuf) {
        self.test_active_mut().buffer.start_fresh_doc(root);
    }

    fn extra(&self) -> &BufferExtra {
        &self.test_active().extra
    }

    pub(in crate::app) fn sync_text_cached(&self) -> bool {
        self.extra().sync_text_cache.is_some()
    }

    pub(in crate::app) fn caret_synced_version(&self) -> u64 {
        self.extra().caret_synced_version
    }

    pub(in crate::app) fn history_preview_value(&self) -> Option<(String, String)> {
        self.extra().history_preview.clone()
    }

    pub(in crate::app) fn history_scroll_before(&self) -> Option<crate::render::ScrollPos> {
        self.extra().history_scroll_before
    }

    pub(in crate::app) fn seed_round_trip_extra(&mut self) {
        self.test_active_mut().extra.shift_selecting = true;
        self.test_active_mut().extra.scroll = crate::render::ScrollPos { row: 11, px_q: 29 };
        self.recompute_spell_cache();
        let version = self.test_active().buffer.version();
        let text = self.test_active().buffer.text();
        let active = self.test_active_mut();
        active.extra.sync_text_cache = Some((version, text));
        active.extra.caret_synced_version = 999;
        active.extra.doc_saved_version = Some(777);
        active.extra.scratch_saved_version = Some(888);
        active.extra.disk_baseline = crate::external::Seen::Present {
            stat: crate::fs::Metadata {
                modified: None,
                len: Some(101),
            },
            digest: Some(101),
        };
        active.extra.scratch_baseline = crate::external::Seen::Present {
            stat: crate::fs::Metadata {
                modified: None,
                len: Some(202),
            },
            digest: Some(202),
        };
        active.extra.doc_autosave_at = None;
        active.extra.history_preview = Some(("42".to_string(), "old text".to_string()));
        active.extra.history_scroll_before = Some(crate::render::ScrollPos::at_row(55));
    }

    pub(in crate::app) fn round_trip_extra_signature(&self) -> TestExtraProjection {
        TestExtraProjection(self.test_active().extra.clone())
    }
}

#[test]
fn every_buffer_extra_field_round_trips_a_b_a_b_c_a() {
    let _guard = crate::testlock::serial();
    let a = PathBuf::from("/session/a.md");
    let b = PathBuf::from("/session/b.md");
    let c = PathBuf::from("/session/c.md");
    let fs = crate::fs::InMemoryFs::new()
        .with_file(&a, "helo alpha\n")
        .with_file(&b, "bravo\n")
        .with_file(&c, "charlie\n");
    let _fs = crate::fs::FsGuard::install(Arc::new(fs));
    let mut session = DocumentSession::new(
        Buffer::from_file(&a),
        crate::external::Seen::Absent,
        crate::external::Seen::Absent,
    );

    session.seed_round_trip_extra();
    session.test_active_mut().extra.doc_autosave_at = Some(Instant::now());
    let expected = session.test_active().extra.clone();
    assert!(
        !expected.spell_cache.is_empty(),
        "fixture must exercise spell cache"
    );

    assert_eq!(
        session.open_path(&b, crate::external::Seen::Absent, Path::new("/")),
        OpenPath::Fresh
    );
    assert_eq!(
        session.open_path(&a, crate::external::Seen::Absent, Path::new("/")),
        OpenPath::Reactivated
    );
    assert_eq!(session.test_active().extra, expected, "A -> B -> A");
    assert_eq!(
        session.open_path(&b, crate::external::Seen::Absent, Path::new("/")),
        OpenPath::Reactivated
    );
    assert_eq!(
        session.open_path(&c, crate::external::Seen::Absent, Path::new("/")),
        OpenPath::Fresh
    );
    assert_eq!(
        session.open_path(&a, crate::external::Seen::Absent, Path::new("/")),
        OpenPath::Reactivated
    );
    assert_eq!(session.test_active().extra, expected, "A -> B -> C -> A");
}

#[test]
fn simultaneous_fresh_buffers_park_and_reactivate_by_distinct_identity() {
    let _guard = crate::testlock::serial();
    let root = PathBuf::from("/notes");
    let mut session = DocumentSession::new(
        Buffer::scratch(),
        crate::external::Seen::Absent,
        crate::external::Seen::Absent,
    );
    session.enrol_active(&root);

    session.start_fresh_document(root.clone());
    let first = session.active_key().expect("first fresh identity");
    session.set_text("first manuscript");
    session.start_fresh_document(root.clone());
    let second = session.active_key().expect("second fresh identity");
    session.set_text("second manuscript");

    assert!(matches!(first, crate::buffers::BufferKey::Fresh(_)));
    assert!(matches!(second, crate::buffers::BufferKey::Fresh(_)));
    assert_ne!(first, second, "each Cmd-N buffer has its own key");
    assert!(session.contains_background(&first));
    assert!(session.activate_key(&first));
    assert_eq!(session.buffer().text(), "first manuscript");
    assert!(session.activate_key(&second));
    assert_eq!(session.buffer().text(), "second manuscript");
    assert_eq!(
        session.working_set().len(),
        3,
        "scratch plus two fresh rows"
    );
}

#[test]
fn successful_fresh_rekey_leaves_no_behavioral_owner_on_the_old_key() {
    let _guard = crate::testlock::serial();
    let root = PathBuf::from("/notes");
    let memory = Arc::new(crate::fs::InMemoryFs::new().with_dir(&root));
    crate::fs::with_fs(memory, || {
        let mut session = DocumentSession::new(
            Buffer::scratch(),
            crate::external::Seen::Absent,
            crate::external::Seen::Absent,
        );
        session.enrol_active(&root);
        session.start_fresh_document(root.clone());
        session.set_text("Named manuscript");
        let old = session.active_key().expect("fresh key");
        assert!(matches!(old, crate::buffers::BufferKey::Fresh(_)));
        #[cfg(not(target_arch = "wasm32"))]
        assert!(
            session.session_buffers().is_empty(),
            "fresh is not restorable from disk"
        );

        session.save().unwrap();
        session.rekey_active_after_naming();

        let path = root.join("named-manuscript.md");
        let new = crate::buffers::BufferKey::path(&path);
        assert_eq!(session.active_key(), Some(new.clone()));
        assert_eq!(session.working_set().index_of(&old), None);
        assert!(session.working_set().index_of(&new).is_some());
        assert!(session.close_facts(&old).is_none());
        assert!(session.close_facts(&new).is_some());
        assert!(!session.contains_background(&old));
        #[cfg(not(target_arch = "wasm32"))]
        {
            let restored = session.session_buffers();
            assert_eq!(restored.len(), 1);
            assert_eq!(
                restored[0].0, path,
                "session records only the committed path"
            );
            assert_eq!(restored[0].1.col, "Named manuscript".chars().count());
        }
    });
}

/// The roster of live transitions that replace the active slot. Each must
/// reversibly park the outgoing manuscript; close has its separate save/refusal
/// laws. The match is exhaustive so a new route added here cannot inherit a
/// hand-waved expectation.
#[test]
fn every_active_replacement_route_parks_the_outgoing_text_byte_for_byte() {
    #[derive(Clone, Copy, Debug)]
    enum ReplacementRoute {
        OpenPath,
        NewDocument,
        ScratchRestore,
        ActivateExisting,
    }

    let _guard = crate::testlock::serial();
    let root = PathBuf::from("/notes");
    let target = root.join("target.md");
    let memory = Arc::new(
        crate::fs::InMemoryFs::new()
            .with_dir(&root)
            .with_file(&target, "target\n"),
    );
    crate::fs::with_fs(memory, || {
        for route in [
            ReplacementRoute::OpenPath,
            ReplacementRoute::NewDocument,
            ReplacementRoute::ScratchRestore,
            ReplacementRoute::ActivateExisting,
        ] {
            let mut session = DocumentSession::new(
                Buffer::scratch(),
                crate::external::Seen::Absent,
                crate::external::Seen::Absent,
            );
            session.enrol_active(&root);
            session.start_fresh_document(root.clone());
            session.set_text("outgoing bytes — 日本語");
            let outgoing = session.active_key().expect("outgoing key");

            match route {
                ReplacementRoute::OpenPath => {
                    session.open_path(&target, crate::external::Seen::Absent, &root);
                }
                ReplacementRoute::NewDocument => session.start_fresh_document(root.clone()),
                ReplacementRoute::ScratchRestore => session.open_scratch(
                    Buffer::scratch(),
                    crate::external::Seen::Absent,
                    root.clone(),
                ),
                ReplacementRoute::ActivateExisting => {
                    session.open_path(&target, crate::external::Seen::Absent, &root);
                    assert!(session.activate_key(&outgoing));
                    session.set_text("arriving edit");
                    assert!(session.activate_key(&crate::buffers::BufferKey::path(&target)));
                }
            }

            assert_eq!(
                session.parked_text(&outgoing).as_deref(),
                Some(if matches!(route, ReplacementRoute::ActivateExisting) {
                    "arriving edit"
                } else {
                    "outgoing bytes — 日本語"
                }),
                "{route:?} must park the only copy before replacement"
            );
        }
    });
}

#[test]
fn autosave_poll_waits_then_consumes_the_due_arm_once() {
    let _guard = crate::testlock::serial();
    let armed = Instant::now();
    let idle = std::time::Duration::from_secs(1);
    let mut session = DocumentSession::new(
        Buffer::scratch(),
        crate::external::Seen::Absent,
        crate::external::Seen::Absent,
    );
    session.arm_doc_autosave(armed);

    assert_eq!(
        session.poll_autosave(armed, idle),
        AutosavePoll::WaitingUntil(armed + idle)
    );
    assert_eq!(session.poll_autosave(armed + idle, idle), AutosavePoll::Due);
    assert_eq!(
        session.poll_autosave(armed + idle, idle),
        AutosavePoll::Idle
    );
}
