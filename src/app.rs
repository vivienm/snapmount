use std::fs;
use std::io;
use std::path::{self, Path, PathBuf};

use structopt::clap::crate_name;
use structopt::clap::Shell;
use structopt::StructOpt;

use crate::cli;
use crate::config::{Config, ConfigMount, ConfigSnapshot};
use crate::error::Result;
use crate::lvm;
use crate::mount::{mount, unmount};

fn create_snapshot(config_snap: &ConfigSnapshot) -> Result<()> {
    match config_snap {
        ConfigSnapshot::Lvm { source, name, size } => {
            let source_lv = lvm::LogicalVolume::from_path(source);
            source_lv.snapshot(name, size)?;
        }
    }
    Ok(())
}

fn remove_snapshot(config_snap: &ConfigSnapshot) -> Result<()> {
    match config_snap {
        ConfigSnapshot::Lvm { source, name, .. } => {
            let target_lv = lvm::LogicalVolume::from_path(source).with_name(name);
            target_lv.remove()?;
        }
    }
    Ok(())
}

fn get_mount_target<P>(toplevel: P, config_mount: &ConfigMount) -> PathBuf
where
    P: AsRef<Path>,
{
    let root_dir: &Path = path::Component::RootDir.as_ref();
    let target = config_mount.target.as_ref().unwrap_or(&config_mount.source);
    if target == root_dir {
        toplevel.as_ref().to_path_buf()
    } else {
        toplevel
            .as_ref()
            .join(target.strip_prefix(root_dir).unwrap_or(&target))
    }
}

fn create_mount<P>(toplevel: P, config_mount: &ConfigMount) -> Result<()>
where
    P: AsRef<Path>,
{
    if config_mount.if_exists && !config_mount.source.exists() {
        log::info!(
            "Skipping mount: {} does not exist",
            &config_mount.source.display()
        );
        return Ok(());
    }
    let target = get_mount_target(toplevel, config_mount);
    mount(
        &config_mount.source,
        &target,
        config_mount.type_.as_deref(),
        &config_mount.options,
    )
}

fn remove_mount<P>(toplevel: P, config_mount: &ConfigMount) -> Result<()>
where
    P: AsRef<Path>,
{
    let target = get_mount_target(toplevel, config_mount);
    unmount(target)
}

fn mount_all(toplevel: &Path, config: &Config) -> Result<()> {
    for config_snap in config.snapshots.iter() {
        create_snapshot(config_snap)?;
    }
    for config_mount in config.mounts.iter() {
        create_mount(toplevel, config_mount)?;
    }
    Ok(())
}

fn unmount_all(toplevel: &Path, config: &Config) -> Result<()> {
    for config_mount in config.mounts.iter().rev() {
        remove_mount(toplevel, config_mount)?;
    }
    for config_snap in config.snapshots.iter().rev() {
        remove_snapshot(config_snap)?;
    }
    Ok(())
}

fn dump_config(config: &Config) -> Result<()> {
    let stdout = io::stdout();
    config.dump(stdout)?;
    Ok(())
}

fn dump_completion(shell: &Shell) {
    cli::Args::clap().gen_completions_to(crate_name!(), *shell, &mut io::stdout());
}

#[inline]
fn log_done() {
    log::info!("All done");
}

pub fn main(args: &cli::Args) -> Result<()> {
    if let cli::ArgsCommand::Completion { shell } = args.command {
        dump_completion(&shell);
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

    match &args.command {
        cli::ArgsCommand::Mount {
            unmount_before,
            target,
        } => {
            if *unmount_before {
                unmount_all(target, &config)?;
            }
            mount_all(target, &config)?;
            log_done();
            Ok(())
        }
        cli::ArgsCommand::Unmount { target } => {
            unmount_all(target, &config)?;
            log_done();
            Ok(())
        }
        cli::ArgsCommand::Config => dump_config(&config),
        cli::ArgsCommand::Completion { .. } => unreachable!(),
    }
}
