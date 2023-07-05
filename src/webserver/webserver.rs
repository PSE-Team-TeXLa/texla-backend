use crate::ast::Ast;
use crate::ast::texla_ast::TexlaAst;

pub struct Webserver<A> where A: Ast {
    ast: A,
    // TODO: hold Managers
}

impl Webserver<TexlaAst> {
    pub fn new(main_file: String) -> Self {
        // TODO: initialize Managers and use them
        // we cannot give them the webserver just now, because the webserver is not yet initialized
        // -> use attach_handler on the managers

        // give main_file to StorageManager, it will hold it

        // get this from StorageManager
        let latex_single_string = "";

        let ast = TexlaAst::from_latex(latex_single_string).expect("Found invalid LaTeX");

        Webserver {
            ast,
        }
    }
}

// TODO: implement handlers
