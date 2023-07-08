use crate::ast::operation::{Operation, Position};
use crate::ast::texla_ast::TexlaAst;
use crate::ast::uuid_provider::Uuid;

pub struct MoveNode {
    target: Uuid,
    destination: Position,
}

impl Operation<TexlaAst> for MoveNode {
    fn execute_on(&self, ast: TexlaAst) {
        todo!()
    }
}
