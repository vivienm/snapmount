use std::path::{Path, PathBuf};
use std::process::Command;

use crate::command::check_run;
use crate::error::Result;

pub struct LogicalVolume {
    pub path: PathBuf,
}

impl LogicalVolume {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    pub fn with_name(&self, name: &str) -> Self {
        Self {
            path: self.path.with_file_name(name),
        }
    }

    pub fn snapshot(&self, name: &str, size: &str) -> Result<()> {
        log::info!("Creating snapshot {} of {}", name, self.path.display());
        let mut command = Command::new("lvcreate");
        command
            .arg("--quiet")
            .arg("--snapshot")
            .arg("--size")
            .arg(size)
            .arg("--name")
            .arg(name)
            .arg(&self.path);
        check_run(command)
    }

    pub fn remove(&self) -> Result<()> {
        if self.exists() {
            log::info!("Removing snapshot {}", self.path.display());
            let mut command = Command::new("lvremove");
            command.arg("--quiet").arg("--force").arg(&self.path);
            check_run(command)?;
        }
        Ok(())
    }
}
