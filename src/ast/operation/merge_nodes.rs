use serde::Deserialize;

use crate::ast::errors::OperationError;
use crate::ast::operation::Operation;
use crate::ast::texla_ast::TexlaAst;
use crate::ast::uuid_provider::Uuid;

#[derive(Deserialize, Debug)]
pub struct MergeNodes {
    pub second_node: Uuid,
}

impl Operation<TexlaAst> for MergeNodes {
    fn execute_on(&self, ast: &mut TexlaAst) -> Result<(), OperationError> {
        todo!()
    }
}
