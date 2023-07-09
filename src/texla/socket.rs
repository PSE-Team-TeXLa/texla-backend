use std::sync::{Arc, Mutex, RwLock};

use socketioxide::adapter::LocalAdapter;
use socketioxide::extensions::Ref;
use socketioxide::{Namespace, Socket, SocketIoLayer};
use tower::layer::util::{Identity, Stack};
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

use crate::ast::options::StringificationOptions;
use crate::ast::texla_ast::TexlaAst;
use crate::ast::Ast;
use crate::infrastructure::storage_manager::{StorageManager, TexlaStorageManager};
use crate::infrastructure::vcs_manager::GitManager;
use crate::texla::core::TexlaCore;
use crate::texla::state::{State, TexlaState};

pub fn socket_service(
    core: Arc<RwLock<TexlaCore>>,
) -> ServiceBuilder<Stack<SocketIoLayer<LocalAdapter>, Stack<CorsLayer, Identity>>> {
    // TODO: Arc<RwLock<TexlaCor>> is technically not needed until here
    let ns = Namespace::builder()
        .add("/", move |socket| handler(socket, core.clone()))
        .build();

    let service = ServiceBuilder::new()
        .layer(CorsLayer::permissive())
        .layer(SocketIoLayer::new(ns));

    service
}

async fn handler(socket: Arc<Socket<LocalAdapter>>, core: Arc<RwLock<TexlaCore>>) {
    println!("Socket connected with id: {}", socket.sid);

    let core = core.read().unwrap();

    // initial parse
    // TODO: error handling! -> close connection if unable to set a state!
    let storage_manager = TexlaStorageManager::new(core.main_file.clone());
    // TODO: asynchronously start StorageManager
    let latex_single_string = storage_manager.multiplex_files().unwrap();
    let ast = TexlaAst::from_latex(&latex_single_string).unwrap();
    // TODO: validate ast

    let state = TexlaState {
        socket: socket.clone(),
        storage_manager,
        ast,
    };

    socket.extensions.insert(state);
    let state = socket.extensions.get::<TexlaState>().unwrap();

    // initial messages
    socket
        .emit("remote_url", state.storage_manager.remote_url())
        .ok();

    send_ast(&socket);

    // TODO: data should be dyn Operation
    socket.on("operation", |socket, data: String, _, _| async move {
        println!("Received operation: {:?}", data);
        todo!();
        // process operation

        send_ast(&socket);
    });
}

fn send_ast(socket: &Arc<Socket<LocalAdapter>>) {
    let state = socket.extensions.get::<TexlaState>().unwrap();
    match state.ast.to_json(StringificationOptions::default()) {
        Ok(ast) => {
            socket.emit("new_ast", ast).ok();
        }
        Err(err) => {
            socket.emit("error", err).ok();
        }
    }
}
