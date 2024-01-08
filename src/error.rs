use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("{reason}")]
    BadEPubFile { reason: String },
    #[error("Error at position {position} in {path}")]
    XMLReadError {
        err: quick_xml::Error,
        position: usize,
        path: String,
    },
}
