use std::sync::{Arc, Mutex};
use std::time::Duration;

use debounced::debounced;
use futures::channel::mpsc::{channel, Sender};
use futures::SinkExt;
use futures::StreamExt;
use tokio::task::JoinHandle;

use crate::infrastructure::storage_manager::TexlaStorageManager;
use crate::infrastructure::vcs_manager::{GitManager, VcsManager};

const WORKSESSION_LENGTH: Duration = Duration::from_secs(5);

pub(crate) enum WorksessionMessage {
    /// Keep the worksession alive for another [WorksessionManager::WORKSESSION_LENGTH]
    Uphold,
    /// Keep the worksession alive indefinitely
    Pause,
}

pub(crate) struct WorksessionManager {
    tx: Sender<WorksessionMessage>,
    join_handle: JoinHandle<()>,
}

impl WorksessionManager {
    pub(crate) fn new(storage_manager: Arc<Mutex<TexlaStorageManager<GitManager>>>) -> Self {
        // TODO: maybe this should be a sync_channel, but then we probably need a crate
        let (tx, rx) = channel(2);
        let mut debounced = debounced(rx, WORKSESSION_LENGTH);

        let join_handle = tokio::spawn(async move {
            while let Some(msg) = debounced.next().await {
                match msg {
                    WorksessionMessage::Uphold => end_worksession(&storage_manager).await,
                    WorksessionMessage::Pause => {
                        // nothing happened in the last WORKSESSION_LENGTH and the last message
                        // was Pause => do not stop the worksession
                    }
                }
            }
        });

        Self { tx, join_handle }
    }

    pub(crate) async fn start_or_uphold(&mut self) {
        self.tx.send(WorksessionMessage::Uphold).await.unwrap();
    }

    pub(crate) async fn pause(&mut self) {
        self.tx.send(WorksessionMessage::Pause).await.unwrap();
    }

    pub(crate) fn disassemble(&self) {
        self.join_handle.abort();
    }
}

// TODO: this seems redundant with (use end_session instead)
/// [crate::infrastructure::storage_manager::StorageManager::end_worksession]
async fn end_worksession(storage_manager: &Arc<Mutex<TexlaStorageManager<GitManager>>>) {
    let storage_manager = storage_manager.lock().unwrap();
    // TODO unwrap every time instead?

    let commit_result = storage_manager.vcs_manager.commit(None);
    if commit_result.is_err() {
        // TODO in case of error, repeat commit? (*)
    }

    let pull_result = storage_manager.vcs_manager.pull();
    if pull_result.is_err() {
        // TODO in case of merge conflict, inform user
        // TODO in case of other error (how to differentiate?), repeat pull only? (*)
    }

    let push_result = storage_manager.vcs_manager.push();
    if push_result.is_err() {
        // TODO in case of push rejection, pull and push again (*)
        // TODO in case of other error (how to differentiate?), repeat push only? (*)
    }
}
