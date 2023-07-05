// TODO: derive Serialize and decide on JSON scheme

use std::cell::RefCell;

use crate::ast::meta_data::MetaData;
use crate::ast::uuid_provider::Uuid;

pub struct Node {
    uuid: Uuid,
    node_type: NodeType,
    meta_data: MetaData,
    // TODO: parent
}

enum NodeType {
    Expandable {
        data: ExpandableData,
        children: Vec<RefCell<Node>>,
    },
    Leaf {
        data: LeafData,
    },
}


enum ExpandableData {
    // TODO
}

enum LeafData {
    // TODO
}
