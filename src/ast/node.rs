// TODO: derive Serialize and decide on JSON scheme

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

use crate::ast::meta_data::MetaData;
use crate::ast::uuid_provider::{Uuid, UuidProvider};

pub type NodeRef = Rc<RefCell<Node>>;
pub type NodeRefWeak = Weak<RefCell<Node>>;

#[derive(Debug)]
pub struct Node {
    pub(crate) uuid: Uuid,
    pub(crate) node_type: NodeType,
    pub(crate) meta_data: MetaData,
    pub(crate) parent: Option<NodeRefWeak>,
}

#[derive(Debug)]
pub enum NodeType {
    Expandable {
        data: ExpandableData,
        children: Vec<NodeRef>,
    },
    Leaf {
        data: LeafData,
    },
}

#[derive(Debug)]
pub enum ExpandableData {
    Segment { heading: String },
    Document { preamble: String, postamble: String },
}

#[derive(Debug)]
pub enum LeafData {
    Text { text: String },
    Image { path: String },
}

impl Node {
    pub fn new_leaf(
        data: LeafData,
        uuid_provider: &mut impl UuidProvider,
        portal: &mut HashMap<Uuid, NodeRefWeak>,
    ) -> NodeRef {
        let uuid = uuid_provider.new_uuid();
        let this = Rc::new(RefCell::new(Node {
            uuid,
            node_type: NodeType::Leaf { data },
            meta_data: MetaData {
                meta_data: Default::default(),
            },
            parent: None,
        }));
        portal.insert(uuid, Rc::downgrade(&this));
        this
    }

    pub fn new_expandable(
        data: ExpandableData,
        children: Vec<NodeRef>,
        uuid_provider: &mut impl UuidProvider,
        portal: &mut HashMap<Uuid, NodeRefWeak>,
    ) -> NodeRef {
        let uuid = uuid_provider.new_uuid();
        let this = Rc::new(RefCell::new(Node {
            uuid,
            node_type: NodeType::Expandable {
                data,
                children: children,
            },
            meta_data: MetaData {
                meta_data: Default::default(),
            },
            parent: None,
        }));
        match &this.borrow_mut().node_type {
            NodeType::Expandable { children, .. } => {
                for child in children {
                    child.borrow_mut().parent = Some(Rc::downgrade(&this.clone()));
                }
            }
            NodeType::Leaf { .. } => {}
        }
        portal.insert(uuid, Rc::downgrade(&this));
        this
    }
}
