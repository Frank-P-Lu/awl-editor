//! The shared contract for every place awl asks the writer to choose a location.
//!
//! A location picker is deliberately not a generic filesystem dialog.  This
//! module owns the parts the consumers genuinely share — directory scope, folder
//! rows, query/focus, breadcrumb presentation, and back navigation — while the
//! typed operation still owns its final effect and filesystem side effect.

use super::OverlayKind;

/// The action that becomes possible only after a location is explicitly
/// accepted.  It is vocabulary for the route roster, not an effect: effects
/// retain their operation-specific payloads and interpreters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CommitVerb {
    OpenFile,
    MoveHere,
    SaveCopyHere,
    ExportHere,
    SwitchHere,
    SetFolder,
}

/// Every consumer of a user-chosen location, including the macOS-native doors.
/// The native rows remain native surfaces; naming them here makes their final
/// verbs auditable beside the in-app navigators rather than leaving a second,
/// unrostered family in `app/menu.rs`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(any(target_os = "macos", test)), allow(dead_code))]
pub(crate) enum LocationConsumer {
    Browse,
    MoveDestination,
    SaveCopyDestination,
    ExportDestination,
    ProjectSwitch,
    SettingPath,
    ProjectBrowse,
    NativeOpen,
    NativeSaveCopy,
    NativeExportDocx,
    NativeExportHtml,
    NativeExportPdf,
}

/// The boundary a chooser presents: active-root relative, workspace absolute,
/// or the system-wide reach of a native panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LocationScope {
    ActiveRoot,
    Workspace,
    System,
}

impl LocationConsumer {
    #[cfg(test)]
    pub(crate) const ALL: [Self; 12] = [
        Self::Browse,
        Self::MoveDestination,
        Self::SaveCopyDestination,
        Self::ExportDestination,
        Self::ProjectSwitch,
        Self::SettingPath,
        Self::ProjectBrowse,
        Self::NativeOpen,
        Self::NativeSaveCopy,
        Self::NativeExportDocx,
        Self::NativeExportHtml,
        Self::NativeExportPdf,
    ];

    /// The in-app card this consumer uses, if it has one.  Save Copy reuses the
    /// export destination card but remains a separate consumer because its
    /// filename/overwrite constraints and final writer are different.
    #[cfg(test)]
    pub(crate) fn overlay_kind(self) -> Option<OverlayKind> {
        match self {
            Self::Browse => Some(OverlayKind::Browse),
            Self::MoveDestination => Some(OverlayKind::MoveDest),
            Self::SaveCopyDestination | Self::ExportDestination => Some(OverlayKind::ExportDest),
            Self::ProjectSwitch | Self::SettingPath => Some(OverlayKind::Project),
            Self::ProjectBrowse => Some(OverlayKind::ProjectBrowse),
            Self::NativeOpen
            | Self::NativeSaveCopy
            | Self::NativeExportDocx
            | Self::NativeExportHtml
            | Self::NativeExportPdf => None,
        }
    }

    pub(crate) fn commit_verb(self) -> CommitVerb {
        match self {
            Self::Browse | Self::NativeOpen => CommitVerb::OpenFile,
            Self::MoveDestination => CommitVerb::MoveHere,
            Self::SaveCopyDestination | Self::NativeSaveCopy => CommitVerb::SaveCopyHere,
            Self::ExportDestination
            | Self::NativeExportDocx
            | Self::NativeExportHtml
            | Self::NativeExportPdf => CommitVerb::ExportHere,
            Self::ProjectSwitch | Self::ProjectBrowse => CommitVerb::SwitchHere,
            Self::SettingPath => CommitVerb::SetFolder,
        }
    }

    pub(crate) fn scope(self) -> LocationScope {
        match self {
            Self::ProjectSwitch | Self::SettingPath | Self::ProjectBrowse => {
                LocationScope::Workspace
            }
            Self::Browse
            | Self::MoveDestination
            | Self::SaveCopyDestination
            | Self::ExportDestination => LocationScope::ActiveRoot,
            Self::NativeOpen
            | Self::NativeSaveCopy
            | Self::NativeExportDocx
            | Self::NativeExportHtml
            | Self::NativeExportPdf => LocationScope::System,
        }
    }

