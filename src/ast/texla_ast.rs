use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde::Serialize;

use crate::ast::errors::AstError;
use crate::ast::node::{Node, NodeRef, NodeRefWeak};
use crate::ast::operation::Operation;
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
}

impl Ast for TexlaAst {
    fn from_latex(latex_single_string: String) -> Result<Self, AstError> {
        Ok(parser::parse_latex(latex_single_string)?)
    }

    fn to_latex(&self, options: StringificationOptions) -> Result<String, AstError> {
        Ok(self.root.lock().unwrap().to_latex(self.highest_level)?)
    }

    fn execute(&mut self, operation: Box<dyn Operation<TexlaAst>>) -> Result<(), AstError> {
        operation.execute_on(self)
    }

    fn get_node(&self, uuid: Uuid) -> NodeRef {
        self.portal
            .get(&uuid)
            .expect("unknown uuid")
            .upgrade()
            .expect("portal should never contain invalid weak pointers when a operation comes")
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::ast::meta_data::MetaData;
    use crate::ast::node::{LeafData, Node, NodeType};
    use crate::ast::options::StringificationOptions;
    use crate::ast::parser::parse_latex;
    use crate::ast::texla_ast::TexlaAst;
    use crate::ast::uuid_provider::Uuid;
    use crate::ast::Ast;

    #[test]
    fn crate_ast() {
        let root = Node {
            uuid: 0,
            node_type: NodeType::Leaf {
                data: LeafData::Text {
                    text: "ROOT".to_string(),
                },
            },
            meta_data: MetaData {
                data: Default::default(),
            },
            parent: None,
            raw_latex: String::new(),
        };
        let ast = TexlaAst::new(root);
        assert!(ast.root.lock().unwrap().parent.is_none());
        assert_eq!(ast.root.lock().unwrap().uuid, 1);
        assert!(ast.portal.get(&(1 as Uuid)).is_some());
        assert!(ast.portal.get(&(2 as Uuid)).is_none());
    }
    #[test]
    fn simple_latex_identical() {
        let latex = fs::read_to_string("latex_test_files/simple_latex.tex").unwrap();
        let ast = parse_latex(latex.clone()).expect("Valid Latex");
        assert!(ast.to_latex(Default::default()).is_ok());
        assert_eq!(ast.to_latex(Default::default()).unwrap(), latex.clone());
    }
    #[test]
    fn only_subsection_identical() {
        let latex = fs::read_to_string("latex_test_files/only_subsection.tex").unwrap();
        let ast = parse_latex(latex.clone()).expect("Valid Latex");
        assert!(ast.to_latex(Default::default()).is_ok());
        assert_eq!(ast.to_latex(Default::default()).unwrap(), latex.clone());
    }
    #[test]
    fn large_latex_identical() {
        let latex = fs::read_to_string("latex_test_files/large_latex.tex").unwrap();
        let ast = parse_latex(latex.clone()).expect("Valid Latex");
        assert!(ast.to_latex(Default::default()).is_ok());
        assert_eq!(ast.to_latex(Default::default()).unwrap(), latex.clone());
    }
    #[test]
    fn lots_identical() {
        let latex = fs::read_to_string("latex_test_files/lots_of_features.tex").unwrap();
        let ast = parse_latex(latex.clone()).expect("Valid Latex");
        assert!(ast.to_latex(Default::default()).is_ok());
        assert_eq!(ast.to_latex(Default::default()).unwrap(), latex.clone());
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
        assert_eq!(out, formatted_latex);
    }
}
