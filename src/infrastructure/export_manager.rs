use crate::infrastructure::errors::InfrastructureError;

pub trait ExportManager {
    fn zip_files(&mut self) -> Result<String, InfrastructureError>;
}

pub struct TexlaExportManager;

impl ExportManager for TexlaExportManager {
    fn zip_files(&mut self) -> Result<String, InfrastructureError> {
        // TODO
        Ok("http://127.0.0.1:3002/src/lib/assets/logo/logo.svg".to_string())
    }
}
