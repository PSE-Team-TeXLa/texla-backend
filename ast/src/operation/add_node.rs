use std::sync::{Arc, Mutex};

use serde::Deserialize;

use crate::errors::OperationError;
use crate::meta_data::MetaData;
use crate::node::{ExpandableData, Node, NodeType};
use crate::operation::{Operation, Position};
use crate::texla_ast::TexlaAst;
use crate::uuid_provider::UuidProvider;
use crate::Ast;

#[derive(Deserialize, Debug)]
pub struct AddNode {
    pub destination: Position,
    pub raw_latex: String,
}

impl Operation<TexlaAst> for AddNode {
    fn execute_on(&self, ast: &mut TexlaAst) -> Result<(), OperationError> {
        // create new node
        // TODO: maybe outsource node creation later
        let uuid = ast.uuid_provider.new_uuid();
        let new_node_ref = Arc::new(Mutex::new(Node {
            uuid,
            node_type: NodeType::Expandable {
                data: ExpandableData::Dummy {
                    before_children: self.raw_latex.clone(),
                    after_children: "".to_string(),
                },
                children: vec![],
            },
            meta_data: MetaData::new(),
            parent: Some(Arc::downgrade(&ast.get_node(self.destination.parent))),
            raw_latex: String::new(), // shouldn't matter since it gets re-parsed instantly
        }));

        // insert into ast
        ast.insert_node_at_position(new_node_ref, self.destination);

        Ok(())
    }
}
