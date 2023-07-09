use crate::ast::errors::AstError;
use crate::ast::uuid_provider::Uuid;
use crate::ast::Ast;

mod add_node;
mod delete_metadata;
mod delete_node;
mod edit_metadata;
mod edit_node;
mod merge_nodes;
mod move_node;

// TODO: derive Deserialize here, serde_traitobject needed for that
pub trait Operation<A>: Send + Sync
where
    A: Ast,
{
    fn execute_on(&self, ast: A) -> Result<(), AstError>;
}

// TODO move into uuid_provider?
pub struct Position {
    pub parent: Uuid,
    pub after_sibling: Option<Uuid>,
}
