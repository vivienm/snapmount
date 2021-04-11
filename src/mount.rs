use std::fs;
use std::io;
use std::os::linux::fs::MetadataExt;
use std::path::Path;
use std::process::Command;

use crate::command::check_run;
use crate::error::Result;

pub fn is_mount<P>(dir: P) -> io::Result<bool>
where
    P: AsRef<Path>,
{
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

pub fn mount<P>(source: P, target: P, type_: Option<&str>, options: &[String]) -> Result<()>
where
    P: AsRef<Path>,
{
    log::info!(
        "Mounting {} to {}",
        source.as_ref().display(),
        target.as_ref().display()
    );
    let mut command = Command::new("mount");
    if let Some(type_) = type_ {
        command.arg("--type");
        command.arg(type_);
    }
    if !options.is_empty() {
        command.arg("--options");
        command.arg(options.join(","));
    }
    command.arg(source.as_ref());
    command.arg(target.as_ref());
    check_run(command)
}

pub fn unmount<P>(target: P) -> Result<()>
where
    P: AsRef<Path>,
{
    if target.as_ref().exists() && is_mount(&target)? {
        log::info!("Unmounting {}", target.as_ref().display());
        let mut command = Command::new("umount");
        command.arg(target.as_ref());
        check_run(command)?;
    }
    Ok(())
}
