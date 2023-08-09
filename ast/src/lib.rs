use errors::AstError;
use node::NodeRef;
use operation::Operation;
use options::StringificationOptions;
use serde::Serialize;
use uuid_provider::Uuid;

pub mod errors;
mod latex_constants;
mod meta_data;
pub mod node;
pub mod operation;
pub mod options;
mod parser;
pub mod texla_ast;
mod uuid_provider;

pub trait Ast: Sized + Send + Sync + Serialize {
    // TODO: we probably want to un-elide lifetimes here
    fn from_latex(latex_single_string: String) -> Result<Self, AstError>;

    fn to_latex(&self, options: StringificationOptions) -> Result<String, AstError>;
    fn execute(&mut self, operation: Box<dyn Operation<Self>>) -> Result<(), AstError>;
}
