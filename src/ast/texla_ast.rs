use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::errors::AstError;
use crate::ast::node::{Node, NodeRef, NodeRefWeak};
use crate::ast::operation::Operation;
use crate::ast::options::StringificationOptions;
use crate::ast::uuid_provider::{TexlaUuidProvider, Uuid};
use crate::ast::{parser, Ast};

pub struct TexlaAst {
    pub(crate) portal: HashMap<Uuid, NodeRefWeak>,
    pub(crate) root: NodeRef,
    pub(crate) uuid_provider: TexlaUuidProvider,
}

impl Ast for TexlaAst {
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
