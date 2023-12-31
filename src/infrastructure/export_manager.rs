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
    main_file_directory: PathBuf,
}

impl TexlaExportManager {
    pub fn new(main_file_directory: PathBuf) -> Self {
        Self {
            main_file_directory,
        }
    }
}

impl ExportManager for TexlaExportManager {
    fn zip_files(&mut self) -> Result<String, InfrastructureError> {
        let file = File::create(self.main_file_directory.join("export.zip"))?;

        let option = FileOptions::default()
            .compression_method(Deflated) // default zip method.
            .unix_permissions(0o755); // shouldn't cause any errors in windows, should work on linux and mac.

        let mut zip = zip::ZipWriter::new(file);

        let walkdir = walkdir::WalkDir::new(self.main_file_directory.clone()).into_iter();

        for entry in walkdir {
            let entry = entry.expect("walkdir gave error");
            let path = entry.path();
            if path.is_file() {
                let name = path
                    .strip_prefix(&self.main_file_directory)
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

// export.zip in test_resources/latex/pflichtenheft is irrelevant
#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::fs;
    use std::path::Path;

    use zip::ZipArchive;

    use crate::infrastructure::export_manager::{ExportManager, TexlaExportManager};
    use crate::infrastructure::file_path::FilePath;

    #[test]
    fn test_zip_files() {
        // prepare directory needed for testing
        let path_to_new_test_directory = "test_resources/latex/pflichtenheft_zip";
        let path_to_exported_directory = "test_resources/latex/pflichtenheft";

        if Path::new(path_to_new_test_directory).is_dir() {
            fs::remove_dir_all(path_to_new_test_directory).unwrap();
        }

        fs::create_dir(path_to_new_test_directory).unwrap();

        let created_zip_path = "test_resources/latex/pflichtenheft/export.zip";
        let copied_zip_path = "test_resources/latex/pflichtenheft_zip/export_copy.zip";
        let main_file = FilePath::from("test_resources/latex/pflichtenheft/main.tex");

        // create zip of test_resources/latex/pflichtenheft
        let mut manager = TexlaExportManager::new(main_file.directory);
        manager.zip_files().unwrap();

        // copy zip created by zip_files() function to pflichtenheft_zip directory
        fs::copy(created_zip_path, copied_zip_path).unwrap();

        // unpack and delete zip
        let mut copied_zip = ZipArchive::new(fs::File::open(copied_zip_path).unwrap()).unwrap();
        copied_zip.extract(path_to_new_test_directory).unwrap();
        fs::remove_file(copied_zip_path).unwrap();

        let original_dir = fs::read_dir(path_to_exported_directory).unwrap();
        let unzipped_dir = fs::read_dir(path_to_new_test_directory).unwrap();

        // compare file names by saving them to HashSet
        let original_files: HashSet<_> = original_dir
            .filter_map(Result::ok)
            .map(|file_or_directory| {
                file_or_directory
                    .path()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string()
            })
            .filter(|name| name != "export.zip")
            // ignore default save path for export.zip which is only used to redirect result zip to
            // frontend and should not be compared
            .collect();

        let unzipped_files: HashSet<_> = unzipped_dir
            .filter_map(Result::ok)
            .map(|file_or_directory| {
                file_or_directory
                    .path()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string()
            })
            .collect();

        assert_eq!(original_files, unzipped_files);

        // delete pflichtenheft_zip directory
        fs::remove_dir_all(path_to_new_test_directory).unwrap();
    }
}
