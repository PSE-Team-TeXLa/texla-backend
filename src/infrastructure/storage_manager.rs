use crate::infrastructure::errors::StorageError;
use crate::infrastructure::vcs_manager::{GitManager, MergeConflictHandler, VcsManager};

pub trait StorageManager {
    fn end_session();
    // TODO method is private in spec, not possible in trait
    fn save(latex_single_string: String) -> Result<(), StorageError>;
    fn multiplex_files() -> Result<String, StorageError>;
    fn stop_timers();
    fn remote_url() -> Option<String>;
    fn start();
}

pub struct TexlaStorageManager<V>
where
    V: VcsManager,
{
    vcs_manager: V,
    directory_change_handler: Option<Box<dyn DirectoryChangeHandler>>,
    merge_conflict_handler: Option<Box<dyn MergeConflictHandler>>,
}

impl TexlaStorageManager<GitManager> {
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
    fn end_session() {
        todo!()
    }

    fn save(latex_single_string: String) -> Result<(), StorageError> {
        todo!()
    }

    fn multiplex_files() -> Result<String, StorageError> {
        todo!()
    }

    fn stop_timers() {
        todo!()
    }

    fn remote_url() -> Option<String> {
        todo!()
    }

    fn start() {
        todo!()
    }
}

pub trait DirectoryChangeHandler {
    fn handle_directory_change(&self);
}
