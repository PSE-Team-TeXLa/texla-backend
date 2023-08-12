use std::sync::{Arc, Mutex};
use std::time::Duration;

use debounced::debounced;
use futures::executor::block_on;
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

const WORKSESSION_EVENT_BUFFER_SIZE: usize = 10;

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
        // Optimally this should be using Sender::blocking_send, but that says that the main
        // thread should not be blocked. I did not find a way to put this outside the main
        // tokio::task::spawn_blocking should do exactly what we want, but somehow it does not work
        block_on(async move {
            self.tx.send(WorksessionMessage::Uphold).await.unwrap();
        });
    }

    pub(crate) fn pause(&mut self) {
        block_on(async move {
            self.tx.send(WorksessionMessage::Pause).await.unwrap();
        });
    }

    pub(crate) fn disassemble(&self) {
        self.join_handle.abort();
    }
}

// TODO: this seems redundant with (use end_session instead)
/// [crate::infrastructure::storage_manager::StorageManager::end_worksession_on_quit]
async fn end_worksession(storage_manager: &Arc<Mutex<TexlaStorageManager<GitManager>>>) {
    let storage_manager = storage_manager.lock().unwrap();
    // TODO unwrap every time instead?

    storage_manager.vcs_manager.commit(None);
    storage_manager.vcs_manager.pull();

    println!("Pushing at end of worksession");
    // TODO: this fails most of the time (not always)
    storage_manager.vcs_manager.push();
}
