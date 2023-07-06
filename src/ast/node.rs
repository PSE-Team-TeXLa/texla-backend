// TODO: derive Serialize and decide on JSON scheme

use std::cell::RefCell;
use std::rc::{Rc, Weak};

use crate::ast::meta_data::MetaData;
use crate::ast::uuid_provider::Uuid;

pub type NodeRef = Rc<RefCell<Node>>;
pub type NodeRefWeak = Weak<RefCell<Node>>;

pub struct Node {
    pub(crate) uuid: Uuid,
    pub(crate) node_type: NodeType,
    pub(crate) meta_data: MetaData,
    pub(crate) parent: Option<NodeRefWeak>,
}

pub enum NodeType {
    Expandable {
        data: ExpandableData,
        children: Vec<NodeRef>,
    },
    Leaf {
        data: LeafData,
    },
}

pub enum ExpandableData {
    Segment { heading: String },
    Document { preamble: String, postamble: String },
}

pub enum LeafData {
    Text { text: String },
    Image { path: String },
}
