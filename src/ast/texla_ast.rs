use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::errors::AstError;
use crate::ast::node::{Node, NodeRef, NodeRefWeak};
use crate::ast::operation::Operation;
use crate::ast::options::StringificationOptions;
use crate::ast::uuid_provider::{TexlaUuidProvider, Uuid, UuidProvider};
use crate::ast::{parser, Ast};

#[derive(Debug)]
pub struct TexlaAst {
    pub(crate) portal: HashMap<Uuid, NodeRefWeak>,
    pub(crate) root: NodeRef,
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
    // TODO: why not move latex_single_string?
    fn from_latex(latex_single_string: &str) -> Result<TexlaAst, AstError> {
        todo!()
    }

    fn to_latex(&self, options: StringificationOptions) -> Result<String, AstError> {
        todo!()
    }

    fn to_json(&self, options: StringificationOptions) -> Result<String, AstError> {
        todo!()
    }

    fn execute(&self, operation: Box<dyn Operation<TexlaAst>>) -> Result<(), AstError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::ast::meta_data::MetaData;
    use crate::ast::node::{LeafData, Node, NodeType};
    use crate::ast::texla_ast::TexlaAst;
    use crate::ast::uuid_provider::Uuid;

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
}
