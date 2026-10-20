use std::path::PathBuf;

use chrono::NaiveDate;

use crate::banlist::FormatRegion;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("i/o error on {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("parse error (line {line}): {message}")]
    Parse { line: usize, message: String },

    #[error("no {region:?} banlist is in effect on {date} (earliest known list: {earliest:?})")]
    NoBanlistForDate {
        region: FormatRegion,
        date: NaiveDate,
        earliest: Option<NaiveDate>,
    },

    #[error("invalid ydke:// url: {0}")]
    InvalidYdke(String),

    #[error("{} deck line(s) could not be resolved: {}", .0.len(), .0.join(", "))]
    UnresolvedCards(Vec<String>),
}

impl Error {
    pub(crate) fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Error::Io { path: path.into(), source }
    }
}
