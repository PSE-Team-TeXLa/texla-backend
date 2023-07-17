use crate::ast::errors::AstError;
use crate::ast::operation::Operation;
use crate::ast::texla_ast::TexlaAst;
use crate::ast::uuid_provider::Uuid;

pub struct DeleteMetadata {
    pub target: Uuid,
    pub key: String,
}

impl Operation<TexlaAst> for DeleteMetadata {
    fn execute_on(&self, ast: &mut TexlaAst) -> Result<(), AstError> {
        todo!()
    }
}
