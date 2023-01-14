use clap::Parser;

#[derive(Parser)]
pub struct RenameParams {
    #[arg(help = "Target directory")]
    pub target_dir: String,
}
