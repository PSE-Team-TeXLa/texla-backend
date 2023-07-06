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
    pub fn new() -> NodeRef {
        Rc::new(RefCell::new(Node {
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
        }))
    }

    pub fn new_text(text: String) -> NodeRef {
        Rc::new(RefCell::new(Node {
            uuid: 0,
            node_type: NodeType::Leaf {
                data: LeafData::Text { text: text },
            },
            meta_data: MetaData {
                meta_data: Default::default(),
            },
            parent: None,
        }))
    }

    pub fn new_segment(heading: String, children: Vec<NodeRef>) -> NodeRef {
        let this = Rc::new(RefCell::new(Node {
            uuid: 0,
            node_type: NodeType::Expandable {
                data: ExpandableData::Segment { heading },
                children: children,
            },
            meta_data: MetaData {
                meta_data: Default::default(),
            },
            parent: None,
        }));
        Self::add_parent_to_children(&this);
        this
    }

    pub fn new_document(preamble: String, postamble: String, children: Vec<NodeRef>) -> NodeRef {
        let this = Rc::new(RefCell::new(Node {
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
        }));
        Self::add_parent_to_children(&this);
        this
    }

    fn add_parent_to_children(parent: &Rc<RefCell<Node>>) {
        match &parent.borrow_mut().node_type {
            NodeType::Expandable { children, .. } => {
                for child in children {
                    child.borrow_mut().parent = Some(Rc::downgrade(&parent.clone()));
                }
            }
            NodeType::Leaf { .. } => {}
        }
    }
}
