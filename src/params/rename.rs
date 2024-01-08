use clap::Parser;

#[derive(Parser)]
pub struct RenameParams {
    #[arg(help = "Input file or directory")]
    pub input: String,
}
