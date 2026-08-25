use std::path::PathBuf;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum ForgeWatchError {
    #[error("project path is not a directory: {0}")]
    NotADirectory(PathBuf),
    #[error("cannot access project path: {0}: {1}")]
    Access(PathBuf, std::io::Error),
    #[error("invalid UTF-8 in {path}: required by {manifest}")]
    InvalidUtf8 {
        path: PathBuf,
        manifest: &'static str,
    },
}

pub type Result<T> = std::result::Result<T, ForgeWatchError>;
