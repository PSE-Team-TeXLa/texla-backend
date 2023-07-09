use std::path::Path;

use crate::ast::texla_ast::TexlaAst;
use crate::ast::Ast;
use crate::infrastructure::errors::InfrastructureError;
use crate::infrastructure::export_manager::{ExportManager, TexlaExportManager};
use crate::infrastructure::storage_manager::{StorageManager, TexlaStorageManager};
use crate::infrastructure::vcs_manager::{GitManager, MergeConflictHandler};

type TexlaCore = Core<TexlaAst, TexlaStorageManager<GitManager>, TexlaExportManager>;

pub struct Core<A, S, E>
where
    A: Ast,
    S: StorageManager,
    E: ExportManager,
{
    ast: A,
    storage_manager: S,
    // TODO: export_manager does not hold state -> could be created everytime when needed
    // let it here either way, for extensibility
    export_manager: E,
}

impl TexlaCore {
    pub fn new(main_file: String) -> Self {
        // TODO is there a shorter way to get the parent directory as String?
        let parent_directory = Path::new(&main_file)
            .parent()
            .expect("No parent directory found")
            .to_str()
            .expect("No parent directory found")
            .to_string();

        let vcs_manager = GitManager::new(parent_directory);
        let storage_manager = TexlaStorageManager::new(vcs_manager, main_file);
        let export_manager = TexlaExportManager;

        // TODO call TexlaStorageManager<T>::attach_handlers() later

        let latex_single_string = storage_manager
            .multiplex_files()
            .expect("Could not build LaTeX single string");
        let ast = TexlaAst::from_latex(&latex_single_string).expect("Found invalid LaTeX");

        Self {
            ast,
            storage_manager,
            export_manager,
        }
    }
}

impl MergeConflictHandler for TexlaCore {
    fn handle_merge_conflict(&self, error: InfrastructureError) {
        todo!()
    }
}
