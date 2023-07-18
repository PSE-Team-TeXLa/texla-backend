use crate::ast::errors::AstError;
use crate::ast::node::NodeRef;
use crate::ast::operation::Operation;
use crate::ast::options::StringificationOptions;
use crate::ast::uuid_provider::Uuid;

pub mod errors;
mod meta_data;
mod node;
pub mod operation;
pub mod options;
mod parser;
pub mod texla_ast;
mod uuid_provider;

pub trait Ast: Sized + Send + Sync {
    // TODO: we probably want to un-elide lifetimes here
    fn from_latex(latex_single_string: String) -> Result<Self, AstError>;

    fn to_latex(&self, options: StringificationOptions) -> Result<String, AstError>;
    fn to_json(&self, options: StringificationOptions) -> Result<String, AstError>;
    fn execute(&mut self, operation: Box<dyn Operation<Self>>) -> Result<(), AstError>;
    fn get_node(&self, uuid: Uuid) -> NodeRef; // TODO define method here?
                                               // TODO add methods for deleting a node and inserting a node?
}
