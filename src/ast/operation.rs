use serde::Deserialize;

use crate::ast::errors::AstError;
use crate::ast::operation::edit_node::EditNode;
use crate::ast::texla_ast::TexlaAst;
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

#[derive(Deserialize)]
enum JsonOperation {
    EditNode { target: Uuid, raw_latex: String },
}
impl JsonOperation {
    fn to_trait_obj(self) -> impl Operation<TexlaAst> {
        match self {
            JsonOperation::EditNode { target, raw_latex } => EditNode { target, raw_latex },
        }
    }
}

// TODO move into uuid_provider?
pub struct Position {
    pub parent: Uuid,
    pub after_sibling: Option<Uuid>,
}
