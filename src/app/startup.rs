pub(super) fn keymap(
    keys: &[(String, Vec<String>)],
    keep: &[String],
    linux_emacs_meta: bool,
) -> crate::keymap::KeymapState {
    crate::keymap::KeymapState::with_overrides_and_keep(keys, keep, linux_emacs_meta)
}

/// Read the persistent scratch stash into a buffer + baseline — the SCRATCH
/// RESTORE a no-argument `App::new` performs, and the live "Open scratch"
/// door's exact counterpart, so the two doors can never disagree on what
/// counts as saved. A corrupt stash is preserved to a `.corrupt-*` sibling
/// before falling back to a blank scratch, identically either way.
pub(super) fn scratch_buffer_from_stash() -> (crate::buffer::Buffer, crate::external::Seen) {
    let stash = crate::fs::scratch_stash_path();
    let buffer = match crate::fs::active().read_to_string(&stash) {
        Ok(s) if !s.is_empty() => crate::buffer::Buffer::from_str(&s),
        Ok(_) => crate::buffer::Buffer::scratch(), // present but empty: nothing to preserve
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => crate::buffer::Buffer::scratch(),
        Err(_) => {
            if let Ok(raw) = crate::fs::active().read(&stash) {
                crate::durable::preserve_corrupt(&stash, &raw);
            }
            crate::buffer::Buffer::scratch()
        }
    };
    (buffer, crate::external::Seen::at(&stash))
}
