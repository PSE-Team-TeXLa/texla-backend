use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::task::JoinHandle;
use tokio::time::sleep;

use crate::infrastructure::storage_manager::TexlaStorageManager;
use crate::infrastructure::vcs_manager::{GitManager, VcsManager};

pub(crate) struct PullTimerManager {
    storage_manager: Arc<Mutex<TexlaStorageManager<GitManager>>>,
    join_handle: Option<JoinHandle<()>>,
}

impl PullTimerManager {
    pub(crate) fn new(storage_manager: Arc<Mutex<TexlaStorageManager<GitManager>>>) -> Self {
        Self {
            storage_manager,
            join_handle: None,
        }
    }

    pub(crate) fn activate(&mut self) {
        self.join_handle = Some(tokio::spawn(pull_repeatedly(self.storage_manager.clone())));
    }

    pub(crate) fn deactivate(&mut self) {
        if let Some(handle) = self.join_handle.take() {
            handle.abort();
        };
    }

    pub(crate) fn disassemble(&mut self) {
        self.deactivate();
    }
}

async fn pull_repeatedly<V: VcsManager>(storage_manager: Arc<Mutex<TexlaStorageManager<V>>>) {
    let duration = Duration::from_millis(storage_manager.lock().unwrap().pull_interval);

    loop {
        storage_manager.lock().unwrap().vcs_manager.pull();

        sleep(duration).await;
    }
}
