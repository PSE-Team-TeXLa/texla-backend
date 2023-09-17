use std::sync::{Arc, RwLock};

use clap::builder::OsStr;
use clap::Parser;

use ast::latex_constants::LATEX_FILE_EXTENSION;

use crate::infrastructure::export_manager::TexlaExportManager;
use crate::infrastructure::file_path::FilePath;
use crate::texla::core::TexlaCore;
use crate::texla::webserver::{start_axum, DEFAULT_PORT};

// the rustdocs are put into the help message of the CLI
#[derive(Parser, Debug)]
#[clap(name = "TeXLa", about = "TeXLa - a graphical LaTeX editor", version)]
struct CliArguments {
    /// The root file of your LaTeX project
    #[arg(short, long, value_names = ["path"])]
    main_file: String,

    /// The time between two subsequent pulls from the git remote (in milliseconds)
    #[arg(short, long, value_name = "duration in ms", default_value = "500")]
    pull_interval: u64,

    /// The minimum time between the last change and the according commit (in milliseconds)
    #[arg(short, long, value_name = "duration in ms", default_value = "5000")]
    worksession_interval: u64,

    /// The time 'notify' is allowed to take for picking up our own file changes and reporting them
    /// (in milliseconds)
    #[arg(short, long, value_name = "duration in ms", default_value = "100")]
    notify_delay: u64,

    /// Disable all git actions (commit, pull, push)
    #[arg(short = 'g', long)]
    no_git: bool,

    /// The port number of the web server
    #[arg(short = 'P', long, value_name = "port number",
    default_value = OsStr::from(& DEFAULT_PORT.to_string()))]
    port: u16,
}

fn verify_main_file(main_file: &FilePath) -> bool {
    if !main_file.exists() {
        println!("Cannot start TeXLa. Path for --main-file is invalid.");
        return false;
    }

    if !main_file.has_extension(LATEX_FILE_EXTENSION) {
        println!("Cannot start TeXLa. Path for --main-file is no LaTeX file.");
        return false;
    }

    true
}

pub async fn start() {
    // append `-- --main-file main.tex` to your run command in CLion to provide the necessary CLI
    // argument
    let args = CliArguments::parse();

    let main_file = FilePath::from(args.main_file);
    if !verify_main_file(&main_file) {
        return;
    }

    println!("Starting TeXLa...");

    let port = args.port;
    if port != DEFAULT_PORT {
        println!(
            "WARNING: Do not use a custom port to start TeXLa multiple times for the same \
        directory!"
        );
    }

    println!("Opening file: {}", main_file.path.to_str().unwrap());

    let core = Arc::new(RwLock::new(TexlaCore {
        export_manager: TexlaExportManager::new(main_file.directory.clone()),
        pull_interval: args.pull_interval,
        worksession_interval: args.worksession_interval,
        notify_delay: args.notify_delay,
        vcs_enabled: !args.no_git, // argument is inverted!
        main_file,
        socket: None,
    }));

    if let Err(err) = open::that(format!("http://localhost:{port}/")) {
        println!("Could not open browser: {err}");
        println!("Please open http://localhost:{port}/ manually");
    } else {
        println!("Opened TeXLa at http://localhost:{port}/");
    }

    start_axum(core, port).await;
}
