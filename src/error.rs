use std::{io, process::Command, result};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("{0}")]
    TomlSer(#[from] toml::ser::Error),
    #[error("{0}")]
    TomlDe(#[from] toml::de::Error),
    #[error("command failure")]
    Command(Command),
}

pub type Result<T> = result::Result<T, Error>;
