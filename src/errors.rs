
#[derive(thiserror::Error, Debug)]
pub enum ApplicationError {
    #[error("Usage: {0}")]
    UsageError(String),

    #[error("IO error {0}")]
    IoError(#[from] std::io::Error),

    #[error("Name resolution error: {0}")]
    NameResolutionError(String),

    #[error("Send error: {0}")]
    SendError(#[from] futures::channel::mpsc::SendError),

    #[error("Lines codec error: {0}")]
    LinesCodecError(#[from] tokio_util::codec::LinesCodecError),

    #[error("Async task join error: {0}")]
    JoinError(#[from] tokio::task::JoinError)
}