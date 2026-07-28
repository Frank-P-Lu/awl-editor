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

    fn count_needle(dir: &Path, needle: &str, hits: &mut BTreeMap<String, usize>) {
        for entry in std::fs::read_dir(dir).expect("read source directory") {
            let entry = entry.expect("read source entry");
            let path = entry.path();
            if path.is_dir() {
                count_needle(&path, needle, hits);
            } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                let source = std::fs::read_to_string(&path).expect("read Rust source");
                let count = source.matches(needle).count();
                if count != 0 {
                    hits.insert(path.display().to_string(), count);
                }
            }
        }
    }

    /// The policy expressions themselves belong only to this module. Consumers
    /// call the owner; a hand-written branch makes this counted law fail.
    #[test]
    fn policy_needles_have_exactly_one_owner_repo_wide() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        for needle in [
            ["if !type", "writer {"].concat(),
            ["checked_version != ", "Some(current_version)"].concat(),
        ] {
            let mut hits = BTreeMap::new();
            count_needle(&root, &needle, &mut hits);
            assert_eq!(
                hits.values().sum::<usize>(),
                1,
                "policy needle `{needle}` must have exactly one owner, found {hits:?}"
            );
            assert_eq!(
                hits.keys().collect::<Vec<_>>(),
                vec![&root.join("view_policy.rs").display().to_string()],
                "policy needle `{needle}` escaped its owner: {hits:?}"
            );
        }
    }
}
