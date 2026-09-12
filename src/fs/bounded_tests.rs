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
fn web_search_cap_fits_the_browser_bounded_read_abi() {
    assert!(
        crate::search_folder::SearchBudget::default().max_file_bytes <= u32::MAX as usize,
        "the browser's cap-sized Uint8Array ABI must represent search's per-file cap"
    );
}

#[test]
fn web_bounded_read_rejects_before_retrieving_the_canonical_file_value() {
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
    assert!(bounded.contains("awl_bounded_local_storage_utf8"));
    assert!(bounded.contains("Uint8Array::new"));
    assert!(bounded.contains("LENGTH_PREFIX"));
    assert!(bounded.contains("known_len > max_bytes as usize"));
    assert!(bounded.contains("BoundedRead::LimitReached(Vec::new())"));
    assert!(!bounded.contains("read_to_string"));
    assert!(
        bounded.find("LENGTH_PREFIX").unwrap()
            < bounded.find("awl_bounded_local_storage_utf8").unwrap(),
        "the tiny length check must precede canonical-value retrieval"
    );

    let javascript = source
        .split_once("export function awl_bounded_local_storage_utf8")
        .expect("raw bounded JS bridge is enrolled")
        .1
        .split_once("\"#)]")
        .expect("inline JS bridge has a closing attribute")
        .0;
    assert!(javascript.contains("new Uint8Array(maxBytes)"));
    assert!(javascript.contains("encodeInto(value, output)"));
    assert!(javascript.contains("result.read === value.length"));
    assert!(
        !javascript.contains("encode(value)"),
        "the browser bridge must not allocate a whole-file encoded array"
    );

    let metadata = backend
        .split_once("fn metadata")
        .expect("metadata reader is enrolled")
        .1
        .split_once("fn remove_file")
        .expect("metadata has a following method")
        .0;
    assert!(metadata.contains("LENGTH_PREFIX"));
    assert!(!metadata.contains("read_to_string"));
    assert!(
        !metadata.contains("get_item(&Self::key(FILE_PREFIX"),
        "metadata may test canonical-key presence but must not retrieve its value"
    );
}

#[test]
fn web_bounded_read_rejects_missing_or_stale_length_truth() {
    use web::WebBoundedState::{Complete, LimitReached, StaleLength};

    assert_eq!(web::classify_web_bounded_read(2, 2, true), Complete);
    assert_eq!(
        web::classify_web_bounded_read(2, 0, false),
        LimitReached,
        "TextEncoder refuses to split a two-byte scalar into one byte"
    );
    assert_eq!(web::classify_web_bounded_read(2, 3, true), StaleLength);
    assert_eq!(web::classify_web_bounded_read(3, 2, true), StaleLength);

    let source = include_str!("web.rs");
    let bounded = source
        .split_once("fn read_bounded")
        .expect("bounded reader is enrolled")
        .1
        .split_once("fn write")
        .expect("bounded reader has a following method")
        .0;
    assert!(bounded.contains("bounded-read length sidecar missing"));
    assert!(bounded.contains("return failed"));
}
