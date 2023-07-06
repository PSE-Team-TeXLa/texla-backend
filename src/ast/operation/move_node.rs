use crate::ast::operation::{Operation, Postion};
use crate::ast::texla_ast::TexlaAst;
use crate::ast::uuid_provider::Uuid;

pub struct MoveNode {
    target: Uuid,
    destination: Postion,
}

impl Operation<TexlaAst> for MoveNode {
    fn execute_on(&self, ast: TexlaAst) {
        todo!()
    }
}
