/// Replace one changed band in a cache that retains exactly one value per
/// logical line. `None` leaves the cache untouched so the caller can reseed
/// it from the current document.
pub(super) fn splice_retained_line_band<T>(
    retained: &mut Vec<T>,
    new_len: usize,
    change: Option<(usize, usize, usize)>,
    replacement: impl FnOnce(&[T], std::ops::Range<usize>) -> Vec<T>,
) -> Option<u64> {
    let (prefix, old_end, new_end) = change?;
    if prefix > old_end || prefix > new_end || new_end > new_len {
        return None;
    }
    let suffix_len = new_len.checked_sub(new_end)?;
    let old_len = old_end.checked_add(suffix_len)?;
    if retained.len() != old_len {
        return None;
    }

    let current_band = prefix..new_end;
    let next = replacement(&retained[prefix..old_end], current_band.clone());
    assert_eq!(
        next.len(),
        current_band.len(),
        "a retained line splice must produce one value per current line"
    );
    let refreshed = next.len() as u64;
    retained.splice(prefix..old_end, next);
    Some(refreshed)
}
