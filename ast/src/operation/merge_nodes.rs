use serde::Deserialize;

use crate::errors::OperationError;
use crate::node::{LeafData, NodeType};
use crate::operation::Operation;
use crate::texla_ast::TexlaAst;
use crate::uuid_provider::Uuid;
use crate::Ast;

/// Merge two text blocks together.
/// Only the second node is specified through its uuid.
/// The first node precedes implicitly.
/// This Struct is a Strategy. It can be created explicitly and should be used on an Ast via the `execute_on()` method.
#[derive(Deserialize, Debug)]
pub struct MergeNodes {
    pub second_node: Uuid,
}

impl Operation<TexlaAst> for MergeNodes {
    fn execute_on(&self, ast: &mut TexlaAst) -> Result<(), OperationError> {
        let second_node_ref = ast.get_node(self.second_node);
        let latex = {
            let second_node = second_node_ref.lock().unwrap();
            match &second_node.node_type {
                NodeType::Leaf {
                    data: LeafData::Text { text },
                } => text.clone(),
                _ => {
                    return Err(OperationError {
                        message: "only Text nodes can be merged".to_string(),
                    });
                }
            }
        };

        let first_uuid = ast
            .remove_node(&second_node_ref)
            .after_sibling
            .ok_or(OperationError {
                message: "no predecessor found to merge into".to_string(),
            })?;
        let first_node_ref = ast.get_node(first_uuid);
        let mut first_node = first_node_ref.lock().unwrap();

        match &mut first_node.node_type {
            NodeType::Leaf {
                data: LeafData::Text { text },
            } => {
                text.push_str(&format!("\n{}", latex.as_str()));
            }
            _ => {
                return Err(OperationError {
                    message: "only Text nodes can be merged".to_string(),
                })
            }
        }

        Ok(())
    }
}
