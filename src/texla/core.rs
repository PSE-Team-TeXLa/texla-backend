use crate::infrastructure::export_manager::TexlaExportManager;
use crate::infrastructure::file_path::FilePath;
use crate::texla::socket::TexlaSocket;

pub struct TexlaCore {
    pub(crate) export_manager: TexlaExportManager,
    pub(crate) pull_interval: u64,
    pub(crate) worksession_interval: u64,
    pub(crate) notify_delay: u64,
    pub(crate) vcs_enabled: bool,

    // only needed for offline version
    // (in online version the main_file would be passed from the frontend)
    pub(crate) main_file: FilePath,
    pub(crate) socket: Option<TexlaSocket>,
}
