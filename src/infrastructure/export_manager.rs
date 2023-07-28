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

        let file = File::create("latex.zip").unwrap();
        let option = FileOptions::default()
            .compression_method(Deflated) // default zip method.
            .unix_permissions(0o755); //shouldn't cause any errors in windows, should work on linux and mac.
        let mut zip = zip::ZipWriter::new(file);

        let walkdir = walkdir::WalkDir::new(main_file_directory).into_iter();
        walkdir
            .filter_map(|e| e.ok()) // filtering (ignoring) invalid files errors
            .filter(|e| {
                let name = e.file_name().to_string_lossy();
                !name.starts_with('.') && !name.ends_with('~') // filtering out hidden files or backup files. Good practice apparently.
            })
            .for_each(|entry| {
                let path = entry.path();
                let name = path.strip_prefix(path).unwrap(); //results in empty string, should strip base directory prefix

                let mut f = File::open(path).unwrap();
                zip.start_file(name.to_str().unwrap(), option).unwrap();

                let mut buffer = Vec::new(); // could be problematic if files are too big.
                f.read_to_end(&mut buffer).unwrap();
                zip.write_all(&buffer).unwrap();
            });

        // TODO
        Ok("http://127.0.0.1:3002/src/lib/assets/logo/logo.svg".to_string())
    }
}
