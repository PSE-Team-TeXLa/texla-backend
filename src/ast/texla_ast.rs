use crate::ast::errors::AstError;
use crate::ast::operation::Operation;
use crate::ast::options::StringificationOptions;
use crate::ast::Ast;

pub struct TexlaAst {}

impl Ast for TexlaAst {
    // TODO: why not move latex_single_string?
    fn from_latex(latex_single_string: &str) -> Result<TexlaAst, AstError> {
        // TODO
        Ok(TexlaAst {})
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
