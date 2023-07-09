use std::sync::{Arc, Mutex, RwLock};

use socketioxide::adapter::LocalAdapter;
use socketioxide::{Namespace, Socket, SocketIoLayer};
use tower::layer::util::{Identity, Stack};
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

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
    // TODO: error handling!
    let storage_manager = TexlaStorageManager::new(core.main_file.clone());
    // TODO: asynchronously start StorageManager
    let latex_single_string = storage_manager.multiplex_files().unwrap();
    let ast = TexlaAst::from_latex(&latex_single_string).unwrap();

    let state = TexlaState {
        socket: socket.clone(),
        storage_manager,
        ast,
    };

    socket.extensions.insert(state);

    // TODO: implement API here

    // data can also be any serde deserializable struct
    socket.on("abc", |socket, data: String, _, _| async move {
        println!("Received abc event: {:?}", data);
        socket.emit("abc", "i am also alive").ok();
    });
}
