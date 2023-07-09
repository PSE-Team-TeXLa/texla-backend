use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

use serde::{Serialize, Serializer};

#[derive(Debug, Serialize)]
pub struct TexlaError {}

impl Error for TexlaError {}

impl Display for TexlaError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "There was an error with the Ast. (Parsing, Operation, Stringification)"
        )
    }
}
