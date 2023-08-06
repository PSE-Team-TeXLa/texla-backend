use std::fs;
use std::ops::Range;
use std::path::Path;
use std::path::{PathBuf, MAIN_SEPARATOR_STR};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use chumsky::prelude::*;
use debounced::debounced;
use futures::future::Ready;
use futures::{channel::mpsc::channel, future, SinkExt, StreamExt};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use tokio::task::JoinHandle;
use tokio::time::sleep;

use crate::infrastructure::errors::{InfrastructureError, VcsError};
use crate::infrastructure::vcs_manager::{GitManager, MergeConflictHandler, VcsManager};

type TexlaFileParserResult = (String, Range<usize>, Range<usize>);

/// The time we wait for file changes to settle before notifying the frontend
const DIRECTORY_WATCHER_DEBOUNCE_DELAY: Duration = Duration::from_millis(100);
/// The time notify is allowed to take for picking up our own file changes and reporting them
const NOTIFY_DELAY_TOLERANCE: Duration = Duration::from_millis(100);

#[async_trait]
pub trait StorageManager {
    fn attach_handlers(
        &mut self,
        dc_handler: Arc<Mutex<dyn DirectoryChangeHandler>>,
        mc_handler: Arc<Mutex<dyn MergeConflictHandler>>,
    );
    async fn start(this: Arc<Mutex<Self>>);
    fn remote_url(&self) -> Option<&String>;
    fn multiplex_files(&self) -> Result<String, InfrastructureError>;
    fn stop_timers(&mut self);
    async fn save(
        this: Arc<Mutex<Self>>,
        latex_single_string: String,
    ) -> Result<(), InfrastructureError>;
    fn end_session(&mut self) -> Result<(), VcsError>;
}

pub struct TexlaStorageManager<V>
where
    V: VcsManager + Send + Sync,
{
    vcs_manager: V,
    directory_change_handler: Option<Arc<Mutex<dyn DirectoryChangeHandler>>>,
    merge_conflict_handler: Option<Arc<Mutex<dyn MergeConflictHandler>>>,
    // TODO use tuple (directory: PathBuf, filename: PathBuf) instead of String for main_file
    main_file: String,
    pull_timer: Option<JoinHandle<()>>,
    worksession_timer: Option<JoinHandle<()>>,
    // TODO: this may become redundant with the pull_timer being active or not
    writing: bool,
}

