use std::path::Path;

/// Every production owner whose write can replace user work or app state that
/// is needed after relaunch. The fault matrix iterates this roster directly,
/// so a new owner cannot arrive outside the sweep through a test-only list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Owner {
    ManualSave,
    Autosave,
    Scratch,
    Recovery,
    History,
    Config,
    Session,
    Export,
}

#[allow(dead_code)] // production roster; its exhaustive consumer is the fault matrix
pub const OWNERS: &[Owner] = &[
    Owner::ManualSave,
    Owner::Autosave,
    Owner::Scratch,
    Owner::Recovery,
    Owner::History,
    Owner::Config,
    Owner::Session,
    Owner::Export,
];

impl Owner {
    pub const fn name(self) -> &'static str {
        match self {
            Self::ManualSave => "manual-save",
            Self::Autosave => "autosave",
            Self::Scratch => "scratch",
            Self::Recovery => "recovery",
            Self::History => "history",
            Self::Config => "config",
            Self::Session => "session",
            Self::Export => "export",
        }
    }
}

/// The named durable-write door. `fs::write_atomic` remains the deliberately
/// small mechanism; the owner argument makes production enrolment data.
pub fn write(owner: Owner, path: &Path, data: &[u8]) -> std::io::Result<()> {
    let _ = owner.name();
    crate::fs::write_atomic(path, data)
}
