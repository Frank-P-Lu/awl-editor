pub(super) fn keymap(
    keys: &[(String, Vec<String>)],
    keep: &[String],
    linux_emacs_meta: bool,
) -> crate::keymap::KeymapState {
    crate::keymap::KeymapState::with_overrides_and_keep(keys, keep, linux_emacs_meta)
}
