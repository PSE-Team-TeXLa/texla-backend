use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub struct AstError {
    message: String,
}
impl Display for AstError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "AST Error: {}", self.message)
    }
}

impl From<ParseError> for AstError {
    fn from(value: ParseError) -> Self {
        Self {
            message: value.to_string(),
        }
    }
}

impl From<StringificationError> for AstError {
    fn from(value: StringificationError) -> Self {
        AstError {
            message: value.to_string(),
        }
    }
}

impl From<OperationError> for AstError {
    fn from(value: OperationError) -> Self {
        AstError {
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
        write!(f, "Could not parse Ast: {}", self.message)
    }
}
impl From<Vec<chumsky::error::Simple<char>>> for ParseError {
    fn from(value: Vec<chumsky::error::Simple<char>>) -> Self {
        Self {
            message: value
                .iter()
                .map(|error| format!("{:?} {}", error.span(), error.to_string()))
                .collect(),
        }
    }
}

#[derive(Debug)]
pub struct StringificationError {
    pub(crate) message: String,
}
impl Display for StringificationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Could not stringify Ast: {}", self.message)
    }
}

impl From<serde_json::Error> for StringificationError {
    fn from(error: serde_json::Error) -> Self {
        Self {
            message: format!("(from serde) {error}"),
        }
    }
}

#[derive(Debug)]
pub struct OperationError {
    message: String,
}

impl Display for OperationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Could not execute operation: {}", self.message)
    }
}
