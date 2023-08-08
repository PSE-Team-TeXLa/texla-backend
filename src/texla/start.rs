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

    // TODO how do we pass the following values to TexlaStorageManager?
    #[arg(short, long, default_value = "500")] // in milliseconds
    pull_interval: u64,

    #[arg(short, long, default_value = "5000")] // in milliseconds
    worksession_interval: u64,
}

pub async fn start() {
    println!("Starting TeXLa...");

    // append `-- --main-file main.tex` to your run command in CLion to provide the necessary CLI
    // argument
    let args = CliArguments::parse();

    // replace separators in path with system-dependent variant
    let main_file = args.main_file.replace(['/', '\\'], MAIN_SEPARATOR_STR);
    // TODO use tuple (directory: PathBuf, filename: PathBuf) instead of String for main_file

    println!("Opening file: {}", main_file);

    let core = TexlaCore {
        export_manager: TexlaExportManager::new(main_file.clone()),
        main_file,
    };

    let core = Arc::new(RwLock::new(core));

    start_axum(core).await;
}
