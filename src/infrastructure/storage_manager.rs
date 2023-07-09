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

    fn attach_handlers(
        &mut self,
        dc_handler: Box<dyn DirectoryChangeHandler>,
        mc_handler: Box<dyn MergeConflictHandler>,
    ) {
        self.directory_change_handler = Some(dc_handler);
        self.merge_conflict_handler = Some(mc_handler);
    }
}

impl StorageManager for TexlaStorageManager<GitManager> {
    fn start(&self) {
        // TODO start async background tasks based on the timers and start DirectoryChangeHandler

        todo!()
    }

    fn remote_url(&self) -> Option<&String> {
        self.vcs_manager.remote_url()
    }

    fn multiplex_files(&self) -> Result<String, InfrastructureError> {
        // TODO handle multiple files

        // dummy implementation for a single file without '\input'
        fs::read_to_string(&self.main_file).map_err(|_| InfrastructureError {})
    }

    fn stop_timers(&mut self) {
        self.pull_timer_running = false;
        self.worksession_timer_running = false;
    }

    fn save(&mut self, latex_single_string: String) -> Result<(), InfrastructureError> {
        // TODO handle multiple files

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
        // TODO stop async background tasks based on the timers and stop DirectoryChangeHandler

        // TODO call save() here as in spec although you can't quit with unsaved changes?
        //  --> if save() should be called, pass latex_single_string as argument to this method

        self.stop_timers();

        self.vcs_manager.commit(None)?;
        self.vcs_manager.push()
    }
}

pub trait DirectoryChangeHandler {
    fn handle_directory_change(&self);
}
