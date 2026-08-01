use crate::app::*;

pub(in crate::app) fn initial_default_folder(cli: &Option<PathBuf>, config: &Config) -> PathBuf {
    crate::resolve_default_folder(&cli.clone().or_else(|| config.default_folder.clone()))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::app) enum TutorialFolderIntent {
    NewDocument,
    KeepTutorial,
}

impl App {
    pub(in crate::app) fn prepare_tutorial_action(&mut self, action: Action) -> Action {
        if action == Action::KeepTutorial {
            self.workspace_state
                .set_tutorial_folder_intent(TutorialFolderIntent::KeepTutorial);
        } else if self.root == crate::fs::data_root() && action == Action::NewDocument {
            self.workspace_state
                .set_tutorial_folder_intent(TutorialFolderIntent::NewDocument);
            return Action::OpenProject;
        }
        action
    }

    pub(in crate::app) fn complete_tutorial_folder_choice(&mut self) {
        match self.workspace_state.take_tutorial_folder_intent() {
            Some(TutorialFolderIntent::NewDocument) => self.new_document(),
            Some(TutorialFolderIntent::KeepTutorial) => self.manual_save(),
            None => {}
        }
    }
}
