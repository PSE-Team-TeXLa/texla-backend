use crate::infrastructure::errors::StorageError;
use crate::infrastructure::vcs_manager::{MergeConflictHandler, VcsManager};

pub trait StorageManager {
    fn end_session();
    // TODO method is private in spec, not possible in trait
    fn save(latex_single_string: String) -> Result<(), StorageError>;
    fn multiplex_files() -> Result<String, StorageError>;
    fn stop_timers();
    fn remote_url() -> Option<String>;
    fn start(dc_handler: dyn DirectoryChangeHandler, mc_handler: dyn MergeConflictHandler);
}

pub struct TexlaStorageManager {
    vcs_manager: dyn VcsManager,
    directory_change_handler: dyn DirectoryChangeHandler,
    merge_conflict_handler: dyn MergeConflictHandler,
}

impl StorageManager for TexlaStorageManager {
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

    fn start(dc_handler: dyn DirectoryChangeHandler, mc_handler: dyn MergeConflictHandler) {
        todo!()
    }
}

pub trait DirectoryChangeHandler {
    fn handle_directory_change();
}
