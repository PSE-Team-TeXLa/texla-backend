use std::sync::Arc;

use serde::Deserialize;

use crate::ast::errors::OperationError;
use crate::ast::node::{ExpandableData, NodeType};
use crate::ast::operation::{Operation, Position};
use crate::ast::texla_ast::TexlaAst;
use crate::ast::uuid_provider::Uuid;
use crate::ast::Ast;

#[derive(Deserialize, Debug)]
pub struct MoveNode {
    pub target: Uuid,
    pub destination: Position,
}

impl Operation<TexlaAst> for MoveNode {
    fn execute_on(&self, ast: &mut TexlaAst) -> Result<(), OperationError> {
        let node_ref = ast.get_node(self.target);
        let node = node_ref.lock().unwrap();
        let parent_ref_weak = &node.parent.as_ref().expect("Root cannot be moved");

        // remove from source
        {
            let parent_ref = parent_ref_weak.upgrade().unwrap();
            let mut parent = parent_ref.lock().unwrap();
            let parent_children = match &mut parent.node_type {
                NodeType::Expandable { children, .. } => children,
                NodeType::Leaf { .. } => panic!("parent is a leaf"),
            };
            let index = parent_children
                .iter()
                .position(|child_ref| Arc::ptr_eq(child_ref, &node_ref))
                .expect("target is not child of parent");
            parent_children.remove(index);
        }

        // add at destination
        {
            let parent_ref = ast.get_node(self.destination.parent);
            let mut parent = parent_ref.lock().unwrap();
            let parent_children = match &mut parent.node_type {
                NodeType::Expandable { children, .. } => children,
                NodeType::Leaf { .. } => panic!("destination parent is a leaf"),
            };
            let index = match self.destination.after_sibling {
                None => parent_children.len(),
                Some(uuid) => {
                    let sibling_ref = ast.get_node(uuid);
                    parent_children
                        .iter()
                        .position(|child_ref| Arc::ptr_eq(child_ref, &sibling_ref))
                        .expect("after_sibling not found")
                        + 1
                }
            };
            parent_children.insert(index, node_ref.clone());
        }

        // the check, whether we are going too deep, is performed to to_latex

        Ok(())
    }
}
