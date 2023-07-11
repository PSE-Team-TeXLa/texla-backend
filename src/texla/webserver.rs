use std::path::PathBuf;

use axum::response::Html;
use axum::routing::{get, MethodRouter};
use axum::Server;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::set_status::SetStatus;

use crate::texla::socket::socket_service;

const PORT: u16 = 13814;
const FRONTEND_PATH: &str = "frontend";

pub async fn start_axum() {
    let app = axum::Router::new()
        // .route("/dummy", get(|| async { Html("This is a dummy file.") }))
        .layer(socket_service())
        .fallback_service(static_files());

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
