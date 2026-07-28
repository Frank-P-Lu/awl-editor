//! Shared pure policies for constructing a rendered editor view.
//!
//! The live [`crate::app::App`] and the bare capture pipeline have different
//! imperative wiring, but they must make the same decisions about what a view
//! shows. Keep those decisions here so a headless pass cannot silently retain a
//! policy fork.

/// Which vertical-scroll strategy cursor-follow applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FollowScroll {
    ShowRow,
    CenterRow,
    /// Centering would move the view while a primary-button press is live.
    Deferred,
}

/// Decide how cursor-follow treats the current row.
pub(crate) fn follow_scroll_strategy(typewriter: bool, dragging: bool) -> FollowScroll {
    if !typewriter {
        FollowScroll::ShowRow
    } else if dragging {
        FollowScroll::Deferred
    } else {
        FollowScroll::CenterRow
    }
}

/// Whether a version-keyed spell cache must be recomputed.
///
/// A capture starts with no cache (`None`), so it takes the same first-scan arm
/// as a live buffer; its caller still owns creating its short-lived checker.
pub(crate) fn spell_recompute_needed(checked_version: Option<u64>, current_version: u64) -> bool {
    checked_version != Some(current_version)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::path::Path;

    #[test]
    fn follow_scroll_sweeps_typewriter_and_dragging() {
        assert_eq!(follow_scroll_strategy(false, false), FollowScroll::ShowRow);
        assert_eq!(follow_scroll_strategy(false, true), FollowScroll::ShowRow);
        assert_eq!(follow_scroll_strategy(true, false), FollowScroll::CenterRow);
        assert_eq!(follow_scroll_strategy(true, true), FollowScroll::Deferred);
    }

    #[test]
    fn spell_recompute_trigger_sweeps_cached_and_changed_versions() {
        assert!(spell_recompute_needed(None, 0));
        assert!(spell_recompute_needed(Some(3), 4));
        assert!(!spell_recompute_needed(Some(4), 4));
    }

    fn production_calls(dir: &Path, needle: &str, hits: &mut BTreeMap<String, usize>) {
        for entry in std::fs::read_dir(dir).expect("read source directory") {
            let entry = entry.expect("read source entry");
            let path = entry.path();
            if path.is_dir() {
                production_calls(&path, needle, hits);
            } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                let source = std::fs::read_to_string(&path).expect("read Rust source");
                let production = source.split("#[cfg(test)]").next().unwrap_or(&source);
                let count = production.matches(needle).count();
                if count != 0 {
                    let relative = path
                        .strip_prefix(Path::new(env!("CARGO_MANIFEST_DIR")).join("src"))
                        .expect("source lives under src")
                        .to_string_lossy()
                        .replace('\\', "/");
                    hits.insert(relative, count);
                }
            }
        }
    }

    fn assert_roster(needle: String, expected: &[(&str, usize)]) {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut actual = BTreeMap::new();
        production_calls(&root, &needle, &mut actual);
        let expected: BTreeMap<String, usize> = expected
            .iter()
            .map(|(path, count)| ((*path).to_string(), *count))
            .collect();
        assert_eq!(
            actual, expected,
            "production consumer roster for `{needle}` changed; route a new consumer \
             through the named owner and update this exhaustive roster consciously"
        );
    }

    /// API ownership and consumer roster. This counts named policy doors rather
    /// than one lexical spelling of their expressions: rewriting a bypass as
    /// `matches!`, a `match`, or a reordered comparison cannot satisfy the law.
    #[test]
    fn every_live_and_capture_consumer_routes_through_the_policy_apis() {
        assert_roster(
            ["follow_scroll_", "strategy("].concat(),
            &[
                ("view_policy.rs", 1),
                ("app/viewstate.rs", 1),
                ("capture/modes.rs", 1),
            ],
        );
        assert_roster(
            ["capture_follow_", "scroll("].concat(),
            &[("capture/animated.rs", 2), ("capture/modes.rs", 2)],
        );
        assert_roster(
            ["spell_recompute_", "needed("].concat(),
            &[
                ("view_policy.rs", 1),
                ("app/viewstate.rs", 1),
                ("capture/modes.rs", 1),
            ],
        );
        assert_roster(
            ["capture_misspell", "ings("].concat(),
            &[("capture/animated.rs", 2), ("capture/modes.rs", 2)],
        );
    }

    /// The imperative primitives may appear only in their declared live/capture
    /// wiring owners. An alternate hand-written policy branch necessarily adds a
    /// raw input or primitive here even if it disguises the boolean expression.
    #[test]
    fn raw_policy_inputs_and_effects_have_an_exhaustive_owner_roster() {
        let app = include_str!("app/viewstate.rs");
        let capture = include_str!("capture/modes.rs");
        let animated = include_str!("capture/animated.rs");

        assert_eq!(app.matches("typewriter_on()").count(), 1);
        assert_eq!(app.matches("spell_checked_version").count(), 1);
        assert_eq!(app.matches("scroll_to_center_row_pos(").count(), 1);
        assert_eq!(app.matches("recompute_spell_cache()").count(), 1);

        assert_eq!(capture.matches("typewriter_on()").count(), 1);
        assert_eq!(capture.matches("scroll_to_show_row_pos(").count(), 1);
        assert_eq!(capture.matches("scroll_to_center_row_pos(").count(), 1);
        assert_eq!(capture.matches("SpellChecker::new(").count(), 1);

        for bypass in [
            "typewriter_on()",
            "scroll_to_show_row_pos(",
            "scroll_to_center_row_pos(",
            "SpellChecker::new(",
            "spell_checked_version",
        ] {
            assert_eq!(
                animated.matches(bypass).count(),
                0,
                "animated capture bypasses the shared capture policy via `{bypass}`"
            );
        }
    }
}
