//! Browser event ordering, independent of winit's unavailable wasm IME preedit.

#[derive(Default)]
pub(super) struct PasteEvents {
    composing: bool,
    epoch: u64,
    serial: u64,
    pending: Option<u64>,
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Paste {
    Unowned,
    Blocked,
    Read(u64),
}

impl PasteEvents {
    /// Composition or blur invalidates already queued deliveries and notices.
    pub(super) fn context_changed(&mut self, composing: bool) {
        self.composing = composing;
        self.epoch = self.epoch.wrapping_add(1);
        self.pending = None;
    }

    pub(super) fn gesture(&mut self, allowed: bool, composing_key: bool) -> Option<u64> {
        if !allowed || composing_key || self.composing {
            return None;
        }
        self.serial = self.serial.wrapping_add(1);
        self.pending = Some(self.serial);
        self.pending
    }

    pub(super) fn paste(&mut self, trusted: bool, focused: bool) -> Paste {
        if !trusted || !focused {
            return Paste::Unowned;
        }
        self.pending = None;
        if self.composing {
            Paste::Blocked
        } else {
            Paste::Read(self.epoch)
        }
    }

    pub(super) fn accepts_delivery(&self, epoch: u64) -> bool {
        !self.composing && self.epoch == epoch
    }

    /// Tested at dequeue time: a timer that already fired cannot leave a stale
    /// notice after a paste, newer gesture, composition, or focus change.
    pub(super) fn claim_timeout(&mut self, token: u64) -> bool {
        if self.pending != Some(token) || self.composing {
            return false;
        }
        self.pending = None;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn browser_paste_composition_blocks_shortcut_and_menu_without_stale_delivery() {
        let _guard = crate::testlock::serial();
        let mut events = PasteEvents::default();
        let Paste::Read(before) = events.paste(true, true) else {
            panic!()
        };
        let timer = events.gesture(true, false).unwrap();
        events.context_changed(true);
        assert_eq!(events.gesture(true, false), None);
        assert_eq!(events.paste(true, true), Paste::Blocked);
        assert!(!events.accepts_delivery(before));
        assert!(!events.claim_timeout(timer));
        events.context_changed(false);
        assert!(!events.accepts_delivery(before));
        let Paste::Read(after) = events.paste(true, true) else {
            panic!()
        };
        assert!(events.accepts_delivery(after));
        assert_eq!(events.gesture(true, true), None);
        events.context_changed(false); // blur
        assert!(!events.accepts_delivery(after));
    }

    #[test]
    fn browser_paste_menu_repetition_and_timeout_races_share_one_owner() {
        let _guard = crate::testlock::serial();
        let mut events = PasteEvents::default();
        assert_eq!(events.gesture(false, false), None); // custom binding/prefix
        assert_eq!(events.paste(false, true), Paste::Unowned);
        assert_eq!(events.paste(true, false), Paste::Unowned);
        let first = events.gesture(true, false).unwrap();
        let second = events.gesture(true, false).unwrap();
        assert!(!events.claim_timeout(first));
        let Paste::Read(epoch) = events.paste(true, true) else {
            panic!()
        };
        assert!(!events.claim_timeout(second));
        assert_eq!(events.paste(true, true), Paste::Read(epoch));
        assert!(events.accepts_delivery(epoch)); // both queued pastes survive
        let missing = events.gesture(true, false).unwrap();
        assert!(events.claim_timeout(missing));
        assert!(!events.claim_timeout(missing));
        assert_eq!(events.paste(true, true), Paste::Read(epoch)); // late real paste
    }
}
