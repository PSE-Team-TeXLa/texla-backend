use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

// yes, this is all necessary
// TODO: more granular errors (do we really need them here?)

#[derive(Debug)]
pub struct AstError {}

impl Error for AstError {}

impl Display for AstError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "There was an error with the Ast. (Parsing, Operation, Stringification)")
    }
}