    /// Folder rows are a navigation step, never an operation commit.  Browse
    /// is the one member that additionally renders files, which only open on
    /// its explicit accept.
    pub(crate) fn folders_only(self) -> bool {
        !matches!(self, Self::Browse | Self::NativeOpen)
    }

    pub(crate) fn shows_relative_breadcrumb(self) -> bool {
        matches!(
            self,
            Self::MoveDestination | Self::SaveCopyDestination | Self::ExportDestination
        )
    }

    pub(crate) fn supports_back(self) -> bool {
        matches!(
            self,
            Self::Browse
                | Self::MoveDestination
                | Self::SaveCopyDestination
                | Self::ExportDestination
                | Self::SettingPath
                | Self::ProjectBrowse
        )
    }

    pub(crate) fn uses_folder_navigation(self) -> bool {
        matches!(
            self,
            Self::MoveDestination
                | Self::SaveCopyDestination
                | Self::ExportDestination
                | Self::SettingPath
                | Self::ProjectBrowse
        )
    }

    pub(crate) fn allows_typed_new_folder(self) -> bool {
        matches!(self, Self::SaveCopyDestination | Self::ExportDestination)
    }

    pub(crate) fn commit_label(self) -> &'static str {
        match self.commit_verb() {
            CommitVerb::OpenFile => "open",
            CommitVerb::MoveHere => "move here",
            CommitVerb::SaveCopyHere => "save a copy here",
            CommitVerb::ExportHere => "export here",
            CommitVerb::SwitchHere => "switch here",
            CommitVerb::SetFolder => "choose",
        }
    }

    #[cfg(test)]
    pub(crate) fn native_menu_id(self) -> Option<&'static str> {
        match self {
            Self::NativeOpen => Some("awl.open"),
            Self::NativeSaveCopy => Some("awl.save_copy"),
            Self::NativeExportPdf => Some("awl.export_pdf"),
            Self::NativeExportDocx => Some("awl.export_word"),
            Self::NativeExportHtml => Some("awl.export_html"),
            _ => None,
        }
    }
}

/// The navigator family represented by `kind`.  `ExportDest` has two typed
/// operations, so callers resolving its commitment use [`consumer_for_card`]
/// to preserve that distinction.
pub(crate) fn navigator_for(kind: OverlayKind) -> Option<LocationConsumer> {
    match kind {
        OverlayKind::Browse => Some(LocationConsumer::Browse),
        OverlayKind::MoveDest => Some(LocationConsumer::MoveDestination),
        OverlayKind::ExportDest => Some(LocationConsumer::ExportDestination),
        OverlayKind::ProjectBrowse => Some(LocationConsumer::ProjectBrowse),
        _ => None,
    }
}

/// The exact operation represented by an existing card.  The `save_copy` bit
/// is carried through every relevel by `Journey`, so this is safe to ask only
/// at explicit acceptance time.
pub(crate) fn consumer_for_card(card: &super::OverlayState) -> Option<LocationConsumer> {
    match card.kind {
        OverlayKind::ExportDest if card.save_copy => Some(LocationConsumer::SaveCopyDestination),
        OverlayKind::Project => Some(LocationConsumer::ProjectSwitch),
        kind => navigator_for(kind),
    }
}

/// Resolve the Project card whose purpose is carried by the parked journey.
pub(crate) fn consumer_for_route(
    card: &super::OverlayState,
    bind: Option<&super::Bind>,
) -> Option<LocationConsumer> {
    match (card.kind, bind) {
        (OverlayKind::Project, Some(super::Bind::Path { .. })) => {
            Some(LocationConsumer::SettingPath)
        }
        _ => consumer_for_card(card),
    }
}

