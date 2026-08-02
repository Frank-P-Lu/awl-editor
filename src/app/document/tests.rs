use super::*;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::app) struct TestExtraProjection(BufferExtra);

impl DocumentSession {
    pub(in crate::app) fn contains_background(&self, key: &crate::buffers::BufferKey) -> bool {
        self.registry.contains(key)
    }

    pub(in crate::app) fn replace_buffer(&mut self, buffer: Buffer) {
        self.active.buffer = buffer;
        self.active.extra = BufferExtra::default();
        self.active.extra.caret_synced_version = self.active.buffer.version();
    }

    pub(in crate::app) fn undo(&mut self) {
        self.active.buffer.undo();
    }

    pub(in crate::app) fn set_mark(&mut self) {
        self.active.buffer.set_mark();
    }

    pub(in crate::app) fn toggle_fold_at_cursor(&mut self) {
        self.active.buffer.toggle_fold_at_cursor();
    }

    pub(in crate::app) fn mark_list_continuation_generated(&mut self) {
        self.active.buffer.mark_list_continuation_generated();
    }

    pub(in crate::app) fn take_list_continuation_generated(&mut self) -> bool {
        self.active.buffer.take_list_continuation_generated()
    }

    pub(in crate::app) fn start_fresh_for_test(&mut self, root: PathBuf) {
        self.active.buffer.start_fresh_doc(root);
    }

    fn extra(&self) -> &BufferExtra {
        &self.active.extra
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
        self.active.extra.shift_selecting = true;
        self.active.extra.scroll = crate::render::ScrollPos { row: 11, px_q: 29 };
        self.recompute_spell_cache();
        self.active.extra.sync_text_cache =
            Some((self.active.buffer.version(), self.active.buffer.text()));
        self.active.extra.caret_synced_version = 999;
        self.active.extra.doc_saved_version = Some(777);
        self.active.extra.scratch_saved_version = Some(888);
        self.active.extra.disk_baseline = crate::external::Seen::Present {
            stat: crate::fs::Metadata {
                modified: None,
                len: Some(101),
            },
            digest: Some(101),
        };
        self.active.extra.scratch_baseline = crate::external::Seen::Present {
            stat: crate::fs::Metadata {
                modified: None,
                len: Some(202),
            },
            digest: Some(202),
        };
        self.active.extra.doc_autosave_at = None;
        self.active.extra.history_preview = Some(("42".to_string(), "old text".to_string()));
        self.active.extra.history_scroll_before = Some(crate::render::ScrollPos::at_row(55));
    }

    pub(in crate::app) fn round_trip_extra_signature(&self) -> TestExtraProjection {
        TestExtraProjection(self.active.extra.clone())
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
    session.active.extra.doc_autosave_at = Some(Instant::now());
    let expected = session.active.extra.clone();
    assert!(
        !expected.spell_cache.is_empty(),
        "fixture must exercise spell cache"
    );

    assert_eq!(
        session.open_path(&b, crate::external::Seen::Absent),
        OpenPath::Fresh
    );
    assert_eq!(
        session.open_path(&a, crate::external::Seen::Absent),
        OpenPath::Reactivated
    );
    assert_eq!(session.active.extra, expected, "A -> B -> A");
    assert_eq!(
        session.open_path(&b, crate::external::Seen::Absent),
        OpenPath::Reactivated
    );
    assert_eq!(
        session.open_path(&c, crate::external::Seen::Absent),
        OpenPath::Fresh
    );
    assert_eq!(
        session.open_path(&a, crate::external::Seen::Absent),
        OpenPath::Reactivated
    );
    assert_eq!(session.active.extra, expected, "A -> B -> C -> A");
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
