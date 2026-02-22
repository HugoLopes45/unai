use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UnaiError {
    #[error("Cannot read '{path}': {source}")]
    FileRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Cannot read stdin: {source}")]
    StdinRead {
        #[source]
        source: std::io::Error,
    },

    #[error("stdin input exceeds 64 MiB size limit")]
    StdinTooLarge,

    #[error("Cannot parse config at '{path}': {source}")]
    ConfigParse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },

    #[error("Invalid config: {0}")]
    ConfigInvalid(String),

    #[error("Invalid rule: {0}")]
    InvalidRule(String),

    #[error("Cannot write output to '{path}': {source}")]
    FileWrite {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

pub type Result<T> = std::result::Result<T, UnaiError>;

/// Exit codes for unai.
/// 0  = success (no findings, or findings auto-fixed)
/// 1  = I/O error
/// 2  = config / rule parse error
/// 10 = findings exist (used with --fail)
pub mod exit_code {
    #[allow(dead_code)]
    pub const SUCCESS: i32 = 0;
    pub const IO_ERROR: i32 = 1;
    pub const CONFIG_ERROR: i32 = 2;
    pub const FINDINGS: i32 = 10;
}
