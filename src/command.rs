use std::process::Command;

use crate::error::{Error, Result};

pub fn check_run(mut command: Command) -> Result<()> {
    log::debug!("Run command {:?}", &command);
    let status = command.status()?;
    if !status.success() {
        return Err(Error::Command(command));
    }
    Ok(())
}
