use crate::ast::errors::AstError;
use crate::ast::operation::Operation;
use crate::ast::texla_ast::TexlaAst;
use crate::ast::uuid_provider::Uuid;

pub struct DeleteNode {
    pub target: Uuid,
}

impl Operation<TexlaAst> for DeleteNode {
    fn execute_on(&self, ast: TexlaAst) -> Result<(), AstError> {
        todo!()
    }
}
