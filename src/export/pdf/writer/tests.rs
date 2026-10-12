use super::finish;

/// `finish` is only reachable from within this module (`fn finish` is private,
/// not even `pub(super)`), so this is the one seam that can drive its own
/// defense-in-depth directly: a hand-built object plan with a hole at id 2
/// (`emit` never produces one) must return `Err`, not unwind.
#[test]
fn a_reserved_object_never_written_is_an_error_not_a_panic() {
    let objects: Vec<Option<Vec<u8>>> = vec![None, Some(b"<< /Type /Catalog >>".to_vec()), None];
    let err = finish(objects).expect_err("a hole in the object plan must not panic");
    assert_eq!(err, "PDF object 2 reserved but never written");
}

/// The companion presence check proves the error above comes from the hole.
#[test]
fn a_fully_written_object_plan_still_succeeds() {
    let objects: Vec<Option<Vec<u8>>> = vec![None, Some(b"<< /Type /Catalog >>".to_vec())];
    finish(objects).expect("every slot is written, so this must succeed");
}
