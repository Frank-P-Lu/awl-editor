//! The command palette's task taxonomy.
//!
//! This is intentionally independent of native/web menu membership: every catalog
//! command has one browse home even when it remains palette-only. Settings rows join
//! the `Settings` category at the union seam because their typed row metadata already
//! distinguishes them from commands.

use crate::facets::{Facet, FacetItem, FacetScheme};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TaskCategory {
    Files,
    Navigate,
    Format,
    View,
    Tools,
    Settings,
}

impl TaskCategory {
    #[cfg(test)]
    pub const ALL: [Self; 6] = [
        Self::Files,
        Self::Navigate,
        Self::Format,
        Self::View,
        Self::Tools,
        Self::Settings,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Files => "Files",
            Self::Navigate => "Navigate",
            Self::Format => "Format",
            Self::View => "View",
            Self::Tools => "Tools",
            Self::Settings => "Settings",
        }
    }
}

use TaskCategory::{Files, Format, Navigate, Settings, Tools, View};

/// The single exhaustive classification table. The coverage law compares this
/// table with `COMMANDS` in both directions, so a new catalog row stays red until
/// it receives exactly one category here.
const COMMAND_TASK_CATEGORIES: &[(&str, TaskCategory)] = &[
    ("Command palette…", Navigate),
    ("Go to…", Navigate),
    ("Open file…", Files),
    ("Open folder…", Files),
    ("Spell suggestions…", Tools),
    ("Version history…", Files),
    ("Compare with version…", Files),
    ("Clean unused assets…", Tools),
    ("Keep version…", Files),
    ("Review the change", Files),
    ("Save your version", Files),
    ("Use disk version", Files),
    ("Last file", Navigate),
    ("New document", Files),
    ("Open scratch", Files),
    ("Keep tutorial…", Files),
    ("Move…", Files),
    ("Rename note…", Files),
    ("Duplicate note", Files),
    ("Move file to Trash", Files),
    ("Reveal in file manager", Files),
    ("Copy file path", Files),
    ("Finish file", Files),
    ("Follow link", Navigate),
    ("Copy link destination", Navigate),
    ("Switch theme…", View),
    ("Caret style…", Settings),
    ("Dictionary…", Settings),
    ("Keymap…", Settings),
    ("Toggle spellcheck", Settings),
    ("Toggle caret style", Settings),
    ("Toggle page mode", View),
    ("Toggle writing nits", Settings),
    ("Widen page", View),
    ("Narrow page", View),
    ("Reset page width", View),
    ("Toggle debug", View),
    ("Toggle outline", View),
    ("Fold section", View),
    ("Collapse other sections", View),
    ("Toggle typewriter scroll", View),
    ("Toggle menu bar", View),
    ("About", Tools),
    ("Credits", Tools),
    ("Lifetime stats", Tools),
    ("Writing streaks", Tools),
    ("Line endings…", Tools),
    ("Align table", Format),
    ("Tag document language", Format),
    ("Insert Date", Format),
    ("Report a Problem", Tools),
    ("Download file", Files),
    ("Check for Updates", Tools),
    ("Blockquote", Format),
    ("Bullet list", Format),
    ("Numbered list", Format),
    ("Task list", Format),
    ("Heading", Format),
    ("Cycle heading", Format),
    ("Code block", Format),
    ("Bold", Format),
    ("Italic", Format),
    ("Inline code", Format),
    ("Highlight", Format),
    ("Strikethrough", Format),
    ("Insert footnote", Format),
    ("Export as Word…", Files),
    ("Export as HTML…", Files),
    ("Export as PDF…", Files),
    ("Insert link…", Format),
    ("Save", Files),
    ("Save a Copy…", Files),
    ("Quit", Files),
    ("Search forward", Navigate),
    ("Search backward", Navigate),
    ("Find and replace…", Navigate),
    ("Undo", Format),
    ("Redo", Format),
    ("Copy", Format),
    ("Cut", Format),
    ("Paste", Format),
    ("Select all", Format),
    ("Zoom in", View),
    ("Zoom out", View),
    ("Reset zoom", View),
    ("Forward word", Navigate),
    ("Backward word", Navigate),
    ("Line start", Navigate),
    ("Line end", Navigate),
    ("Document start", Navigate),
    ("Document end", Navigate),
    ("Forward char", Navigate),
    ("Backward char", Navigate),
    ("Next line", Navigate),
    ("Previous line", Navigate),
    ("Delete word forward", Navigate),
    ("Delete word backward", Navigate),
    ("Settings…", Settings),
    ("Keybindings…", Settings),
];

