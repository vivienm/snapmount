use std::fs;
use std::io;
use std::os::linux::fs::MetadataExt;
use std::path::Path;
use std::process::Command;

use crate::command::check_run;
use crate::error::Result;

pub fn is_mount<P: AsRef<Path>>(dir: P) -> io::Result<bool> {
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

pub fn mount<P: AsRef<Path>>(source: P, target: P, bind: bool) -> Result<()> {
    log::info!(
        "{} {} to {}",
        if bind { "Bind mounting" } else { "Mounting" },
        source.as_ref().display(),
        target.as_ref().display()
    );
    let mut command = Command::new("mount");
    command
        .arg("-o")
        .arg(if bind { "bind,ro" } else { "ro" })
        .arg(source.as_ref())
        .arg(target.as_ref());
    check_run(command)
}

pub fn unmount<P: AsRef<Path>>(target: P, recursive: bool) -> Result<()> {
    if target.as_ref().exists() && is_mount(&target)? {
        log::info!(
            "{} {}",
            if recursive {
                "Recursively unmounting"
            } else {
                "Unmounting"
            },
            target.as_ref().display()
        );
        let mut command = Command::new("umount");
        if recursive {
            command.arg("--recursive");
        }
        command.arg(target.as_ref());
        check_run(command)?;
    }
    Ok(())
}
