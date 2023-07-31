use crate::infrastructure::errors::InfrastructureError;
use crate::infrastructure::export_manager::TexlaExportManager;
use crate::infrastructure::storage_manager::DirectoryChangeHandler;
use crate::infrastructure::vcs_manager::MergeConflictHandler;

pub struct TexlaCore {
    pub export_manager: TexlaExportManager,
    // only needed for offline version
    // not clean (maybe pass main_file over frontend)
    pub main_file: String, // TODO use Path instead of String
}
