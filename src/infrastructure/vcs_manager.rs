use crate::infrastructure::errors::InfrastructureError;

pub trait VcsManager {
    fn pull(&self) -> Result<(), InfrastructureError>;
    fn push(&self) -> Result<(), InfrastructureError>;
    fn commit(&self, message: String) -> Result<(), InfrastructureError>;
}

pub struct GitManager;

impl VcsManager for GitManager {
    fn pull(&self) -> Result<(), InfrastructureError> {
        todo!()
    }

    fn push(&self) -> Result<(), InfrastructureError> {
        todo!()
    }

    fn commit(&self, message: String) -> Result<(), InfrastructureError> {
        todo!()
    }
}

pub trait MergeConflictHandler {
    fn handle_merge_conflict(&self, error: InfrastructureError);
}
