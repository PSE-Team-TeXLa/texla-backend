use std::fmt::Debug;

use serde::Deserialize;

use crate::errors::OperationError;
use crate::texla_ast::TexlaAst;
use crate::uuid_provider::Uuid;
use crate::Ast;

pub mod add_node;
pub mod delete_metadata;
pub mod delete_node;
pub mod edit_metadata;
pub mod edit_node;
pub mod merge_nodes;
pub mod move_node;

// TODO: if struggling with lifetimes, let execute_on consume self
pub trait Operation<A>: Send + Sync + Debug
where
    A: Ast,
{
    fn execute_on(&self, ast: &mut A) -> Result<(), OperationError>;
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
#[derive(Deserialize, Debug, Clone, Copy)]
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
            "arguments": {
                "target": 42,
                "raw_latex": "this is the new latex"
            }
        }
        "#;
        let operation: super::JsonOperation = serde_json::from_str(json).unwrap();
        operation.to_trait_obj();
        // if this test runs, the deserialization worked
    }
}
