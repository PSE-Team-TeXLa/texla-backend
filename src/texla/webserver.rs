use axum::http::{header, Request, Response, StatusCode};
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use axum::body::StreamBody;
use axum::{Error, Server};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::set_status::SetStatus;
use tower_http::trace::TraceLayer;

use crate::texla::core::TexlaCore;
use crate::texla::socket::socket_service;

const PORT: u16 = 13814;
const FRONTEND_PATH: &str = "frontend";

pub async fn start_axum(core: Arc<RwLock<TexlaCore>>) {
    let app = axum::Router::new()
        // .route("/dummy", get(|| async { Html("This is a dummy file.") }))
        .fallback_service(static_files())
        .route("/user-assets/*path", get(user_assets_handler))
        .layer(
            TraceLayer::new_for_http().on_body_chunk(()).on_eos(()), // .on_request(log_request)
        )
        .layer(socket_service(core));

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
    axum::extract::Path(path): axum::extract::Path<String>,
) -> Result<axum::response::Response, StatusCode> {
    println!("Serving user file: {}", path);
    let main_file = "latex_test_files/lots_of_features.tex";
    let main_file_path = PathBuf::from(main_file)
        .canonicalize()
        .expect("Could not find main file directory");
    let main_file_directory = main_file_path
        .parent()
        .expect("main_file cannot be a root directory");

    let file = match tokio::fs::File::open(main_file_directory.join(&path)).await {
        Ok(file) => file,
        Err(err) => return Err(StatusCode::BAD_REQUEST),
    };
    // convert the `AsyncRead` into a `Stream`
    let stream = tokio_util::io::ReaderStream::new(file);
    // convert the `Stream` into an `axum::body::HttpBody`
    let body = StreamBody::new(stream);

    let header = format!("attachment; filename=\"{}\"", path);
    let headers = [
        (header::CONTENT_TYPE, "text/toml; charset=utf-8"),
        (header::CONTENT_DISPOSITION, header.as_str()),
    ];

    Ok((headers, body).into_response())
}
