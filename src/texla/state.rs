use std::sync::Arc;

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

pub type TexlaState = State<TexlaAst, TexlaStorageManager<GitManager>>;

pub struct State<A, SM>
where
    A: Ast,
    SM: StorageManager,
{
    pub ast: A,
    pub storage_manager: SM,
    pub socket: Arc<Socket<LocalAdapter>>,
}

impl State<TexlaAst, TexlaStorageManager<GitManager>> {
    fn send_error(&self, error: TexlaError) {
        self.socket.emit("error", error).ok();
    }
}

impl DirectoryChangeHandler for TexlaState {
    fn handle_directory_change(&self) {
        todo!("read files, update ast, validate ast, send ast to client");
    }
}

impl MergeConflictHandler for TexlaState {
    fn handle_merge_conflict(&self, error: InfrastructureError) {
        self.send_error(error.into());
    }
}
