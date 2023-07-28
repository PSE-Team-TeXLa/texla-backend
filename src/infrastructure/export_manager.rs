use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use zip::write::FileOptions;
use zip::CompressionMethod::Deflated;

use crate::infrastructure::errors::InfrastructureError;

pub trait ExportManager {
    fn zip_files(&mut self) -> Result<String, InfrastructureError>;
}

pub struct TexlaExportManager {
    main_file: String,
}

impl TexlaExportManager {
    pub fn new(main_file: String) -> Self {
        Self { main_file }
    }
}

impl ExportManager for TexlaExportManager {
    fn zip_files(&mut self) -> Result<String, InfrastructureError> {
        let main_file_directory = PathBuf::from(&self.main_file)
            .parent()
            .expect("Could not find parent directory")
            .to_path_buf();

        let file = File::create(main_file_directory.join("export.zip")).unwrap();

        let option = FileOptions::default()
            .compression_method(Deflated) // default zip method.
            .unix_permissions(0o755); // shouldn't cause any errors in windows, should work on linux and mac.

        let mut zip = zip::ZipWriter::new(file);

        let walkdir = walkdir::WalkDir::new(main_file_directory.clone()).into_iter();

        //TODO make it work with ? instead of panic with .unwrap()
        for entry in walkdir {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                let name = path
                    .strip_prefix(&main_file_directory)
                    .unwrap()
                    .to_str()
                    .unwrap();

                if !name.starts_with('.') && !name.ends_with('~') {
                    let mut file = File::open(path).unwrap();
                    zip.start_file(name, option).unwrap();

                    let mut buffer = Vec::new(); // could be problematic if files are too big.
                    file.read_to_end(&mut buffer).unwrap();
                    zip.write_all(&buffer).unwrap();
                }
            }
        }

        Ok("/user-assets/export.zip".to_string())
    }
}

//TODO Add test zip_files() -> unzip -> compare with latex_text_files
//TODO Remove useless url test.
#[cfg(test)]
mod tests {
    use crate::infrastructure::export_manager::{ExportManager, TexlaExportManager};

    #[test]
    fn zip_files() {
        let main_file = "latex_test_files/latex_with_inputs.tex".to_string();
        let mut export_manager = TexlaExportManager::new(main_file);

        let url = export_manager.zip_files();
        let should_url = "http://127.0.0.1:3002/src/lib/assets/logo/latex.zip";
    }
}
