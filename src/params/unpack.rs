use clap::Parser;

#[derive(Parser)]
pub struct UnpackParams {
    #[arg(help = "Input directory")]
    pub input: String,
}
