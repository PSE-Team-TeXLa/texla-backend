use serde::Serialize;

use errors::AstError;
use operation::Operation;
use options::StringificationOptions;

pub mod errors;
pub mod latex_constants;
mod meta_data;
pub mod node;
pub mod operation;
pub mod options;
mod parser;
pub mod texla_ast;
pub mod texla_constants;
mod uuid_provider;

pub trait Ast: Sized + Send + Sync + Serialize {
    fn from_latex(latex_single_string: String) -> Result<Self, AstError>;

    fn to_latex(&self, options: StringificationOptions) -> Result<String, AstError>;
    fn execute(&mut self, operation: Box<dyn Operation<Self>>) -> Result<(), AstError>;
}
