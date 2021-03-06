use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use crate::{command::Runner, error::Result};

pub struct LogicalVolume<P> {
    pub path: P,
}

impl<P> LogicalVolume<P> {
    pub fn from_path(path: P) -> Self {
        Self { path }
    }
}

impl<P> LogicalVolume<P>
where
    P: AsRef<Path>,
{
    pub fn exists(&self) -> bool {
        self.path.as_ref().exists()
    }

    pub fn with_name(&self, name: &str) -> LogicalVolume<PathBuf> {
        LogicalVolume::from_path(self.path.as_ref().with_file_name(name))
    }

    pub fn snapshot<R>(&self, runner: &R, name: &str, size: &str) -> Result<()>
    where
        R: Runner,
    {
        log::info!(
            "Creating snapshot {} of {}",
            name,
            self.path.as_ref().display()
        );
        let mut command = Command::new("lvcreate");
        command
            .arg("--quiet")
            .arg("--snapshot")
            .arg("--size")
            .arg(size)
            .arg("--name")
            .arg(name)
            .arg(self.path.as_ref())
            .stdout(Stdio::null());
        runner.check_run(command)
    }

    pub fn remove<R>(&self, runner: &R) -> Result<()>
    where
        R: Runner,
    {
        if self.exists() {
            log::info!("Removing snapshot {}", self.path.as_ref().display());
            let mut command = Command::new("lvremove");
            command
                .arg("--quiet")
                .arg("--force")
                .arg(self.path.as_ref())
                .stdout(Stdio::null());
            runner.check_run(command)?;
        }
        Ok(())
    }
}
