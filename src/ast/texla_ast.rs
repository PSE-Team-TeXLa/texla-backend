use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde::Serialize;

use crate::ast::errors::AstError;
use crate::ast::node::{Node, NodeRef, NodeRefWeak, NodeType};
use crate::ast::operation::{Operation, Position};
use crate::ast::options::StringificationOptions;
use crate::ast::uuid_provider::{TexlaUuidProvider, Uuid, UuidProvider};
use crate::ast::{parser, Ast};

#[derive(Debug, Serialize, Clone)]
pub struct TexlaAst {
    #[serde(skip_serializing)]
    pub(crate) portal: HashMap<Uuid, NodeRefWeak>,
    // TODO can we safely call '.upgrade().unwrap()' on any weak pointer from the portal?
    pub(crate) root: NodeRef,
    #[serde(skip_serializing)]
    pub(crate) uuid_provider: TexlaUuidProvider,
    pub(crate) highest_level: i8,
}

impl TexlaAst {
    pub fn new(mut root: Node) -> Self {
        let mut portal: HashMap<Uuid, NodeRefWeak> = HashMap::new();
        let mut uuid_provider = TexlaUuidProvider::new();
        root.uuid = uuid_provider.new_uuid();
        let root_ref = Arc::new(Mutex::new(root));
        portal.insert(
            root_ref.lock().unwrap().uuid,
            Arc::downgrade(&root_ref.clone()),
        );
        TexlaAst {
            portal,
            root: root_ref,
            uuid_provider,
            highest_level: 0,
        }
    }

    // TODO: maybe replace unwraps by expect or error returning
    pub fn insert_node_at_position(&mut self, node_ref: NodeRef, position: Position) {
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
    pub fn remove_node(&mut self, node_ref: &NodeRef) -> Position {
        let node = node_ref.lock().unwrap();
        let parent_ref_weak = &node.parent.as_ref().expect("root cannot be removed");
        let parent_ref = parent_ref_weak.upgrade().unwrap();
        let mut parent = parent_ref.lock().unwrap();
        let parent_children = match &mut parent.node_type {
            NodeType::Expandable { children, .. } => children,
            NodeType::Leaf { .. } => panic!("parent is a leaf"),
        };
        let index = parent_children
            .iter()
            .position(|child_ref| Arc::ptr_eq(child_ref, &node_ref))
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
        Ok(self.root.lock().unwrap().to_latex(self.highest_level)?)
    }

    fn execute(&mut self, operation: Box<dyn Operation<TexlaAst>>) -> Result<(), AstError> {
        Ok(operation.execute_on(self)?)
    }

    // TODO: move from trait to impl
    fn get_node(&self, uuid: Uuid) -> NodeRef {
        self.portal
            .get(&uuid)
            .expect("unknown uuid")
            .upgrade()
            .expect("portal should never contain invalid weak pointers when an operation comes")
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::ast::options::StringificationOptions;
    use crate::ast::parser::parse_latex;
    use crate::ast::Ast;

    fn lf(s: String) -> String {
        s.replace("\r\n", "\n")
    }

    #[test]
    fn simple_latex_identical() {
        let latex = fs::read_to_string("latex_test_files/simple_latex.tex").unwrap();
        let ast = parse_latex(latex.clone()).expect("Valid Latex");
        assert!(ast.to_latex(Default::default()).is_ok());
        assert_eq!(
            lf(ast.to_latex(Default::default()).unwrap()),
            lf(latex.clone())
        );
    }
    #[test]
    fn only_subsection_identical() {
        let latex = fs::read_to_string("latex_test_files/only_subsection.tex").unwrap();
        let ast = parse_latex(latex.clone()).expect("Valid Latex");
        assert!(ast.to_latex(Default::default()).is_ok());
        assert_eq!(
            lf(ast.to_latex(Default::default()).unwrap()),
            lf(latex.clone())
        );
    }
    #[test]
    fn large_latex_identical() {
        let latex = fs::read_to_string("latex_test_files/large_latex.tex").unwrap();
        let ast = parse_latex(latex.clone()).expect("Valid Latex");
        assert!(ast.to_latex(Default::default()).is_ok());
        assert_eq!(
            lf(ast.to_latex(Default::default()).unwrap()),
            lf(latex.clone())
        );
    }
    #[test]
    fn lots_identical() {
        let latex = fs::read_to_string("latex_test_files/lots_of_features.tex").unwrap();
        let ast = parse_latex(latex.clone()).expect("Valid Latex");
        assert!(ast.to_latex(Default::default()).is_ok());
        assert_eq!(
            lf(ast.to_latex(Default::default()).unwrap()),
            lf(latex.clone())
        );
    }
    #[test]
    fn parse_and_to_json() {
        let latex = fs::read_to_string("latex_test_files/lots_of_features.tex").unwrap();
        let ast = parse_latex(latex.clone()).expect("Valid Latex");
        let out = serde_json::to_string_pretty(&ast).unwrap();
        fs::write("out.json", out).expect("File write error");
    }
    #[test]
    fn simple_latex_mod_formatting() {
        let formatted_latex = fs::read_to_string("latex_test_files/simple_latex.tex").unwrap();
        let unformatted_latex =
            fs::read_to_string("latex_test_files/simple_latex_unformatted.tex").unwrap();
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
