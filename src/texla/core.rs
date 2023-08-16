use crate::infrastructure::export_manager::TexlaExportManager;

pub struct TexlaCore {
    pub export_manager: TexlaExportManager,
    // only needed for offline version
    // not clean (maybe pass main_file over frontend)
    pub main_file: String,
    // TODO use tuple (directory: PathBuf, filename: PathBuf) instead of String for main_file
    pub(crate) pull_interval: u64,
    pub(crate) worksession_interval: u64,
}