impl<V> TexlaStorageManager<V>
where
    V: VcsManager + Send + Sync,
{
    const LATEX_FILE_EXTENSION: &'static str = "tex";
    const LATEX_PATH_SEPARATOR: &'static str = "/";
    const FILE_BEGIN_MARK: &'static str = "% TEXLA FILE BEGIN ";
    const FILE_END_MARK: &'static str = "% TEXLA FILE END ";
    const INPUT_COMMAND: &'static str = "\\input";

    pub fn new(vcs_manager: V, main_file: String) -> Self {
        // TODO use tuple (directory: PathBuf, filename: PathBuf) instead of String for main_file
        Self {
            vcs_manager,
            directory_change_handler: None,
            merge_conflict_handler: None,
            main_file,
            pull_timer: None,
            worksession_timer: None,
            writing: false,
        }
    }

    fn lf(s: String) -> String {
        s.replace("\r\n", "\n")
    }

    fn char_len(s: &str) -> usize {
        s.chars().count()
    }

    fn char_range_to_byte_range(s: &str, r: Range<usize>) -> Range<usize> {
        let start = s
            .char_indices()
            .nth(r.start)
            .map(|(index, _)| index)
            .unwrap();
        let end = s.char_indices().nth(r.end).map(|(index, _)| index).unwrap();
        start..end
    }

    fn curly_braces_parser() -> BoxedParser<'static, char, String, Simple<char>> {
        none_of::<_, _, Simple<char>>("}")
            .repeated()
            .at_least(1)
            .delimited_by(just("{"), just("}"))
            .collect::<String>()
            .boxed()
    }

    fn latex_input_parser() -> BoxedParser<'static, char, (String, Range<usize>), Simple<char>> {
        take_until(just::<_, _, Simple<char>>(Self::INPUT_COMMAND))
            .map_with_span(|_, span| -> usize {
                span.end() - Self::char_len(Self::INPUT_COMMAND) // = input_start
            })
            // TODO allow white spaces (but no newlines?) around curly braces?
            .then(Self::curly_braces_parser())
            .map_with_span(|(start, path), span| -> (String, Range<usize>) {
                (path, start..span.end()) // span.end() = input_end
            })
            .boxed()
    }

    fn texla_file_parser() -> BoxedParser<'static, char, TexlaFileParserResult, Simple<char>> {
        recursive(|input| {
            take_until(just(Self::FILE_BEGIN_MARK))
                .map_with_span(|(_, _), span: Range<usize>| -> usize {
                    span.end() - Self::char_len(Self::FILE_BEGIN_MARK) // = input_start
                })
                .then(Self::curly_braces_parser())
                .map_with_span(
                    |(input_start, path_begin), span| -> (String, usize, usize) {
                        (path_begin, input_start, span.end() + 1) // span.end() + 1 = text_start
                    },
                )
                .then_with(move |(path_begin, input_start, text_start)| {
                    take_until(
                        input.clone().or(just(Self::FILE_END_MARK)
                            .map_with_span(|_, span: Range<usize>| -> usize {
                                span.start() - 1 // = text_end
                            })
                            .then(Self::curly_braces_parser())
                            .map_with_span(
                                move |(text_end, path_end), span| -> (String, usize, usize) {
                                    (path_end, span.end(), text_end) // span.end() = input_end
                                },
                            )
                            .try_map(move |(path_end, input_end, text_end), span| {
                                if path_begin != path_end {
                                    Err(Simple::custom(span, "Invalid latex single string"))
                                } else {
                                    Ok((
                                        path_begin.clone(),
                                        input_start..input_end,
                                        text_start..text_end,
                                    ))
                                }
                            })),
                    )
                })
                .map(|(_, result)| result)
        })
        .boxed()
    }

    fn get_paths(&self, input_path: String) -> (PathBuf, PathBuf) {
        // replace separators in path (LaTeX und Unix use forward slashes, Windows uses backslashes)
        // and set file extension (optional in LaTeX)
        let path = PathBuf::from({
            if MAIN_SEPARATOR_STR == Self::LATEX_PATH_SEPARATOR {
                input_path
            } else {
                input_path.replace(Self::LATEX_PATH_SEPARATOR, MAIN_SEPARATOR_STR)
            }
        })
        .with_extension(Self::LATEX_FILE_EXTENSION);

        // get relative and absolute path
        let main_file_directory = PathBuf::from(&self.main_file)
            .parent()
            .expect("Could not find parent directory")
            .to_path_buf();
        let path_abs_os; // absolute path, platform-dependent slashes
        let path_rel; // relative path, converted to path_rel_latex with forward slashes

        if path.is_relative() {
            path_rel = path.clone();
            path_abs_os = main_file_directory
                .canonicalize()
                .expect("Could not create absolute path")
                .join(path);
        } else {
            path_abs_os = path.clone();
            path_rel = path
                .strip_prefix(main_file_directory)
                .expect("Could not create relative path")
                .to_path_buf();
            // TODO also support paths that are no child of 'main_file_directory'?
        }

        // replace separators in path and remove file extension again
        let path_rel_latex = {
            if MAIN_SEPARATOR_STR == Self::LATEX_PATH_SEPARATOR {
                path_rel.with_extension("")
            } else {
                PathBuf::from(
                    path_rel
                        .with_extension("")
                        .to_str()
                        .unwrap()
                        .replace(MAIN_SEPARATOR_STR, Self::LATEX_PATH_SEPARATOR),
                )
            }
        };

        (path_abs_os, path_rel_latex)
    }

    // TODO: use this everywhere. Maybe make it even more global
    fn main_file_directory(&self) -> PathBuf {
        PathBuf::from(&self.main_file)
            .parent()
            .expect("Could not find parent directory")
            .to_path_buf()
    }

    fn start_timers(_this: Arc<Mutex<Self>>) {
        // TODO (or after refactor)
    }

    async fn watch<P: AsRef<Path>>(
        path: P,
        storage_manager: Arc<Mutex<Self>>,
        handler: Arc<Mutex<dyn DirectoryChangeHandler>>,
    ) -> notify::Result<()> {
        // TODO: any reason for buffer size only 1?
        let (mut tx, rx) = channel(1);

        let mut watcher = RecommendedWatcher::new(
            move |res| {
                // TODO: block_on should never be used inside a tokio runtime
                futures::executor::block_on(async {
                    tx.send(res).await.unwrap();
                })
            },
            Config::default(),
        )?;

        watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;

        // TODO: save watcher in storage manager; unwatch when socket disconnects

        // TODO: refactor into own function
        // filter the events for foreign events
        // FIXME: received a "Detected foreign change" from a change in the frontend
        let filtered = rx.filter_map(|res| -> Ready<Option<Event>> {
            future::ready::<Option<Event>>(match res {
                Ok(event) => {
                    // TODO: we also want to ignore changes, when the frontend is currently active
                    if storage_manager.lock().unwrap().writing {
                        // this is our own change => ignore it
                        None
                    } else {
                        let only_git_files = event
                            .paths
                            .iter()
                            .all(|p| p.to_str().expect("non UTF-8 path").contains(".git"));

                        if only_git_files {
                            // these were only git changes => ignore them
                            None
                        } else {
                            Some(event)
                        }
                    }
                }
                Err(err) => {
                    eprintln!("watch error (not propagating): {:?}", err);
                    None
                }
            })
        });

        let mut debounced = debounced(filtered, DIRECTORY_WATCHER_DEBOUNCE_DELAY);

        while let Some(_event) = debounced.next().await {
            println!("Detected foreign change (debounced)");
            handler.lock().unwrap().handle_directory_change();
        }

        Ok(())
    }
}

