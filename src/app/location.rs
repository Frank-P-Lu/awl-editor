//! The two runtime owners behind an editor's working location.
//!
//! `ConfigurationRuntime` owns persisted configuration together with the CLI
//! overrides and first-run default-folder policy. `ProjectLocation` owns the
//! live root and everything derived from it. The typed policy is the only
//! value that crosses that boundary: configuration says *how* to choose a
//! workspace, while `App::resync_project_location` remains the one place that
//! applies that choice to the current root.

use crate::config::Config;
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::app) struct LocationPolicy {
    workspace: Option<PathBuf>,
}

impl LocationPolicy {
    fn from_sources(cli_workspace: &Option<PathBuf>, config: &Config) -> Self {
        Self {
            workspace: cli_workspace.clone().or_else(|| config.workspace.clone()),
        }
    }

    pub(in crate::app) fn workspace_root(&self, root: &std::path::Path) -> PathBuf {
        crate::resolve_workspace(&self.workspace, root)
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(in crate::app) fn workspace_override(&self) -> Option<PathBuf> {
        self.workspace.clone()
    }
}

/// What changed when configuration was re-read from disk.
pub(in crate::app) struct ReloadOutcome {
    pub(in crate::app) location_policy: LocationPolicy,
}

/// Configuration plus the startup-only inputs that take precedence over it.
///
/// `Deref<Target = Config>` keeps ordinary configuration readers concise,
/// while the CLI/default-folder policy remains visibly owned here.
pub(in crate::app) struct ConfigurationRuntime {
    config: Config,
    pub(in crate::app) default_folder: PathBuf,
    cli_default_folder: Option<PathBuf>,
    cli_workspace: Option<PathBuf>,
}

/// Configuration facts consumed by the frame scheduler, detached from the
/// mutable persisted configuration owner for the duration of one poll.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::app) struct SchedulingSnapshot {
    ambient_motion_on: bool,
}

impl SchedulingSnapshot {
    pub(in crate::app) fn ambient_motion_on(self) -> bool {
        self.ambient_motion_on
    }
}

impl ConfigurationRuntime {
    pub(in crate::app) fn new(
        config: Config,
        cli_workspace: Option<PathBuf>,
        cli_default_folder: Option<PathBuf>,
    ) -> Self {
        let default_folder = crate::resolve_default_folder(
            &cli_default_folder
                .clone()
                .or_else(|| config.default_folder.clone()),
        );
        Self {
            config,
            default_folder,
            cli_default_folder,
            cli_workspace,
        }
    }

    pub(in crate::app) fn location_policy(&self) -> LocationPolicy {
        LocationPolicy::from_sources(&self.cli_workspace, &self.config)
    }

    pub(in crate::app) fn scheduling_snapshot(&self) -> SchedulingSnapshot {
        SchedulingSnapshot {
            ambient_motion_on: self.config.ambient_motion_on(),
        }
    }

    /// THE ONE READ of the LOCAL-USAGE PRIVACY TOGGLE anywhere under
    /// `src/app/`, handed to `UsageLedger`'s transitions as a typed value —
    /// the same shape as [`Self::scheduling_snapshot`] and
    /// [`Self::location_policy`]: configuration states a policy, and the
    /// domain that acts on it never re-derives one of its own.
    ///
    /// It used to be re-read at eight sites across the odometer and streaks
    /// wiring, where a tracking hook that forgot the `if` was a privacy defect
    /// one missing line away. `the_usage_privacy_gate_has_exactly_one_reader`
    /// in `app/tests/domains.rs` keeps it singular.
    #[cfg(not(target_arch = "wasm32"))]
    pub(in crate::app) fn usage_recording(&self) -> super::usage::Recording {
        super::usage::Recording::from_config(self.config.stats_on())
    }

    pub(in crate::app) fn apply_loaded(&mut self, config: Config) -> ReloadOutcome {
        self.config = config;
        self.default_folder = crate::resolve_default_folder(
            &self
                .cli_default_folder
                .clone()
                .or_else(|| self.config.default_folder.clone()),
        );
        ReloadOutcome {
            location_policy: self.location_policy(),
        }
    }
}

impl Deref for ConfigurationRuntime {
    type Target = Config;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

impl DerefMut for ConfigurationRuntime {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.config
    }
}

/// The live project root and its derived, project-scoped state.
pub(in crate::app) struct ProjectLocation {
    pub(in crate::app) root: PathBuf,
    pub(in crate::app) project: crate::project::Project,
    pub(in crate::app) file_index: Vec<String>,
    pub(in crate::app) workspace_root: Option<PathBuf>,
    pub(in crate::app) recent_projects: Vec<PathBuf>,
    pub(in crate::app) recent_files: Vec<PathBuf>,
}

impl ProjectLocation {
    pub(in crate::app) fn new(root: PathBuf, policy: &LocationPolicy) -> Self {
        Self {
            project: crate::project::Project::resolve(&root),
            file_index: crate::index::build_index(&root),
            workspace_root: Some(policy.workspace_root(&root)),
            recent_projects: crate::recents::load(&crate::recents::recents_path()),
            recent_files: crate::recent_files::load(),
            root,
        }
    }

    pub(in crate::app) fn rescan_file_index(&mut self) {
        self.file_index = crate::index::build_index(&self.root);
    }

    pub(in crate::app) fn resync(&mut self, policy: LocationPolicy) {
        self.project = crate::project::Project::resolve(&self.root);
        self.rescan_file_index();
        self.workspace_root = Some(policy.workspace_root(&self.root));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_workspace_wins_in_the_location_policy() {
        let mut config = Config::empty();
        config.workspace = Some(PathBuf::from("/configured"));
        let runtime = ConfigurationRuntime::new(config, Some(PathBuf::from("/explicit")), None);

        assert_eq!(
            runtime
                .location_policy()
                .workspace_root(std::path::Path::new("/root/project")),
            PathBuf::from("/explicit")
        );
    }

    #[test]
    fn reload_outcome_carries_the_reloaded_workspace_policy() {
        let runtime = ConfigurationRuntime::new(Config::empty(), None, None);
        let mut reloaded = Config::empty();
        reloaded.workspace = Some(PathBuf::from("/new-workspace"));
        let mut runtime = runtime;

        let outcome = runtime.apply_loaded(reloaded);

        assert_eq!(
            outcome
                .location_policy
                .workspace_root(std::path::Path::new("/root/project")),
            PathBuf::from("/new-workspace")
        );
    }
}
