use clap::Parser;

#[derive(Parser)]
pub struct InfoParams {
    #[arg(help = "Input directory")]
    pub input: String,
}
