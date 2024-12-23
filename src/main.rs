mod error;
mod executor;
mod params;
mod util;
use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Show rename commands for epub files
    Rename(params::rename::RenameParams),
    Info(params::info::InfoParams),
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::Rename(v)) => {
            executor::rename::execute(v)?;
        }
        Some(Commands::Info(v)) => {
            executor::info::execute(v)?;
        }
        None => eprintln!("No subcommand provided!\nCheck the subcommands with `repub -h`"),
    }

    Ok(())
}
