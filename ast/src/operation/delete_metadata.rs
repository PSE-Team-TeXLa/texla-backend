use serde::Deserialize;

use crate::errors::OperationError;
use crate::operation::Operation;
use crate::texla_ast::TexlaAst;
use crate::uuid_provider::Uuid;
use crate::Ast;

/// Tries to delete a key-value pair from the Metadata Hashmap of some Node.
/// The Node is specified by its `target` Uuid, the key value pair is specified by its `key`.
/// This Struct is a Strategy. It can be created explicitly and should be used on an Ast via the `execute_on()` method.
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
