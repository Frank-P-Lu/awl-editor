//! Test-only counts at the real work owners, never elapsed-time assertions.

use std::cell::Cell;

#[derive(Clone, Copy)]
pub(crate) enum Op {
    Splice,
    Refilter,
    Recompute,
    Preview,
    Mirror,
}

thread_local! { static COUNTS: Cell<[usize; 5]> = const { Cell::new([0; 5]) }; }

pub(crate) fn note(op: Op) {
    COUNTS.with(|counts| {
        let mut value = counts.get();
        value[op as usize] += 1;
        counts.set(value);
    });
}

pub(crate) fn take() -> [usize; 5] {
    COUNTS.with(|counts| counts.replace([0; 5]))
}
