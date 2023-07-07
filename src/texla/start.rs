use clap::Parser;

use crate::ast;
use crate::infrastructure;
use crate::texla::core::Core;
use crate::texla::webserver::start_axum;

#[derive(Parser, Debug)]
struct CliArguments {
    #[arg(short, long)]
    main_file: String,
}

pub async fn start() {
    println!("Starting TeXLa...");

    // append `-- --main-file main.tex` to your run command in CLion to provide the necessary CLI
    // argument
    let args = CliArguments::parse();
    println!("Opening file: {}", args.main_file);

    // HERE: start multiple tasks and join them
    // let core = Core::new(args.main_file);
    start_axum().await;
}
