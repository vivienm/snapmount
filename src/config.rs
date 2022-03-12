use std::{
    io::{Read, Write},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

use crate::error;

fn default_lvm_snapshot_size() -> String {
    "1G".to_string()
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, tag = "type")]
pub enum ConfigSnapshot {
    #[serde(rename = "lvm")]
    Lvm {
        source: PathBuf,
        name: String,
        #[serde(default = "default_lvm_snapshot_size")]
        size: String,
    },
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigMount {
    pub source: PathBuf,
    #[serde(default)]
    pub target: Option<PathBuf>,
    #[serde(default, rename = "type")]
    pub type_: Option<String>,
    #[serde(default)]
    pub options: Vec<String>,
    #[serde(default)]
    pub if_exists: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub snapshots: Vec<ConfigSnapshot>,
    #[serde(default)]
    pub mounts: Vec<ConfigMount>,
}

impl Config {
    pub fn load<R: Read>(mut file: R) -> error::Result<Self> {
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        let config: Config = toml::from_slice(&data)?;
        Ok(config)
    }

    pub fn dump<W: Write>(&self, mut file: W) -> error::Result<()> {
        let s = toml::to_string_pretty(self)?;
        file.write_all(s.as_bytes())?;
        Ok(())
    }
}
