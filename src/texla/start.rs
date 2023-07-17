use std::sync::{Arc, RwLock};

use clap::Parser;

use crate::infrastructure::export_manager::TexlaExportManager;
use crate::texla::core::TexlaCore;
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

    let core = TexlaCore {
        export_manager: TexlaExportManager,
        main_file: args.main_file,
    };

    let core = Arc::new(RwLock::new(core));

    start_axum(core).await;
}
