use std::fs;
use std::ops::Range;
use std::path::{PathBuf, MAIN_SEPARATOR_STR};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use async_trait::async_trait;
use chumsky::prelude::*;
use tokio::time::sleep;
use tracing::debug;

use ast::latex_constants::*;
use ast::texla_constants::*;

use crate::infrastructure::dir_watcher::DirectoryWatcher;
use crate::infrastructure::errors::InfrastructureError;
use crate::infrastructure::file_path::FilePath;
use crate::infrastructure::pull_timer::PullTimerManager;
use crate::infrastructure::vcs_manager::{GitErrorHandler, GitManager, VcsManager};
use crate::infrastructure::work_session::WorksessionManager;

#[async_trait]
pub trait StorageManager {
    fn attach_handlers(
        &mut self,
        dc_handler: Arc<RwLock<dyn DirectoryChangeHandler>>,
        ge_handler: Arc<RwLock<dyn GitErrorHandler>>,
    );
    async fn start(this: Arc<Mutex<Self>>) -> Result<(), InfrastructureError>;
    fn remote_url(&self) -> Option<&String>;
    fn multiplex_files(&self) -> Result<String, InfrastructureError>;
    fn wait_for_frontend(&mut self);
    fn frontend_aborted(&mut self);
    async fn save(
        this: Arc<Mutex<Self>>,
        latex_single_string: String,
    ) -> Result<(), InfrastructureError>;
    fn end_worksession(&mut self);
    fn disassemble(&mut self);
}

pub struct TexlaStorageManager<V>
where
    V: VcsManager,
{
    pub(super) vcs_manager: V,
    pub(crate) directory_change_handler: Option<Arc<RwLock<dyn DirectoryChangeHandler>>>,
    main_file: FilePath,
    pull_timer_manager: Option<PullTimerManager>,
    pub(crate) pull_interval: u64,
    worksession_manager: Option<WorksessionManager>,
    pub(crate) worksession_interval: u64,
    dir_watcher: Option<DirectoryWatcher>,
    pub(crate) writing: bool,
    pub(crate) waiting_for_frontend: bool,
    notify_delay: u64,
}

impl TexlaStorageManager<GitManager> {
    pub fn new(
        vcs_manager: GitManager,
        main_file: FilePath,
        pull_interval: u64,
        worksession_interval: u64,
        notify_delay: u64,
    ) -> Self {
        Self {
            vcs_manager,
            directory_change_handler: None,
            main_file,
            pull_timer_manager: None,
            pull_interval,
            worksession_manager: None,
            worksession_interval,
            dir_watcher: None,
            writing: false,
            waiting_for_frontend: false,
            notify_delay,
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

    fn curly_brackets_parser() -> BoxedParser<'static, char, String, Simple<char>> {
        none_of::<_, _, Simple<char>>("}")
            .repeated()
            .at_least(1)
            .delimited_by(just("{"), just("}"))
            .collect::<String>()
            .boxed()
    }

    fn latex_input_parser() -> BoxedParser<'static, char, (String, Range<usize>), Simple<char>> {
        take_until(just::<_, _, Simple<char>>(INPUT))
            .map_with_span(|_, span| -> usize {
                span.end() - Self::char_len(INPUT) // = input_start
            })
            .then(Self::curly_brackets_parser())
            .map_with_span(|(start, path), span| -> (String, Range<usize>) {
                (path, start..span.end()) // span.end() = input_end
            })
            .boxed()
    }

    fn find_texla_file_marks(string: &str) -> Option<(String, Range<usize>, Range<usize>)> {
        let end_start = string.find(FILE_END_MARK)?;
        let (path, end_end) = {
            let string = &string[end_start + FILE_END_MARK.len()..];
            if !string.starts_with(TEXLA_COMMENT_DELIMITER_LEFT) {
                return None;
            }
            let brace_close = string.find(TEXLA_COMMENT_DELIMITER_RIGHT)?;
            let path = string[1..brace_close].to_string();
            (path, end_start + FILE_END_MARK.len() + brace_close + 1)
        };

        let begin_mark = format!("{FILE_BEGIN_MARK}{{{path}}}");
        let begin_start = string[..end_start].rfind(&begin_mark)?;
        let begin_end = begin_start + begin_mark.len();

        // This assumes newlines between markers and content, which is okay, because we only
        // process our own stringification results here.
        Some((path, begin_start..end_end, begin_end + 1..end_start))
    }

