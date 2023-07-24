use serde::Deserialize;

use crate::ast::errors::OperationError;
use crate::ast::errors::AstError;
use crate::ast::operation::{Operation, Position};
use crate::ast::texla_ast::TexlaAst;
use crate::ast::uuid_provider::Uuid;

#[derive(Deserialize, Debug)]
pub struct MoveNode {
    pub target: Uuid,
    pub destination: Position,
}

impl Operation<TexlaAst> for MoveNode {
    fn execute_on(&self, ast: &mut TexlaAst) -> Result<(), OperationError> {
        todo!()
    }
}
