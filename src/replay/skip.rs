use crate::keymap::Action;

use super::{Classified, EffectClass};

/// One live-only effect a permissive replay skipped, emitted in the sidecar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkippedEffect {
    pub effect: &'static str,
    pub action: String,
}

/// Converts one Unsupported classification into the sidecar's permissive record.
pub fn permissive_skip(action: &Action, c: &Classified) -> Option<SkippedEffect> {
    matches!(c.class, EffectClass::Unsupported { .. }).then(|| SkippedEffect {
        effect: c.name,
        action: format!("{action:?}"),
    })
}
