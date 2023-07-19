use std::sync::{Arc, Mutex};

use serde::Deserialize;

use crate::ast::errors::AstError;
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
    fn execute_on(&self, ast: &mut TexlaAst) -> Result<(), AstError> {
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
            let parent_ref = parent_ref_weak.upgrade().unwrap(); // FIXME: this happens sometimes
            let mut parent = parent_ref.lock().unwrap();

            let parent_children = match &mut parent.node_type {
                NodeType::Expandable { children, .. } => children,
                NodeType::Leaf { .. } => panic!("Parent is a leaf"),
            };

            drop(node); // drop lock, because we need want to lock again in the next step

            let child_index = parent_children
                .iter()
                .position(|child_ref| child_ref.lock().unwrap().uuid == self.target)
                .expect("Could not find child");

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
