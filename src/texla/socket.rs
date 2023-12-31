use std::process::exit;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use serde::Serialize;
use socketioxide::adapter::LocalAdapter;
use socketioxide::extensions::Ref;
use socketioxide::{Namespace, Socket, SocketIoLayer};
use tokio::time::sleep;
use tower::layer::util::{Identity, Stack};
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

use ast::operation::{JsonOperation, Operation};
use ast::options::StringificationOptions;
use ast::texla_ast::TexlaAst;
use ast::Ast;

use crate::infrastructure::export_manager::ExportManager;
use crate::infrastructure::storage_manager::{StorageManager, TexlaStorageManager};
use crate::infrastructure::vcs_manager::GitManager;
use crate::texla::core::TexlaCore;
use crate::texla::errors::TexlaError;
use crate::texla::state::{SharedTexlaState, TexlaState};

const QUIT_DELAY: Duration = Duration::from_secs(1);

pub type TexlaSocket = Arc<Socket<LocalAdapter>>;

pub fn socket_service(
    core: Arc<RwLock<TexlaCore>>,
) -> ServiceBuilder<Stack<SocketIoLayer<LocalAdapter>, Stack<CorsLayer, Identity>>> {
    let ns = Namespace::builder()
        .add("/", move |socket| handler(socket, core.clone()))
        .build();

    // ServiceBuilder executes layer top to bottom
    ServiceBuilder::new()
        .layer(CorsLayer::permissive())
        .layer(SocketIoLayer::new(ns))
}

// This is a fairly long function, but its complexity is low and its control flow is very linear.
// All actions belong here, as they handle the basics of communication.
// Thus, it would be artificial and useless to split it up into multiple functions.
async fn handler(socket: TexlaSocket, core: Arc<RwLock<TexlaCore>>) {
    println!("Socket connected with ID '{}'", socket.sid);

    let storage_manager = {
        let core = core.read().unwrap();

        let vcs_manager = GitManager::new(core.vcs_enabled, core.main_file.directory.clone());
        TexlaStorageManager::new(
            vcs_manager,
            core.main_file.clone(),
            core.pull_interval,
            core.worksession_interval,
            core.notify_delay,
        )
    };

    let ast = match parse_ast_from_disk(&storage_manager) {
        Ok(ast) => ast,
        Err(err) => {
            println!("Found invalid ast: {err}");
            send(&socket, "error", err).ok();
            return;
            // this will display the error in the frontend
            // the frontend will not receive any further messages
        }
    };

    let state = TexlaState {
        socket: socket.clone(),
        storage_manager: Arc::new(Mutex::new(storage_manager)),
        ast,
    };
    socket.extensions.insert(Arc::new(RwLock::new(state)));

    {
        let mut core = core.write().unwrap();

        if let Some(old_socket) = core.socket.clone() {
            let err = TexlaError {
                message: "This frontend is replaced by another one. \
                Click the TeXLa logo to regain control."
                    .to_string(),
            };
            send(&old_socket, "error", err).ok();
            send(&old_socket, "quit", "quit").ok();

            let state = extract_state(&old_socket);
            let state = state.read().unwrap();
            let mut sm = state.storage_manager.lock().unwrap();
            sm.disassemble();
            println!("Disconnected old socket");
        }

        core.socket = Some(socket.clone());
    }

    let storage_manager_handle = {
        let state_ref = extract_state(&socket);
        let state = state_ref.read().unwrap();

        state
            .storage_manager
            .lock()
            .unwrap()
            .attach_handlers(state_ref.clone(), state_ref.clone());
        StorageManager::start(state.storage_manager.clone())
    };

    {
        let state_ref = extract_state(&socket);
        let state = state_ref.read().unwrap();
        let remote_url = {
            let storage_manager = state.storage_manager.lock().unwrap();
            storage_manager.remote_url().map(|url| url.to_string())
        };

        // initial messages
        send(&socket, "remote_url", remote_url).ok();
        send(&socket, "new_ast", &state.ast).ok();
    }

    socket.on("active", |socket, _: String, _, _| async move {
        let state_ref = extract_state(&socket);
        let state = state_ref.read().unwrap();
        // stop synchronization in order to prevent losing changes
        state.storage_manager.lock().unwrap().wait_for_action();
        println!("Waiting for frontend to finalize operation...");
    });

    socket.on("operation", |socket, json: String, _, _| async move {
        print!("Received operation: ");

        let operation = serde_json::from_str::<JsonOperation>(&json)
            .expect("Got invalid operation from frontend")
            .to_trait_obj();
        println!("{operation:?}");

        let state = extract_state(&socket).clone();
        match perform_and_check_operation(state.clone(), operation).await {
            Ok(()) => {
                send(&socket, "new_ast", &state.read().unwrap().ast).ok();
                println!("Operation was okay");
                println!("Saved changes");
            }
            Err(err) => {
                println!("Operation was not okay: {err}");
                send(&socket, "error", err).ok();
                // send old ast in order to enable frontend to roll back to it
                send(&socket, "new_ast", &state.read().unwrap().ast).ok();
            }
        }
    });

    let core_clone = core.clone();
    socket.on("prepare_export", move |socket, json: String, _, _| {
        let options = serde_json::from_str::<StringificationOptions>(&json)
            .expect("Got invalid options from frontend");
        handle_export(socket, options, core_clone.clone())
    });

    socket.on("quit", |socket, _: String, _, _| async move {
        println!("Received quit");
        {
            let storage_manager = {
                let state_ref = extract_state(&socket);
                let state = state_ref.read().unwrap();
                state.storage_manager.clone()
            };
            let mut storage_manager = storage_manager.lock().unwrap();
            storage_manager.end_worksession();
            storage_manager.disassemble();
        };
        println!("Quitting...");
        send(&socket, "quit", "ok").ok();
        sleep(QUIT_DELAY).await;
        socket.disconnect().ok();
        exit(0);
    });

    if let Err(err) = storage_manager_handle.await {
        send(&socket, "error", TexlaError::from(err)).ok();
    };
}

