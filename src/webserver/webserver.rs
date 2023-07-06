use crate::ast::Ast;
use crate::ast::texla_ast::TexlaAst;
use crate::infrastructure::errors::MergeConflictError;
use crate::infrastructure::export_manager::ExportManager;
use crate::infrastructure::storage_manager::StorageManager;
use crate::infrastructure::vcs_manager::MergeConflictHandler;

pub struct Webserver<A> where A: Ast {
    ast: A,
    storage_manager: dyn StorageManager,
    export_manager: dyn ExportManager,
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

impl MergeConflictHandler for Webserver<TexlaAst> {
    fn handle_merge_conflict(error: MergeConflictError) {
        todo!()
    }
}
