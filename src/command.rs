use std::process::{self, Command};

use crate::error::{Error, Result};

pub fn run(command: &mut Command) -> Result<process::ExitStatus> {
    log::debug!("Run command {:?}", command);
    let status = command.status()?;
    Ok(status)
}

pub fn check_run(mut command: Command) -> Result<()> {
    let status = run(&mut command)?;
    if !status.success() {
        return Err(Error::Command(command));
    }
    Ok(())
}
