use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

use serde::{Serialize, Serializer};

// TODO implement and use same errors as in spec?

#[derive(Debug)]
pub struct AstError {}

impl Error for AstError {}

impl Display for AstError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "There was an error with the Ast. (Parsing, Operation, Stringification)"
        )
    }
}

impl Serialize for AstError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        todo!("Implement Into for AstError and make TexlaError serializable")
    }
}
