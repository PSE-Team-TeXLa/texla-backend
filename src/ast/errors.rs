use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

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
