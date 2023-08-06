mod executor;
mod params;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Fix epub issues
    Fix(params::fix::FixParams),
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Some(Commands::Fix(v)) => {
            executor::fix::execute(v);
        }
        None => eprintln!("No subcommand provided!\nCheck the subcommands with `repub -h`"),
    }
}
