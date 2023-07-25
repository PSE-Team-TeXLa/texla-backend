use std::collections::HashMap;

use crate::ast::Ast;
use serde::Deserialize;

use crate::ast::errors::OperationError;
use crate::ast::operation::Operation;
use crate::ast::texla_ast::TexlaAst;
use crate::ast::uuid_provider::Uuid;

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
