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
}

fn row(label: &'static str, action: Action) -> ContextRow {
    ContextRow { label, action }
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

/// Copy the buffer's own absolute path onto the kill ring — the same
/// set-kill shape as [`copy_link_destination`], sharing the ordinary
/// `WriteKillRing` mirror to the OS clipboard rather than a second write
/// path. A no-op for a path-less scratch buffer (nothing to copy); the
/// palette/context-menu gate keeps the row from firing in the first place,
/// this is the core's own floor for a chord that reaches it directly.
pub fn copy_file_path(buffer: &mut crate::buffer::Buffer) {
    if let Some(path) = buffer.path() {
        buffer.set_kill(&path.display().to_string());
    }
}

/// The Reveal command's platform label: macOS names its own file browser
/// ("Reveal in Finder", matching how every other native Mac app's context
/// menu phrases this exact verb); every other native target gets the
/// platform-neutral name. `cfg!(target_os)`, not [`crate::convention::Convention`]
/// — the underlying capability itself (`mac_chrome::reveal_in_file_viewer`)
/// is gated on the real OS, an axis `Convention` (⌘ vs Ctrl chord reading)
/// does not model, so the label tracks the same compile-time fact the
/// behavior does. Deliberately NOT threaded into the catalog's own `name`
/// (see `commands::COMMANDS`): that field is read by `guide::tests`/
/// `reference::law`'s cross-host doc-drift laws, which regenerate on
/// whichever CI runner's `target_os` happens to run them — a host-dependent
/// catalog name would make those laws disagree with themselves between the
/// mac and linux jobs.
fn reveal_label() -> &'static str {
    if cfg!(target_os = "macos") {
        "Reveal in Finder"
    } else {
        "Reveal in file manager"
    }
}

pub fn overlay(rows: Vec<ContextRow>, anchor: (f32, f32)) -> crate::overlay::OverlayState {
    let mut state = crate::overlay::OverlayState::new(
        crate::overlay::OverlayKind::Context,
        rows.iter().map(|row| row.label.to_string()).collect(),
        Vec::new(),
        Vec::new(),
    );
    state.context_actions = rows.into_iter().map(|row| row.action).collect();
    state.context_anchor = Some(anchor);
    state
}