    fn get_paths(&self, input_path: String) -> (PathBuf, PathBuf) {
        // replace separators in path (LaTeX und Unix use forward slashes, Windows uses backslashes)
        // and set file extension (optional in LaTeX)
        let path = PathBuf::from({
            if MAIN_SEPARATOR_STR == LATEX_PATH_SEPARATOR {
                input_path
            } else {
                input_path.replace(LATEX_PATH_SEPARATOR, MAIN_SEPARATOR_STR)
            }
        })
        .with_extension(LATEX_FILE_EXTENSION);

        // get absolute path for OS and transform path for LaTeX
        let path_abs_os = if path.is_relative() {
            self.main_file
                .directory
                .canonicalize() // this will use platform-dependent slashes
                .expect("Could not create absolute path")
                .join(path.clone())
        } else {
            path.clone()
        };

        // We normally want relative paths in LaTeX but we don't convert given absolute paths to
        // relative ones as this might be unintended by the user. Though, 'path_latex' is just a
        // copy of 'path' with replaced path separators and no file extension.

        let path_latex = {
            if MAIN_SEPARATOR_STR == LATEX_PATH_SEPARATOR {
                path.with_extension("")
            } else {
                PathBuf::from(
                    path.with_extension("")
                        .to_str()
                        .unwrap()
                        .replace(MAIN_SEPARATOR_STR, LATEX_PATH_SEPARATOR),
                )
            }
        };

        (path_abs_os, path_latex)
    }

    fn pull_timer_manager(&mut self) -> &mut PullTimerManager {
        self.pull_timer_manager
            .as_mut()
            .expect("Pull timer manager not initialized")
    }
    fn worksession_manager(&mut self) -> &mut WorksessionManager {
        self.worksession_manager
            .as_mut()
            .expect("Worksession manager not initialized")
    }
    fn dir_watcher(&mut self) -> &mut DirectoryWatcher {
        self.dir_watcher
            .as_mut()
            .expect("Directory watcher not initialized")
    }
}

#[async_trait]
impl StorageManager for TexlaStorageManager<GitManager> {
    fn attach_handlers(
        &mut self,
        dc_handler: Arc<RwLock<dyn DirectoryChangeHandler>>,
        ge_handler: Arc<RwLock<dyn GitErrorHandler>>,
    ) {
        self.directory_change_handler = Some(dc_handler);
        self.vcs_manager.attach_handler(ge_handler);
    }

    async fn start(this: Arc<Mutex<Self>>) -> Result<(), InfrastructureError> {
        let directory = this.lock().unwrap().main_file.directory.clone();
        let directory_watcher = DirectoryWatcher::new(directory, this.clone())?;

        let mut sm = this.lock().unwrap();
        sm.pull_timer_manager = Some(PullTimerManager::new(this.clone()));
        sm.worksession_manager = Some(WorksessionManager::new(
            this.clone(),
            sm.worksession_interval,
        ));
        sm.dir_watcher = Some(directory_watcher);

        sm.pull_timer_manager().activate();

        Ok(())
    }

    fn remote_url(&self) -> Option<&String> {
        self.vcs_manager.remote_url()
    }

    fn multiplex_files(&self) -> Result<String, InfrastructureError> {
        // To further improve performance, regex could be used for searching inputs instead of
        // Chumsky. Additionally, inputs could be replaced recursively.
        let parser = Self::latex_input_parser();
        let mut latex_single_string =
            fs::read_to_string(&self.main_file.path).expect("Could not read file");

        loop {
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
                    "{FILE_BEGIN_MARK}{{{path_str}}}\n{input_text}\n{FILE_END_MARK}{{{path_str}}}"
                ),
            );
        }

        Ok(Self::lf(latex_single_string))
    }

    fn wait_for_frontend(&mut self) {
        self.waiting_for_frontend = true;
        self.pull_timer_manager().deactivate();
        self.worksession_manager().pause();
    }

    fn frontend_aborted(&mut self) {
        self.waiting_for_frontend = false;
        self.pull_timer_manager().activate();
    }

    // note: This method could be accidentally used to perform multiple saves simultaneously.
    async fn save(
        this: Arc<Mutex<Self>>,
        mut latex_single_string: String,
    ) -> Result<(), InfrastructureError> {
        // To further improve performance, async file I/O could be used.
        {
            this.lock().unwrap().writing = true;

            loop {
                let find_res = TexlaStorageManager::find_texla_file_marks(&latex_single_string);
                if find_res.is_none() {
                    break;
                }

                let (path, input_byte_range, text_byte_range) = find_res.unwrap();
                let (path_abs_os, path_rel_latex) = this.lock().unwrap().get_paths(path);
                debug!("writing file: {:?}", path_abs_os);
                debug!("string length: {}", latex_single_string.len());
                debug!("input range: {:?} bytes", input_byte_range);
                debug!(
                    "input: {:?}",
                    &latex_single_string[input_byte_range.clone()]
                );
                debug!("text range: {:?} bytes", text_byte_range);
                debug!("text: {:?}", &latex_single_string[text_byte_range.clone()]);

                fs::write(path_abs_os, &latex_single_string[text_byte_range])
                    .expect("Could not write file");

                // replace '% TEXLA FILE BEGIN ... % TEXLA FILE END' in string with '\input{...}'
                latex_single_string.replace_range(
                    input_byte_range,
                    &format!("{}{{{}}}", INPUT, path_rel_latex.to_str().unwrap()),
                )
            }

            fs::write(&this.lock().unwrap().main_file.path, latex_single_string)
                .expect("Could not write file");
        }

        // this is frankly needed, because notify does not pick up all changes immediately
        let duration = Duration::from_millis(this.lock().unwrap().notify_delay);
        sleep(duration).await;
        this.lock().unwrap().writing = false;

        let mut sm = this.lock().unwrap();
        sm.pull_timer_manager().activate();
        sm.worksession_manager().start_or_uphold();

        Ok(())
    }

    fn end_worksession(&mut self) {
        // don't call save() here since all changes are already saved at end of worksession

        println!("End of worksession");

        if self.vcs_manager.has_local_changes() {
            self.vcs_manager.commit(None);
            self.vcs_manager.pull();
            self.vcs_manager.push();
        } else {
            println!("Nothing to commit");
        }
    }

    fn disassemble(&mut self) {
        println!("Disassembling, freeing resources...");
        self.worksession_manager().disassemble();
        self.pull_timer_manager().disassemble();
        self.dir_watcher().disassemble();
    }
}

