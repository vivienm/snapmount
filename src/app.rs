use std::fs;
use std::io;
use std::path::PathBuf;

use structopt::clap::crate_name;
use structopt::clap::Shell;
use structopt::StructOpt;

use crate::cli;
use crate::config::{Config, ConfigMount};
use crate::error::Result;
use crate::lvm;
use crate::mount::{mount, unmount};

fn create_toplevel_mountpoint(config: &Config) -> Result<()> {
    if config.mountpoint.create && !config.mountpoint.path.exists() {
        log::info!(
            "Creating toplevel mount directory {}",
            config.mountpoint.path.display()
        );
        fs::create_dir(&config.mountpoint.path)?;
    }
    Ok(())
}

fn remove_toplevel_mountpoint(config: &Config) -> Result<()> {
    if config.mountpoint.create && config.mountpoint.path.exists() {
        log::info!(
            "Removing toplevel mount directory {}",
            config.mountpoint.path.display()
        );
        fs::remove_dir(&config.mountpoint.path)?;
    }
    Ok(())
}

fn get_mount_target(config: &Config, config_mount: &ConfigMount) -> PathBuf {
    let target = match config_mount {
        ConfigMount::Bind { source, target } => target.as_ref().unwrap_or(source),
        ConfigMount::Lvm { target, .. } => target,
    };
    config
        .mountpoint
        .path
        .join(target.strip_prefix("/").unwrap_or(target))
}

fn create_snapshot(config_mount: &ConfigMount) -> Result<()> {
    match config_mount {
        ConfigMount::Lvm {
            source, snapshot, ..
        } => {
            let source_lv = lvm::LogicalVolume::from_path(source);
            source_lv.snapshot(&snapshot.lv_name, &snapshot.size)?;
        }
        ConfigMount::Bind { .. } => {}
    }
    Ok(())
}

fn remove_snapshot(config_mount: &ConfigMount) -> Result<()> {
    match config_mount {
        ConfigMount::Lvm {
            source, snapshot, ..
        } => {
            let target_lv = lvm::LogicalVolume::from_path(source).with_name(&snapshot.lv_name);
            target_lv.remove()?;
        }
        ConfigMount::Bind { .. } => {}
    }
    Ok(())
}

fn create_mount(config: &Config, config_mount: &ConfigMount) -> Result<()> {
    let target = get_mount_target(config, config_mount);
    match config_mount {
        ConfigMount::Lvm {
            source, snapshot, ..
        } => {
            let target_lv = lvm::LogicalVolume::from_path(source).with_name(&snapshot.lv_name);
            mount(&target_lv.path, &target, false)?;
        }
        ConfigMount::Bind { source, .. } => {
            mount(source, &target, true)?;
        }
    }
    Ok(())
}

fn command_mount(config: &Config) -> Result<()> {
    create_toplevel_mountpoint(config)?;
    for mount in config.mounts.iter() {
        create_snapshot(mount)?;
    }
    for mount in config.mounts.iter() {
        create_mount(config, mount)?;
    }
    log::info!("All done");
    Ok(())
}

fn command_unmount(config: &Config) -> Result<()> {
    unmount(&config.mountpoint.path, true)?;
    for mount in config.mounts.iter().rev() {
        remove_snapshot(mount)?;
    }
    remove_toplevel_mountpoint(config)?;
    log::info!("All done");
    Ok(())
}

fn command_config(config: &Config) -> Result<()> {
    let stdout = io::stdout();
    config.dump(stdout)?;
    Ok(())
}

fn command_completion(shell: &Shell) {
    cli::Args::clap().gen_completions_to(crate_name!(), *shell, &mut io::stdout());
}

pub fn main(args: &cli::Args) -> Result<()> {
    if let cli::ArgsCommand::Completion { shell } = args.command {
        command_completion(&shell);
        return Ok(());
    }

    env_logger::Builder::new()
        .format_module_path(false)
        .format_timestamp(None)
        .filter_level(args.log_level)
        .init();

    log::debug!("Loading configuration file {}", args.config_path.display());
    let config = {
        let config_file = fs::File::open(&args.config_path)?;
        Config::load(config_file)?
    };

    match args.command {
        cli::ArgsCommand::Mount => command_mount(&config),
        cli::ArgsCommand::Unmount => command_unmount(&config),
        cli::ArgsCommand::Config => command_config(&config),
        cli::ArgsCommand::Completion { .. } => unreachable!(),
    }
}
