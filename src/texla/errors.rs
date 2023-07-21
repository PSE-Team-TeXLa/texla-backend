use std::fmt::{Debug, Display, Formatter};

use serde::Serialize;

use crate::ast::errors::AstError;
use crate::infrastructure::errors::InfrastructureError;
use crate::texla::socket::TexlaSocket;

#[derive(Debug, Serialize)]
pub struct TexlaError {
    message: String,
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
