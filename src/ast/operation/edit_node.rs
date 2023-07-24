use std::sync::{Arc, Mutex};

use serde::Deserialize;

use crate::ast::errors::OperationError;
use crate::ast::meta_data::MetaData;
use crate::ast::node::{ExpandableData, Node, NodeType};
use crate::ast::operation::Operation;
use crate::ast::texla_ast::TexlaAst;
use crate::ast::uuid_provider::Uuid;
use crate::ast::Ast;

#[derive(Deserialize, Debug)]
pub struct EditNode {
    pub target: Uuid,
    pub raw_latex: String,
}

impl Operation<TexlaAst> for EditNode {
    // TODO: some of the results, that are expected or unwrapped should be propagated using ?
    fn execute_on(&self, ast: &mut TexlaAst) -> Result<(), OperationError> {
        // get information of current node
        let node_ref = ast.get_node(self.target);
        let node = node_ref.lock().unwrap();
        let node_meta_data_map = &node.meta_data.data;
        let node_parent = &node.parent;

        // create new node
        let new_node_ref = Arc::new(Mutex::new(Node {
            uuid: self.target,
            node_type: NodeType::Expandable {
                data: ExpandableData::Dummy {
                    text: self.raw_latex.clone(),
                },
                children: match &node.node_type {
                    NodeType::Expandable { children, .. } => children.clone(), //copies children from old node
                    NodeType::Leaf { .. } => {
                        vec![]
                    }
                },
            },
            meta_data: MetaData {
                data: node_meta_data_map.clone(),
            },
            parent: node_parent.clone(),
            raw_latex: String::new(), //shouldn't matter since it gets re-parsed instantly
        }));

        // update child of parent
        if let Some(parent_ref_weak) = node_parent.as_ref() {
            // FIXME: this happens sometimes (maybe fixed by now, if not fix in other operations as well)
            let parent_ref = parent_ref_weak.upgrade().unwrap();
            let mut parent = parent_ref.lock().unwrap();

            let parent_children = match &mut parent.node_type {
                NodeType::Expandable { children, .. } => children,
                NodeType::Leaf { .. } => panic!("parent is a leaf"),
            };

            let child_index = parent_children
                .iter()
                .position(|child_ref| Arc::ptr_eq(child_ref, &node_ref))
                .expect("target is not child of parent");

            parent_children[child_index] = new_node_ref.clone();
        } else {
            // if parent is None, then this node is the root node
            ast.root = new_node_ref.clone();
        }

        // update node reference in portal
        ast.portal
            .insert(self.target, Arc::downgrade(&new_node_ref));

        Ok(())
    }
}
