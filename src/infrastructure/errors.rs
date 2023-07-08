use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

// TODO implement and use same errors as in spec?

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

pub struct ExportZipError;

pub struct StorageError;

pub struct VcsError;

pub struct MergeConflictError;
