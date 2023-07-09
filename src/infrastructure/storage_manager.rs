use crate::infrastructure::errors::InfrastructureError;
use crate::infrastructure::vcs_manager::{GitManager, MergeConflictHandler, VcsManager};

pub trait StorageManager {
    fn end_session(&self);
    fn save(&self, latex_single_string: String) -> Result<(), InfrastructureError>;
    fn multiplex_files(&self) -> Result<String, InfrastructureError>;
    fn stop_timers(&self);
    fn remote_url(&self) -> Option<String>;
    fn start(&self);
}

pub struct TexlaStorageManager<V>
where
    V: VcsManager,
{
    vcs_manager: V,
    directory_change_handler: Option<Box<dyn DirectoryChangeHandler>>,
    merge_conflict_handler: Option<Box<dyn MergeConflictHandler>>,
    main_file: String,
}

impl<V> TexlaStorageManager<V>
where
    V: VcsManager,
{
    pub fn new(vcs_manager: V, main_file: String) -> Self {
        Self {
            vcs_manager,
            directory_change_handler: None,
            merge_conflict_handler: None,
            main_file,
        }
    }

    fn attach_handlers(
        &mut self,
        dc_handler: Box<dyn DirectoryChangeHandler>,
        mc_handler: Box<dyn MergeConflictHandler>,
    ) {
        self.directory_change_handler = Some(dc_handler);
        self.merge_conflict_handler = Some(mc_handler);
    }
}

impl StorageManager for TexlaStorageManager<GitManager> {
    fn end_session(&self) {
        todo!()
    }

    fn save(&self, latex_single_string: String) -> Result<(), InfrastructureError> {
        todo!()
    }

    fn multiplex_files(&self) -> Result<String, InfrastructureError> {
        todo!()
    }

    fn stop_timers(&self) {
        todo!()
    }

    fn remote_url(&self) -> Option<String> {
        todo!()
    }

    fn start(&self) {
        todo!()
    }
}

pub trait DirectoryChangeHandler {
    fn handle_directory_change(&self);
}
