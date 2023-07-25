use serde::Deserialize;

use crate::ast::errors::OperationError;
use crate::ast::operation::{Operation, Position};
use crate::ast::texla_ast::TexlaAst;
use crate::ast::uuid_provider::Uuid;
use crate::ast::Ast;

#[derive(Deserialize, Debug)]
pub struct MoveNode {
    pub target: Uuid,
    pub destination: Position,
}

impl Operation<TexlaAst> for MoveNode {
    fn execute_on(&self, ast: &mut TexlaAst) -> Result<(), OperationError> {
        let node_ref = ast.get_node(self.target);
        ast.remove_node(&node_ref);
        ast.insert_node_at_position(node_ref.clone(), self.destination);
        Ok(())
    }
}
