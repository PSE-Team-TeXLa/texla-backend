use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use serde::{Deserialize, Serialize};

use crate::ast::errors::{AstError, StringificationError};
use crate::ast::node::{Node, NodeRef, NodeRefWeak};
use crate::ast::operation::Operation;
use crate::ast::options::StringificationOptions;
use crate::ast::uuid_provider::{TexlaUuidProvider, Uuid, UuidProvider};
use crate::ast::{parser, Ast};

#[derive(Debug, Serialize)]
pub struct TexlaAst {
    #[serde(skip_serializing)]
    pub(crate) portal: HashMap<Uuid, NodeRefWeak>,
    pub(crate) root: NodeRef,
    #[serde(skip_serializing)]
    pub(crate) uuid_provider: TexlaUuidProvider,
}

impl TexlaAst {
    pub fn new(mut root: Node) -> Self {
        let mut portal: HashMap<Uuid, NodeRefWeak> = HashMap::new();
        let mut uuid_provider = TexlaUuidProvider::new();
        root.uuid = uuid_provider.new_uuid();
        let root_ref = Rc::new(RefCell::new(root));
        portal.insert(root_ref.borrow().uuid, Rc::downgrade(&root_ref.clone()));
        TexlaAst {
            portal,
            root: root_ref,
            uuid_provider,
        }
    }
}

impl Ast for TexlaAst {
    fn from_latex(latex_single_string: &str) -> Result<Self, AstError> {
        Ok(parser::parse_latex(latex_single_string.to_string())?)
    }

    fn to_latex(&self, options: StringificationOptions) -> Result<String, AstError> {
        Ok(self.root.borrow().to_latex(1)?)
    }

    fn to_json(&self, options: StringificationOptions) -> Result<String, AstError> {
        match serde_json::to_string_pretty(self) {
            Ok(json_string) => Ok(json_string),
            Err(error) => Err(AstError::from(StringificationError::from(error))),
        }
    }

    fn execute(&self, operation: Box<dyn Operation<TexlaAst>>) -> Result<(), AstError> {
        todo!()
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
                meta_data: Default::default(),
            },
            parent: None,
        };
        let ast = TexlaAst::new(root);
        assert!(ast.root.borrow().parent.is_none());
        assert_eq!(ast.root.borrow().uuid, 1);
        assert!(ast.portal.get(&(1 as Uuid)).is_some());
        assert!(ast.portal.get(&(2 as Uuid)).is_none());
    }
    #[test]
    fn parse_and_print() {
        let latex = fs::read_to_string("simple_latex").unwrap();
        let ast = parse_latex(latex.clone()).expect("Valid Latex");
        assert!(ast.to_latex(StringificationOptions {}).is_ok());
        assert_eq!(
            ast.to_latex(StringificationOptions {}).unwrap(),
            latex.clone()
        );
    }
    #[test]
    fn parse_and_to_json() {
        let latex = fs::read_to_string("simple_latex").unwrap();
        let ast = parse_latex(latex.clone()).expect("Valid Latex");
        let out = ast.to_json(StringificationOptions {}).unwrap();
        fs::write("out.json", out).expect("File write error");
    }
}
