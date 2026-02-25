use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IotError {

    #[error("read file error:{0}")]
    IoError(#[from] io::Error),

    #[error("toml parse error:{0}")]
    TomlParseError(#[from] toml::de::Error),
}

pub type IotResult<T> = Result<T, IotError>;