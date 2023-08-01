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
    // TODO: pass main_file here
    fn zip_files(&mut self) -> Result<String, InfrastructureError> {
        let main_file_directory = PathBuf::from(&self.main_file)
            .parent()
            .expect("Could not find parent directory")
            .to_path_buf();

        let file = File::create(main_file_directory.join("export.zip"))?;

        let option = FileOptions::default()
            .compression_method(Deflated) // default zip method.
            .unix_permissions(0o755); // shouldn't cause any errors in windows, should work on linux and mac.

        let mut zip = zip::ZipWriter::new(file);

        let walkdir = walkdir::WalkDir::new(main_file_directory.clone()).into_iter();

        for entry in walkdir {
            let entry = entry.expect("walkdir gave error");
            let path = entry.path();
            if path.is_file() {
                let name = path
                    .strip_prefix(&main_file_directory)
                    .expect("walkdir gave file outside main_file_directory")
                    .to_str()
                    .expect("found non-utf8 file name");

                //skip hidden files
                if name.split('/').any(|part| part.starts_with('.')) {
                    continue;
                }

                if name.ends_with(".zip") {
                    continue;
                }

                if !name.starts_with('.') && !name.ends_with('~') {
                    let mut file = File::open(path)?;
                    zip.start_file(name, option)?;

                    let mut buffer = Vec::new(); // could be problematic if files are too big.
                    file.read_to_end(&mut buffer)?;
                    zip.write_all(&buffer)?;
                }
            }
        }

        Ok("/user-assets/export.zip".to_string())
    }
}

//TODO: Add test: 1. Define self-made zip in [test] -> use zip_files() -> compare.
//TODO Remove useless url test.
#[cfg(test)]
mod tests {
    use crate::infrastructure::export_manager::{ExportManager, TexlaExportManager};
    use std::fs;
    use std::io::Read;
    use zip::ZipArchive;

    //all stringify options should be off so that latex_test-files.zip in test_resources is valid.
    //large.tex configuration is needed.
    #[test]
    fn zip_files() {
        let main_file_directory = "test_resources/latex";
        let mut manager = TexlaExportManager::new("test_resources/latex/large.tex".to_string());
        let path_in_frontend_placeholder = manager.zip_files().unwrap();

        let zip_path = "test_resources/latex/export.zip";

        let mut expected_zip =
            ZipArchive::new(fs::File::open("test_resources/zip/latex_test_files.zip").unwrap())
                .unwrap();
        let mut actual_zip = ZipArchive::new(fs::File::open(zip_path).unwrap()).unwrap();

        //assert_eq!(expected_zip.len(), actual_zip.len());

        for i in 0..expected_zip.len() {
            let mut expected_file = expected_zip.by_index(i).unwrap();
            let file_name = expected_file.name();

            //            if file_name.starts_with('.')
            //                || expected_file.is_dir()
            //                || file_name.ends_with(".jpg")
            //                || file_name.ends_with(".zip")
            //            {
            //                continue; //Skip hidden files
            //            }

            // println!("Expected file: {}", file_name);

            let mut actual_file = actual_zip.by_name(expected_file.name()).unwrap();

            let actual_file_name = actual_file.name();

            // println!("Actual file: {}", actual_file.name());

            //            let mut expected_content = String::new();
            //            let mut actual_content = String::new();
            //
            //            expected_file.read_to_string(&mut expected_content).unwrap();
            //            actual_file.read_to_string(&mut actual_content).unwrap();
            //
            //            assert_eq!(expected_content, actual_content);

            assert_eq!(file_name, actual_file_name);
        }
    }
}
