use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

// TODO after VS: implement and use same errors as in spec?

#[derive(Debug)]
pub struct InfrastructureError {}

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
