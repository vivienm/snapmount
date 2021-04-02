use std::io;
use std::process::Command;
use std::result;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Io(#[from] io::Error),
    #[error("{0}")]
    Toml(#[from] toml::de::Error),
    #[error("command failure")]
    Command(Command),
}

pub type Result<T> = result::Result<T, Error>;
