use serde::Deserialize;

use crate::ast::errors::OperationError;
use crate::ast::node::{LeafData, NodeType};
use crate::ast::operation::Operation;
use crate::ast::texla_ast::TexlaAst;
use crate::ast::uuid_provider::Uuid;
use crate::ast::Ast;

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
                text.push_str(latex.as_str());
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
