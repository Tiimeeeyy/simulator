use std::path::PathBuf;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("duel creation failed with status {0}")]
    DuelCreationFailed(i32),

    #[error("failed to parse core message: {0}")]
    InvalidMessage(String),

    #[error("i/o error on {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("script not found: '{0}'")]
    ScriptNotFound(String),

    #[error("unexpected null pointer from core engine")]
    NullPointer,

    #[error("duel has already ended")]
    DuelAlreadyEnded,
}

impl Error {
    #[allow(dead_code)]
    pub(crate) fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Error::Io {
            path: path.into(),
            source,
        }
    }
}
