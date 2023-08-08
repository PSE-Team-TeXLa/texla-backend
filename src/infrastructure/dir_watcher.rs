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
    storage_manager: Arc<Mutex<TexlaStorageManager<GitManager>>>,
    passer_join_handle: JoinHandle<()>,
}

impl DirectoryWatcher {
    pub(crate) fn new(
        storage_manager: Arc<Mutex<TexlaStorageManager<GitManager>>>,
    ) -> Result<Self, notify::Error> {
        let (path, handler) = {
            let sm = storage_manager.lock().unwrap();
            let path = sm.main_file_directory();
            println!("Starting directory watcher for {:?}", path);
            let handler = sm
                .directory_change_handler
                .as_ref()
                .expect("Starting directory watcher without directory change handler")
                .clone();
            (path, handler)
        };

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
                    eprintln!("watch error (not propagating): {:?}", err);
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
            storage_manager,
            passer_join_handle,
        })
    }

    fn is_notify_event_interesting(
        sm: &Arc<Mutex<TexlaStorageManager<GitManager>>>,
        event: &Event,
    ) -> bool {
        // TODO: we also want to ignore changes, when the frontend is currently active

        if sm.lock().unwrap().writing {
            // this is our own change => ignore it
            false
        } else {
            let only_git_files = event
                .paths
                .iter()
                .all(|p| p.to_str().expect("non UTF-8 path").contains(".git"));

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
