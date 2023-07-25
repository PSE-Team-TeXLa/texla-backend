use crate::infrastructure::errors::InfrastructureError;

pub trait VcsManager {
    fn pull(&self) -> Result<(), InfrastructureError>;
    fn commit(&self, message: Option<String>) -> Result<(), InfrastructureError>;
    fn push(&self) -> Result<(), InfrastructureError>;
}

pub struct GitManager {
    repository_path: String,
    remote_url: Option<String>,
}

impl GitManager {
    pub fn new(main_file: String) -> Self {
        // TODO: find repository_path from main_file
        // Linus: we probably want to go up the file tree until we find a .git folder
        // (or get the git repository_path root from a git command directly)
        // if there is no repository, all git operations should do nothing

        Self {
            repository_path: format!("TEMPORARY, deduce from main_file: {}", main_file),
            remote_url: None, // TODO!
        }
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
