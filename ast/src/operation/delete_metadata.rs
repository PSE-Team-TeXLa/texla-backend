use crate::Ast;
use serde::Deserialize;

use crate::errors::OperationError;
use crate::operation::Operation;
use crate::texla_ast::TexlaAst;
use crate::uuid_provider::Uuid;

#[derive(Deserialize, Debug)]
pub struct DeleteMetadata {
    pub target: Uuid,
    pub key: String,
}

impl Operation<TexlaAst> for DeleteMetadata {
    fn execute_on(&self, ast: &mut TexlaAst) -> Result<(), OperationError> {
        let node_ref = ast.get_node(self.target);
        let mut node = node_ref.lock().unwrap();
        node.meta_data.data.remove(&self.key);

        Ok(())
    }
}
