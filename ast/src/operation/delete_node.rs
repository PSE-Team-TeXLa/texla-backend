use serde::Deserialize;

use crate::errors::OperationError;
use crate::operation::Operation;
use crate::texla_ast::TexlaAst;
use crate::uuid_provider::Uuid;
use crate::Ast;

#[derive(Deserialize, Debug)]
pub struct DeleteNode {
    pub target: Uuid,
}

impl Operation<TexlaAst> for DeleteNode {
    fn execute_on(&self, ast: &mut TexlaAst) -> Result<(), OperationError> {
        let node_ref = &ast.get_node(self.target);
        ast.remove_node(node_ref);
        Ok(())
    }
}
