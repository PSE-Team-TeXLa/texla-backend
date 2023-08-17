use std::path::{PathBuf, MAIN_SEPARATOR_STR};

const PATH_SEPARATORS: [char; 2] = ['/', '\\'];

pub struct FilePath {
    pub path: PathBuf,
    pub directory: PathBuf,
    pub filename: String,
}

impl From<&str> for FilePath {
    fn from(value: &str) -> Self {
        // replace separators in path with system-dependent variant and create PathBuf
        let path = PathBuf::from(value.replace(PATH_SEPARATORS, MAIN_SEPARATOR_STR));

        return FilePath {
            path: path.clone(),
            directory: path
                .parent()
                .expect("Could not find parent directory")
                .to_path_buf(),
            filename: path
                .file_name()
                .expect("Could not extract file name")
                .to_str()
                .unwrap()
                .to_string(),
        };
    }
}

impl From<String> for FilePath {
    fn from(value: String) -> Self {
        FilePath::from(value.as_str())
    }
}

impl Clone for FilePath {
    fn clone(&self) -> Self {
        Self {
            path: self.path.clone(),
            directory: self.directory.clone(),
            filename: self.filename.clone(),
        }
    }
}
