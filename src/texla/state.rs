use std::sync::{Arc, Mutex, RwLock};

use ast::texla_ast::TexlaAst;
use ast::Ast;

use crate::infrastructure::errors::VcsError;
use crate::infrastructure::storage_manager::{
    DirectoryChangeHandler, StorageManager, TexlaStorageManager,
};
use crate::infrastructure::vcs_manager::{GitErrorHandler, GitManager};
use crate::texla::errors::TexlaError;
use crate::texla::socket::{parse_ast_from_disk, send, TexlaSocket};

pub type TexlaState = State<TexlaAst, TexlaStorageManager<GitManager>>;
pub type SharedTexlaState = Arc<RwLock<TexlaState>>;

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
                send(&self.socket, "error", err).ok();
            }
        };
    }
}

impl GitErrorHandler for TexlaState {
    fn handle_git_error(&self, error: VcsError) {
        send(&self.socket, "error", TexlaError::from(error)).ok();
    }
}
