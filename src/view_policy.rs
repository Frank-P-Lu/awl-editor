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

    fn capture_production_sources() -> BTreeMap<String, String> {
        fn visit(root: &Path, dir: &Path, sources: &mut BTreeMap<String, String>) {
            for entry in std::fs::read_dir(dir).expect("read capture source directory") {
                let entry = entry.expect("read capture source entry");
                let path = entry.path();
                if path.is_dir() {
                    if path.file_name().and_then(|name| name.to_str()) != Some("tests") {
                        visit(root, &path, sources);
                    }
                } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                    let source = std::fs::read_to_string(&path).expect("read capture Rust source");
                    let production = source.split("#[cfg(test)]").next().unwrap_or(&source);
                    let relative = path
                        .strip_prefix(root)
                        .expect("capture source lives under src")
                        .to_string_lossy()
                        .replace('\\', "/");
                    sources.insert(relative, production.to_string());
                }
            }
        }

        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut sources = BTreeMap::new();
        let root_module = root.join("capture.rs");
        sources.insert(
            "capture.rs".to_string(),
            std::fs::read_to_string(root_module).expect("read capture root module"),
        );
        visit(&root, &root.join("capture"), &mut sources);
        sources
    }

    fn identifier_count(source: &str, identifier: &str) -> usize {
        source
            .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
            .filter(|token| *token == identifier)
            .count()
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
                ("capture/policy.rs", 1),
            ],
        );
        assert_roster(
            ["policy::follow_", "scroll("].concat(),
            &[("capture/animated.rs", 2), ("capture/modes.rs", 1)],
        );
        assert_roster(
            ["spell_recompute_", "needed("].concat(),
            &[
                ("view_policy.rs", 1),
                ("app/viewstate.rs", 1),
                ("capture/policy.rs", 1),
            ],
        );
        assert_roster(
            "policy::misspellings(".to_string(),
            &[("capture/animated.rs", 2), ("capture/modes.rs", 1)],
        );
    }

    /// The COMPLETE production capture tree is enumerated first, so adding a new
    /// capture module cannot silently escape this audit. Raw identifier tokens are
    /// then rostered without depending on call punctuation, qualification, aliases,
    /// or the spelling of a surrounding `if`/`match`/`matches!`.
    #[test]
    fn every_capture_source_has_an_identifier_level_policy_owner() {
        let sources = capture_production_sources();
        let expected_files = [
            "capture.rs",
            "capture/animated.rs",
            "capture/background_sidecar.rs",
            "capture/film.rs",
            "capture/frames.rs",
            "capture/gpu.rs",
            // Serializes the renderer's own `LayoutReport` and decides nothing:
            // it holds none of the rostered policy identifiers, so it contributes
            // zero to every count below rather than needing an owner of its own.
            "capture/layout_sidecar.rs",
            "capture/modes.rs",
            "capture/opts.rs",
            "capture/oracle.rs",
            "capture/policy.rs",
            // Serializes replay-skip records and owns no view policy.
            "capture/replay_sidecar.rs",
            "capture/scroll_sidecar.rs",
            "capture/sidecar.rs",
        ];
        assert_eq!(
            sources.keys().map(String::as_str).collect::<Vec<_>>(),
            expected_files,
            "the production capture source roster changed; enroll every new .rs file \
             in the policy ownership audit before it can ship"
        );

        for identifier in [
            "typewriter_on",
            "scroll_to_show_row_pos",
            "scroll_to_center_row_pos",
            "SpellChecker",
            "spell_checked_version",
            "checked_version",
        ] {
            let actual: BTreeMap<&str, usize> = sources
                .iter()
                .filter_map(|(path, source)| {
                    let count = identifier_count(source, identifier);
                    (count != 0).then_some((path.as_str(), count))
                })
                .collect();
            let expected = match identifier {
                "spell_checked_version" => BTreeMap::new(),
                _ => BTreeMap::from([(
                    "capture/policy.rs",
                    if identifier == "checked_version" {
                        2
                    } else {
                        1
                    },
                )]),
            };
            assert_eq!(
                actual, expected,
                "raw capture policy identifier `{identifier}` escaped its sole declared \
                 owner; animated/timeline/held/frames/film may call only shared helpers"
            );
        }

        for (helper, expected) in [
            (
                "follow_scroll",
                BTreeMap::from([
                    ("capture/animated.rs", 2),
                    ("capture/modes.rs", 1),
                    ("capture/policy.rs", 1),
                ]),
            ),
            (
                "misspellings",
                BTreeMap::from([
                    ("capture/animated.rs", 2),
                    ("capture/modes.rs", 1),
                    ("capture/policy.rs", 1),
                ]),
            ),
        ] {
            let actual: BTreeMap<&str, usize> = sources
                .iter()
                .filter_map(|(path, source)| {
                    let count = identifier_count(source, helper);
                    (count != 0).then_some((path.as_str(), count))
                })
                .collect();
            assert_eq!(
                actual, expected,
                "shared helper roster changed for `{helper}`"
            );
        }
    }
}
