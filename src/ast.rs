use crate::ast::errors::AstError;
use crate::ast::operation::Operation;
use crate::ast::options::StringificationOptions;

mod errors;
mod meta_data;
mod node;
mod operation;
pub mod options;
pub mod texla_ast;
mod uuid_provider;

pub trait Ast: Sized {
    // TODO: we probably want to un-elide lifetimes here
    fn from_latex(latex_single_string: &str) -> Result<Self, AstError>;

    fn to_latex(&self, options: StringificationOptions) -> Result<String, AstError>;
    fn to_json(&self, options: StringificationOptions) -> Result<String, AstError>;
    fn execute(&self, operation: Box<dyn Operation<Self>>) -> Result<(), AstError>;
}
