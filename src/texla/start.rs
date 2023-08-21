use std::sync::{Arc, RwLock};

use clap::Parser;

use crate::infrastructure::export_manager::TexlaExportManager;
use crate::infrastructure::file_path::FilePath;
use crate::texla::core::TexlaCore;
use crate::texla::webserver::{start_axum, PORT};

// the rustdocs are put into the help message of the CLI
#[derive(Parser, Debug)]
#[clap(name = "TeXLa", about = "TeXLa - a graphical LaTeX editor", version)]
struct CliArguments {
    /// The root file of your LaTeX project
    #[arg(value_names = ["path"], short, long)]
    main_file: String,

    /// The time between two subsequent pulls from the git remote (in milliseconds)
    #[arg(value_names = ["duration in ms"], short, long, default_value = "500")]
    pull_interval: u64,

    /// The minimum time between the last change and the according commit (in milliseconds)
    #[arg(value_names = ["duration in ms"], short, long, default_value = "5000")]
    worksession_interval: u64,

    /// The time 'notify' is allowed to take for picking up our own file changes and reporting them
    /// (in milliseconds)
    #[arg(value_names = ["duration in ms"], short, long, default_value = "100")]
    notify_delay: u64,
}

pub async fn start() {
    // append `-- --main-file main.tex` to your run command in CLion to provide the necessary CLI
    // argument
    let args = CliArguments::parse();

    println!("Starting TeXLa...");

    let main_file = FilePath::from(args.main_file);
    println!("Opening file: {}", main_file.path.to_str().unwrap());

    let core = Arc::new(RwLock::new(TexlaCore {
        export_manager: TexlaExportManager::new(main_file.directory.clone()),
        pull_interval: args.pull_interval,
        worksession_interval: args.worksession_interval,
        notify_delay: args.notify_delay,
        main_file,
        socket: None,
    }));

    if let Err(err) = open::that(format!("http://localhost:{}/", PORT)) {
        println!("Could not open browser: {err}");
        println!("Please open http://localhost:{}/ manually", PORT);
    } else {
        println!("Opened TeXLa at http://localhost:{}/", PORT);
    }

    start_axum(core).await;
}
