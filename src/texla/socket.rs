use std::sync::{Arc, Mutex, RwLock};

use socketioxide::adapter::LocalAdapter;
use socketioxide::extensions::Ref;
use socketioxide::{Namespace, Socket, SocketIoLayer};
use tokio::join;
use tower::layer::util::{Identity, Stack};
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

use crate::ast::operation::{JsonOperation, Operation};
use crate::ast::options::StringificationOptions;
use crate::ast::texla_ast::TexlaAst;
use crate::ast::Ast;
use crate::infrastructure::errors::InfrastructureError;
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
    // TODO: Arc<RwLock<TexlaCor>> is technically not needed until here
    let ns = Namespace::builder()
        .add("/", move |socket| handler(socket, core.clone()))
        .build();

    // ServiceBuilder executes layer top to bottom
    let service = ServiceBuilder::new()
        .layer(CorsLayer::permissive())
        .layer(SocketIoLayer::new(ns));

    service
}

async fn handler(socket: TexlaSocket, core: Arc<RwLock<TexlaCore>>) {
    println!("Socket connected with id: {}", socket.sid);

    let storage_manager = {
        let core = core.read().unwrap();

        let vcs_manager = GitManager::new(core.main_file.clone());
        TexlaStorageManager::new(vcs_manager, core.main_file.clone())
    };

    let ast = {
        let latex_single_string = storage_manager.multiplex_files().unwrap();
        let ast = TexlaAst::from_latex(latex_single_string).unwrap();
        // TODO: validate ast (by calling to_latex())
        // TODO: error handling! -> close connection if unable to set a state!
        ast
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

    socket.on("operation", |socket, json: String, _, _| async move {
        print!("Received operation:");

        let operation = serde_json::from_str::<JsonOperation>(&json)
            .expect("Got invalid operation from frontend")
            .to_trait_obj();
        println!("{:?}", operation);

        let state_ref = extract_state(&socket);
        let mut state = state_ref.lock().unwrap();
        match perform_and_check_operation(&state.ast, operation) {
            Ok(ast) => {
                state.ast = ast;
                socket.emit("new_ast", &state.ast).ok();
                println!(
                    "Operation was okay, new_ast {:?}",
                    serde_json::to_string_pretty(&state.ast).unwrap()
                );
            }
            Err(err) => {
                println!("Operation was not okay: {}", err);
                socket.emit("error", err).ok();
            }
        }
    });

    {
        socket.on(
            "prepare_export",
            move |socket, options: StringificationOptions, _, _| {
                handle_export(socket, options, core.clone())
            },
        );
    }

    join!(storage_manager_handle);
}

fn extract_state(socket: &TexlaSocket) -> Ref<SharedTexlaState> {
    socket.extensions.get::<SharedTexlaState>().unwrap()
}

fn perform_and_check_operation(
    ast: &TexlaAst,
    operation: Box<dyn Operation<TexlaAst>>,
) -> Result<TexlaAst, TexlaError> {
    // TODO alternative to cloning: mutable reference + atomic operations
    let mut cloned_ast = ast.clone();

    cloned_ast.execute(operation)?;
    let latex_single_string = cloned_ast.to_latex(Default::default())?;
    let reparsed_ast = TexlaAst::from_latex(latex_single_string)?;

    Ok(reparsed_ast)
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