/// The complete target × state × platform owner. Every row routes through a
/// catalog Action. A row exists only when it can apply at this target; native
/// filesystem operations are omitted on web.
pub fn rows(target: ContextTarget, state: ContextState, platform: Platform) -> Vec<ContextRow> {
    use Action::*;
    use ContextTarget::*;
    match target {
        Misspelling => Vec::new(), // the established spell picker owns this target
        Selection => vec![
            row("Cut", KillRegion),
            row("Copy", CopyRegion),
            row("Paste", Yank),
            row("Select all", SelectAll),
        ],
        Link => vec![
            row("Follow link", FollowLink),
            row("Edit link…", InsertLink),
            row("Copy destination", CopyLinkDestination),
        ],
        Heading => vec![
            row(
                if state.heading_folded {
                    "Expand section"
                } else {
                    "Fold section"
                },
                ToggleFold,
            ),
            row("Collapse other sections", CollapseOtherSections),
            row("Go to heading…", OpenOutline),
        ],
        Body => vec![row("Paste", Yank), row("Select all", SelectAll)],
        Filename if platform == Platform::Native && state.named_file => {
            let mut rows = vec![
                row("Rename file…", OpenRenameNote),
                row("Move file…", MoveFile),
                row("Duplicate file", DuplicateNote),
                row("Version history…", OpenHistory),
                row(reveal_label(), RevealInFileManager),
                row("Copy file path", CopyFilePath),
            ];
            if cfg!(target_os = "macos") {
                rows.insert(3, row("Move file to Trash", TrashFile));
            }
            rows
        }
        Filename => Vec::new(),
        Folder if platform == Platform::Native => vec![
            row("Go to folders…", OpenProject),
            row("Open file…", OpenBrowse),
        ],
        Folder => vec![row("Go to folders…", OpenProject)],
        LeftEdge | RightEdge => vec![
            row("Narrow", PageNarrower),
            row("Widen", PageWider),
            row("Reset page width", PageReset),
            row("Toggle page mode", TogglePageMode),
            row("Page width settings…", OpenSettingsMenu),
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
                                    .any(|c| c.action == row.action)
                                    || matches!(
                                        row.action,
                                        crate::keymap::Action::OpenOutline
                                            | crate::keymap::Action::OpenProject
                                    ),
                                "{target:?}/{platform:?}: {:?} is neither catalog-routed nor a \
                                 contextual Go-to lens deep link",
                                row.action
                            );
                        }
                    }
                }
            }
        }
    }

    /// The context roster is an applicability roster, not a disabled-command
    /// roster: every target × state × platform cell contains only actions that
    /// work at that target. The two formerly disabled cases are named by their
    /// observable lists so this goes red if either gate becomes a grey row.
    #[test]
    fn roster_omits_inapplicable_actions_across_every_target_state_and_platform() {
        for target in ContextTarget::ALL {
            for platform in [Platform::Native, Platform::Web] {
                for bits in 0u8..64 {
                    let s = ContextState {
                        has_selection: bits & 1 != 0,
                        link: bits & 2 != 0,
                        heading: bits & 4 != 0,
                        heading_folded: bits & 8 != 0,
                        misspelled: bits & 16 != 0,
                        named_file: bits & 32 != 0,
                    };
                    let got = rows(target, s, platform);
                    let actions: Vec<_> = got.iter().map(|row| row.action.clone()).collect();
                    assert!(
                        !actions.iter().any(|action| {
                            matches!(action, Action::KillRegion | Action::CopyRegion)
                                && target == ContextTarget::Body
                        }),
                        "{target:?}/{platform:?}/{bits:06b}: Body exposed Cut/Copy: {actions:?}"
                    );
                    if target == ContextTarget::Body {
                        assert_eq!(actions, [Action::Yank, Action::SelectAll]);
                    }
                    if target == ContextTarget::Filename
                        && (!s.named_file || platform == Platform::Web)
                    {
                        assert!(
                            actions.is_empty(),
                            "{target:?}/{platform:?}/{bits:06b}: filename rows: {actions:?}"
                        );
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

    /// Reveal + Copy-path join the working-set stack's shared filename menu
    /// alongside Rename/Move/Duplicate/Version-history — same catalog
    /// actions, same door, no second menu-wiring mechanism.
    #[test]
    fn filename_menu_carries_reveal_and_copy_path_on_native() {
        let rows = rows(ContextTarget::Filename, state(), Platform::Native);
        let entries: Vec<(&str, Action)> =
            rows.iter().map(|r| (r.label, r.action.clone())).collect();
        assert!(
            entries
                .iter()
                .any(|(_, a)| *a == Action::RevealInFileManager),
            "Filename menu is missing Reveal in file manager: {entries:?}"
        );
        assert!(
            entries.iter().any(|(_, a)| *a == Action::CopyFilePath),
            "Filename menu is missing Copy file path: {entries:?}"
        );
    }

    /// An unnamed scratch document has nowhere to reveal and no path to copy,
    /// so its filename target is absent rather than a card of disabled rows.
    #[test]
    fn filename_actions_are_omitted_for_an_unnamed_scratch_document() {
        let mut s = state();
        s.named_file = false;
        let rows = rows(ContextTarget::Filename, s, Platform::Native);
        assert!(
            rows.is_empty(),
            "an unnamed filename target has no applicable actions: {rows:?}"
        );
    }

    /// A named document includes both rows, and the visible label tracks the
    /// current build's OS: macOS says "Reveal in Finder" (matching every
    /// other native app's own context menu), any other native target gets the
    /// platform-neutral name. `cfg!(target_os)`, not `Convention` — see
    /// `reveal_label`'s own doc for why the two axes are not interchangeable
    /// here.
    #[test]
    fn reveal_label_matches_the_build_os_and_both_rows_exist_for_a_named_document() {
        let rows = rows(ContextTarget::Filename, state(), Platform::Native);
        let reveal = rows
            .iter()
            .find(|r| r.action == Action::RevealInFileManager)
            .expect("Reveal row is present");
        let want = if cfg!(target_os = "macos") {
            "Reveal in Finder"
        } else {
            "Reveal in file manager"
        };
        assert_eq!(reveal.label, want);

        assert!(
            rows.iter().any(|r| r.action == Action::CopyFilePath),
            "Copy file path row is present"
        );
    }
}
