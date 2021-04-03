
#[derive(thiserror::Error, Debug)]
pub enum ApplicationError {
    #[error("Usage: {0}")]
    UsageError(String),

    #[error("IO error {0}")]
    IoError(#[from] std::io::Error),

    #[error("Name resolution error: {0}")]
    NameResolutionError(String),
}