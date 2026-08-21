use std::path::Path;

enum_with_all! {
    /// Every production owner whose write can replace user work or app state.
    /// `ALL` comes from this variant list, so the fault matrix cannot drift.
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
}

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

/// The durable create-if-absent door. It shares the temporary sibling write
/// with [`write`], but never lets a racing creator be replaced at publish.
#[cfg(not(target_arch = "wasm32"))]
pub fn write_new(owner: Owner, path: &Path, data: &[u8]) -> std::io::Result<()> {
    let _ = owner.name();
    crate::fs::write_atomic_new(path, data)
}