#[async_trait]
impl StorageManager for TexlaStorageManager<GitManager> {
    fn attach_handlers(
        &mut self,
        dc_handler: Arc<Mutex<dyn DirectoryChangeHandler>>,
        mc_handler: Arc<Mutex<dyn MergeConflictHandler>>,
    ) {
        self.directory_change_handler = Some(dc_handler);
        self.merge_conflict_handler = Some(mc_handler);
    }

    async fn start(this: Arc<Mutex<Self>>) {
        // TODO get duration intervals from CLI arguments
        let pull_duration = Duration::from_millis(500);
        let worksession_duration = Duration::from_millis(5000);

        let mut guard_outer = this.lock().unwrap();
        // TODO unwrap every time instead?

        // TODO error handling (report errors via callback function in socket?)
        let that = this.clone();
        guard_outer.pull_timer = Some(tokio::spawn(async move {
            loop {
                let pull_result = that.lock().unwrap().vcs_manager.pull();
                if pull_result.is_err() {
                    // TODO in case of merge conflict, inform user
                    // TODO in case of other error (how to differentiate?), repeat pull only? (*)
                }

                sleep(pull_duration).await;
            }
        }));

        let that = this.clone();
        guard_outer.worksession_timer = Some(tokio::spawn(async move {
            sleep(worksession_duration).await;

            let guard_inner = that.lock().unwrap();
            // TODO unwrap every time instead?

            let commit_result = guard_inner.vcs_manager.commit(None);
            if commit_result.is_err() {
                // TODO in case of error, repeat commit? (*)
            }

            let pull_result = guard_inner.vcs_manager.pull();
            if pull_result.is_err() {
                // TODO in case of merge conflict, inform user
                // TODO in case of other error (how to differentiate?), repeat pull only? (*)
            }

            let push_result = guard_inner.vcs_manager.push();
            if push_result.is_err() {
                // TODO in case of push rejection, pull and push again (*)
                // TODO in case of other error (how to differentiate?), repeat push only? (*)
            }
        }));

        // TODO (*): inform user after several unsuccessful tries
        //  (maximum number of repetitions stored in a constant or as CLI argument)

        // TODO start DirectoryChangeHandler
        // TODO after VS: start async timer-based background tasks and start DirectoryChangeHandler
        // we probably want to use tokio::spawn() here

        let sm = this.clone();
        tokio::spawn(async move {
            // TODO: do this outside the task
            let (path, handler) = {
                let sm = sm.lock().unwrap();
                let path = sm.main_file_directory();
                println!("Starting directory watcher for {:?}", path);
                let handler = sm
                    .directory_change_handler
                    .as_ref()
                    .expect("Starting directory watcher without directory change handler")
                    .clone();
                (path, handler)
            };
            if let Err(e) = Self::watch(path, sm, handler).await {
                println!("error: {:?}", e)
            }
        });
    }

