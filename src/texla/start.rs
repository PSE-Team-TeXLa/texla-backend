use clap::Parser;

use crate::ast;
use crate::infrastructure;
use crate::texla::webserver::Webserver;

#[derive(Parser, Debug)]
struct CliArguments {
    #[arg(short, long)]
    main_file: String,
}

pub fn start() {
    println!("Starting TeXLa...");

    // append `-- --main-file main.tex` to your run command in CLion to provide the necessary CLI
    // argument
    let args = CliArguments::parse();
    println!("Opening file: {}", args.main_file);

    let webserver = Webserver::new(args.main_file);
}
