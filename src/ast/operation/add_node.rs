use serde::Deserialize;

use crate::ast::errors::OperationError;
use crate::ast::operation::{Operation, Position};
use crate::ast::texla_ast::TexlaAst;

#[derive(Deserialize, Debug)]
pub struct AddNode {
    pub destination: Position,
    pub raw_latex: String,
}

impl Operation<TexlaAst> for AddNode {
    fn execute_on(&self, ast: &mut TexlaAst) -> Result<(), OperationError> {
        todo!()
    }
}