pub fn parse_ast_from_disk(
    storage_manager: &TexlaStorageManager<GitManager>,
) -> Result<TexlaAst, TexlaError> {
    let latex_single_string = storage_manager.multiplex_files()?;
    let ast = TexlaAst::from_latex(latex_single_string)?;

    // Verify the ast by converting it to latex again.
    // (It should never happen, that our output cannot be parsed.)
    // If operations return ParseErrors even though they should be legal, check this
    // TexlaAst::from_latex(ast.to_latex(Default::default())?)?;

    Ok(ast)
}

fn extract_state(socket: &TexlaSocket) -> Ref<SharedTexlaState> {
    socket.extensions.get::<SharedTexlaState>().unwrap()
}

async fn perform_and_check_operation(
    state: SharedTexlaState,
    operation: Box<dyn Operation<TexlaAst>>,
) -> Result<(), TexlaError> {
    let backup_latex = state.read().unwrap().ast.to_latex(Default::default())?;

    match perform_operation(state.clone(), operation).await {
        Ok(new_ast) => {
            state.write().unwrap().ast = new_ast;
            Ok(())
        }
        Err(err) => {
            let mut state = state.write().unwrap();
            state.ast = TexlaAst::from_latex(backup_latex)?;
            state.storage_manager.lock().unwrap().action_aborted();
            Err(err)
        }
    }
}

async fn perform_operation(
    state: SharedTexlaState,
    operation: Box<dyn Operation<TexlaAst>>,
) -> Result<TexlaAst, TexlaError> {
    let reparsed_ast = {
        let mut locked = state.write().unwrap();
        locked.ast.execute(operation)?;
        let latex_single_string = locked.ast.to_latex(Default::default())?;
        TexlaAst::from_latex(latex_single_string)?
    };
    tokio::spawn(async move {
        if let Err(err) = stringify_and_save(state.clone(), Default::default()).await {
            println!("Error while saving: {err}");
            let state = state.read().unwrap();
            let socket = &state.socket;
            send(socket, "error", err).ok();
        }
    });
    Ok(reparsed_ast)
}

async fn stringify_and_save(
    state: SharedTexlaState,
    options: StringificationOptions,
) -> Result<(), TexlaError> {
    let latex_single_string = state.read().unwrap().ast.to_latex(options)?;
    let storage_manager = state.read().unwrap().storage_manager.clone();
    StorageManager::save(storage_manager, latex_single_string).await?;

    Ok(())
}

// this function is correctly placed here, because it contains coordination and communication
async fn handle_export(
    socket: TexlaSocket,
    options: StringificationOptions,
    core: Arc<RwLock<TexlaCore>>,
) {
    println!("Preparing export with options: {options:?}");
    let state_ref = extract_state(&socket).clone();

    // copy complete AST with all options enabled
    let ast_copy = {
        let state = state_ref.read().unwrap();
        // freeze git actions until the original AST is reverted
        state.storage_manager.lock().unwrap().wait_for_action();
        state_ref.read().unwrap().ast.clone()
    };

    // apply given options to AST
    if let Err(err) = stringify_and_save(state_ref.clone(), options).await {
        send(&socket, "error", err).ok();
        return;
    }

    // export AST to zip file
    let zip_result = core.write().unwrap().export_manager.zip_files();

    // revert original AST with all options enabled
    extract_state(&socket).write().unwrap().ast = ast_copy;

    // send message to frontend
    match zip_result {
        Ok(url) => {
            send(&socket, "export_ready", url).ok();
        }
        Err(err) => {
            send(&socket, "error", TexlaError::from(err)).ok();
        }
    }

    // unfreeze git actions
    let state_ref = extract_state(&socket).clone();
    state_ref
        .read()
        .unwrap()
        .storage_manager
        .lock()
        .unwrap()
        .action_aborted();

    // save reverted AST to local files again
    if let Err(err) = stringify_and_save(state_ref.clone(), Default::default()).await {
        send(&socket, "error", err).ok();
    }
}

