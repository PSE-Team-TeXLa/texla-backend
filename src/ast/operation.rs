use crate::ast::errors::AstError;
use crate::ast::texla_ast::TexlaAst;
use crate::ast::uuid_provider::Uuid;
use crate::ast::Ast;

mod move_node;

pub trait Operation<A>
where
    A: Ast,
{
    fn execute_on(&self, ast: A);
}

struct MoveOperation {
    new_string: String,
    target: Uuid,
}

impl Operation<TexlaAst> for MoveOperation {
    fn execute_on(&self, ast: TexlaAst) {
        todo!()
    }
}

#[derive(Deserialize)]
enum JsonOperation {
    MoveOperation { new_string: String, target: Uuid },
}
impl JsonOperation {
    fn to_trait_obj(self) -> impl Operation<TexlaAst> {
        match self {
            JsonOperation::MoveOperation { new_string, target } => {
                MoveOperation { new_string, target }
            }
        }
    }
}
// ? move into uuid_provider?
pub struct Postion {
    pub parent: Uuid,
    pub after_sibling: Option<Uuid>,
}
