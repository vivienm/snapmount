use std::fmt;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::{Deserialize, Deserializer};

use crate::error;

#[derive(Deserialize)]
#[serde(untagged)]
enum StringOrAnything<T> {
    String(String),
    Anything(T),
}

impl<'de, T> StringOrAnything<T> {
    fn eval<D>(self) -> Result<T, D::Error>
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
    StringOrAnything::<T>::deserialize(deserializer)?.eval::<D>()
}

fn opt_expand_env_vars<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr + serde::Deserialize<'de>,
    <T as FromStr>::Err: fmt::Display,
{
    Option::<StringOrAnything<T>>::deserialize(deserializer)?
        .map(StringOrAnything::eval::<D>)
        .transpose()
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigMountPoint {
    #[serde(deserialize_with = "expand_env_vars")]
    pub path: PathBuf,
    #[serde(default)]
    pub create: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigMountLvmSnapshot {
    #[serde(deserialize_with = "expand_env_vars")]
    pub lv_name: String,
    #[serde(default, deserialize_with = "opt_expand_env_vars")]
    pub size: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum ConfigMount {
    #[serde(rename = "bind")]
    Bind {
        #[serde(deserialize_with = "expand_env_vars")]
        source: PathBuf,
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

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub mountpoint: ConfigMountPoint,
    pub mounts: Vec<ConfigMount>,
}

impl Config {
    pub fn from_path<P: AsRef<Path>>(path: P) -> error::Result<Self> {
        let mut file = fs::File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        let config: Config = toml::from_slice(&data)?;
        Ok(config)
    }
}
