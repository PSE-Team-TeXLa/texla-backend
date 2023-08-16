use std::collections::HashMap;
use std::string::String;
use std::sync::{Arc, Mutex, Weak};

use serde::Serialize;

use crate::errors::StringificationError;
use crate::latex_constants::*;
use crate::meta_data::MetaData;
use crate::options::StringificationOptions;
use crate::texla_constants::*;
use crate::uuid_provider::{Uuid, UuidProvider};

pub(crate) type NodeRef = Arc<Mutex<Node>>;
pub(crate) type NodeRefWeak = Weak<Mutex<Node>>;

#[derive(Debug, Serialize)]
pub(crate) struct Node {
    pub(crate) uuid: Uuid,
    pub(crate) node_type: NodeType,
    #[serde(flatten)]
    pub(crate) meta_data: MetaData,
    #[serde(skip_serializing)]
    pub(crate) parent: Option<NodeRefWeak>,
    pub(crate) raw_latex: String,
}

impl Node {
    pub(crate) fn to_latex(
        &self,
        level: i8,
        options: &StringificationOptions,
    ) -> Result<String, StringificationError> {
        if options.include_metadata && !self.meta_data.data.is_empty() {
            Ok(format!(
                "% TEXLA METADATA {}\n{}",
                self.meta_data,
                self.node_type.to_latex(level, options)?
            ))
        } else {
            Ok(self.node_type.to_latex(level, options)?)
        }
    }

    pub(crate) fn new_leaf(
        data: LeafData,
        uuid_provider: &mut impl UuidProvider,
        portal: &mut HashMap<Uuid, NodeRefWeak>,
        raw_latex: String,
        metadata: HashMap<String, String>,
    ) -> NodeRef {
        let uuid = uuid_provider.new_uuid();
        let this = Arc::new(Mutex::new(Node {
            uuid,
            node_type: NodeType::Leaf { data },
            meta_data: MetaData { data: metadata },
            parent: None,
            raw_latex,
        }));
        portal.insert(uuid, Arc::downgrade(&this));
        this
    }

    pub(crate) fn new_expandable(
        data: ExpandableData,
        children: Vec<NodeRef>,
        uuid_provider: &mut impl UuidProvider,
        portal: &mut HashMap<Uuid, NodeRefWeak>,
        raw_latex: String,
        metadata: HashMap<String, String>,
    ) -> NodeRef {
        let uuid = uuid_provider.new_uuid();
        let this = Arc::new(Mutex::new(Node {
            uuid,
            node_type: NodeType::Expandable { data, children },
            meta_data: MetaData { data: metadata },
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
pub(crate) enum NodeType {
    Expandable {
        data: ExpandableData,
        children: Vec<NodeRef>,
    },
    Leaf {
        data: LeafData,
    },
}

impl NodeType {
    pub(crate) fn children_to_latex(
        &self,
        level: i8,
        options: &StringificationOptions,
    ) -> Result<String, StringificationError> {
        match self {
            NodeType::Expandable { children, .. } => children
                .iter()
                .map(|child| child.lock().unwrap().to_latex(level, options))
                .collect(),
            NodeType::Leaf { .. } => Ok(String::new()),
        }
    }

    pub(crate) fn to_latex(
        &self,
        level: i8,
        options: &StringificationOptions,
    ) -> Result<String, StringificationError> {
        match self {
            NodeType::Leaf { data } => Ok(data.to_latex(options)),
            // TODO: this should be in NodeType::Expandable just as with the leaves
            NodeType::Expandable { data, .. } => {
                match data {
                    ExpandableData::Segment { heading, counted } => {
                        // under a segment the expected next level is increased by one
                        let children = self.children_to_latex(level + 1, options)?;
                        let count = match counted {
                            false => String::from(UNCOUNTED_SEGMENT_MARKER),
                            true => String::new(),
                        };
                        let keyword = SEGMENT_LEVELS
                            .iter()
                            .find(|(lvl, _)| *lvl == level)
                            .map(|(_, keyword)| keyword)
                            .ok_or(StringificationError {
                                message: format!("Invalid nesting level: {}", level),
                            })?;
                        Ok(format!("\\{keyword}{count}{{{heading}}}\n{children}"))
                    }
                    ExpandableData::Document {
                        preamble,
                        postamble,
                    } => {
                        let children = self.children_to_latex(level, options)?;
                        Ok(format!("{preamble}\\begin{{document}}\n{children}\\end{{document}}\n{postamble}"
                    ))
                    }
                    ExpandableData::File { path } => {
                        let children = self.children_to_latex(level, options)?; //Dont increase the
                                                                                // nesting level since file is not in hierarchy
                        Ok(format!(
                            "{FILE_BEGIN_MARK}{{{path}}}\n{children}{FILE_END_MARK}{{{path}}}\n"
                        ))
                    }
                    ExpandableData::Environment { name } => {
                        let children = self.children_to_latex(level, options)?;
                        Ok(format!("\\begin{{{name}}}\n{children}\\end{{{name}}}\n"))
                    }
                    ExpandableData::Dummy {
                        before_children,
                        after_children,
                    } => {
                        let children = self.children_to_latex(level, options)?;
                        Ok(format!("{before_children}\n{children}{after_children}\n"))
                    }
                }
            }
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub(crate) enum ExpandableData {
    Document {
        preamble: String,
        postamble: String,
    },
    Segment {
        heading: String,
        counted: bool,
    },
    File {
        path: String,
    },
    Environment {
        name: String,
    },
    Dummy {
        before_children: String,
        after_children: String,
    },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub(crate) enum LeafData {
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
    Comment {
        comment: String,
    },
}

impl LeafData {
    // This does not consume the node
    fn to_latex(&self, options: &StringificationOptions) -> String {
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
                MathKind::DoubleDollars => format!("$${content}$$\n"),
                MathKind::Displaymath => {
                    format!("\\begin{{displaymath}}{content}\\end{{displaymath}}\n")
                }
                MathKind::Equation => {
                    format!("\\begin{{equation}}{content}\\end{{equation}}\n")
                }
            },
            LeafData::Comment { comment } => {
                if options.include_comments {
                    // TODO: is this just to replace new lines?
                    comment.lines().fold(String::new(), |mut acc, line| {
                        acc.push_str(format!("{line}\n").as_str());
                        acc
                    }) + "\n"
                } else {
                    String::new()
                }
            }
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub(crate) enum MathKind {
    SquareBrackets,
    DoubleDollars,
    Displaymath,
    Equation,
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::node::{LeafData, Node};
    use crate::options::StringificationOptions;
    use crate::uuid_provider::TexlaUuidProvider;

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
            Default::default(),
        );
        assert_eq!(
            node.lock()
                .unwrap()
                .to_latex(1, &StringificationOptions::default()),
            Ok("Test\n\n".to_string())
        );
    }
}
