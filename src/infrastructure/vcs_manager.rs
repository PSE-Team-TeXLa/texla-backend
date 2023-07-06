use crate::infrastructure::errors::{MergeConflictError, VcsError};

pub trait VcsManager {
    fn pull() -> Result<(), VcsError>;
    fn push() -> Result<(), VcsError>;
    fn commit(message: String) -> Result<(), VcsError>;
}

pub struct GitManager;

impl VcsManager for GitManager {
    fn pull() -> Result<(), VcsError> {
        todo!()
    }

    fn push() -> Result<(), VcsError> {
        todo!()
    }

    fn commit(message: String) -> Result<(), VcsError> {
        todo!()
    }
}

pub trait MergeConflictHandler {
    fn handle_merge_conflict(error: MergeConflictError);
}
