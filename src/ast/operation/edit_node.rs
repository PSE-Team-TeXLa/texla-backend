use std::sync::{Arc, Mutex};

use serde::Deserialize;

use crate::ast::errors::AstError;
use crate::ast::meta_data::MetaData;
use crate::ast::node::{LeafData, Node, NodeType};
use crate::ast::operation::Operation;
use crate::ast::texla_ast::TexlaAst;
use crate::ast::uuid_provider::Uuid;
use crate::ast::Ast;

#[derive(Deserialize)]
pub struct EditNode {
    pub target: Uuid,
    pub raw_latex: String,
}

impl Operation<TexlaAst> for EditNode {
    fn execute_on(&self, ast: &mut TexlaAst) -> Result<(), AstError> {
        // get information of current node
        let node_ref = ast.get_node(self.target);
        let node = node_ref.lock().unwrap();
        let node_meta_data_map = &node.meta_data.data;
        let node_parent = &node.parent;

        // create new node
        let new_node_ref = Arc::new(Mutex::new(Node {
            uuid: self.target,
            node_type: NodeType::Leaf {
                data: LeafData::Text {
                    text: self.raw_latex.clone(),
                },
            },
            meta_data: MetaData {
                data: node_meta_data_map.clone(),
            },
            parent: node_parent.clone(),
        }));

        // update child of parent
        let parent_ref = node_parent
            .as_ref()
            .expect("Could not find parent")
            .upgrade()
            .expect("Could not upgrade weak pointer");
        let parent_node_type = &mut parent_ref.lock().expect("Could not acquire lock").node_type;

        let parent_children;
        if let NodeType::Expandable { children, .. } = parent_node_type {
            parent_children = children;
        } else {
            panic!("Invalid parent which is no Expandable")
        }

        let child_index = parent_children
            .iter()
            .position(|child_ref| {
                child_ref.lock().expect("Could not acquire lock").uuid == self.target
            })
            .expect("Could not find child");
        parent_children[child_index] = new_node_ref.clone();

        // update node reference in portal
        ast.portal
            .insert(self.target, Arc::downgrade(&new_node_ref));

        Ok(())
    }
}
