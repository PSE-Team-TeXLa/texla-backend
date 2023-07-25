use serde::Deserialize;

use crate::ast::Ast;
use crate::ast::errors::OperationError;
use crate::ast::operation::Operation;
use crate::ast::texla_ast::TexlaAst;
use crate::ast::uuid_provider::Uuid;

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
