use std::fs;
use std::ops::Range;
use std::path::{PathBuf, MAIN_SEPARATOR_STR};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chumsky::prelude::*;

use crate::infrastructure::errors::InfrastructureError;
use crate::infrastructure::vcs_manager::{GitManager, MergeConflictHandler, VcsManager};

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
    fn save(&mut self, latex_single_string: String) -> Result<(), InfrastructureError>;
    fn end_session(&mut self) -> Result<(), InfrastructureError>;
}

pub struct TexlaStorageManager<V>
where
    V: VcsManager,
{
    vcs_manager: V,
    directory_change_handler: Option<Arc<Mutex<dyn DirectoryChangeHandler>>>,
    merge_conflict_handler: Option<Arc<Mutex<dyn MergeConflictHandler>>>,
    main_file: String,
    // TODO use Path instead of String
    pull_timer_running: bool,
    worksession_timer_running: bool,
}

impl<V> TexlaStorageManager<V>
where
    V: VcsManager,
{
    const FILE_BEGIN_MARK: &'static str = "% TEXLA FILE BEGIN";
    const FILE_END_MARK: &'static str = "% TEXLA FILE END";
    const LATEX_PATH_SEPARATOR: &'static str = "/";

    pub fn new(vcs_manager: V, main_file: String) -> Self {
        // TODO use Path instead of String for main_file
        Self {
            vcs_manager,
            directory_change_handler: None,
            merge_conflict_handler: None,
            main_file,
            pull_timer_running: false,
            worksession_timer_running: false,
        }
    }

    fn lf(s: String) -> String {
        s.replace("\r\n", "\n")
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
        take_until(just::<_, _, Simple<char>>("\\input"))
            .map_with_span(|_, span| -> usize { span.end() - 6 })
            // "\\input".to_string().len() = 6
            .then(Self::curly_braces_parser())
            .map_with_span(|(start, text), span| -> (String, Range<usize>) {
                (text, start..span.end())
            })
            .boxed()
    }

    fn get_paths(&self, input_path: String) -> (PathBuf, PathBuf) {
        // replace separators in path (LaTeX und Unix use forward slashes, Windows uses backslashes)
        let mut path = PathBuf::from({
            if MAIN_SEPARATOR_STR == Self::LATEX_PATH_SEPARATOR {
                input_path
            } else {
                input_path.replace(Self::LATEX_PATH_SEPARATOR, MAIN_SEPARATOR_STR)
            }
        });

        // append file extension (optional in LaTeX)
        if path.extension().is_none() {
            path.set_extension("tex");
        }

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
                .join(path)
                .canonicalize()
                .expect("Could not create absolute path");
        } else {
            path_abs_os = path.canonicalize().expect("Invalid path given");
            path_rel = path
                .strip_prefix(main_file_directory)
                .expect("Could not create relative path")
                .to_path_buf();
            // TODO also support paths that are no child of 'main_file_directory'?
        }

        // replace separators in path again
        let path_rel_latex = {
            if MAIN_SEPARATOR_STR == Self::LATEX_PATH_SEPARATOR {
                path_rel
            } else {
                PathBuf::from(
                    path_rel
                        .to_str()
                        .unwrap()
                        .replace(MAIN_SEPARATOR_STR, Self::LATEX_PATH_SEPARATOR),
                )
            }
        };

        (path_abs_os, path_rel_latex)
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
        // TODO after VS: start async timer-based background tasks and start DirectoryChangeHandler
        // we probably want to use tokio::spawn() here
    }

    fn remote_url(&self) -> Option<&String> {
        self.vcs_manager.remote_url()
    }

    fn multiplex_files(&self) -> Result<String, InfrastructureError> {
        // define parser for '\input{...}'
        let parser = Self::latex_input_parser();

        // start with content of main file as latex single string
        let mut latex_single_string =
            fs::read_to_string(&self.main_file).expect("Could not read file");

        loop {
            // search for '\input{...}'
            let parse_res = parser.parse(latex_single_string.clone());
            if parse_res.is_err() {
                break;
            }

            // get paths
            let (path, path_range) = parse_res.unwrap();
            let (path_abs_os, path_rel_latex) = self.get_paths(path);

            // read content from inputted file
            let input_text = fs::read_to_string(path_abs_os).expect("Could not read file");

            // replace '\input{...}' in string with file content surrounded by begin and end marks
            latex_single_string.replace_range(
                path_range,
                &format!(
                    "{} {{{}}}\n{}\n{}",
                    // TODO insert path after file end mark as well?
                    Self::FILE_BEGIN_MARK,
                    path_rel_latex.to_str().unwrap(),
                    input_text,
                    Self::FILE_END_MARK
                ),
            );
        }

        // return final latex single string with uniform line separators (LF only)
        Ok(Self::lf(latex_single_string))
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

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::infrastructure::storage_manager::{StorageManager, TexlaStorageManager};
    use crate::infrastructure::vcs_manager::GitManager;

    fn lf(s: String) -> String {
        s.replace("\r\n", "\n")
    }

    #[test]
    fn multiplex_files() {
        let main_file = "latex_test_files/latex_with_inputs.tex".to_string();
        let vcs_manager = GitManager::new(main_file.clone());
        let storage_manager = TexlaStorageManager::new(vcs_manager, main_file);

        assert_eq!(
            lf(storage_manager.multiplex_files().unwrap()),
            lf(fs::read_to_string("latex_test_files/latex_single_string.tex").unwrap())
        )
    }
}
