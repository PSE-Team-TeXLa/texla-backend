use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, PartialEq)]
pub struct InfrastructureError {
    message: String,
}

impl Display for InfrastructureError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Infrastructure Error: {}", self.message)
    }
}

impl From<StorageError> for InfrastructureError {
    fn from(value: StorageError) -> Self {
        Self {
            message: value.to_string(),
        }
    }
}

impl From<VcsError> for InfrastructureError {
    fn from(value: VcsError) -> Self {
        Self {
            message: value.to_string(),
        }
    }
}

impl From<PushRejectionError> for InfrastructureError {
    fn from(value: PushRejectionError) -> Self {
        Self {
            message: value.to_string(),
        }
    }
}

impl From<MergeConflictError> for InfrastructureError {
    fn from(value: MergeConflictError) -> Self {
        Self {
            message: value.to_string(),
        }
    }
}

impl From<ExportZipError> for InfrastructureError {
    fn from(value: ExportZipError) -> Self {
        Self {
            message: value.to_string(),
        }
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

#[derive(Debug, PartialEq)]
pub struct StorageError {
    pub message: String,
}

impl Display for StorageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Could not perform storage operation: {}", self.message)
    }
}

#[derive(Debug, PartialEq)]
pub struct VcsError {
    pub message: String,
}

impl Display for VcsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Could not perform VCS operation: {}", self.message)
    }
}

impl From<PushRejectionError> for VcsError {
    fn from(value: PushRejectionError) -> Self {
        Self {
            message: value.to_string(),
        }
    }
}

impl From<MergeConflictError> for VcsError {
    fn from(value: MergeConflictError) -> Self {
        Self {
            message: value.to_string(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct PushRejectionError;

impl Display for PushRejectionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Could not push changes: remote changes need to be pulled first"
        )
    }
}

#[derive(Debug, PartialEq)]
pub struct MergeConflictError {}

impl Display for MergeConflictError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Could not merge changes: conflicts need to be solved manually"
        )
    }
}

#[derive(Debug, PartialEq)]
pub struct ExportZipError {
    pub message: String,
}

impl Display for ExportZipError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Could not export ZIP file: {}", self.message)
    }
}
