mod executor;
mod params;
mod util;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Show rename commands for each epub file
    Rename(params::rename::RenameParams),
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::Rename(v)) => {
            executor::rename::execute(v);
        }
        None => eprintln!("No subcommand provided!\nCheck the subcommands with `repub -h`"),
    }
}
