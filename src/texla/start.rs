use std::path::MAIN_SEPARATOR_STR;
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

    // replace separators in path with system-dependent variant
    let main_file = args.main_file.replace(['/', '\\'], MAIN_SEPARATOR_STR);
    // TODO use Path instead of String

    println!("Opening file: {}", main_file);

    let core = TexlaCore {
        export_manager: TexlaExportManager,
        main_file,
    };

    let core = Arc::new(RwLock::new(core));

    start_axum(core).await;
}
