use std::fs;
use std::io;
use std::os::linux::fs::MetadataExt;
use std::path::Path;
use std::process::Command;

use crate::command::check_run;
use crate::error::Result;

pub fn mount_ro<P: AsRef<Path>>(source: P, target: P) -> Result<()> {
    log::info!(
        "Mounting {} to {}",
        source.as_ref().display(),
        target.as_ref().display()
    );
    let mut command = Command::new("mount");
    command
        .arg("-o")
        .arg("ro")
        .arg(source.as_ref())
        .arg(target.as_ref());
    check_run(command)
}

pub fn mount_bind<P: AsRef<Path>>(source: P, target: P) -> Result<()> {
    log::info!(
        "Bind mounting {} to {}",
        source.as_ref().display(),
        target.as_ref().display()
    );
    let mut command = Command::new("mount");
    command
        .arg("--bind")
        .arg(source.as_ref())
        .arg(target.as_ref());
    check_run(command)
}

pub fn is_mountpoint<P: AsRef<Path>>(dir: P) -> io::Result<bool> {
    let dir_meta = fs::metadata(&dir)?;
    let file_type = dir_meta.file_type();

    if file_type.is_symlink() {
        // A symlink can never be a mount point.
        return Ok(false);
    }

    Ok(if let Some(parent) = dir.as_ref().parent() {
        let parent_meta = fs::metadata(parent)?;
        parent_meta.st_dev() != dir_meta.st_dev()
    } else {
        // If the directory does not have a parent, then it is the root filesystem.
        false
    })
}

pub fn unmount<P: AsRef<Path>>(target: P) -> Result<()> {
    if target.as_ref().exists() && is_mountpoint(&target)? {
        log::info!("Unmount {}", target.as_ref().display());
        let mut command = Command::new("umount");
        command.arg(target.as_ref());
        check_run(command)?;
    }
    Ok(())
}
