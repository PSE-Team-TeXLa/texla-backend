use crate::ast::errors::AstError;
use crate::ast::operation::Operation;
use crate::ast::texla_ast::TexlaAst;
use crate::ast::uuid_provider::Uuid;

pub struct EditNode {
    target: Uuid,
    raw_latex: String,
}

impl Operation<TexlaAst> for EditNode {
    fn execute_on(&self, ast: TexlaAst) -> Result<(), AstError> {
        todo!()
    }
}
