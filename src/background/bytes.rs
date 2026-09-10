/// Marker for types safe to reinterpret as bytes.
///
/// # Safety
/// Implementors must have a stable layout with no padding and only
/// plain-old-data fields.
pub(super) unsafe trait Pod: Copy + 'static {}

pub(super) fn of<T: Pod>(value: &T) -> &[u8] {
    unsafe {
        core::slice::from_raw_parts((value as *const T).cast::<u8>(), core::mem::size_of::<T>())
    }
}
