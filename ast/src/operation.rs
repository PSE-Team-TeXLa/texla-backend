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
/// Structs that implement this Trait can modify an [Ast] in some way.
/// This specifies the Operation Interface in the Strategy pattern.
pub trait Operation<A>: Send + Sync + Debug
where
    A: Ast,
{
    /// Execute this Operation on some [Ast].
    fn execute_on(&self, ast: &mut A) -> Result<(), OperationError>;
}

/// Enum to represent the different Operations.
/// This Representation is used since rust currently doesn't support serialization of trait objects directly.
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
    /// This maps the `JsonOperation` to the equivalent Trait Object. The return type is guaranteed to be an [Operation].
    /// This conversion is used since rust currently doesn't support serialization of trait objects directly.
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
/// Represents a Position in an [Ast]. The Position points between to nodes or behind a node in order to allow specifying positions which are not currently occupied.
/// As a result this can not be used to specify the position of a node that is already in the Ast.
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
        let _operation = operation.to_trait_obj();
        // if this test runs, the deserialization worked
    }
}