pub(crate) fn join_browse(dir: Option<&str>, name: &str) -> String {
    match dir {
        Some(dir) if !dir.is_empty() => format!("{dir}/{name}"),
        _ => name.to_string(),
    }
}

pub(crate) fn browse_parent(dir: Option<&str>) -> Option<Option<String>> {
    match dir {
        None => None,
        Some(dir) => match dir.rsplit_once('/') {
            Some((parent, _)) => Some(Some(parent.to_string())),
            None => Some(None),
        },
    }
}

/// `Project` and `ProjectBrowse` carry an absolute directory in `browse_dir`;
/// every active-root navigator carries a root-relative path instead.
fn walks_absolute(kind: OverlayKind) -> bool {
    match navigator_for(kind) {
        Some(route) => route.scope() == LocationScope::Workspace,
        // The Settings folder-value picker is not a location consumer, but it
        // also walks absolute workspace paths.
        None => kind == OverlayKind::Project,
    }
}

/// The path named by a highlighted child row. Remembered project rows already
/// contain an absolute path, which `Path::join` deliberately preserves.
pub(crate) fn descend_target(card: &super::OverlayState, name: &str) -> String {
    match walks_absolute(card.kind) {
        true => std::path::Path::new(card.browse_dir.as_deref().unwrap_or(""))
            .join(name)
            .to_string_lossy()
            .to_string(),
        false => join_browse(card.browse_dir.as_deref(), name),
    }
}

pub(crate) fn ascend_target(card: &super::OverlayState) -> Option<Option<String>> {
    match walks_absolute(card.kind) {
        true => std::path::Path::new(card.browse_dir.as_deref().unwrap_or("/"))
            .parent()
            .map(|parent| Some(parent.to_string_lossy().to_string())),
        false => browse_parent(card.browse_dir.as_deref()),
    }
}

/// The folder a destination card's accept names: its highlighted folder, a
/// permitted typed new folder, or the level currently being viewed.
pub(crate) fn dest_value(card: &super::OverlayState, allow_new: bool) -> Option<String> {
    if let Some(name) = card.selected_value()
        && card.selected_is_dir()
    {
        return Some(join_browse(card.browse_dir.as_deref(), name));
    }
    let query = card.query.text().trim();
    let route_allows_new =
        consumer_for_card(card).is_some_and(LocationConsumer::allows_typed_new_folder);
    if allow_new && route_allows_new && !query.is_empty() {
        return Some(join_browse(card.browse_dir.as_deref(), query));
    }
    Some(card.browse_dir.clone().unwrap_or_default())
}

