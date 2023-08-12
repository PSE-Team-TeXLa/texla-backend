use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Output};
use std::sync::{Arc, Mutex};

use chrono::Local;

use crate::infrastructure::errors::VcsError;

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

pub trait VcsManager: Send + Sync {
    fn attach_handler(&mut self, ge_handler: Arc<Mutex<dyn GitErrorHandler>>);
    fn pull(&self);
    fn commit(&self, message: Option<String>);
    fn push(&self);
}

pub struct GitManager {
    active: bool,
    main_file_directory: PathBuf,
    remote_url: Option<String>,
    git_error_handler: Option<Arc<Mutex<dyn GitErrorHandler>>>,
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
                git_error_handler: None,
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
            git_error_handler: None,
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

    pub fn remote_url(&self) -> Option<&String> {
        self.remote_url.as_ref()
    }
}

impl VcsManager for GitManager {
    fn attach_handler(&mut self, ge_handler: Arc<Mutex<dyn GitErrorHandler>>) {
        self.git_error_handler = Some(ge_handler);
    }

    fn pull(&self) {
        if !self.active {
            return;
        }

        println!("Pulling...");
        let pull_output = self.git(Self::GIT_PULL.to_vec());
        println!("Pull over");

        if !pull_output.status.success() {
            self.git_error_handler
                .as_ref()
                .expect("No git error handler present")
                .lock()
                .unwrap()
                .handle_git_error(VcsError {
                    message: "unable to pull remote changes".to_string(),
                });
        }
    }

    fn commit(&self, custom_message: Option<String>) {
        if !self.active {
            return;
        }

        let message = {
            if let Some(_) = custom_message {
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
            self.git_error_handler
                .as_ref()
                .expect("No git error handler present")
                .lock()
                .unwrap()
                .handle_git_error(VcsError {
                    message: "unable to add local files to staging area".to_string(),
                });
        }

        let mut command = Self::GIT_COMMIT.to_vec();
        command.append(&mut vec![&message]);
        let commit_output = self.git(command);

        if !commit_output.status.success() {
            self.git_error_handler
                .as_ref()
                .expect("No git error handler present")
                .lock()
                .unwrap()
                .handle_git_error(VcsError {
                    message: "unable to commit local changes to repository".to_string(),
                });
        }
    }

    fn push(&self) {
        if !self.active {
            return;
        }

        let push_output = self.git(Self::GIT_PUSH.to_vec());

        if !push_output.status.success() {
            self.git_error_handler
                .as_ref()
                .expect("No git error handler present")
                .lock()
                .unwrap()
                .handle_git_error(VcsError {
                    message: "unable to push local changes".to_string(),
                });
        }
    }
}

pub trait GitErrorHandler: Send + Sync {
    fn handle_git_error(&self, error: VcsError);
}