pub(crate) fn send(socket: &TexlaSocket, event: &str, data: impl Serialize) -> Result<(), ()> {
    // this only works with a modified main branch of socketioxide (see Cargo.toml)
    // with the upcoming release (after 0.3.0) you could relax this check and instead free
    // resources in an on_disconnect handler (see https://github.com/Totodore/socketioxide/pull/41).
    match socket.emit(event, data) {
        Ok(_) => {
            println!("Successfully sent {} to '{}'", event, socket.sid)
        }
        Err(_) => {
            println!("Detected a closed socket: '{}'", socket.sid);
            // make sure locks are released before doing this
            let socket = socket.clone();
            tokio::spawn(async move {
                let state = extract_state(&socket);
                let state = state.read().unwrap();
                let mut sm = state.storage_manager.lock().unwrap();
                sm.disassemble();
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use std::fs;
    use std::io::Read;
    use std::path::Path;
    use std::sync::{Arc, Mutex};

    use fs_extra::dir::*;
    use fs_extra::error::*;
    use tokio::runtime::Runtime;
    use walkdir::WalkDir;

    use ast::options::StringificationOptions;
    use ast::Ast;

    use crate::infrastructure::file_path::FilePath;
    use crate::infrastructure::storage_manager::{StorageManager, TexlaStorageManager};
    use crate::infrastructure::vcs_manager::GitManager;

    extern crate fs_extra;

    #[test]
    fn parse_pflichtenheft_from_disk() {
        let main_file = FilePath::from("test_resources/latex/pflichtenheft/main.tex");
        let sm = TexlaStorageManager::new(
            GitManager::new(true, main_file.directory.clone()),
            main_file,
            500,
            5000,
            100,
        );
        assert!(super::parse_ast_from_disk(&sm).is_ok());
    }

    #[test]
    fn pflichtenheft_read_save() {
        let main_file = FilePath::from("test_resources/latex/pflichtenheft/main.tex");

        let main_file_directory = "test_resources/latex/pflichtenheft";
        let copy_main_file_directory = "test_resources/latex/pflichtenheft_copy";
        if Path::new(copy_main_file_directory).is_dir() {
            fs::remove_dir_all(copy_main_file_directory).unwrap();
        }
        fs::create_dir(copy_main_file_directory).unwrap();

        let sm = TexlaStorageManager::new(
            GitManager::new(true, main_file.directory.clone()),
            main_file,
            500,
            5000,
            100,
        );
        let ast = super::parse_ast_from_disk(&sm).unwrap();

        let latex_single_string = ast.to_latex(StringificationOptions::default()).unwrap();

        let rt = Runtime::new().unwrap();
        rt.spawn(async move {
            StorageManager::save(Arc::new(Mutex::new(sm)), latex_single_string)
                .await
                .ok();
        });

        let mut options = CopyOptions::new();
        options.overwrite = true;
        copy(main_file_directory, copy_main_file_directory, &options).unwrap();

        let path_to_copied_directory = "test_resources/latex/pflichtenheft_copy/pflichtenheft";

        let are_directories_equal =
            compare_directories(main_file_directory, path_to_copied_directory);

        assert!(
            are_directories_equal.unwrap(),
            "Directories have differences"
        );

        fs::remove_dir_all(copy_main_file_directory).unwrap();
    }

    fn compare_directories(dir1: &str, dir2: &str) -> Result<bool> {
        let walker1 = WalkDir::new(dir1).into_iter();
        let walker2 = WalkDir::new(dir2).into_iter();

        // merge iterators into one
        for (entry1, entry2) in walker1.zip(walker2) {
            let entry1 = entry1.expect("should be a valid entry");
            let entry2 = entry2.expect("should be a valid entry");

            if entry1.file_type().is_file() && entry2.file_type().is_file() {
                let mut file1 = fs::File::open(entry1.path())?;
                let mut file2 = fs::File::open(entry2.path())?;

                let mut contents1 = Vec::new();
                let mut contents2 = Vec::new();

                file1.read_to_end(&mut contents1)?;
                file2.read_to_end(&mut contents2)?;

                if contents1 != contents2 {
                    return Ok(false);
                }
            } else if entry1.file_type().is_dir() && entry2.file_type().is_dir() {
                continue;
            } else {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
