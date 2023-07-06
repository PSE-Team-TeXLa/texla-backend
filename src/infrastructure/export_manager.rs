use crate::infrastructure::errors::ExportZipError;

pub trait ExportManager {
    fn zip_files() -> Result<String, ExportZipError>;
}

pub struct TexlaExportManager;

impl ExportManager for TexlaExportManager {
    fn zip_files() -> Result<String, ExportZipError> {
        todo!()
    }
}
