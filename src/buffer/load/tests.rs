use super::*;
use crate::fs::{FileSystem, InMemoryFs};
use std::sync::Arc;

#[test]
fn failed_decode_has_no_path_bound_fallback_and_missing_is_explicitly_new() {
    let path = Path::new("/probe/existing.md");
    let mem = InMemoryFs::new();
    crate::fs::with_fs(Arc::new(mem.clone()), || {
        for bytes in [&[0xe9, 0xe9][..], &[0xe2, 0x82][..], b"text\0binary"] {
            mem.write(path, bytes).unwrap();
            assert!(Buffer::load_file(path).is_err());
            assert!(Buffer::open_file(path).is_err());
            let mut scratch = Buffer::from_file(path);
            assert_eq!(scratch.path(), None);
            scratch.set_text("edit");
            assert!(scratch.save().is_err());
            assert_eq!(mem.read(path).unwrap(), bytes);
        }
        let missing = Path::new("/probe/new.md");
        assert!(Buffer::load_file(missing).is_err());
        let (mut new, seen) = Buffer::open_file(missing).unwrap();
        assert_eq!(seen, crate::external::Seen::Absent);
        assert_eq!(new.path(), Some(missing));
        new.set_text("new text");
        new.save().unwrap();
        assert_eq!(mem.read(missing).unwrap(), b"new text");
    });
}

#[test]
fn successful_load_observes_exact_original_bytes_before_eol_normalization() {
    let path = Path::new("/probe/mixed.md");
    let bytes = "é\r\nmixed\nlast\r\n".as_bytes();
    let mem = InMemoryFs::new();
    mem.write(path, bytes).unwrap();
    crate::fs::with_fs(Arc::new(mem), || {
        let (buffer, seen) = Buffer::load_file(path).unwrap();
        assert_eq!(buffer.text(), "é\nmixed\nlast\n");
        assert_eq!(
            crate::external::look(path, &seen).0,
            crate::external::Change::Unchanged
        );
        let expected = crate::external::digest(bytes);
        assert!(matches!(seen, crate::external::Seen::Present {
            digest: Some(value), ..
        } if value == expected));
    });
}
