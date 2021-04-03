use std::fmt;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize};

use crate::error;

#[derive(Deserialize)]
#[serde(untagged)]
enum StringOrAnything<T> {
    String(String),
    Anything(T),
}

impl<'de, T> StringOrAnything<T> {
    fn expand_deserialize<D>(self) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: FromStr + serde::Deserialize<'de>,
        <T as FromStr>::Err: fmt::Display,
    {
        match self {
            StringOrAnything::String(s) => match shellexpand::env(&s) {
                Ok(value) => value.parse::<T>().map_err(serde::de::Error::custom),
                Err(err) => Err(serde::de::Error::custom(err)),
            },
            StringOrAnything::Anything(anything) => Ok(anything),
        }
    }
}

fn expand_env_vars<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + serde::Deserialize<'de>,
    <T as FromStr>::Err: fmt::Display,
{
    StringOrAnything::<T>::deserialize(deserializer)?.expand_deserialize::<D>()
}

fn opt_expand_env_vars<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + serde::Deserialize<'de>,
    <T as FromStr>::Err: fmt::Display,
{
    Option::<StringOrAnything<T>>::deserialize(deserializer)?
        .map(StringOrAnything::expand_deserialize::<D>)
        .transpose()
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigMountPoint {
    #[serde(deserialize_with = "expand_env_vars")]
    pub path: PathBuf,
    #[serde(default)]
    pub create: bool,
}

fn default_lvm_snapshot_size() -> String {
    "1G".to_string()
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigMountLvmSnapshot {
    #[serde(deserialize_with = "expand_env_vars")]
    pub lv_name: String,
    #[serde(
        default = "default_lvm_snapshot_size",
        deserialize_with = "expand_env_vars"
    )]
    pub size: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum ConfigMount {
    #[serde(rename = "bind")]
    Bind {
        #[serde(deserialize_with = "expand_env_vars")]
        source: PathBuf,
        #[serde(default)]
        writable: bool,
        #[serde(default, deserialize_with = "opt_expand_env_vars")]
        target: Option<PathBuf>,
    },
    #[serde(rename = "lvm")]
    Lvm {
        #[serde(deserialize_with = "expand_env_vars")]
        source: PathBuf,
        #[serde(deserialize_with = "expand_env_vars")]
        target: PathBuf,
        snapshot: ConfigMountLvmSnapshot,
    },
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub mountpoint: ConfigMountPoint,
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
