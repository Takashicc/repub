use clap::Parser;

#[derive(Parser)]
pub struct FixParams {
    #[arg(help = "Target directory")]
    pub input_dir: String,
}
