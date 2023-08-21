use std::fmt::{Debug, Display, Formatter};

use serde::Serialize;

use crate::infrastructure::errors::{InfrastructureError, VcsError};
use ast::errors::AstError;

#[derive(Debug, PartialEq, Serialize)]
pub struct TexlaError {
    pub(crate) message: String,
}

impl Display for TexlaError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<AstError> for TexlaError {
    fn from(value: AstError) -> Self {
        Self {
            message: value.to_string(),
        }
    }
}

impl From<InfrastructureError> for TexlaError {
    fn from(value: InfrastructureError) -> Self {
        Self {
            message: value.to_string(),
        }
    }
}

impl From<VcsError> for TexlaError {
    fn from(value: VcsError) -> Self {
        Self {
            message: value.to_string(),
        }
    }
}
