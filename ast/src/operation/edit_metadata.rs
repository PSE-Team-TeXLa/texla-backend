use std::collections::HashMap;

use serde::Deserialize;

use crate::errors::OperationError;
use crate::operation::Operation;
use crate::texla_ast::TexlaAst;
use crate::uuid_provider::Uuid;
use crate::Ast;

/// Modify the Metadata Hashmap of some Node.
/// The Node is specified by its `target` Uuid.
/// This sets all the keys in `new` to their value in `new`. If these keys didn't exist before, the are created.
/// This Struct is a Strategy. It can be created explicitly and should be used on an Ast via the `execute_on()` method.
#[derive(Deserialize, Debug)]
pub struct EditMetadata {
    pub target: Uuid,
    pub new: HashMap<String, String>,
}

impl Operation<TexlaAst> for EditMetadata {
    fn execute_on(&self, ast: &mut TexlaAst) -> Result<(), OperationError> {
        let node_ref = ast.get_node(self.target);
        let mut node = node_ref.lock().unwrap();
        node.meta_data.data.extend(self.new.clone());

        Ok(())
    }
}