    fn remote_url(&self) -> Option<&String> {
        self.vcs_manager.remote_url()
    }

    fn multiplex_files(&self) -> Result<String, InfrastructureError> {
        let parser = Self::latex_input_parser();
        let mut latex_single_string =
            fs::read_to_string(&self.main_file).expect("Could not read file");

        loop {
            // TODO use regex instead of chumsky to search inputs
            // TODO replace inputs recursively
            let parse_res = parser.parse(latex_single_string.clone());
            if parse_res.is_err() {
                break;
            }

            let (path, path_char_range) = parse_res.unwrap();
            let (path_abs_os, path_rel_latex) = self.get_paths(path);

            // convert range to handle non-ASCII characters correctly
            let path_byte_range =
                Self::char_range_to_byte_range(&latex_single_string, path_char_range);

            let input_text = fs::read_to_string(path_abs_os).expect("Could not read file");

            // replace '\input{...}' in string with file content surrounded by begin and end marks
            let path_str = path_rel_latex.to_str().unwrap();
            latex_single_string.replace_range(
                path_byte_range,
                &format!(
                    "{}{{{}}}\n{}\n{}{{{}}}",
                    Self::FILE_BEGIN_MARK,
                    path_str,
                    input_text,
                    Self::FILE_END_MARK,
                    path_str
                ),
            );
        }

        Ok(Self::lf(latex_single_string))
    }

    fn stop_timers(&mut self) {
        if let Some(handle) = &self.pull_timer {
            handle.abort();
            self.pull_timer = None;
        }

        if let Some(handle) = &self.worksession_timer {
            handle.abort();
            self.worksession_timer = None;
        }
    }

    // TODO: problem: this storage manager could be used to perform multiple saves simultaneously
    async fn save(
        this: Arc<Mutex<Self>>,
        mut latex_single_string: String,
    ) -> Result<(), InfrastructureError> {
        // define parser for % TEXLA FILE BEGIN ...'
        {
            let parser = Self::texla_file_parser();

            // TODO: get rid of some of the locks, but do not lock the storage_manager for too long!
            this.lock().unwrap().writing = true;

            loop {
                let parse_res = parser.parse(latex_single_string.clone());
                if parse_res.is_err() {
                    break;
                }

                let (path, input_char_range, text_char_range) = parse_res.unwrap();
                let (path_abs_os, path_rel_latex) = this.lock().unwrap().get_paths(path);

                // convert ranges to handle non-ASCII characters correctly
                let input_byte_range =
                    Self::char_range_to_byte_range(&latex_single_string, input_char_range);
                let text_byte_range =
                    Self::char_range_to_byte_range(&latex_single_string, text_char_range);

                fs::write(path_abs_os, &latex_single_string[text_byte_range])
                    .expect("Could not write file");

                // replace '% TEXLA FILE BEGIN ... % TEXLA FILE END' in string with '\input{...}'
                latex_single_string.replace_range(
                    input_byte_range,
                    &format!(
                        "{}{{{}}}",
                        Self::INPUT_COMMAND,
                        path_rel_latex.to_str().unwrap()
                    ),
                )
            }

            fs::write(&this.lock().unwrap().main_file.clone(), latex_single_string)
                .expect("Could not write file");
        }

        // this is frankly needed, because notify does not pick up all changes immediately
        sleep(NOTIFY_DELAY_TOLERANCE).await;
        this.lock().unwrap().writing = false;

        // TODO IMPORTANT: start timers

        Ok(())
    }

