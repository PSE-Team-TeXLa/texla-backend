use crate::ast::errors::AstError;
use crate::ast::operation::{Operation, Position};
use crate::ast::texla_ast::TexlaAst;

pub struct AddNode {
    destination: Position,
    raw_latex: String,
}

impl Operation<TexlaAst> for AddNode {
    fn execute_on(&self, ast: TexlaAst) -> Result<(), AstError> {
        todo!()
    }
}
