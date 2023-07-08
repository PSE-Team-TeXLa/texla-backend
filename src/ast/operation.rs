use crate::ast::uuid_provider::Uuid;
use crate::ast::Ast;

mod move_node;
// TODO add modules and signatures for other operations

// TODO: derive Deserialize here, serde_traitobject needed for that
pub trait Operation<A>
where
    A: Ast,
{
    fn execute_on(&self, ast: A);
}

// TODO move into uuid_provider?
pub struct Postion {
    pub parent: Uuid,
    pub after_sibling: Option<Uuid>,
}