/// The macOS File-menu panel route, resolved from its catalog action.  This is
/// intentionally pure so the roster law can sweep it on Linux and wasm too.
#[cfg(any(target_os = "macos", test))]
pub(crate) fn native_consumer_for_action(
    action: &crate::keymap::Action,
) -> Option<LocationConsumer> {
    use crate::keymap::Action;
    match action {
        Action::OpenBrowse => Some(LocationConsumer::NativeOpen),
        Action::SaveCopy => Some(LocationConsumer::NativeSaveCopy),
        Action::ExportPdf => Some(LocationConsumer::NativeExportPdf),
        Action::ExportWord => Some(LocationConsumer::NativeExportDocx),
        Action::ExportHtml => Some(LocationConsumer::NativeExportHtml),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Roster-derived consumer × verb law.  The no-wildcard matches make a new
    /// route or a changed native panel fail to compile here instead of silently
    /// joining the product without an explicit commitment verb.
    #[test]
    fn every_location_consumer_declares_its_commit_verb_and_navigation_contract() {
        for consumer in LocationConsumer::ALL {
            match consumer {
                LocationConsumer::Browse
                | LocationConsumer::MoveDestination
                | LocationConsumer::SaveCopyDestination
                | LocationConsumer::ExportDestination
                | LocationConsumer::ProjectSwitch
                | LocationConsumer::SettingPath
                | LocationConsumer::ProjectBrowse
                | LocationConsumer::NativeOpen
                | LocationConsumer::NativeSaveCopy
                | LocationConsumer::NativeExportDocx
                | LocationConsumer::NativeExportHtml
                | LocationConsumer::NativeExportPdf => {}
            }
            match consumer.commit_verb() {
                CommitVerb::OpenFile
                | CommitVerb::MoveHere
                | CommitVerb::SaveCopyHere
                | CommitVerb::ExportHere
                | CommitVerb::SwitchHere
                | CommitVerb::SetFolder => {}
            }
            let expected_scope = match consumer {
                LocationConsumer::Browse
                | LocationConsumer::MoveDestination
                | LocationConsumer::SaveCopyDestination
                | LocationConsumer::ExportDestination => LocationScope::ActiveRoot,
                LocationConsumer::ProjectSwitch
                | LocationConsumer::SettingPath
                | LocationConsumer::ProjectBrowse => LocationScope::Workspace,
                LocationConsumer::NativeOpen
                | LocationConsumer::NativeSaveCopy
                | LocationConsumer::NativeExportDocx
                | LocationConsumer::NativeExportHtml
                | LocationConsumer::NativeExportPdf => LocationScope::System,
            };
            assert_eq!(consumer.scope(), expected_scope, "{consumer:?}: scope");
        }
        let project =
            super::super::OverlayState::new_project("/workspace".to_string(), Vec::new(), &[]);
        assert_eq!(
            consumer_for_route(&project, None),
            Some(LocationConsumer::ProjectSwitch)
        );
        assert_eq!(
            consumer_for_route(
                &project,
                Some(&super::super::Bind::Path {
                    key: "default_folder".to_string(),
                })
            ),
            Some(LocationConsumer::SettingPath)
        );
        assert!(!LocationConsumer::ProjectSwitch.supports_back());
        assert!(LocationConsumer::SettingPath.supports_back());
        assert!(LocationConsumer::MoveDestination.shows_relative_breadcrumb());
        assert!(!LocationConsumer::ProjectBrowse.shows_relative_breadcrumb());
    }

    /// Save Copy and Export deliberately share the typed-folder path: both
    /// callers pass `allow_new = true` to the same destination resolver.  The
    /// other navigators use their query as a filter or their own named-folder
    /// row, never as permission to invent a destination.
    #[test]
    fn only_save_copy_and_export_admit_a_typed_new_destination_folder() {
        let admitting: Vec<LocationConsumer> = LocationConsumer::ALL
            .into_iter()
            .filter(|consumer| consumer.allows_typed_new_folder())
            .collect();
        assert_eq!(
            admitting,
            vec![
                LocationConsumer::SaveCopyDestination,
                LocationConsumer::ExportDestination,
            ],
            "the typed-folder admission roster changed; update the operation and byte-safety laws"
        );
        for consumer in [
            LocationConsumer::Browse,
            LocationConsumer::MoveDestination,
            LocationConsumer::ProjectSwitch,
            LocationConsumer::SettingPath,
            LocationConsumer::ProjectBrowse,
        ] {
            assert!(
                !consumer.allows_typed_new_folder(),
                "{consumer:?}: search/navigation must not become a create-and-commit path"
            );
        }
    }

    #[test]
    fn every_native_location_panel_has_the_rostered_action_and_verb() {
        for consumer in LocationConsumer::ALL {
            let Some(id) = consumer.native_menu_id() else {
                continue;
            };
            let action = crate::menu::resolve(id)
                .unwrap_or_else(|| panic!("{consumer:?}: {id:?} does not resolve"));
            assert!(
                crate::menu::opens_native_panel(id),
                "{consumer:?}: {id:?} must stay a native location panel"
            );
            assert_eq!(
                native_consumer_for_action(&action),
                Some(consumer),
                "{consumer:?}: native action must route through the shared roster"
            );
            assert!(
                matches!(
                    consumer.commit_verb(),
                    CommitVerb::OpenFile | CommitVerb::SaveCopyHere | CommitVerb::ExportHere
                ),
                "{consumer:?}: a native file panel must promise a file operation"
            );
        }
    }
}
