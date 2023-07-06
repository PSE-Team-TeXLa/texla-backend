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

impl Node {
    pub fn new() -> Self {
        Node {
            uuid: 0,
            node_type: NodeType::Leaf {
                data: LeafData::Text {
                    text: "".to_string(),
                },
            },
            meta_data: MetaData {
                meta_data: Default::default(),
            },
            parent: None,
        }
    }
    pub fn new_text(text: String) -> Self {
        Node {
            uuid: 0,
            node_type: NodeType::Leaf {
                data: LeafData::Text { text: text },
            },
            meta_data: MetaData {
                meta_data: Default::default(),
            },
            parent: None,
        }
    }

    pub fn new_image(path: String) -> Self {
        Node {
            uuid: 0,
            node_type: NodeType::Leaf {
                data: LeafData::Image { path },
            },
            meta_data: MetaData {
                meta_data: Default::default(),
            },
            parent: None,
        }
    }
    pub fn new_segment(heading: String, children: Vec<NodeRef>) -> Self {
        Node {
            uuid: 0,
            node_type: NodeType::Expandable {
                data: ExpandableData::Segment { heading },
                children,
            },
            meta_data: MetaData {
                meta_data: Default::default(),
            },
            parent: None,
        }
    }
    pub fn new_document(preamble: String, postamble: String, children: Vec<NodeRef>) -> Self {
        Node {
            uuid: 0,
            node_type: NodeType::Expandable {
                data: ExpandableData::Document {
                    preamble,
                    postamble,
                },
                children,
            },
            meta_data: MetaData {
                meta_data: Default::default(),
            },
            parent: None,
        }
    }
}
