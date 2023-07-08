use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub struct InfrastructureError {}

impl Error for InfrastructureError {}

impl Display for InfrastructureError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "There was an error with the Infrastructure. (Parsing, Operation, \
        Stringification)"
        )
    }
}

// TODO: more granular errors (do we really need them here?)
pub struct ExportZipError;

pub struct StorageError;

pub struct VcsError;

pub struct MergeConflictError;