pub fn task_category_of(name: &str) -> Option<TaskCategory> {
    COMMAND_TASK_CATEGORIES
        .iter()
        .find_map(|(candidate, category)| (*candidate == name).then_some(*category))
}

const COMMAND_FACET_STRIP: [Facet; 8] = [
    Facet {
        label: "All",
        id: "all",
        sections: &[],
    },
    Facet {
        label: "Files",
        id: "files",
        sections: &["Files"],
    },
    Facet {
        label: "Navigate",
        id: "navigate",
        sections: &["Navigate"],
    },
    Facet {
        label: "Format",
        id: "format",
        sections: &["Format"],
    },
    Facet {
        label: "View",
        id: "view",
        sections: &["View"],
    },
    Facet {
        label: "Tools",
        id: "tools",
        sections: &["Tools"],
    },
    Facet {
        label: "Settings",
        id: "settings",
        sections: &["Settings"],
    },
    Facet {
        label: "Recent",
        id: "recent",
        sections: &["Recent"],
    },
];

pub(super) fn command_bucket(item: FacetItem, lens_idx: usize) -> Option<&'static str> {
    let category = match lens_idx {
        1 => TaskCategory::Files,
        2 => TaskCategory::Navigate,
        3 => TaskCategory::Format,
        4 => TaskCategory::View,
        5 => TaskCategory::Tools,
        6 => TaskCategory::Settings,
        7 => return item.recent.then_some("Recent"),
        _ => return None,
    };
    if task_category_of(item.accept) == Some(category)
        || (category == TaskCategory::Settings
            && crate::settings::palette_rows()
                .iter()
                .any(|row| row.name == item.accept))
    {
        Some(category.label())
    } else {
        None
    }
}

pub static COMMAND_FACETS: FacetScheme = FacetScheme {
    strip: &COMMAND_FACET_STRIP,
    bucket: command_bucket,
};

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn every_command_has_exactly_one_task_category() {
        let mut counts = HashMap::new();
        for (name, _) in COMMAND_TASK_CATEGORIES {
            *counts.entry(*name).or_insert(0usize) += 1;
        }
        for command in crate::commands::COMMANDS.iter() {
            assert_eq!(
                counts.get(command.name),
                Some(&1),
                "{:?} must have exactly one task category",
                command.name
            );
        }
        let catalog: HashSet<_> = crate::commands::COMMANDS.iter().map(|c| c.name).collect();
        for name in counts.keys() {
            assert!(
                catalog.contains(name),
                "task taxonomy contains unknown command {name:?}"
            );
        }
    }

    #[test]
    fn category_roster_is_no_wildcard_and_every_category_is_occupied() {
        for category in TaskCategory::ALL {
            let label = match category {
                TaskCategory::Files => "Files",
                TaskCategory::Navigate => "Navigate",
                TaskCategory::Format => "Format",
                TaskCategory::View => "View",
                TaskCategory::Tools => "Tools",
                TaskCategory::Settings => "Settings",
            };
            assert_eq!(category.label(), label);
            assert!(COMMAND_TASK_CATEGORIES.iter().any(|(_, c)| *c == category));
        }
    }

    #[test]
    fn native_web_and_settings_rows_each_have_one_browse_home() {
        for platform in [
            crate::commands::Platform::Native,
            crate::commands::Platform::Web,
        ] {
            let visible = crate::commands::visible_on(platform);
            for command in visible {
                assert!(
                    task_category_of(command.name).is_some(),
                    "{platform:?} command {:?} has no browse home",
                    command.name
                );
            }
        }
        for row in crate::settings::palette_rows() {
            assert_eq!(
                TaskCategory::Settings.label(),
                "Settings",
                "setting {:?} has exactly the Settings browse home",
                row.name
            );
        }
    }
}
