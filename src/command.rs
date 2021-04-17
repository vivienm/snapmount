use std::os::unix::process::ExitStatusExt;
use std::process::{self, Command};

use crate::error::{Error, Result};

pub trait Runner {
    fn run(&self, command: &mut Command) -> Result<process::ExitStatus>;

    fn check_run(&self, mut command: Command) -> Result<()> {
        let status = self.run(&mut command)?;
        if !status.success() {
            return Err(Error::Command(command));
        }
        Ok(())
    }
}

pub struct ProcessRunner;

impl Runner for ProcessRunner {
    fn run(&self, command: &mut Command) -> Result<process::ExitStatus> {
        log::debug!("Run command {:?}", command);
        let status = command.status()?;
        Ok(status)
    }
}

pub struct FakeRunner;

impl Runner for FakeRunner {
    fn run(&self, command: &mut Command) -> Result<process::ExitStatus> {
        log::info!("Run command {:?} [DRY RUN]", command);
        Ok(ExitStatusExt::from_raw(0))
    }

    fn check_run(&self, mut command: Command) -> Result<()> {
        let _ = self.run(&mut command);
        Ok(())
    }
}
