use std::sync::Arc;

use socketioxide::adapter::LocalAdapter;
use socketioxide::{Namespace, Socket, SocketIoLayer};
use tower::layer::util::{Identity, Stack};
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

pub fn socket_service(
) -> ServiceBuilder<Stack<SocketIoLayer<LocalAdapter>, Stack<CorsLayer, Identity>>> {
    let ns = Namespace::builder().add("/", handler).build();

    let service = ServiceBuilder::new()
        .layer(CorsLayer::permissive())
        .layer(SocketIoLayer::new(ns));

    service
}

async fn handler(socket: Arc<Socket<LocalAdapter>>) {
    println!("Socket connected with id: {}", socket.sid);

    // TODO: implement API here
    // TODO: how do i come in here from outside, e.g. for sending an error?

    // data can also be any serde deserializable struct
    socket.on("abc", |socket, data: String, _, _| async move {
        println!("Received abc event: {:?}", data);
        socket.emit("abc", "i am also alive").ok();
    });
}
