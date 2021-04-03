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

#[derive(Default)]
pub struct Mounter {
    bind: bool,
    read_only: bool,
}

impl Mounter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bind(&mut self) -> &mut Self {
        self.bind = true;
        self
    }

    pub fn read_only(&mut self) -> &mut Self {
        self.read_only = true;
        self
    }

    pub fn mount<P: AsRef<Path>>(&self, source: P, target: P) -> Result<()> {
        log::info!(
            "Mounting {} to {}",
            source.as_ref().display(),
            target.as_ref().display()
        );
        let mut command = Command::new("mount");
        if self.bind {
            command.arg("--bind");
        }
        if self.read_only {
            command.arg("--read-only");
        }
        command.arg(source.as_ref()).arg(target.as_ref());
        check_run(command)
    }
}

#[derive(Default)]
pub struct Unmounter {
    recursive: bool,
}

impl Unmounter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn recursive(&mut self) -> &mut Self {
        self.recursive = true;
        self
    }

    pub fn unmount<P: AsRef<Path>>(&self, target: P) -> Result<()> {
        if target.as_ref().exists() && is_mount(&target)? {
            log::info!("Unmounting {}", target.as_ref().display());
            let mut command = Command::new("umount");
            if self.recursive {
                command.arg("--recursive");
            }
            command.arg(target.as_ref());
            check_run(command)?;
        }
        Ok(())
    }
}
