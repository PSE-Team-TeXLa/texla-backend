use chumsky::prelude::todo;

use crate::ast::texla_ast::TexlaAst;
use crate::ast::Ast;
use crate::infrastructure::errors::MergeConflictError;
use crate::infrastructure::export_manager::{ExportManager, TexlaExportManager};
use crate::infrastructure::storage_manager::{StorageManager, TexlaStorageManager};
use crate::infrastructure::vcs_manager::{GitManager, MergeConflictHandler};

// TODO: rename to TexlaCore?
type TexlaWebserver = Webserver<TexlaAst, TexlaStorageManager<GitManager>, TexlaExportManager>;

pub struct Webserver<A, S, E>
where
    A: Ast,
    S: StorageManager,
    E: ExportManager,
{
    ast: A,
    storage_manager: S,
    export_manager: E,
}

impl TexlaWebserver {
    pub fn new(main_file: String) -> Self {
        // TODO: initialize Managers and use them
        // we cannot give them the webserver just now, because the webserver is not yet initialized
        // -> use attach_handler on the managers

        // give main_file to StorageManager, it will hold it

        // get this from StorageManager
        let latex_single_string = "";

        let ast = TexlaAst::from_latex(latex_single_string).expect("Found invalid LaTeX");

        TexlaWebserver {
            ast,
            storage_manager: todo!(),
            export_manager: todo!(),
        }
    }
}

impl MergeConflictHandler for TexlaWebserver {
    fn handle_merge_conflict(&self, error: MergeConflictError) {
        todo!()
    }
}
