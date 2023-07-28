use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

// TODO after VS: implement and use same errors as in spec?

#[derive(Debug, PartialEq)]
pub struct InfrastructureError {
    message: String,
}

impl Error for InfrastructureError {}

impl Display for InfrastructureError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "There was an error with the Infrastructure. (Storage, Vcs, PushRejection, \
            MergeConflict, ExportZip)"
        )
    }
}

impl From<std::io::Error> for InfrastructureError {
    fn from(err: std::io::Error) -> Self {
        Self {
            message: err.to_string(),
        }
    }
}

impl From<zip::result::ZipError> for InfrastructureError {
    fn from(err: zip::result::ZipError) -> Self {
        Self {
            message: err.to_string(),
        }
    }
}
