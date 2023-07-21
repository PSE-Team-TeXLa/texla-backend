use std::collections::HashMap;
use std::string::String;
use std::sync::{Arc, Mutex, Weak};

use serde::Serialize;

use crate::ast::errors::StringificationError;
use crate::ast::meta_data::MetaData;
use crate::ast::uuid_provider::{Uuid, UuidProvider};

pub type NodeRef = Arc<Mutex<Node>>;
pub type NodeRefWeak = Weak<Mutex<Node>>;

#[derive(Debug, Serialize)]
pub struct Node {
    pub(crate) uuid: Uuid,
    pub(crate) node_type: NodeType,
    #[serde(flatten)]
    pub(crate) meta_data: MetaData,
    #[serde(skip_serializing)]
    pub(crate) parent: Option<NodeRefWeak>,
    pub(crate) raw_latex: String,
}
impl Node {
    pub(crate) fn to_latex(&self, level: u8) -> Result<String, StringificationError> {
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
    pub fn children_to_latex(&self, level: u8) -> Result<String, StringificationError> {
        match self {
            NodeType::Expandable { children, .. } => children
                .iter()
                .map(|child| child.lock().unwrap().to_latex(level))
                .collect(),
            NodeType::Leaf { .. } => Ok(String::new()),
        }
    }
    pub fn to_latex(&self, level: u8) -> Result<String, StringificationError> {
        match self {
            NodeType::Expandable { data, .. } => match data {
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
                    let children = self.children_to_latex(level + 1)?;
                    Ok(format!("\\{keyword}{{{heading}}}\n{children}"))
                }
                ExpandableData::Document {
                    preamble,
                    postamble,
                } => {
                    let children: String = self.children_to_latex(level)?;
                    Ok(format!(
                        "{preamble}\\begin{{document}}\n{children}\\end{{document}}{postamble}"
                    ))
                }
                ExpandableData::Dummy { text } => {
                    let children: String = self.children_to_latex(level + 1)?;
                    Ok(format!("{text}\n{children}"))
                }
            },
            NodeType::Leaf { data } => match data {
                LeafData::Text { text } => Ok(text.to_string() + "\n\n"),
                LeafData::Image { path } => Ok(format!("\\includegraphics{{{}}}\n", path)),
            },
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum ExpandableData {
    Document { preamble: String, postamble: String },
    Segment { heading: String },
    Dummy { text: String }, // File { path: String },
                            // Environment { name: String },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum LeafData {
    Text { text: String },
    // Math { kind: MathKind, content: String },
    Image { path: String },
    // Label { label: String },
    // Caption { caption: String },
}

enum MathKind {
    SquareBrackets,
    DoubleDollars,
    Displaymath,
    Equation,
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
        raw_latex: String,
    ) -> NodeRef {
        let uuid = uuid_provider.new_uuid();
        let this = Arc::new(Mutex::new(Node {
            uuid,
            node_type: NodeType::Leaf { data },
            meta_data: MetaData {
                data: Default::default(),
            },
            parent: None,
            raw_latex,
        }));
        portal.insert(uuid, Arc::downgrade(&this));
        this
    }

    pub fn new_expandable(
        data: ExpandableData,
        children: Vec<NodeRef>,
        uuid_provider: &mut impl UuidProvider,
        portal: &mut HashMap<Uuid, NodeRefWeak>,
        raw_latex: String,
    ) -> NodeRef {
        let uuid = uuid_provider.new_uuid();
        let this = Arc::new(Mutex::new(Node {
            uuid,
            node_type: NodeType::Expandable {
                data,
                children: children,
            },
            meta_data: MetaData {
                data: Default::default(),
            },
            parent: None,
            raw_latex,
        }));
        match &this.lock().unwrap().node_type {
            NodeType::Expandable { children, .. } => {
                for child in children {
                    child.lock().unwrap().parent = Some(Arc::downgrade(&this.clone()));
                }
            }
            NodeType::Leaf { .. } => {}
        }
        portal.insert(uuid, Arc::downgrade(&this));
        this
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::ast::errors::StringificationError;
    use crate::ast::node::{LeafData, Node};
    use crate::ast::uuid_provider::TexlaUuidProvider;

    #[test]
    fn print_text() {
        let mut uuidprov = TexlaUuidProvider::new();
        let mut portal = HashMap::new();
        let node = Node::new_leaf(
            LeafData::Text {
                text: "Test".to_string(),
            },
            &mut uuidprov,
            &mut portal,
            "raw".to_string(),
        );
        assert_eq!(node.lock().unwrap().to_latex(1), Ok("Test".to_string()));
    }
}
