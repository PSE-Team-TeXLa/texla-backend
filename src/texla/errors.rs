use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

use serde::Serialize;

use crate::ast::errors::AstError;

#[derive(Debug, Serialize)]
pub struct TexlaError {
    message: String,
}

impl From<AstError> for TexlaError {
    fn from(value: AstError) -> Self {
        Self {
            message: value.to_string(),
        }
    }
}

impl Display for TexlaError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
