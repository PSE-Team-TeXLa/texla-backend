use errors::AstError;
use operation::Operation;
use options::StringificationOptions;
use serde::Serialize;

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

/// The **Ast (Abstract Syntax Tree)** is the central Data structure of this Library and is used to represent LaTeX Documents programmatically.\
/// This Trait requires methods to convert to LaTeX as well as to create the Ast from LaTeX.\
/// Additionally it requires the implementation of the Strategy Pattern in order to make modifications on the Ast.\
/// Interaction with the Ast can work in the following way:
/// - `from_latex()` to get Ast Representation of some LaTeX String.
/// - Serialize using [serde] to get the representation in Json format including Uuids.
/// - Construct Operations to modify the Ast, use Uuids from Json.
/// - Execute Operation on Ast using `execute()`
/// - Generate LaTeX Source code for the new Ast using `to_latex()`
///
/// [texla_ast::TexlaAst] implementation can be used.
pub trait Ast: Sized + Send + Sync + Serialize {
    /// Generates an Ast representation of the provided `latex_single_string`.
    fn from_latex(latex_single_string: String) -> Result<Self, AstError>;
    /// Generates LaTeX Source code from this Ast.
    fn to_latex(&self, options: StringificationOptions) -> Result<String, AstError>;
    /// Modifies this Ast by applying the provided `Operation`.
    /// [Operation] is a Strategy template. To implement this, `execute_on()` should be called on the operation and this Ast should be passed.
    fn execute(&mut self, operation: Box<dyn Operation<Self>>) -> Result<(), AstError>;
}
