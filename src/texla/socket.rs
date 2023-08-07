use serde::Serialize;
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

use crate::infrastructure::export_manager::ExportManager;
use crate::infrastructure::storage_manager::{StorageManager, TexlaStorageManager};
use crate::infrastructure::vcs_manager::GitManager;
use crate::texla::core::TexlaCore;
use crate::texla::errors::TexlaError;
use crate::texla::state::{SharedTexlaState, TexlaState};
use ast::operation::{JsonOperation, Operation};
use ast::options::StringificationOptions;
use ast::texla_ast::TexlaAst;
use ast::Ast;

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
// (in more files, some into state)
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
        send(&socket, "remote_url", storage_manager.remote_url()).ok();
        send(&socket, "new_ast", &state.ast).ok();
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
        match perform_and_check_operation(&mut state, operation) {
            Ok(()) => {
                send(&socket, "new_ast", &state.ast).ok();
                println!("Operation was okay");
                println!("Saved changes");
                // println!("new_ast {:#?}", &state.ast);
            }
            Err(err) => {
                println!("Operation was not okay: {}", err);
                send(&socket, "error", err).ok();
                // send old ast in order to enable frontend to roll back to it
                send(&socket, "new_ast", &state.ast).ok();
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
                send(&socket, "quit", "ok").ok();
                sleep(std::time::Duration::from_secs(1)).await;
                socket.disconnect().ok();
                exit(0);
            }
            Err(err) => {
                send(&socket, "error", TexlaError::from(err)).ok();
            }
        };
    });

    // let the tasks in storage_manager be executed
    join!(storage_manager_handle);
}

pub fn parse_ast_from_disk(
    storage_manager: &TexlaStorageManager<GitManager>,
) -> Result<TexlaAst, TexlaError> {
    let latex_single_string = storage_manager.multiplex_files()?;
    let ast = TexlaAst::from_latex(latex_single_string)?;
    // verify the ast by converting it to latex again
    // TODO: the reparsing should be temporary.
    // It should never happen, that our output cannot be parsed
    TexlaAst::from_latex(ast.to_latex(Default::default())?)?;
    Ok(ast)
}

fn extract_state(socket: &TexlaSocket) -> Ref<SharedTexlaState> {
    socket.extensions.get::<SharedTexlaState>().unwrap()
}

fn perform_and_check_operation(
    state: &mut TexlaState,
    operation: Box<dyn Operation<TexlaAst>>,
) -> Result<(), TexlaError> {
    let backup_latex = state.ast.to_latex(Default::default())?;

    let perform = || -> Result<TexlaAst, TexlaError> {
        state.ast.execute(operation)?;
        let latex_single_string = state.ast.to_latex(Default::default())?;
        let reparsed_ast = TexlaAst::from_latex(latex_single_string)?;
        stringify_and_save(state, Default::default())?;
        Ok(reparsed_ast)
    };

    match perform() {
        Ok(new_ast) => {
            state.ast = new_ast;
            Ok(())
        }
        Err(err) => {
            state.ast = TexlaAst::from_latex(backup_latex)?;
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

// TODO: move into export handler?
async fn handle_export(
    socket: TexlaSocket,
    options: StringificationOptions,
    core: Arc<RwLock<TexlaCore>>,
) {
    println!("Preparing export with options: {:?}", options);
    let state_ref = extract_state(&socket);
    let state = state_ref.lock().unwrap();

    if let Err(err) = stringify_and_save(&state, options) {
        send(&socket, "error", err).ok();
        return;
    }

    // TODO: save original files again
    match core.write().unwrap().export_manager.zip_files() {
        Ok(url) => {
            send(&socket, "export_ready", url).ok();
        }
        Err(err) => {
            send(&socket, "error", TexlaError::from(err)).ok();
        }
    }
}

pub(crate) fn send(socket: &TexlaSocket, event: &str, data: impl Serialize) -> Result<(), ()> {
    match socket.emit(event, data) {
        Ok(_) => {
            println!("Successfully sent")
        }
        Err(_err) => {
            // TODO: extremely temporary!
            // this only works with the unreleased main branch of socketioxide and would be
            // obsolet with an almost finished pull request:
            // https://github.com/Totodore/socketioxide/pull/41
            println!("Detected a closed socket!");
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::infrastructure::storage_manager::TexlaStorageManager;
    use crate::infrastructure::vcs_manager::GitManager;

    #[test]
    fn pflichtenheft() {
        let file = "test_resources/latex/pflichtenheft/main.tex".to_string();
        // TODO replace separator?
        let sm = TexlaStorageManager::new(GitManager::new(file.clone()), file);
        assert!(super::parse_ast_from_disk(&sm).is_ok());
    }
}
