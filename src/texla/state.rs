use std::sync::Arc;

use socketioxide::adapter::LocalAdapter;
use socketioxide::Socket;

use crate::ast::texla_ast::TexlaAst;
use crate::ast::Ast;
use crate::infrastructure::storage_manager::{StorageManager, TexlaStorageManager};
use crate::infrastructure::vcs_manager::GitManager;

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

impl State<TexlaAst, TexlaStorageManager<GitManager>> {}