use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::string::String;

use serde::Serialize;

use crate::ast::errors::StringificationError;
use crate::ast::meta_data::MetaData;
use crate::ast::uuid_provider::{Uuid, UuidProvider};

pub type NodeRef = Rc<RefCell<Node>>;
pub type NodeRefWeak = Weak<RefCell<Node>>;

#[derive(Debug, Serialize)]
pub struct Node {
    pub(crate) uuid: Uuid,
    pub(crate) node_type: NodeType,
    #[serde(flatten)]
    pub(crate) meta_data: MetaData,
    #[serde(skip_serializing)]
    pub(crate) parent: Option<NodeRefWeak>,
}
impl Node {
    pub(crate) fn to_latex(&self, level: i32) -> Result<String, StringificationError> {
        self.node_type.to_latex(level)
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum NodeType {
    Expandable {
        data: ExpandableData,
        children: Vec<NodeRef>,
    },
    Leaf {
        data: LeafData,
    },
}

impl NodeType {
    pub fn to_latex(&self, level: i32) -> Result<String, StringificationError> {
        match self {
            NodeType::Expandable { data, children } => match data {
                ExpandableData::Segment { heading } => {
                    let keyword = match level {
                        2 => "section".to_string(),
                        3 => "subsection".to_string(),
                        other => {
                            return Err(StringificationError {
                                message: format!("Invalid Nesting Level: {}", other),
                            })
                        }
                    };
                    let children: String = children
                        .iter()
                        .map(|child_node| child_node.borrow().to_latex(level + 1))
                        .collect::<Result<String, StringificationError>>()?;
                    Ok(format!("\\{keyword}{{{heading}}}\n{children}"))
                }
                ExpandableData::Document {
                    preamble,
                    postamble,
                } => {
                    let children: String = children
                        .iter()
                        .map(|child_node| child_node.borrow().to_latex(level + 1))
                        .collect::<Result<String, StringificationError>>()?;
                    Ok(format!(
                        "{preamble}\\begin{{document}}\n{children}\\end{{document}}{postamble}"
                    ))
                }
            },
            NodeType::Leaf { data } => match data {
                LeafData::Text { text } => Ok(text.to_string()),
                LeafData::Image { path } => Ok(format!("\\includegraphics{{{}}}\n", path)),
            },
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum ExpandableData {
    Segment { heading: String },
    Document { preamble: String, postamble: String },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum LeafData {
    Text { text: String },
    Image { path: String },
}

impl LeafData {
    // This does not consume the node
    fn to_latex(&self) -> String {
        match self {
            LeafData::Text { text } => text.to_string(),
            LeafData::Image { path } => format!("\\includegraphics{{{}}}\n", path),
        }
    }
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::ast::node::{LeafData, Node};
    use crate::ast::uuid_provider::TexlaUuidProvider;

    #[test]
    fn printText() {
        let mut uuidprov = TexlaUuidProvider::new();
        let mut portal = HashMap::new();
        let node = Node::new_leaf(
            LeafData::Text {
                text: "Test".to_string(),
            },
            &mut uuidprov,
            &mut portal,
        );
        assert_eq!(node.borrow().to_latex(1), Ok("Test".to_string()));
    }
}
