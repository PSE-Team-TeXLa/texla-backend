use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use debounced::debounced;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::sync::mpsc::channel;
use tokio::task::JoinHandle;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;

use crate::infrastructure::storage_manager::TexlaStorageManager;
use crate::infrastructure::vcs_manager::GitManager;

/// The time we wait for file changes to settle before notifying the frontend
const DIRECTORY_WATCHER_DEBOUNCE_DELAY: Duration = Duration::from_millis(100);

pub(crate) struct DirectoryWatcher {
    path: PathBuf,
    watcher: RecommendedWatcher,
    passer_join_handle: JoinHandle<()>,
}

impl DirectoryWatcher {
    pub(crate) fn new(
        path: PathBuf,
        storage_manager: Arc<Mutex<TexlaStorageManager<GitManager>>>,
    ) -> Result<Self, notify::Error> {
        println!("Starting directory watcher for {path:?}");
        let handler = storage_manager
            .lock()
            .unwrap()
            .directory_change_handler
            .as_ref()
            .expect("Starting directory watcher without directory change handler")
            .clone();

        let (tx, rx) = channel(10);

        let mut watcher = RecommendedWatcher::new(
            move |res| {
                tx.blocking_send(res).unwrap();
            },
            Config::default(),
        )?;

        watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

        let stream = ReceiverStream::new(rx);

        let sm = storage_manager.clone();
        let filtered = stream.filter(move |res: &notify::Result<Event>| -> bool {
            match res {
                Ok(event) => Self::is_notify_event_interesting(&sm, event),
                Err(err) => {
                    eprintln!("watch error (not propagating): {err:?}");
                    false
                }
            }
        });

        let mut debounced = debounced(filtered, DIRECTORY_WATCHER_DEBOUNCE_DELAY);

        let passer_join_handle = tokio::spawn(async move {
            while let Some(_event) = debounced.next().await {
                println!("Detected foreign change (debounced)");
                handler.lock().unwrap().handle_directory_change();
            }
        });

        Ok(Self {
            path,
            watcher,
            passer_join_handle,
        })
    }

    fn is_notify_event_interesting(
        sm: &Arc<Mutex<TexlaStorageManager<GitManager>>>,
        event: &Event,
    ) -> bool {
        if sm.lock().unwrap().writing || sm.lock().unwrap().waiting_for_frontend {
            // this is our own change => ignore it
            // or we are still waiting for the frontend to finish its operation => ignore it
            false
        } else {
            let only_git_files = event
                .paths
                .iter()
                .all(|p| p.components().any(|c| c.as_os_str() == ".git"));

            !only_git_files
        }
    }

    pub(crate) fn disassemble(&mut self) {
        self.watcher
            .unwatch(self.path.as_path())
            .expect("Could not unwatch directory");
        self.passer_join_handle.abort();
    }
}
