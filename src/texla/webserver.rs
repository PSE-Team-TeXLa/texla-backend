use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use axum;
use axum::body::StreamBody;
use axum::handler::Handler;
use axum::http::{header, StatusCode};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Extension, Server};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::set_status::SetStatus;
use tower_http::trace::TraceLayer;

use crate::texla::core::TexlaCore;
use crate::texla::socket::socket_service;

const PORT: u16 = 13814;
const FRONTEND_PATH: &str = "frontend";

pub async fn start_axum(core: Arc<RwLock<TexlaCore>>) {
    let app = axum::Router::new()
        .fallback_service(static_files())
        .route("/user-assets/*path", get(user_assets_handler))
        .layer(
            TraceLayer::new_for_http().on_body_chunk(()).on_eos(()), // .on_request(log_request)
        )
        .layer(socket_service(core.clone()))
        .layer(Extension(core.clone()));

    let res = Server::bind(&([127, 0, 0, 1], PORT).into())
        .serve(app.into_make_service())
        .await;

    res.expect("Could not start webserver");
}

fn static_files() -> ServeDir<SetStatus<ServeFile>> {
    let frontend_path = PathBuf::from(FRONTEND_PATH)
        .canonicalize()
        .expect("Could not find frontend path");
    println!("Serving static files from: {}", frontend_path.display());

    // TODO: is index.html really the root of our svelte app?
    ServeDir::new(frontend_path).not_found_service(ServeFile::new(
        PathBuf::from(FRONTEND_PATH)
            .join(PathBuf::from("index.html"))
            .canonicalize()
            .expect("Could not find frontend index.html"),
    ))
}

async fn user_assets_handler(
    Extension(core): Extension<Arc<RwLock<TexlaCore>>>,
    axum::extract::Path(path): axum::extract::Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    println!("Serving user asset: {}", path);

    let main_file_directory = {
        let main_file = &core.read().unwrap().main_file;
        let main_file_path = PathBuf::from(main_file)
            .canonicalize()
            .expect("Could not find main file directory");
        main_file_path
            .parent()
            .expect("main_file cannot be a root directory")
            .to_path_buf()
    };

    let file = match tokio::fs::File::open(main_file_directory.join(&path)).await {
        Ok(file) => file,
        Err(err) => return Err(StatusCode::IM_A_TEAPOT),
    };
    // convert the `AsyncRead` into a `Stream`
    let stream = tokio_util::io::ReaderStream::new(file);
    // convert the `Stream` into an `axum::body::HttpBody`
    let body = StreamBody::new(stream);

    let content_disposition_header = format!("attachment; filename=\"{}\"", path);
    let headers = [
        (header::CONTENT_TYPE, "text/toml; charset=utf-8"),
        (
            header::CONTENT_DISPOSITION,
            content_disposition_header.as_str(),
        ),
    ];

    Ok((headers, body).into_response())
}
