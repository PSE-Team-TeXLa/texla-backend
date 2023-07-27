mod ast;
mod infrastructure;
mod texla;

#[tokio::main]
async fn main() {
    // this logs debug information according to the RUST_LOG environment variable
    tracing_subscriber::fmt::init();
    texla::start::start().await;
}
