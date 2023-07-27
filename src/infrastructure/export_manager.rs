use crate::infrastructure::errors::InfrastructureError;

use std::io::{Read, Write};
use zip::result::ZipError;
use zip::write::FileOptions;

use std::fs::File;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};
use zip::CompressionMethod::Deflated;

pub trait ExportManager {
    fn zip_files(&mut self) -> Result<String, InfrastructureError>;
}

pub struct TexlaExportManager;

impl ExportManager for TexlaExportManager {
    fn zip_files(&mut self) -> Result<String, InfrastructureError> {
        let path = "backend";
        let file = File::create("latex.zip").unwrap();
        let option = FileOptions::default()
            .compression_method(Deflated)
            .unix_permissions(0o755);
        let mut zip = zip::ZipWriter::new(file);

        let walkdir = walkdir::WalkDir::new(path).into_iter();
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
