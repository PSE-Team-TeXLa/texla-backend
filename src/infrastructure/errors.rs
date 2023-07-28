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

#[derive(Debug, PartialEq)]
pub struct PushRejectionError {
    pub message: String,
}

impl Display for PushRejectionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Could not push changes: {}", self.message)
    }
}

#[derive(Debug, PartialEq)]
pub struct MergeConflictError {
    pub message: String,
}

impl Display for MergeConflictError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Could not merge changes: {}", self.message)
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
