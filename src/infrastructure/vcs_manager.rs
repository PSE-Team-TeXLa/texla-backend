use crate::infrastructure::errors::{MergeConflictError, VcsError};

pub trait VcsManager {
    fn pull(&self) -> Result<(), VcsError>;
    fn push(&self) -> Result<(), VcsError>;
    fn commit(&self, message: String) -> Result<(), VcsError>;
}

pub struct GitManager;

impl VcsManager for GitManager {
    fn pull(&self) -> Result<(), VcsError> {
        todo!()
    }

    fn push(&self) -> Result<(), VcsError> {
        todo!()
    }

    fn commit(&self, message: String) -> Result<(), VcsError> {
        todo!()
    }
}

pub trait MergeConflictHandler {
    fn handle_merge_conflict(&self, error: MergeConflictError);
}
