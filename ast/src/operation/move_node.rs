use serde::Deserialize;

use crate::errors::OperationError;
use crate::operation::{Operation, Position};
use crate::texla_ast::TexlaAst;
use crate::uuid_provider::Uuid;
use crate::Ast;

/// Move an existing Node to a new [Position].
/// The Node is specified by its `target` Uuid.
/// This Struct is a Strategy. It can be created explicitly and should be used on an Ast via the `execute_on()` method.
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
