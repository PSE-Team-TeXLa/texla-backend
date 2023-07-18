use std::fmt::Debug;

use serde::Deserialize;

use crate::ast::errors::AstError;
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
pub trait Operation<A>: Send + Sync + Debug
where
    A: Ast,
{
    fn execute_on(&self, ast: &mut A) -> Result<(), AstError>;
}

#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum JsonOperation {
    EditNode {
        arguments: edit_node::EditNode,
    },
    MoveNode {
        arguments: move_node::MoveNode,
    },
    AddNode {
        arguments: add_node::AddNode,
    },
    DeleteNode {
        arguments: delete_node::DeleteNode,
    },
    EditMetadata {
        arguments: edit_metadata::EditMetadata,
    },
    DeleteMetadata {
        arguments: delete_metadata::DeleteMetadata,
    },
    MergeNodes {
        arguments: merge_nodes::MergeNodes,
    },
}

// we do this, just because serde_traitobject requires nightly
impl JsonOperation {
    pub fn to_trait_obj(self) -> Box<dyn Operation<TexlaAst>> {
        match self {
            JsonOperation::EditNode {
                arguments: operation,
            } => Box::new(operation),
            JsonOperation::MoveNode {
                arguments: operation,
            } => Box::new(operation),
            JsonOperation::AddNode {
                arguments: operation,
            } => Box::new(operation),
            JsonOperation::DeleteNode {
                arguments: operation,
            } => Box::new(operation),
            JsonOperation::EditMetadata {
                arguments: operation,
            } => Box::new(operation),
            JsonOperation::DeleteMetadata {
                arguments: operation,
            } => Box::new(operation),
            JsonOperation::MergeNodes {
                arguments: operation,
            } => Box::new(operation),
        }
    }
}

// TODO move into uuid_provider?
#[derive(Deserialize, Debug)]
pub struct Position {
    pub parent: Uuid,
    pub after_sibling: Option<Uuid>,
}

#[cfg(test)]
mod test {
    #[test]
    fn from_json() {
        let json = r#"
        {
            "type": "EditNode",
            "operation": {
                "target": 42,
                "raw_latex": "this is the new latex"
            }
        }
        "#;
        let operation: super::JsonOperation = serde_json::from_str(json).unwrap();
        let operation = operation.to_trait_obj();
        // if this test runs, the deserialization worked
    }
}
