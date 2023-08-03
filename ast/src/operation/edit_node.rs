use std::sync::{Arc, Mutex};

use serde::Deserialize;

use crate::errors::OperationError;
use crate::meta_data::MetaData;
use crate::node::{ExpandableData, Node, NodeType};
use crate::operation::Operation;
use crate::texla_ast::TexlaAst;
use crate::uuid_provider::Uuid;
use crate::Ast;

#[derive(Deserialize, Debug)]
pub struct EditNode {
    pub target: Uuid,
    pub raw_latex: String,
}

impl Operation<TexlaAst> for EditNode {
    fn execute_on(&self, ast: &mut TexlaAst) -> Result<(), OperationError> {
        let node_ref = ast.get_node(self.target);

        // create new node from old node
        let new_node_ref = {
            let node = node_ref.lock().unwrap();
            let node_meta_data_map = &node.meta_data.data;
            let node_parent = &node.parent;

            let mut parts = self.raw_latex.split("...");
            let before_children = parts.next().unwrap_or("").to_string();
            let after_children = parts.next().unwrap_or("").to_string();

            Arc::new(Mutex::new(Node {
                uuid: self.target,
                node_type: NodeType::Expandable {
                    data: ExpandableData::Dummy {
                        before_children,
                        after_children,
                    },
                    children: match &node.node_type {
                        NodeType::Expandable { children, .. } => children.clone(), // copies children from old node
                        NodeType::Leaf { .. } => {
                            vec![]
                        }
                    },
                },
                meta_data: MetaData {
                    data: node_meta_data_map.clone(),
                },
                parent: node_parent.clone(),
                raw_latex: String::new(), // shouldn't matter since it gets re-parsed instantly
            }))
        };

        if node_ref.lock().unwrap().parent.as_ref().is_some() {
            // update node in ast
            let position = ast.remove_node(&node_ref);
            ast.insert_node_at_position(new_node_ref, position);
        } else {
            // if parent is None, then this node is the root node
            ast.root = new_node_ref;
        }

        Ok(())
    }
}
