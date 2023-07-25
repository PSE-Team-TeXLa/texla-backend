use std::sync::{Arc, Mutex};

use socketioxide::adapter::LocalAdapter;
use socketioxide::Socket;

use crate::ast::texla_ast::TexlaAst;
use crate::ast::Ast;
use crate::infrastructure::errors::InfrastructureError;
use crate::infrastructure::storage_manager::{
    DirectoryChangeHandler, StorageManager, TexlaStorageManager,
};
use crate::infrastructure::vcs_manager::{GitManager, MergeConflictHandler};
use crate::texla::errors::TexlaError;
use crate::texla::socket::TexlaSocket;

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
    fn handle_directory_change(&self) {
        todo!("read files, update ast, validate ast, send ast to client");
    }
}

impl MergeConflictHandler for TexlaState {
    fn handle_merge_conflict(&self, error: InfrastructureError) {
        self.socket.emit("error", TexlaError::from(error)).ok();
    }
}
