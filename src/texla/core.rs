use std::cell::RefCell;
use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Mutex, RwLock};

use crate::ast::texla_ast::TexlaAst;
use crate::ast::Ast;
use crate::infrastructure::errors::InfrastructureError;
use crate::infrastructure::export_manager::{ExportManager, TexlaExportManager};
use crate::infrastructure::storage_manager::{
    DirectoryChangeHandler, StorageManager, TexlaStorageManager,
};
use crate::infrastructure::vcs_manager::{GitManager, MergeConflictHandler};
use crate::texla::state::{State, TexlaState};
use crate::texla::webserver::start_axum;

pub struct TexlaCore {
    pub export_manager: TexlaExportManager,
    // only needed for offline versoion
    // not clean (maybe pass main_file over frontend)
    pub main_file: String,
}

impl DirectoryChangeHandler for TexlaCore {
    fn handle_directory_change(&self) {
        todo!()
    }
}

impl MergeConflictHandler for TexlaCore {
    fn handle_merge_conflict(&self, error: InfrastructureError) {
        todo!()
    }
}
