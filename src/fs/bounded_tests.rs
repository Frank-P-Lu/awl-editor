use super::*;

struct ObservedNativeInput {
    bytes: Vec<u8>,
    cursor: usize,
    initial_len: u64,
    final_len: u64,
    metadata_calls: usize,
    fail_after: Option<usize>,
    bytes_served: usize,
}

impl ObservedNativeInput {
    fn stable(bytes: &[u8]) -> Self {
        Self {
            bytes: bytes.to_vec(),
            cursor: 0,
            initial_len: bytes.len() as u64,
            final_len: bytes.len() as u64,
            metadata_calls: 0,
            fail_after: None,
            bytes_served: 0,
        }
    }

    fn growing(bytes: &[u8], initial_len: u64) -> Self {
        Self {
            initial_len,
            ..Self::stable(bytes)
        }
    }

    fn failing(bytes: &[u8], fail_after: usize) -> Self {
        Self {
            fail_after: Some(fail_after),
            ..Self::stable(bytes)
        }
    }
}

impl std::io::Read for ObservedNativeInput {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        if self.fail_after.is_some_and(|limit| self.cursor >= limit) {
            return Err(std::io::Error::other("scripted partial read failure"));
        }
        if self.cursor == self.bytes.len() {
            return Ok(0);
        }
        let before_failure = self
            .fail_after
            .map_or(usize::MAX, |limit| limit.saturating_sub(self.cursor));
        let len = out
            .len()
            .min(self.bytes.len() - self.cursor)
            .min(before_failure);
        out[..len].copy_from_slice(&self.bytes[self.cursor..self.cursor + len]);
        self.cursor += len;
        self.bytes_served += len;
        Ok(len)
    }
}

impl native::NativeBoundedInput for ObservedNativeInput {
    fn current_len(&mut self) -> std::io::Result<u64> {
        let len = if self.metadata_calls == 0 {
            self.initial_len
        } else {
            self.final_len
        };
        self.metadata_calls += 1;
        Ok(len)
    }
}

#[test]
fn native_reader_caps_actual_bytes_at_exact_and_cap_plus_one() {
    let _guard = crate::testlock::serial();
    let mut exact = ObservedNativeInput::stable(b"12345");
    assert!(matches!(
        native::read_bounded_input(&mut exact, 5),
        BoundedRead::Complete(bytes) if bytes == b"12345"
    ));
    assert_eq!(exact.bytes_served, 5);

    let mut larger = ObservedNativeInput::stable(b"123456");
    assert!(matches!(
        native::read_bounded_input(&mut larger, 5),
        BoundedRead::LimitReached(bytes) if bytes == b"12345"
    ));
    assert_eq!(
        larger.bytes_served, 5,
        "the native reader must not read the cap+1 byte and truncate afterward"
    );
}

#[test]
fn native_reader_rejects_growth_without_crossing_the_byte_cap() {
    let _guard = crate::testlock::serial();
    let mut growing = ObservedNativeInput::growing(b"12345", 4);
    assert!(matches!(
        native::read_bounded_input(&mut growing, 8),
        BoundedRead::LimitReached(bytes) if bytes == b"12345"
    ));
    assert_eq!(growing.bytes_served, 5);
    assert_eq!(growing.metadata_calls, 2);
}

#[test]
fn native_reader_reports_only_bytes_served_before_a_partial_error() {
    let _guard = crate::testlock::serial();
    let mut failing = ObservedNativeInput::failing(b"12345", 2);
    assert!(matches!(
        native::read_bounded_input(&mut failing, 5),
        BoundedRead::Failed { bytes, .. } if bytes == b"12"
    ));
    assert_eq!(failing.bytes_served, 2);
}

#[test]
fn in_memory_bounded_read_distinguishes_exact_size_from_cap_plus_one() {
    let fs = InMemoryFs::new()
        .with_file("/exact.md", "12345")
        .with_file("/larger.md", "123456");
    match fs.read_bounded(Path::new("/exact.md"), 5) {
        BoundedRead::Complete(bytes) => assert_eq!(bytes, b"12345"),
        other => panic!("exact-size file must be complete: {other:?}"),
    }
    match fs.read_bounded(Path::new("/larger.md"), 5) {
        BoundedRead::LimitReached(bytes) => assert_eq!(bytes, b"12345"),
        other => panic!("cap+1 file must be limited: {other:?}"),
    }
}

#[test]
fn native_bounded_read_never_returns_more_than_the_cap() {
    let _guard = crate::testlock::serial();
    let dir = crate::testscratch::ScratchDir::new(
        std::env::temp_dir().join(format!("awl-native-bounded-read-{}", std::process::id())),
    );
    let exact = dir.join("exact.md");
    let larger = dir.join("larger.md");
    std::fs::write(&exact, b"12345").unwrap();
    std::fs::write(&larger, vec![b'x'; 1_000_000]).unwrap();
    assert!(matches!(
        NativeFs.read_bounded(&exact, 5),
        BoundedRead::Complete(bytes) if bytes.len() == 5
    ));
    assert!(matches!(
        NativeFs.read_bounded(&larger, 5),
        BoundedRead::LimitReached(bytes) if bytes.len() == 5
    ));
}

#[test]
fn native_bounded_read_only_accepts_a_stable_complete_size() {
    assert!(native::stable_complete(5, 5, 5));
    assert!(!native::stable_complete(6, 5, 5));
    assert!(!native::stable_complete(5, 6, 5));
    assert!(!native::stable_complete(4, 4, 5));
}

#[test]
fn web_bounded_read_is_inert_without_a_transactional_storage_backend() {
    let source = include_str!("web.rs");
    let backend = source
        .split_once("impl FileSystem for WebFs")
        .expect("WebFs implementation is enrolled")
        .1;
    let bounded = backend
        .split_once("fn read_bounded")
        .expect("bounded reader is enrolled")
        .1
        .split_once("fn write")
        .expect("bounded reader has a following method")
        .0;
    assert!(
        bounded.contains("bounded browser reads are unavailable"),
        "no synchronous localStorage bounded-read substitute is safe"
    );
    assert!(!bounded.contains("storage()"));
    assert!(!bounded.contains("get_item"));
    assert!(!source.contains("awl_bounded_local_storage_utf8"));
    assert!(!source.contains("LENGTH_PREFIX"));
}

#[test]
fn browser_search_refusal_precedes_the_picker_gather() {
    let source = include_str!("../app/apply.rs");
    let refusal = source
        .split_once("if matches!(action, Action::OpenSearchFolder)")
        .expect("the browser search action has a refusal")
        .1
        .split_once("// ESC COLLAPSES")
        .expect("the refusal ends before ordinary action handling")
        .0;
    assert!(refusal.contains("search in folder is unavailable in the browser"));
    assert!(
        refusal.contains("self.sync_view(false);"),
        "a direct Search in folder binding must publish its notice without a palette transition"
    );
    assert!(refusal.contains("return false"));
    assert!(
        source
            .find("if matches!(action, Action::OpenSearchFolder)")
            .unwrap()
            < source.find("self.run_action_core(&action, shift)").unwrap(),
        "the browser refusal must run before picker gathering can materialize a corpus"
    );
}