pub trait DirectoryChangeHandler: Send + Sync {
    fn handle_directory_change(&mut self);
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::{Arc, Mutex};

    use ast::texla_ast::TexlaAst;
    use ast::Ast;

    use crate::infrastructure::file_path::FilePath;
    use crate::infrastructure::pull_timer::PullTimerManager;
    use crate::infrastructure::storage_manager::{StorageManager, TexlaStorageManager};
    use crate::infrastructure::vcs_manager::GitManager;
    use crate::infrastructure::work_session::WorksessionManager;

    fn lf(s: String) -> String {
        s.replace("\r\n", "\n")
    }

    #[test]
    fn multiplex_files() {
        let main_file = FilePath::from("test_resources/latex/with_inputs.tex");
        let vcs_manager = GitManager::new(true, main_file.directory.clone());
        let storage_manager = TexlaStorageManager::new(vcs_manager, main_file, 500, 5000, 100);

        let correct_latex_single_string =
            fs::read_to_string("test_resources/latex/latex_single_string.txt").unwrap();
        assert_eq!(
            lf(storage_manager.multiplex_files().unwrap()),
            lf(correct_latex_single_string.clone())
        );

        // check that the latex single string is also parsable
        TexlaAst::from_latex(correct_latex_single_string).unwrap();
    }

    #[test]
    fn multiplex_files_huge() {
        let main_file = FilePath::from("test_resources/latex/with_inputs_huge.tex");
        let vcs_manager = GitManager::new(true, main_file.directory.clone());
        let storage_manager = TexlaStorageManager::new(vcs_manager, main_file, 500, 5000, 100);

        let correct_latex_single_string =
            fs::read_to_string("test_resources/latex/latex_single_string_huge.txt").unwrap();
        assert_eq!(
            lf(storage_manager.multiplex_files().unwrap()),
            lf(correct_latex_single_string.clone())
        );

        // check that the latex single string is also parsable
        TexlaAst::from_latex(correct_latex_single_string).unwrap();
    }

    #[tokio::test]
    async fn save() {
        // rebuild test directory
        fs::remove_dir_all("test_resources/latex/out").ok();
        fs::create_dir_all("test_resources/latex/out/sections/section2")
            .expect("Could not create directory");

        let main_file = FilePath::from("test_resources/latex/out/with_inputs.tex");
        let vcs_manager = GitManager::new(true, main_file.directory.clone());
        let worksession_interval = 5000;
        let storage_manager =
            TexlaStorageManager::new(vcs_manager, main_file, 500, worksession_interval, 100);
        let shared = Arc::new(Mutex::new(storage_manager));
        let latex_single_string =
            lf(fs::read_to_string("test_resources/latex/latex_single_string.txt").unwrap());

        shared.lock().unwrap().pull_timer_manager = Some(PullTimerManager::new(shared.clone()));
        shared.lock().unwrap().worksession_manager = Some(WorksessionManager::new(
            shared.clone(),
            worksession_interval,
        ));

        // this is needed, because we use some blocking calls and you cannot block the main thread
        tokio::spawn(async move {
            StorageManager::save(shared, latex_single_string)
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
                lf(fs::read_to_string(
                    "test_resources/latex/out/sections/section2/subsection1.tex"
                )
                .unwrap())
            );
        });
    }
}
