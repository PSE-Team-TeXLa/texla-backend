use crate::infrastructure::export_manager::TexlaExportManager;
use crate::infrastructure::file_path::FilePath;

pub struct TexlaCore {
    pub export_manager: TexlaExportManager,
    // only needed for offline version
    // not clean (maybe pass main_file over frontend)
    pub(crate) main_file: FilePath,
    pub(crate) pull_interval: u64,
    pub(crate) worksession_interval: u64,
}
