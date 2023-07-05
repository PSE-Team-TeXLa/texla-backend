use crate::ast::Ast;
use crate::ast::uuid_provider::Uuid;

mod move_node;

// TODO: derive Deserialize here, serde_traitobject needed for that
pub trait Operation<A> where A: Ast {
    fn execute_on(&self, ast: A);
}

// ? move into uuid_provider?
pub struct Postion {
    pub parent: Uuid,
    pub after_sibling: Option<Uuid>,
}
