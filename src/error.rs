use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{reason}")]
    BadEPubFile { reason: String },
}
