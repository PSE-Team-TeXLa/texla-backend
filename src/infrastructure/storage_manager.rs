use std::fs;

use crate::infrastructure::errors::InfrastructureError;
use crate::infrastructure::vcs_manager::{GitManager, MergeConflictHandler, VcsManager};

pub trait StorageManager {
    fn start(&self);
    fn remote_url(&self) -> Option<&String>;
    fn multiplex_files(&self) -> Result<String, InfrastructureError>;
    fn stop_timers(&mut self);
    fn save(&mut self, latex_single_string: String) -> Result<(), InfrastructureError>;
    fn end_session(&mut self) -> Result<(), InfrastructureError>;
}

pub struct TexlaStorageManager<V>
where
    V: VcsManager,
{
    vcs_manager: V,
    directory_change_handler: Option<Box<dyn DirectoryChangeHandler>>,
    merge_conflict_handler: Option<Box<dyn MergeConflictHandler>>,
    main_file: String,
    // TODO: should maybe be a reference and maybe a Path
    pull_timer_running: bool,
    worksession_timer_running: bool,
}

impl<V> TexlaStorageManager<V>
where
    V: VcsManager,
{
    pub fn new(vcs_manager: V, main_file: String) -> Self {
        Self {
            vcs_manager,
            directory_change_handler: None,
            merge_conflict_handler: None,
            main_file,
            pull_timer_running: false,
            worksession_timer_running: false,
        }
    }

    pub async fn attach_handlers_and_start(
        &mut self,
        dc_handler: Box<dyn DirectoryChangeHandler>,
        mc_handler: Box<dyn MergeConflictHandler>,
    ) {
        self.directory_change_handler = Some(dc_handler);
        self.merge_conflict_handler = Some(mc_handler);

        // TODO: start async timer-based background tasks and start DirectoryChangeHandler
    }
}

impl StorageManager for TexlaStorageManager<GitManager> {
    fn start(&self) {
        // TODO after VS: start async timer-based background tasks and start DirectoryChangeHandler

        todo!()
    }

    fn remote_url(&self) -> Option<&String> {
        self.vcs_manager.remote_url()
    }

    fn multiplex_files(&self) -> Result<String, InfrastructureError> {
        // TODO after VS: handle multiple files

        // dummy implementation for a single file without '\input'
        fs::read_to_string(&self.main_file).map_err(|_| InfrastructureError {})
    }

    fn stop_timers(&mut self) {
        self.pull_timer_running = false;
        self.worksession_timer_running = false;
    }

    fn save(&mut self, latex_single_string: String) -> Result<(), InfrastructureError> {
        // TODO after VS: handle multiple files

        // dummy implementation for a single file without '\input'
        let res =
            fs::write(&self.main_file, latex_single_string).map_err(|_| InfrastructureError {});

        if res.is_ok() {
            self.pull_timer_running = true;
            self.worksession_timer_running = true;
        }

        res
    }

    fn end_session(&mut self) -> Result<(), InfrastructureError> {
        // TODO after VS: stop async timer-based background tasks and stop DirectoryChangeHandler

        // don't call save() here since you can't quit (i.e. end the session) with unsaved changes

        self.stop_timers();

        self.vcs_manager.commit(None)?;
        self.vcs_manager.push()
    }
}

pub trait DirectoryChangeHandler: Send + Sync {
    fn handle_directory_change(&self);
}
