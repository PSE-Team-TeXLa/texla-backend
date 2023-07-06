use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::meta_data::MetaData;
use crate::ast::node::LeafData::Text;
use crate::ast::node::{Node, NodeType};
use crate::ast::texla_ast::TexlaAst;
use crate::ast::uuid_provider::TexlaUuidProvider;

pub fn parse_latex(latex_string: String) -> Node {
    todo!()
}
