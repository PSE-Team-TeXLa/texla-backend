use std::collections::HashMap;
use std::sync::Arc;

use serde::Serialize;

use crate::errors::AstError;
use crate::node::{NodeRef, NodeRefWeak, NodeType};
use crate::operation::Operation;
use crate::options::StringificationOptions;
use crate::uuid_provider::{Position, TexlaUuidProvider, Uuid};
use crate::{parser, Ast};

/// `TexlaAst` Implements [Ast] and can represent LaTex Documents which follow a number of specifications in the Pflichtenheft Document.
#[derive(Debug, Serialize, Clone)]
pub struct TexlaAst {
    // by reparsing after each operation all weak references in this hashmap are always valid
    #[serde(skip_serializing)]
    pub(crate) portal: HashMap<Uuid, NodeRefWeak>,
    pub(crate) root: NodeRef,
    #[serde(skip_serializing)]
    pub(crate) uuid_provider: TexlaUuidProvider,
    pub(crate) highest_level: i8,
}

/// The methods here shall be atomar, which is why they panic instead of returning errors.
/// They assert the validity of certain invariants, namely:
/// - A parent weak reference must be valid and must be an Expandable Node.
/// - A portal weak reference must be valid.
/// - No non-existing UUIDs are queried.
impl TexlaAst {
    pub(crate) fn get_node(&self, uuid: Uuid) -> NodeRef {
        self.portal
            .get(&uuid)
            .expect("unknown uuid")
            .upgrade()
            .expect("portal should never contain invalid weak pointers when an operation comes")
    }

    pub(crate) fn insert_node_at_position(&mut self, node_ref: NodeRef, position: Position) {
        let parent_ref = self.get_node(position.parent);
        let mut parent = parent_ref.lock().unwrap();
        let parent_children = match &mut parent.node_type {
            NodeType::Expandable { children, .. } => children,
            NodeType::Leaf { .. } => panic!("position parent is a leaf"),
        };
        let index = match position.after_sibling {
            None => 0,
            Some(uuid) => {
                let sibling_ref = self.get_node(uuid);
                parent_children
                    .iter()
                    .position(|child_ref| Arc::ptr_eq(child_ref, &sibling_ref))
                    .expect("after_sibling not found")
                    + 1
            }
        };
        self.portal
            .insert(node_ref.lock().unwrap().uuid, Arc::downgrade(&node_ref));
        parent_children.insert(index, node_ref);
    }

    /// returns the [Position] of the removed node
    pub(crate) fn remove_node(&mut self, node_ref: &NodeRef) -> Position {
        let node = node_ref.lock().unwrap();
        let parent_ref_weak = &node.parent.as_ref().expect("root cannot be removed");
        let parent_ref = parent_ref_weak.upgrade().expect("parent ref is invalid");
        let mut parent = parent_ref.lock().unwrap();
        let parent_children = match &mut parent.node_type {
            NodeType::Expandable { children, .. } => children,
            NodeType::Leaf { .. } => panic!("parent is a leaf"),
        };
        let index = parent_children
            .iter()
            .position(|child_ref| Arc::ptr_eq(child_ref, node_ref))
            .expect("target is not child of parent");
        parent_children.remove(index);
        self.portal.remove(&node.uuid);
        Position {
            after_sibling: match index {
                0 => None,
                _ => Some(parent_children[index - 1].lock().unwrap().uuid),
            },
            parent: parent.uuid, // the order of properties matters here because of borrowing
        }
    }
}

impl Ast for TexlaAst {
    fn from_latex(latex_single_string: String) -> Result<Self, AstError> {
        Ok(parser::parse_latex(latex_single_string)?)
    }

    fn to_latex(&self, options: StringificationOptions) -> Result<String, AstError> {
        Ok(self
            .root
            .lock()
            .unwrap()
            .to_latex(self.highest_level, &options)?)
    }

    fn execute(&mut self, operation: Box<dyn Operation<TexlaAst>>) -> Result<(), AstError> {
        Ok(operation.execute_on(self)?)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::options::StringificationOptions;
    use crate::parser::parse_latex;
    use crate::Ast;

    fn lf(s: String) -> String {
        s.replace("\r\n", "\n")
    }

    fn test_for_identity_after_parse_and_stringify(latex: String) {
        let ast = parse_latex(latex.clone()).expect("Valid Latex");
        assert!(ast.to_latex(Default::default()).is_ok());
        assert_eq!(
            lf(ast.to_latex(Default::default()).unwrap()),
            lf(latex.clone())
        );
    }

    #[test]
    fn simple_latex_identical() {
        let latex = fs::read_to_string("../test_resources/latex/simple.tex").unwrap();
        test_for_identity_after_parse_and_stringify(latex);
    }

    #[test]
    fn empty_document_identical() {
        let latex = fs::read_to_string("../test_resources/latex/empty_document.tex").unwrap();
        test_for_identity_after_parse_and_stringify(latex);
    }

    #[test]
    fn only_subsection_identical() {
        let latex = fs::read_to_string("../test_resources/latex/only_subsection.tex").unwrap();
        test_for_identity_after_parse_and_stringify(latex);
    }

    #[test]
    fn large_latex_identical() {
        let latex = fs::read_to_string("../test_resources/latex/large.tex").unwrap();
        test_for_identity_after_parse_and_stringify(latex);
    }

    #[test]
    fn lots_identical() {
        let latex = fs::read_to_string("../test_resources/latex/lots_of_features.tex").unwrap();
        test_for_identity_after_parse_and_stringify(latex);
    }

    #[test]
    fn align_identical() {
        let latex = fs::read_to_string("../test_resources/latex/align.tex").unwrap();
        test_for_identity_after_parse_and_stringify(latex);
    }

    #[test]
    fn sectioning() {
        let latex = fs::read_to_string("../test_resources/latex/sectioning.tex").unwrap();
        test_for_identity_after_parse_and_stringify(latex);
    }

    #[test]
    fn parse_and_to_json() {
        let latex = fs::read_to_string("../test_resources/latex/lots_of_features.tex").unwrap();
        let ast = parse_latex(latex.clone()).expect("Valid Latex");
        let out = serde_json::to_string_pretty(&ast).unwrap();
        fs::create_dir("../test_resources/json").ok();
        fs::write("../test_resources/json/out.json", out).expect("File write error");
    }

    #[test]
    fn simple_latex_mod_formatting() {
        let formatted_latex = fs::read_to_string("../test_resources/latex/simple.tex").unwrap();
        let unformatted_latex =
            fs::read_to_string("../test_resources/latex/simple_unformatted.tex").unwrap();
        let ast = parse_latex(unformatted_latex.clone()).expect("Valid Latex");
        let out = ast
            .to_latex(StringificationOptions {
                include_comments: false,
                include_metadata: false,
            })
            .unwrap();
        assert_eq!(lf(out), lf(formatted_latex));
    }
}
