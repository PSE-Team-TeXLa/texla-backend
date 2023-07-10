use std::error::Error;
use std::fmt::{Debug, Display, Formatter};

use chumsky::error::Simple;

// yes, this is all necessary
// TODO: more granular errors (do we really need them here?)

#[derive(Debug)]
pub struct AstError {
    message: String,
}

impl Display for AstError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "There was an error with the Ast. (Parsing, Operation, Stringification)"
        )
    }
}
impl From<ParseError> for AstError {
    fn from(value: ParseError) -> Self {
        Self {
            message: value.to_string(),
        }
    }
}

//TODO Decide on Chumsky Error Strategy, then make this nicer (after VS)
#[derive(Debug)]
pub struct ParseError {
    message: String,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "AST Could not be parsed)")
    }
}

impl From<Vec<chumsky::error::Simple<char>>> for ParseError {
    fn from(value: Vec<Simple<char>>) -> Self {
        Self {
            message: value
                .iter()
                .map(|error| format!("{:?} {}", error.span(), error.to_string()))
                .collect(),
        }
    }
}

impl From<StringificationError> for AstError {
    fn from(value: StringificationError) -> Self {
        AstError {
            message: value.message,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct StringificationError {
    pub(crate) message: String,
}

impl Display for StringificationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Stringification Error: {}", self.message)
    }
}
