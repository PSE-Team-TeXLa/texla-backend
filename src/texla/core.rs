use crate::infrastructure::errors::InfrastructureError;
use crate::infrastructure::export_manager::TexlaExportManager;
use crate::infrastructure::storage_manager::DirectoryChangeHandler;
use crate::infrastructure::vcs_manager::MergeConflictHandler;

pub struct TexlaCore {
    pub export_manager: TexlaExportManager,
    // only needed for offline versoion
    // not clean (maybe pass main_file over frontend)
    pub main_file: String,
}

impl DirectoryChangeHandler for TexlaCore {
    fn handle_directory_change(&self) {
        todo!()
    }
}

impl MergeConflictHandler for TexlaCore {
    fn handle_merge_conflict(&self, error: InfrastructureError) {
        todo!()
    }
}
