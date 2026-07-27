use std::sync::atomic::{AtomicU32, Ordering};

static BITS: AtomicU32 = AtomicU32::new(1.0f32.to_bits());

pub(super) fn get() -> f32 {
    f32::from_bits(BITS.load(Ordering::Relaxed))
}

pub(super) fn set(value: f32) {
    let value = crate::range::SCROLL_SENSITIVITY.quantize(value);
    BITS.store(value.to_bits(), Ordering::Relaxed);
}
