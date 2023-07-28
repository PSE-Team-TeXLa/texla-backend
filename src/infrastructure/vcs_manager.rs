use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Output};

use crate::infrastructure::errors::InfrastructureError;

struct StringOutput {
    status: ExitStatus,
    stdout: String,
    stderr: String,
}

impl From<Output> for StringOutput {
    fn from(value: Output) -> Self {
        Self {
            status: value.status,
            stdout: Self::byte_vec_to_string(value.stdout),
            stderr: Self::byte_vec_to_string(value.stderr),
        }
    }
}

impl StringOutput {
    fn byte_vec_to_string(vec: Vec<u8>) -> String {
        String::from_utf8(vec)
            .unwrap()
            .trim_end_matches('\n')
            .to_string()
    }
}

pub trait VcsManager {
    fn pull(&self) -> Result<(), InfrastructureError>;
    fn commit(&self, message: Option<String>) -> Result<(), InfrastructureError>;
    fn push(&self) -> Result<(), InfrastructureError>;
}

pub struct GitManager {
    active: bool,
    main_file_directory: PathBuf,
    remote_url: Option<String>,
}

impl GitManager {
    const GIT: &'static str = "git";

    pub fn new(main_file: String) -> Self {
        let main_file_directory = PathBuf::from(main_file)
            .parent()
            .expect("Could not find parent directory")
            .to_path_buf();

        // check if main file is inside a git repository
        let inside_work_tree = Self::git_inside_dir(
            vec!["rev-parse", "--is-inside-work-tree"],
            &main_file_directory,
        )
        .stdout;

        if inside_work_tree != "true" {
            return Self {
                active: false,
                main_file_directory,
                remote_url: None,
            };
        }

        // get remote repository url if present
        let origins = Self::git_inside_dir(vec!["remote"], &main_file_directory).stdout;
        let remote_url = if origins.is_empty() {
            None
        } else {
            let first_origin = {
                if origins.contains('\n') {
                    origins.split_once('\n').unwrap().0
                    // TODO is it okay to take the first origin when there are multiple ones?
                } else {
                    &origins
                }
            };

            Some(
                Self::git_inside_dir(
                    vec!["remote", "get-url", first_origin],
                    &main_file_directory,
                )
                .stdout,
            )
        };

        Self {
            active: true,
            main_file_directory,
            remote_url,
        }
    }

    fn git_inside_dir(args: Vec<&str>, dir: &Path) -> StringOutput {
        StringOutput::from(
            Command::new(Self::GIT)
                .current_dir(dir)
                .args(args)
                .output()
                .expect("Could not execute command"),
        )
    }

    fn git(&self, args: Vec<&str>) -> StringOutput {
        Self::git_inside_dir(args, &self.main_file_directory)
    }

    // TODO after VS: is this an acceptable getter?
    pub fn remote_url(&self) -> Option<&String> {
        self.remote_url.as_ref()
    }
}

impl VcsManager for GitManager {
    fn pull(&self) -> Result<(), InfrastructureError> {
        todo!()
    }

    fn commit(&self, message: Option<String>) -> Result<(), InfrastructureError> {
        // TODO after VS: use default commit message if no message is given
        Ok(()) // TODO!
    }

    fn push(&self) -> Result<(), InfrastructureError> {
        Ok(()) // TODO!
    }
}

pub trait MergeConflictHandler: Send + Sync {
    fn handle_merge_conflict(&self, error: InfrastructureError);
}
