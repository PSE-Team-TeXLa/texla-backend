use std::sync::{Arc, Mutex};

use crate::infrastructure::errors::InfrastructureError;
use crate::infrastructure::storage_manager::{
    DirectoryChangeHandler, StorageManager, TexlaStorageManager,
};
use crate::infrastructure::vcs_manager::{GitManager, MergeConflictHandler};
use crate::texla::errors::TexlaError;
use crate::texla::socket::{parse_ast_from_disk, send, TexlaSocket};
use ast::texla_ast::TexlaAst;
use ast::Ast;

pub type TexlaState = State<TexlaAst, TexlaStorageManager<GitManager>>;
// TODO: maybe Mutex is not needed (if it is, use RwLock instead)
pub type SharedTexlaState = Arc<Mutex<TexlaState>>;

pub struct State<A, SM>
where
    A: Ast,
    SM: StorageManager,
{
    pub ast: A,
    pub storage_manager: Arc<Mutex<SM>>,
    pub socket: TexlaSocket,
}

impl State<TexlaAst, TexlaStorageManager<GitManager>> {}

impl DirectoryChangeHandler for TexlaState {
    fn handle_directory_change(&mut self) {
        let storage_manager = self.storage_manager.lock().unwrap();

        match parse_ast_from_disk(&storage_manager) {
            Ok(ast) => {
                self.ast = ast;
                send(&self.socket, "new_ast", self.ast.clone()).ok();
            }
            Err(err) => {
                // TODO: prepend information that files were changed on disk/remote?
                send(&self.socket, "error", err).ok();
            }
        };
    }
}

impl MergeConflictHandler for TexlaState {
    fn handle_merge_conflict(&self, error: InfrastructureError) {
        send(&self.socket, "error", TexlaError::from(error)).ok();
    }
}
