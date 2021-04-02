use std::fs;
use std::path::PathBuf;

use crate::cli;
use crate::config::{Config, ConfigMount};
use crate::error::Result;
use crate::lvm;
use crate::mount;

fn create_mountpoint(config: &Config) -> Result<()> {
    if config.mountpoint.create && !config.mountpoint.path.exists() {
        log::info!(
            "Creating toplevel mount directory {}",
            config.mountpoint.path.display()
        );
        fs::create_dir(&config.mountpoint.path)?;
    }
    Ok(())
}

fn remove_mountpoint(config: &Config) -> Result<()> {
    if config.mountpoint.create && config.mountpoint.path.exists() {
        log::info!(
            "Removing toplevel mount directory {}",
            config.mountpoint.path.display()
        );
        fs::remove_dir(&config.mountpoint.path)?;
    }
    Ok(())
}

fn mount_target(config: &Config, mount: &ConfigMount) -> PathBuf {
    let target = match mount {
        ConfigMount::Bind {
            source,
            target: None,
        } => source,
        ConfigMount::Bind {
            target: Some(target),
            ..
        } => target,
        ConfigMount::Lvm { target, .. } => target,
    };
    config
        .mountpoint
        .path
        .join(target.strip_prefix("/").unwrap_or(target))
}

fn handle_mount(config: &Config, mount: &ConfigMount) -> Result<()> {
    let target = mount_target(config, mount);
    match mount {
        ConfigMount::Lvm {
            source, snapshot, ..
        } => {
            // Create the LV snapshot.
            let source_lv = lvm::LogicalVolume::from_path(source);
            source_lv.snapshot(&snapshot.lv_name, &snapshot.size)?;
            // Mount it.
            let target_lv = source_lv.with_name(&snapshot.lv_name);
            mount::mount_ro(&target_lv.path, &target)?;
        }
        ConfigMount::Bind { source, .. } => {
            // Bind mount.
            mount::mount_bind(source, &target)?;
        }
    }
    Ok(())
}

pub fn command_mount(config: &Config) -> Result<()> {
    create_mountpoint(config)?;
    for mount in config.mounts.iter() {
        handle_mount(config, mount)?;
    }
    Ok(())
}

fn handle_unmount(config: &Config, mount: &ConfigMount) -> Result<()> {
    let target = mount_target(config, mount);
    mount::unmount(&target)?;
    match mount {
        ConfigMount::Lvm {
            source, snapshot, ..
        } => {
            // Remove the LV snapshot.
            let target_lv = lvm::LogicalVolume::from_path(source).with_name(&snapshot.lv_name);
            target_lv.remove()?;
        }
        ConfigMount::Bind { .. } => {}
    }
    Ok(())
}

pub fn command_unmount(config: &Config) -> Result<()> {
    for mount in config.mounts.iter().rev() {
        handle_unmount(config, mount)?;
    }
    remove_mountpoint(config)?;
    Ok(())
}

pub fn main(args: &cli::Args) -> Result<()> {
    env_logger::Builder::new()
        .format_module_path(false)
        .format_timestamp(None)
        .filter_level(args.log_level)
        .init();

    log::info!("Loading configuration file {}", args.config_path.display());
    let config = Config::from_path(&args.config_path)?;

    match args.command {
        cli::ArgsCommand::Mount => command_mount(&config),
        cli::ArgsCommand::Unmount => command_unmount(&config),
    }
}
