use super::*;

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
