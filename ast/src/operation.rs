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
    use crate::node::{NodeRef, NodeType};
    use crate::texla_ast::TexlaAst;
    use crate::uuid_provider::Uuid;
    use crate::Ast;

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

    pub(in crate::operation) fn find_uuid_by_content(
        ast: &TexlaAst,
        content: &str,
    ) -> Option<Uuid> {
        find_uuid_by_content_recursive(&ast.root, content)
    }

    pub(in crate::operation) fn find_uuid_by_content_recursive(
        node_ref: &NodeRef,
        content: &str,
    ) -> Option<Uuid> {
        let node = node_ref.lock().unwrap();
        let current_raw_latex = &node.raw_latex.to_string();

        // Check if the raw_latex of the current node matches the content
        if current_raw_latex.contains(content) {
            return Some(node.uuid);
        }

        // If not, continue the traversal based on the node type
        match &node.node_type {
            NodeType::Expandable { children, .. } => {
                for child_ref in children {
                    if let Some(uuid) = find_uuid_by_content_recursive(child_ref, content) {
                        return Some(uuid);
                    }
                }
            }
            NodeType::Leaf { .. } => {
                // For Leaf nodes, we've already checked the raw_latex above.
                // So, there's no need for additional checks here.
            }
        }
        None
    }

    pub(in crate::operation) fn get_node_and_count_children(
        ast: &TexlaAst,
        content: &str,
    ) -> usize {
        let node_uuid = find_uuid_by_content(ast, content).expect("Failed to find");
        let node_ref = ast.get_node(node_uuid);
        count_children_of_node(&node_ref)
    }

    pub(in crate::operation) fn count_children_of_node(node_ref: &NodeRef) -> usize {
        match &node_ref.lock().unwrap().node_type {
            NodeType::Expandable { children, .. } => children.len(),
            _ => 0, // Return 0 for non-Expandable nodes
        }
    }
}
