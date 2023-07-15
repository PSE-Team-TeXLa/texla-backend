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
    pub fn new(repository_path: String) -> Self {
        // TODO after VS: check that the given directory is a repository and get remote url

        Self {
            repository_path,
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

        todo!()
    }

    fn push(&self) -> Result<(), InfrastructureError> {
        todo!()
    }
}

pub trait MergeConflictHandler: Send + Sync {
    fn handle_merge_conflict(&self, error: InfrastructureError);
}
