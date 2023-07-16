use crate::ast::errors::AstError;
use crate::ast::operation::Operation;
use crate::ast::texla_ast::TexlaAst;
use crate::ast::uuid_provider::Uuid;

pub struct MergeNodes {
    pub second_node: Uuid,
}

impl Operation<TexlaAst> for MergeNodes {
    fn execute_on(&self, ast: TexlaAst) -> Result<(), AstError> {
        todo!()
    }
}
