use std::collections::HashMap;
use std::fmt::format;
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
    pub(crate) fn to_latex(&self, level: i8) -> Result<String, StringificationError> {
        self.node_type.to_latex(level)
    }

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
    pub fn children_to_latex(&self, level: i8) -> Result<String, StringificationError> {
        match self {
            NodeType::Expandable { children, .. } => children
                .iter()
                .map(|child| child.lock().unwrap().to_latex(level))
                .collect(),
            NodeType::Leaf { .. } => Ok(String::new()),
        }
    }
    pub fn to_latex(&self, level: i8) -> Result<String, StringificationError> {
        match self {
            NodeType::Leaf { data } => Ok(data.to_latex()),
            NodeType::Expandable { data, children } => {
                let children = self.children_to_latex(level + 1)?;
                match data {
                    ExpandableData::Segment { heading } => {
                        let keyword = match level {
                            3 => "section".to_string(),
                            4 => "subsection".to_string(),
                            other => {
                                return Err(StringificationError {
                                    message: format!("Invalid Nesting Level: {}", other),
                                })
                            }
                        };
                        Ok(format!("\\{keyword}{{{heading}}}\n{children}"))
                    }
                    ExpandableData::Document {
                        preamble,
                        postamble,
                    } => Ok(format!(
                        "{preamble}\\begin{{document}}\n{children}\\end{{document}}{postamble}"
                    )),
                    ExpandableData::File { path } => Ok(format!(
                        "% TEXLA FILE BEGIN ({path})\n{children}\n% TEXLA FILE END"
                    )),
                    ExpandableData::Environment { name } => {
                        Ok(format!("\\begin{{{name}}}\n{children}\n\\end{{{name}}}"))
                    }
                    ExpandableData::Dummy { text } => Ok(format!("{text}\n{children}")),
                }
            }
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum ExpandableData {
    Document { preamble: String, postamble: String },
    Segment { heading: String },
    File { path: String },
    Environment { name: String },
    Dummy { text: String },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum LeafData {
    Text {
        text: String,
    },
    Math {
        kind: MathKind,
        content: String,
    },
    Image {
        path: String,
        options: Option<String>,
    },
    Label {
        label: String,
    },
    Caption {
        caption: String,
    },
}
impl LeafData {
    // This does not consume the node
    fn to_latex(&self) -> String {
        match self {
            LeafData::Text { text } => format!("{text}\n\n"),
            LeafData::Image { path, options } => match options {
                None => format!("\\includegraphics{{{path}}}\n"),
                Some(option) => format!("\\includegraphics[{option}]{{{path}}}\n"),
            },
            LeafData::Label { label } => format!("\\label{{{label}}}\n"),
            LeafData::Caption { caption } => format!("\\caption{{{caption}}}\n"),
            LeafData::Math { kind, content } => match kind {
                MathKind::SquareBrackets => format!("\\[{content}\\]\n"),
                MathKind::DoubleDollars => format!("$$\n{content}\n$$\n"),
                MathKind::Displaymath => {
                    format!("\\begin{{displaymath}}\n{content}\n\\end{{displaymath}}\n")
                }
                MathKind::Equation => {
                    format!("\\begin{{equation}}\n{content}\n\\end{{equation}}\n")
                }
            },
        }
    }
}

#[derive(Debug, Serialize)]
pub enum MathKind {
    SquareBrackets,
    DoubleDollars,
    Displaymath,
    Equation,
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
