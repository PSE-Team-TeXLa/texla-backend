use std::process::exit;
use std::sync::{Arc, Mutex, RwLock};

use socketioxide::adapter::LocalAdapter;
use socketioxide::extensions::Ref;
use socketioxide::{Namespace, Socket, SocketIoLayer};
use tokio::join;
use tokio::time::sleep;
use tower::layer::util::{Identity, Stack};
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

use crate::ast::operation::{JsonOperation, Operation};
use crate::ast::options::StringificationOptions;
use crate::ast::texla_ast::TexlaAst;
use crate::ast::Ast;
use crate::infrastructure::export_manager::ExportManager;
use crate::infrastructure::storage_manager::{StorageManager, TexlaStorageManager};
use crate::infrastructure::vcs_manager::GitManager;
use crate::texla::core::TexlaCore;
use crate::texla::errors::TexlaError;
use crate::texla::state::{SharedTexlaState, TexlaState};

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

// TODO: a bit of reorganization, maybe split into multiple functions
async fn handler(socket: TexlaSocket, core: Arc<RwLock<TexlaCore>>) {
    println!("Socket connected with id: {}", socket.sid);

    let storage_manager = {
        let core = core.read().unwrap();

        let vcs_manager = GitManager::new(core.main_file.clone());
        TexlaStorageManager::new(vcs_manager, core.main_file.clone())
    };

    let ast = match parse_ast_from_disk(&storage_manager) {
        Ok(ast) => ast,
        Err(err) => {
            println!("Found invalid ast: {}", err);
            socket.emit("error", err).ok();
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
    socket.extensions.insert(Arc::new(Mutex::new(state)));

    let storage_manager_handle = {
        let state_ref = extract_state(&socket);
        let state = state_ref.lock().unwrap();

        state
            .storage_manager
            .lock()
            .unwrap()
            .attach_handlers(state_ref.clone(), state_ref.clone());
        StorageManager::start(state.storage_manager.clone())
    };

    {
        let state_ref = extract_state(&socket);
        let state = state_ref.lock().unwrap();
        let storage_manager = state.storage_manager.lock().unwrap();

        // initial messages
        socket.emit("remote_url", storage_manager.remote_url()).ok();
        socket.emit("new_ast", &state.ast).ok();
    }

    socket.on("active", |socket, _: String, _, _| async move {
        let state_ref = extract_state(&socket);
        let state = state_ref.lock().unwrap();
        // stop synchronization in order to prevent losing changes
        state.storage_manager.lock().unwrap().stop_timers();
        println!("Waiting for frontend to finalize operation...");
    });

    socket.on("operation", |socket, json: String, _, _| async move {
        print!("Received operation:");

        let operation = serde_json::from_str::<JsonOperation>(&json)
            .expect("Got invalid operation from frontend")
            .to_trait_obj();
        println!("{:?}", operation);

        let state_ref = extract_state(&socket);
        let mut state = state_ref.lock().unwrap();
        match perform_and_check_operation(&mut state.ast, operation) {
            Ok(()) => {
                socket.emit("new_ast", &state.ast).ok();
                println!("Operation was okay");
                // println!("new_ast {:#?}", &state.ast);
            }
            Err(err) => {
                println!("Operation was not okay: {}", err);
                socket.emit("error", err).ok();
                // send old ast in order to enable frontend to roll back to it
                socket.emit("new_ast", &state.ast).ok();
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
        println!("Saving Changes...");
        let result = {
            let state_ref = extract_state(&socket);
            let state = state_ref.lock().unwrap();
            let mut storage_manager = state.storage_manager.lock().unwrap();
            storage_manager.end_session()
        };
        match result {
            Ok(_) => {
                println!("Quitting...");
                socket.emit("quit", "ok").ok();
                sleep(std::time::Duration::from_secs(1)).await;
                socket.disconnect().ok();
                exit(0);
            }
            Err(err) => {
                socket.emit("error", TexlaError::from(err)).ok();
            }
        };
    });

    // let the tasks in storage_manager be executed
    join!(storage_manager_handle);
}

fn parse_ast_from_disk(
    storage_manager: &TexlaStorageManager<GitManager>,
) -> Result<TexlaAst, TexlaError> {
    let latex_single_string = storage_manager.multiplex_files()?;
    let ast = TexlaAst::from_latex(latex_single_string)?;
    // verify the ast by converting it to latex again
    ast.to_latex(Default::default())?;
    Ok(ast)
}

fn extract_state(socket: &TexlaSocket) -> Ref<SharedTexlaState> {
    socket.extensions.get::<SharedTexlaState>().unwrap()
}

fn perform_and_check_operation(
    ast: &mut TexlaAst,
    operation: Box<dyn Operation<TexlaAst>>,
) -> Result<(), TexlaError> {
    let backup_latex = ast.to_latex(Default::default())?;

    let perform = || -> Result<TexlaAst, TexlaError> {
        ast.execute(operation)?;
        let latex_single_string = ast.to_latex(Default::default())?;
        let reparsed_ast = TexlaAst::from_latex(latex_single_string)?;
        Ok(reparsed_ast)
    };

    match perform() {
        Ok(new_ast) => {
            *ast = new_ast;
            Ok(())
        }
        Err(err) => {
            *ast = TexlaAst::from_latex(backup_latex)?;
            Err(err)
        }
    }
}

fn stringify_and_save(
    state: &TexlaState,
    options: StringificationOptions,
) -> Result<(), TexlaError> {
    let latex_single_string = state.ast.to_latex(options)?;
    state
        .storage_manager
        .lock()
        .unwrap()
        .save(latex_single_string)?;

    Ok(())
}

async fn handle_export(
    socket: TexlaSocket,
    options: StringificationOptions,
    core: Arc<RwLock<TexlaCore>>,
) {
    println!("Preparing export with options: {:?}", options);
    let state_ref = extract_state(&socket);
    let state = state_ref.lock().unwrap();

    if let Err(err) = stringify_and_save(&state, options) {
        socket.emit("error", err).ok();
        return;
    }

    match core.write().unwrap().export_manager.zip_files() {
        Ok(url) => {
            socket.emit("export_ready", url).ok();
        }
        Err(err) => {
            socket.emit("error", TexlaError::from(err)).ok();
        }
    }
}

#[cfg(test)]
mod test {
    use crate::infrastructure::storage_manager::TexlaStorageManager;
    use crate::infrastructure::vcs_manager::GitManager;

    // TODO: comment in when endless loop is fixed
    // (this not only fails the test, but stops the entire testsuite from terminating)
    // #[test]
    fn pflichtenheft() {
        let file = "latex_test_files/pflichtenheft-main/main.tex".to_string();
        let sm = TexlaStorageManager::new(GitManager::new(file.clone()), file);
        assert!(super::parse_ast_from_disk(&sm).is_ok());
    }
}
