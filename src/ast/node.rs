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
    Document { preamble: String, postamble: String },
    Segment { heading: String },
    File { path: String },
    Environment { name: String },
}

enum LeafData {
    Text { text: String },
    Math { kind: MathKind, content: String },
    Image { path: String },
    Label { label: String },
    Caption { caption: String },
}

enum MathKind {
    SquareBrackets,
    DoubleDollars,
    Displaymath,
    Equation,
}
