use clap::{Parser, Subcommand};

mod file_finder;
mod init;
mod update;

#[derive(Debug, Parser)]
#[command(name = "cup")]
#[command(about = "A good way to update tags in files", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Init {},
    Update {},
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Some(Commands::Update {}) => {
            update::update(init::load_config().unwrap());
        }
        Some(Commands::Init {}) => {
            if let Err(e) = init::init() {
                eprintln!("Error initializing configuration: {e}");
                std::process::exit(1);
            }
        }
        None => {
            update::update(init::load_config().unwrap());
        }
    }
}
