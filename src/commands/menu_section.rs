pub(super) const FILE_COMMANDS: &[&str] = &[
    "New document",
    "Go to…",
    "Open file…",
    "Open folder…",
    "Save",
    "Finish file",
    "Export as PDF…",
    "Export as Word…",
    "Export as HTML…",
    "Version history…",
    "Move…",
    "Rename note…",
    "Duplicate note",
];

pub(super) const EDIT_COMMANDS: &[&str] = &[
    "Undo",
    "Redo",
    "Cut",
    "Copy",
    "Paste",
    "Select all",
    "Bold",
    "Italic",
    "Inline code",
    "Highlight",
    "Strikethrough",
    "Insert link…",
    "Heading",
    "Blockquote",
    "Bullet list",
    "Numbered list",
    "Task list",
    "Code block",
    "Align table",
];

pub(super) const VIEW_COMMANDS: &[&str] = &[
    "Toggle page mode",
    "Switch theme…",
    "Zoom in",
    "Zoom out",
    "Reset zoom",
    "Toggle debug",
    "Narrow page",
    "Widen page",
    "Reset page width",
    "Toggle outline",
    "Fold section",
    "Collapse other sections",
    "Toggle typewriter scroll",
    "Toggle menu bar",
];

pub fn menu_section(name: &str) -> Option<&'static str> {
    if FILE_COMMANDS.contains(&name) {
        Some("File")
    } else if EDIT_COMMANDS.contains(&name) {
        Some("Edit")
    } else if VIEW_COMMANDS.contains(&name) {
        Some("View")
    } else {
        None
    }
}
