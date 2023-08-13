use std::process::exit;
use std::sync::{Arc, Mutex, RwLock};

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
        let state = state_ref.lock().unwrap();
        // stop synchronization in order to prevent losing changes
        state.storage_manager.lock().unwrap().wait_for_frontend();
        println!("Waiting for frontend to finalize operation...");
    });

    socket.on("operation", |socket, json: String, _, _| async move {
        print!("Received operation:");

        let operation = serde_json::from_str::<JsonOperation>(&json)
            .expect("Got invalid operation from frontend")
            .to_trait_obj();
        println!("{:?}", operation);

        let state = extract_state(&socket).clone();
        match perform_and_check_operation(state.clone(), operation).await {
            Ok(()) => {
                send(&socket, "new_ast", &state.lock().unwrap().ast).ok();
                println!("Operation was okay");
                println!("Saved changes");
                // println!("new_ast {:#?}", &state.ast);
            }
            Err(err) => {
                println!("Operation was not okay: {}", err);
                send(&socket, "error", err).ok();
                // send old ast in order to enable frontend to roll back to it
                send(&socket, "new_ast", &state.lock().unwrap().ast).ok();
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
        println!("Ending worksession...");
        let result = {
            let state_ref = extract_state(&socket);
            let state = state_ref.lock().unwrap();
            let mut storage_manager = state.storage_manager.lock().unwrap();
            storage_manager.end_worksession()
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

    if let Err(err) = storage_manager_handle.await {
        send(&socket, "error", TexlaError::from(err)).ok();
    };
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

async fn perform_and_check_operation(
    state: Arc<Mutex<TexlaState>>,
    operation: Box<dyn Operation<TexlaAst>>,
) -> Result<(), TexlaError> {
    let backup_latex = state.lock().unwrap().ast.to_latex(Default::default())?;

    match perform_operation(state.clone(), operation).await {
        Ok(new_ast) => {
            state.lock().unwrap().ast = new_ast;
            Ok(())
        }
        Err(err) => {
            state.lock().unwrap().ast = TexlaAst::from_latex(backup_latex)?;
            Err(err)
        }
    }
}

async fn perform_operation(
    state: Arc<Mutex<TexlaState>>,
    operation: Box<dyn Operation<TexlaAst>>,
) -> Result<TexlaAst, TexlaError> {
    let reparsed_ast = {
        let mut locked = state.lock().unwrap();
        locked.ast.execute(operation)?;
        let latex_single_string = locked.ast.to_latex(Default::default())?;
        TexlaAst::from_latex(latex_single_string)?
    };
    // TODO: this should be done in parallel
    stringify_and_save(state, Default::default()).await?;
    Ok(reparsed_ast)
}

async fn stringify_and_save(
    state: Arc<Mutex<TexlaState>>,
    options: StringificationOptions,
) -> Result<(), TexlaError> {
    let latex_single_string = state.lock().unwrap().ast.to_latex(options)?;
    let storage_manager = state.lock().unwrap().storage_manager.clone();
    StorageManager::save(storage_manager, latex_single_string).await?;

    Ok(())
}

// TODO: move into export handler?
async fn handle_export(
    socket: TexlaSocket,
    options: StringificationOptions,
    core: Arc<RwLock<TexlaCore>>,
) {
    println!("Preparing export with options: {:?}", options);
    let state = extract_state(&socket).clone();

    if let Err(err) = stringify_and_save(state, options).await {
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
    // this only works with a modified main branch of socketioxide (see Cargo.toml)
    // with the upcoming release (after 0.3.0) you could relax this check and instead free
    // resources in a on_disconnect handler (see https://github.com/Totodore/socketioxide/pull/41).
    match socket.emit(event, data) {
        Ok(_) => {
            println!("Successfully sent {} to {}", event, socket.sid)
        }
        Err(_) => {
            println!("Detected a closed socket: {}", socket.sid);
            // make sure locks are released before doing this
            let socket = socket.clone();
            tokio::spawn(async move {
                let state = extract_state(&socket);
                let state = state.lock().unwrap();
                let mut sm = state.storage_manager.lock().unwrap();
                sm.disassemble();
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use tokio::runtime::Runtime;

    use ast::options::StringificationOptions;
    use ast::Ast;

    use crate::infrastructure::storage_manager::TexlaStorageManager;
    use crate::infrastructure::vcs_manager::GitManager;

    #[test]
    fn pflichtenheft() {
        let file = "test_resources/latex/pflichtenheft/main.tex".to_string();
        // TODO replace separator?
        let sm = TexlaStorageManager::new(GitManager::new(file.clone()), file);
        assert!(super::parse_ast_from_disk(&sm).is_ok());
    }

    #[test]
    fn pflichtenheft_read_save() {
        let file = "test_resources/latex/pflichtenheft/main.tex".to_string();
        // TODO replace separator?
        let sm = TexlaStorageManager::new(GitManager::new(file.clone()), file);
        let ast = super::parse_ast_from_disk(&sm);
        let ast = ast.unwrap();

        let latex_single_string = ast.to_latex(StringificationOptions::default());
        let latex_single_string = latex_single_string.unwrap();

        let rt = Runtime::new().unwrap();
        rt.spawn(async move {
            // StorageManager::save(Arc::new(Mutex::new(sm)), latex_single_string)
            //     .await
            //     .ok();
        });

        // TODO: check that there are no changes
    }
}
