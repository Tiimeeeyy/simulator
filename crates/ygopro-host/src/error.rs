//! Host session error types.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum HostError {
    #[error("duel error: {0}")]
    Duel(#[from] ygopro_core::Error),

    #[error("network I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("session has already ended")]
    AlreadyEnded,

    #[error("no human response received")]
    NoResponse,
}

pub type Result<T, E = HostError> = std::result::Result<T, E>;
