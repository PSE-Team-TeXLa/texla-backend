use crate::infrastructure::errors::InfrastructureError;

pub trait ExportManager {
    fn zip_files(&self) -> Result<String, InfrastructureError>;
}

pub struct TexlaExportManager;

impl ExportManager for TexlaExportManager {
    fn zip_files(&self) -> Result<String, InfrastructureError> {
        todo!()
    }
}
