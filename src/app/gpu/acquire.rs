//! Release an acquired swapchain image before replacing its surface configuration.

/// `Suboptimal` still owns a surface texture. Its destructor must run while the
/// old swapchain is configured; reconfiguring first invalidates its discard path.
pub(super) fn reconfigure_after_discard<T, R>(acquired: T, configure: impl FnOnce() -> R) -> R {
    drop(acquired);
    configure()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    struct AcquiredTexture<'a> {
        outstanding: &'a Cell<bool>,
        discards: &'a Cell<usize>,
    }

    impl Drop for AcquiredTexture<'_> {
        fn drop(&mut self) {
            assert!(self.outstanding.replace(false), "texture discarded twice");
            self.discards.set(self.discards.get() + 1);
        }
    }

    #[test]
    fn suboptimal_texture_is_discarded_before_each_surface_reconfiguration() {
        let _guard = crate::testlock::serial();
        let outstanding = Cell::new(false);
        let discards = Cell::new(0);
        let configurations = Cell::new(0);
        // Outdated has no image; Suboptimal owns one. Interleave both outcomes
        // as a live resize can, rather than only testing a single recovery.
        for has_texture in [true, true, false, true, false, false, true] {
            outstanding.set(has_texture);
            let acquired = has_texture.then(|| AcquiredTexture {
                outstanding: &outstanding,
                discards: &discards,
            });
            reconfigure_after_discard(acquired, || {
                assert!(
                    !outstanding.get(),
                    "reconfiguration must not invalidate an acquired texture"
                );
                configurations.set(configurations.get() + 1);
            });
        }
        assert_eq!(discards.get(), 4);
        assert_eq!(configurations.get(), 7);
    }
}
