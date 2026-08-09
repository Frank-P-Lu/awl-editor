//! Workspace shapes. `OverlayKind::workspace_shell() -> bool` became
//! `workspace_shape() -> Option<WorkspaceShape>`, and `WorkspaceShape::
//! rows_are_primary()` is the single fact every consumer (geometry, keyboard
//! handling, the footer hint) reduces to instead of re-deriving which region
//! holds a workspace's own rows. This file is the classification's own law:
//! the wildcard-free roster, and the "one match, one owner" bypass ban that
//! keeps a second reader from silently disagreeing with it.
//!
//! History still returns `None` from `workspace_shape` — nothing routes to
//! `TimelineOverComparison` yet, so `render::tests::workspace`'s
//! existing pixel laws are this slice's byte-identity proof: unmodified, and
//! still green, over the one kind (`Settings`) that reaches this geometry.

use crate::overlay::OverlayKind;
use crate::overlay::workspace::WorkspaceShape;

/// THE CLASSIFICATION ITSELF, wildcard-free over both variants — the thing a
/// third shape cannot dodge.
#[test]
fn rows_are_primary_classifies_both_shapes() {
    assert!(
        !WorkspaceShape::RailOverRows.rows_are_primary(),
        "Settings' rail carries LABELS, not rows"
    );
    assert!(
        WorkspaceShape::TimelineOverComparison.rows_are_primary(),
        "a timeline's rows are the primary list"
    );
}

/// THE ROSTER, swept wildcard-free over every `OverlayKind` — a new kind must
/// declare which side of the line it falls on before it compiles (the
/// classification itself is enforced by `OverlayKind::workspace_shape`'s own
/// exhaustive match; this proves today's roster reads the way the brief
/// states it does, so a future edit that quietly widens it is caught here).
#[test]
fn workspace_shape_roster_is_exact() {
    for kind in OverlayKind::ALL {
        let shape = kind.workspace_shape();
        let expected = match kind {
            OverlayKind::Settings => Some(WorkspaceShape::RailOverRows),
            OverlayKind::History | OverlayKind::Conflict => {
                Some(WorkspaceShape::TimelineOverComparison)
            }
            _ => None,
        };
        assert_eq!(shape, expected, "{kind:?} workspace_shape() drifted");
    }
    // Named directly, so a reader does not have to trust the loop above: the
    // workspace roster is exactly the two members DESIGN.md §5 names, and each
    // draws the shape that section describes for it (History uses its
    // timeline/comparison shape rather than a contextual card).
    assert_eq!(
        OverlayKind::History.workspace_shape(),
        Some(WorkspaceShape::TimelineOverComparison)
    );
    // THREE, deliberately. The gate is "a workspace is not a default", not
    // "there are exactly two workspaces": the conflict surface earns one on the
    // same grounds History does — sustained reading, in two coordinated regions,
    // with the document behind it being the very thing under comparison. It
    // reuses `TimelineOverComparison` unchanged, so the shape roster itself did
    // not grow; only its membership did.
    assert_eq!(
        OverlayKind::ALL
            .iter()
            .filter(|k| k.workspace_shape().is_some())
            .count(),
        3,
        "the workspace roster is deliberately short (DESIGN.md §5)"
    );
    // …and it is still exactly TWO shapes for those three members, which is the
    // half of this gate that has not moved.
    assert_eq!(
        OverlayKind::ALL
            .iter()
            .filter_map(|k| k.workspace_shape())
            .filter(|s| s.rows_are_primary())
            .count(),
        2,
        "History and the conflict share one shape; Settings has the other"
    );
}

/// THE BYPASS IS MODULE-PRIVATE. `rows_are_primary`'s match over
/// `WorkspaceShape`'s two variants is the ONLY place in the crate allowed to
/// name them directly — every other reader (geometry, keyboard, hints, a
/// must go through the method, or a third shape could
/// silently carry a different answer in two places (CLAUDE.md's
/// same-behavior-same-code rule). Scoped to the whole of `src/`, minus the two
/// files that legitimately construct a literal: the type's own definition and
/// this law's own roster test above.
#[test]
fn workspace_shape_variants_are_named_in_exactly_two_files() {
    let allowed = [
        "src/overlay/workspace.rs",
        "src/render/tests/workspace_shape.rs",
    ];
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut offenders: Vec<String> = Vec::new();
    let mut scanned = 0usize;
    fn walk(dir: &std::path::Path, out: &mut Vec<String>, scanned: &mut usize, allowed: &[&str]) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        let mut entries: Vec<_> = entries.flatten().collect();
        entries.sort_by_key(|e| e.path());
        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out, scanned, allowed);
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            *scanned += 1;
            let rel = path
                .strip_prefix(std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")))
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| path.to_string_lossy().to_string());
            if allowed.contains(&rel.as_str()) {
                continue;
            }
            for (i, line) in text.lines().enumerate() {
                let code = line.trim_start();
                if code.starts_with("//") || code.starts_with("///") || code.starts_with("//!") {
                    continue; // prose may name the shapes
                }
                if code.contains("WorkspaceShape::RailOverRows")
                    || code.contains("WorkspaceShape::TimelineOverComparison")
                {
                    out.push(format!("{rel}:{}", i + 1));
                }
            }
        }
    }
    walk(&root, &mut offenders, &mut scanned, &allowed);
    assert!(
        offenders.is_empty(),
        "a second place names a `WorkspaceShape` variant directly, bypassing \
         `WorkspaceShape::rows_are_primary` — the one owner every consumer must \
         route through. Offending lines: {offenders:?}"
    );
    assert!(
        scanned >= 100,
        "the scanner only read {scanned} files under src/ — it is looking in the \
         wrong place, not confirming a clean sweep"
    );
}
