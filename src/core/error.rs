use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("configuration error: {0}")]
    Config(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("hash mismatch: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },

    #[error("manifest error: {0}")]
    Manifest(String),

    #[error("platform error: {0}")]
    Platform(String),

    #[error("policy error: {0}")]
    Policy(String),

    #[error("verification failed: {0}")]
    Verification(String),

    #[error("already initialized")]
    AlreadyInitialized,

    #[error("not initialized")]
    NotInitialized,

    #[error("unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, Error>;
