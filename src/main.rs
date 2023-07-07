mod ast;
mod infrastructure;
mod texla;

#[tokio::main]
async fn main() {
    texla::start::start().await;
}
