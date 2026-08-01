//! Pure contextual-menu policy. Pointer hit-testing names a [`ContextTarget`];
//! this module is the single owner of the ordered catalog actions exposed for
//! that target on a given platform and editor state.

use crate::commands::Platform;
use crate::keymap::Action;

enum_with_all! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ContextTarget {
        Misspelling,
        Selection,
        Link,
        Heading,
        Body,
        Filename,
        Folder,
        LeftEdge,
        RightEdge,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContextState {
    pub has_selection: bool,
    pub link: bool,
    pub heading: bool,
    pub heading_folded: bool,
    pub misspelled: bool,
    pub named_file: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextRow {
    pub label: &'static str,
    pub action: Action,
    pub enabled: bool,
}

fn row(label: &'static str, action: Action, enabled: bool) -> ContextRow {
    ContextRow {
        label,
        action,
        enabled,
    }
}

/// Resolve overlapping document facts to exactly one target. Spelling and the
/// selection surface retain their established priority over links/headings/body.
pub fn document_target(state: ContextState) -> ContextTarget {
    if state.misspelled {
        ContextTarget::Misspelling
    } else if state.has_selection {
        ContextTarget::Selection
    } else if state.link {
        ContextTarget::Link
    } else if state.heading {
        ContextTarget::Heading
    } else {
        ContextTarget::Body
    }
}

pub const fn modified_link_hover(command_down: bool, over_link: bool) -> bool {
    command_down && over_link
}

pub fn copy_link_destination(buffer: &mut crate::buffer::Buffer) {
    let byte = buffer.char_to_byte(buffer.cursor_char());
    if let Some(url) = crate::markdown::link_at(&buffer.text(), byte) {
        buffer.set_kill(&url);
    }
}

pub fn overlay(rows: Vec<ContextRow>, anchor: (f32, f32)) -> crate::overlay::OverlayState {
    let mut state = crate::overlay::OverlayState::new(
        crate::overlay::OverlayKind::Context,
        rows.iter().map(|row| row.label.to_string()).collect(),
        Vec::new(),
        Vec::new(),
    );
    state.context_actions = rows
        .iter()
        .map(|row| row.enabled.then_some(row.action.clone()))
        .collect();
    state.set_secondaries(
        rows.into_iter()
            .map(|row| {
                (!row.enabled)
                    .then_some("unavailable")
                    .unwrap_or_default()
                    .to_string()
            })
            .collect(),
    );
    state.context_anchor = Some(anchor);
    state
}

/// The complete target × state × platform owner. Every row routes through a
/// catalog Action; unavailable native filesystem operations are omitted on web.
pub fn rows(target: ContextTarget, state: ContextState, platform: Platform) -> Vec<ContextRow> {
    use Action::*;
    use ContextTarget::*;
    match target {
        Misspelling => Vec::new(), // the established spell picker owns this target
        Selection => vec![
            row("Cut", KillRegion, state.has_selection),
            row("Copy", CopyRegion, state.has_selection),
            row("Paste", Yank, true),
            row("Select all", SelectAll, true),
        ],
        Link => vec![
            row("Follow link", FollowLink, true),
            row("Edit link…", InsertLink, true),
            row("Copy destination", CopyLinkDestination, true),
        ],
        Heading => vec![
            row(
                if state.heading_folded {
                    "Expand section"
                } else {
                    "Fold section"
                },
                ToggleFold,
                true,
            ),
            row("Collapse other sections", CollapseOtherSections, true),
            row("Go to heading…", OpenOutline, true),
        ],
        Body => vec![
            row("Cut", KillRegion, false),
            row("Copy", CopyRegion, false),
            row("Paste", Yank, true),
            row("Select all", SelectAll, true),
        ],
        Filename if platform == Platform::Native => vec![
            row("Rename file…", OpenRenameNote, state.named_file),
            row("Move file…", MoveFile, state.named_file),
            row("Duplicate file", DuplicateNote, state.named_file),
            row("Version history…", OpenHistory, state.named_file),
        ],
        Filename => Vec::new(),
        Folder if platform == Platform::Native => vec![
            row("Switch folder…", OpenProject, true),
            row("Browse files…", OpenBrowse, true),
        ],
        Folder => vec![row("Browse files…", OpenBrowse, true)],
        LeftEdge | RightEdge => vec![
            row("Narrow", PageNarrower, true),
            row("Widen", PageWider, true),
            row("Reset page width", PageReset, true),
            row("Toggle page mode", TogglePageMode, true),
            row("Page width settings…", OpenSettingsMenu, true),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> ContextState {
        ContextState {
            has_selection: false,
            link: false,
            heading: false,
            heading_folded: false,
            misspelled: false,
            named_file: true,
        }
    }

    #[test]
    fn priority_is_exhaustive_over_every_document_fact_cell() {
        for bits in 0u8..32 {
            let s = ContextState {
                misspelled: bits & 1 != 0,
                has_selection: bits & 2 != 0,
                link: bits & 4 != 0,
                heading: bits & 8 != 0,
                heading_folded: bits & 16 != 0,
                named_file: true,
            };
            let want = if s.misspelled {
                ContextTarget::Misspelling
            } else if s.has_selection {
                ContextTarget::Selection
            } else if s.link {
                ContextTarget::Link
            } else if s.heading {
                ContextTarget::Heading
            } else {
                ContextTarget::Body
            };
            assert_eq!(document_target(s), want, "bits={bits:05b}");
        }
    }

    #[test]
    fn link_cursor_affordance_sweeps_modifier_and_hit_state() {
        for command_down in [false, true] {
            for over_link in [false, true] {
                assert_eq!(
                    modified_link_hover(command_down, over_link),
                    command_down && over_link
                );
            }
        }
    }

    #[test]
    fn every_target_state_platform_cell_has_only_catalog_actions() {
        for target in ContextTarget::ALL {
            for platform in [Platform::Native, Platform::Web] {
                for folded in [false, true] {
                    for selected in [false, true] {
                        let mut s = state();
                        s.heading_folded = folded;
                        s.has_selection = selected;
                        for row in rows(target, s, platform) {
                            assert!(
                                crate::commands::COMMANDS
                                    .iter()
                                    .any(|c| c.action == row.action),
                                "{target:?}/{platform:?}: {:?} is not catalog-routed",
                                row.action
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn edge_sides_are_identical_and_selection_never_duplicates_formatting() {
        let s = state();
        assert_eq!(
            rows(ContextTarget::LeftEdge, s, Platform::Native),
            rows(ContextTarget::RightEdge, s, Platform::Native)
        );
        let mut selected = s;
        selected.has_selection = true;
        let labels: Vec<_> = rows(ContextTarget::Selection, selected, Platform::Native)
            .into_iter()
            .map(|r| r.label)
            .collect();
        assert_eq!(labels, ["Cut", "Copy", "Paste", "Select all"]);
    }
}
