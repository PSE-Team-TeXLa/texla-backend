use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Output};

use chrono::Local;

use crate::infrastructure::errors::{
    InfrastructureError, MergeConflictError, PushRejectionError, VcsError,
};

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
    fn pull(&self) -> Result<(), VcsError>;
    fn commit(&self, message: Option<String>) -> Result<(), VcsError>;
    fn push(&self) -> Result<(), VcsError>;
}

pub struct GitManager {
    active: bool,
    main_file_directory: PathBuf,
    remote_url: Option<String>,
}

impl GitManager {
    const GIT: &'static str = "git";

    const DEFAULT_COMMIT_MESSAGE_PREFIX: &'static str = "TeXLa ";
    const DEFAULT_COMMIT_MESSAGE_TIME_FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

    const GIT_IS_INSIDE_WORK_TREE: [&'static str; 2] = ["rev-parse", "--is-inside-work-tree"];
    const GIT_LIST_REMOTES: [&'static str; 1] = ["remote"];
    const GIT_GET_REMOTE_URL: [&'static str; 2] = ["remote", "get-url"];
    const GIT_PULL: [&'static str; 3] = ["pull", "--rebase", "--autostash"];
    const GIT_ADD: [&'static str; 2] = ["add", "--all"];
    const GIT_COMMIT: [&'static str; 2] = ["commit", "--message"];
    const GIT_PUSH: [&'static str; 1] = ["push"];

    pub fn new(main_file: String) -> Self {
        let main_file_directory = PathBuf::from(main_file)
            .parent()
            .expect("Could not find parent directory")
            .to_path_buf();

        // check if main file is inside a git repository
        let inside_work_tree =
            Self::git_inside_dir(Self::GIT_IS_INSIDE_WORK_TREE.to_vec(), &main_file_directory)
                .stdout;

        if inside_work_tree != "true" {
            return Self {
                active: false,
                main_file_directory,
                remote_url: None,
            };
        }

        // get remote repository url if present
        let remotes =
            Self::git_inside_dir(Self::GIT_LIST_REMOTES.to_vec(), &main_file_directory).stdout;
        let remote_url = if remotes.is_empty() {
            None
        } else {
            let first_remote = {
                if remotes.contains('\n') {
                    remotes.split_once('\n').unwrap().0
                    // TODO is it okay to take the first remote when there are multiple ones?
                } else {
                    &remotes
                }
            };

            let mut command = Self::GIT_GET_REMOTE_URL.to_vec();
            command.append(&mut vec![first_remote]);
            Some(Self::git_inside_dir(command, &main_file_directory).stdout)
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
    fn pull(&self) -> Result<(), VcsError> {
        if !self.active {
            return Ok(());
        }

        // TODO adapt return type to accept MergeConflictError and VcsError without into()?
        println!("Pulling...");
        let pull_output = self.git(Self::GIT_PULL.to_vec());
        println!("Pull over");

        if !pull_output.status.success() {
            let stderr = pull_output.stderr;
            return if stderr.starts_with("error")
                || (stderr.contains('\n')
                    && stderr.split_once('\n').unwrap().1.starts_with("CONFLICT"))
            {
                Err(MergeConflictError.into())
            } else {
                Err(VcsError {
                    message: "unable to push local changes".to_string(),
                })
            };
        }

        Ok(())
    }

    fn commit(&self, custom_message: Option<String>) -> Result<(), VcsError> {
        if !self.active {
            return Ok(());
        }

        let message = {
            if let Some(..) = custom_message {
                custom_message.unwrap()
            } else {
                format!(
                    "{}{}",
                    Self::DEFAULT_COMMIT_MESSAGE_PREFIX,
                    Local::now().format(Self::DEFAULT_COMMIT_MESSAGE_TIME_FORMAT)
                )
            }
        };

        println!("Committing...");
        let add_output = self.git(Self::GIT_ADD.to_vec());
        println!("Commit over");

        if !add_output.status.success() {
            return Err(VcsError {
                message: "unable to add local files to staging area".to_string(),
            });
        }

        let mut command = Self::GIT_COMMIT.to_vec();
        command.append(&mut vec![&message]);
        let commit_output = self.git(command);

        if !commit_output.status.success() {
            return Err(VcsError {
                message: "unable to commit local changes to repository".to_string(),
            });
        }

        Ok(())
    }

    fn push(&self) -> Result<(), VcsError> {
        if !self.active {
            return Ok(());
        }

        // TODO adapt return type to accept PushRejectionError and VcsError without into()?
        let push_output = self.git(Self::GIT_PUSH.to_vec());

        if !push_output.status.success() {
            let stderr = push_output.stderr;
            return if stderr.contains('\n')
                && stderr
                    .split_once('\n')
                    .unwrap()
                    .1
                    .starts_with(" ! [rejected]")
            {
                Err(PushRejectionError.into())
            } else {
                Err(VcsError {
                    message: "unable to push local changes, \
                    possibly because remote repository can't be reached"
                        .to_string(),
                })
            };
        }

        Ok(())
    }
}

pub trait MergeConflictHandler: Send + Sync {
    fn handle_merge_conflict(&self, error: InfrastructureError);
}
