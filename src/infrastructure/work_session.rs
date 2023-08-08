use std::sync::{Arc, Mutex};
use std::time::Duration;

use debounced::debounced;
use futures::StreamExt;
use tokio::sync::mpsc::{channel, Sender};
use tokio::task::JoinHandle;
use tokio_stream::wrappers::ReceiverStream;

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

const WORKSESSION_EVENT_BUFFER_SIZE: usize = 2;

impl WorksessionManager {
    pub(crate) fn new(storage_manager: Arc<Mutex<TexlaStorageManager<GitManager>>>) -> Self {
        let (tx, rx) = channel(WORKSESSION_EVENT_BUFFER_SIZE);
        let stream = ReceiverStream::new(rx);
        let mut debounced = debounced(stream, WORKSESSION_LENGTH);

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

    pub(crate) fn start_or_uphold(&mut self) {
        self.tx.blocking_send(WorksessionMessage::Uphold).unwrap();
    }

    pub(crate) fn pause(&mut self) {
        self.tx.blocking_send(WorksessionMessage::Pause).unwrap();
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
    println!("Pushing at end of worksession");
    // TODO: this fails most of the time (not always)
    if push_result.is_err() {
        println!("Push error");
        // TODO in case of push rejection, pull and push again (*)
        // TODO in case of other error (how to differentiate?), repeat push only? (*)
    }
}
