use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IotError {

    #[error("io error: {0}")]
    IoError(#[from] io::Error),

    #[error("toml parse error: {0}")]
    TomlParseError(#[from] toml::de::Error),

    #[error("frame error: {0}")]
    FrameError(String),
}

pub type IotResult<T> = Result<T, IotError>;