    fn end_session(&mut self) -> Result<(), VcsError> {
        // TODO after VS: stop async timer-based background tasks and stop DirectoryChangeHandler

        // don't call save() here since you can't quit (i.e. end the session) with unsaved changes

        self.stop_timers();

        self.vcs_manager.commit(None)?;
        self.vcs_manager.push()
    }
}

pub trait DirectoryChangeHandler: Send + Sync {
    fn handle_directory_change(&mut self);
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::{Arc, Mutex};

    use crate::infrastructure::storage_manager::{StorageManager, TexlaStorageManager};
    use crate::infrastructure::vcs_manager::GitManager;

    fn lf(s: String) -> String {
        s.replace("\r\n", "\n")
    }

    #[test]
    fn multiplex_files() {
        let main_file = "test_resources/latex/with_inputs.tex".to_string();
        // TODO replace separator?
        let vcs_manager = GitManager::new(main_file.clone());
        let storage_manager = TexlaStorageManager::new(vcs_manager, main_file);

        assert_eq!(
            lf(storage_manager.multiplex_files().unwrap()),
            lf(fs::read_to_string("test_resources/latex/latex_single_string.txt").unwrap())
        )
    }

    #[test]
    fn multiplex_files_huge() {
        let main_file = "test_resources/latex/with_inputs_huge.tex".to_string();
        // TODO replace separator?
        let vcs_manager = GitManager::new(main_file.clone());
        let storage_manager = TexlaStorageManager::new(vcs_manager, main_file);

        assert_eq!(
            lf(storage_manager.multiplex_files().unwrap()),
            lf(fs::read_to_string("test_resources/latex/latex_single_string_huge.txt").unwrap())
        )
    }

    #[tokio::test]
    async fn save() {
        // rebuild test directory
        fs::remove_dir_all("test_resources/latex/out").ok();
        fs::create_dir_all("test_resources/latex/out/sections/section2")
            .expect("Could not create directory");

        let main_file = "test_resources/latex/out/with_inputs.tex".to_string();
        // TODO replace separator?
        let vcs_manager = GitManager::new(main_file.clone());
        let storage_manager = TexlaStorageManager::new(vcs_manager, main_file);
        let latex_single_string =
            lf(fs::read_to_string("test_resources/latex/latex_single_string.txt").unwrap());

        StorageManager::save(Arc::new(Mutex::new(storage_manager)), latex_single_string)
            .await
            .unwrap();

        assert_eq!(
            lf(fs::read_to_string("test_resources/latex/with_inputs.tex").unwrap()),
            lf(fs::read_to_string("test_resources/latex/out/with_inputs.tex").unwrap())
        );
        assert_eq!(
            lf(fs::read_to_string("test_resources/latex/sections/section1.tex").unwrap()),
            lf(fs::read_to_string("test_resources/latex/out/sections/section1.tex").unwrap())
        );
        assert_eq!(
            lf(fs::read_to_string("test_resources/latex/sections/section2.tex").unwrap()),
            lf(fs::read_to_string("test_resources/latex/out/sections/section2.tex").unwrap())
        );
        assert_eq!(
            lf(
                fs::read_to_string("test_resources/latex/sections/section2/subsection1.tex")
                    .unwrap()
            ),
            lf(
                fs::read_to_string("test_resources/latex/out/sections/section2/subsection1.tex")
                    .unwrap()
            )
        );
    }
}
