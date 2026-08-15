//! Replay trace and truthfulness bookkeeping.
//!
//! One chord owns one trace row. Search and prefix outcomes write that row
//! directly; an action seeds it from the transition's primary effect, then a
//! later non-applied effect in the same depth-first stream replaces it. This
//! preserves the existing trace contract while keeping classification,
//! warnings, intercepted handoffs, and permissive skips in one owner.

use super::*;

fn chord_trace(
    chord: &str,
    action: &Action,
    classified: &crate::replay::Classified,
) -> crate::storyboard::ChordTrace {
    crate::storyboard::ChordTrace {
        chord: chord.to_string(),
        action: Some(format!("{action:?}")),
        effect: classified.name.to_string(),
        class: match &classified.class {
            crate::replay::EffectClass::Applied => "applied",
            crate::replay::EffectClass::Intercepted { .. } => "intercepted",
            crate::replay::EffectClass::Unsupported { .. } => "unsupported",
        },
        detail: match &classified.class {
            crate::replay::EffectClass::Intercepted { detail } => detail.clone(),
            crate::replay::EffectClass::Applied
            | crate::replay::EffectClass::Unsupported { .. } => String::new(),
        },
    }
}

impl ReplaySession<'_> {
    pub(super) fn record_search_trace(&mut self, chord: &crate::keyspec::Chord) {
        self.records.push(crate::storyboard::ChordTrace {
            chord: chord.spec.clone(),
            action: None,
            effect: "search_input".to_string(),
            class: "applied",
            detail: String::new(),
        });
    }

    pub(super) fn record_prefix_trace(&mut self, chord: &crate::keyspec::Chord) {
        self.records.push(crate::storyboard::ChordTrace {
            chord: chord.spec.clone(),
            action: None,
            effect: "prefix".to_string(),
            class: "applied",
            detail: String::new(),
        });
    }

    pub(super) fn record_action_trace(
        &mut self,
        chord: &crate::keyspec::Chord,
        action: &Action,
        primary: &actions::Effect,
    ) {
        let classified = crate::replay::classify_for(primary, self.filesystem);
        self.records
            .push(chord_trace(&chord.spec, action, &classified));
    }

    pub(super) fn classify_effect(
        &mut self,
        action: &Action,
        chord: &crate::keyspec::Chord,
        effect: &actions::Effect,
    ) -> Result<()> {
        let classified = crate::replay::classify_for(effect, self.filesystem);
        if !matches!(classified.class, crate::replay::EffectClass::Applied) {
            *self.records.last_mut().expect("this chord has a trace") =
                chord_trace(&chord.spec, action, &classified);
        }
        if let crate::replay::EffectClass::Intercepted { detail } = &classified.class {
            self.intercepts.push(crate::replay::Intercept {
                effect: classified.name,
                detail: detail.clone(),
            });
        }
        if self.mode == crate::replay::Mode::Strict
            && let crate::replay::EffectClass::Unsupported { .. } = classified.class
        {
            return Err(crate::replay::strict_error(action, &classified));
        }
        if self.mode == crate::replay::Mode::Permissive
            && let Some(skip) = crate::replay::permissive_skip(action, &classified)
        {
            self.replay_skips.push(skip);
        }
        if self.mode == crate::replay::Mode::Permissive
            && let Some(warning) = crate::replay::warn_line(action, &classified)
        {
            eprintln!("{warning}");
            self.warnings.push(warning);
        }
        Ok(())
    }
}
