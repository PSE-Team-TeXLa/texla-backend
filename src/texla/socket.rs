use std::path::Path;
use std::sync::{Arc, RwLock};

use socketioxide::{Namespace, Socket, SocketIoLayer};
use socketioxide::adapter::LocalAdapter;
use tower::layer::util::{Identity, Stack};
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

use crate::ast::Ast;
use crate::ast::operation::{JsonOperation, Operation};
use crate::ast::options::StringificationOptions;
use crate::ast::texla_ast::TexlaAst;
use crate::infrastructure::storage_manager::{StorageManager, TexlaStorageManager};
use crate::infrastructure::vcs_manager::GitManager;
use crate::texla::core::TexlaCore;
use crate::texla::errors::TexlaError;
use crate::texla::state::TexlaState;

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

async fn handler(socket: Arc<Socket<LocalAdapter>>, core: Arc<RwLock<TexlaCore>>) {
    println!("Socket connected with id: {}", socket.sid);

    let core = core.read().unwrap();

    // initial parse
    // TODO call TexlaStorageManager<T>::attach_handlers() later
    // TODO: error handling! -> close connection if unable to set a state!

    // TODO after VS: is there a shorter way to get the parent directory as String?
    // Linus: i think we should not assume, that we only want to watch the surrounding directory
    let parent_directory = Path::new(&core.main_file)
        .parent()
        .expect("No parent directory found")
        .to_str()
        .expect("No parent directory found")
        .to_string();

    // TODO: allow asynchronicity here
    let vcs_manager = GitManager::new(parent_directory);
    let storage_manager = TexlaStorageManager::new(vcs_manager, core.main_file.clone());

    let latex_single_string = storage_manager.multiplex_files().unwrap();
    // let ast = TexlaAst::from_latex(latex_single_string).unwrap();
    let ast = TexlaAst::trivial();
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

    if let Ok(ast) = state.ast.to_json(StringificationOptions::default()) {
        socket.emit("new_ast", ast).ok();
    } else {
        panic!("This error should have been caught before creating state")
    }

    socket.on(
        "operation",
        |socket, operation: JsonOperation, _, _| async move {
            println!("Received operation");
            let mut state = socket.extensions.get_mut::<TexlaState>().unwrap();
            match perform_and_check_operation(&state.ast, operation.to_trait_obj()) {
                Ok(ast) => {
                    state.ast = ast;
                    socket.emit("new_ast", &state.ast).ok();
                }
                Err(err) => {
                    socket.emit("error", err).ok();
                }
            }
        },
    );
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
