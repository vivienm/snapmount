use std::fs;
use std::io;
use std::path::{self, Path, PathBuf};
use std::process;

use structopt::clap::crate_name;
use structopt::StructOpt;

use crate::cli;
use crate::config::{Config, ConfigMount, ConfigSnapshot};
use crate::error::Result;
use crate::lvm;
use crate::mount::{mount, unmount};

fn load_config<P>(config_path: P) -> Result<Config>
where
    P: AsRef<Path>,
{
    log::debug!(
        "Loading configuration file {}",
        config_path.as_ref().display()
    );
    let config_file = fs::File::open(&config_path)?;
    Config::load(config_file)
}

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

#[inline]
fn log_done() {
    log::info!("All done");
}

trait CliCommand {
    fn run(&self, config_path: &Path) -> Result<()>;
}

impl CliCommand for cli::ArgsCommandMount {
    fn run(&self, config_path: &Path) -> Result<()> {
        let config = load_config(config_path)?;
        if self.unmount_before {
            unmount_all(&self.target, &config)?;
        }
        mount_all(&self.target, &config)?;
        log_done();
        Ok(())
    }
}

impl CliCommand for cli::ArgsCommandUnmount {
    fn run(&self, config_path: &Path) -> Result<()> {
        let config = load_config(config_path)?;
        unmount_all(&self.target, &config)?;
        log_done();
        Ok(())
    }
}

impl CliCommand for cli::ArgsCommandRun {
    fn run(&self, config_path: &Path) -> Result<()> {
        let config = load_config(config_path)?;
        if self.unmount_before {
            unmount_all(&self.target, &config)?;
        }
        mount_all(&self.target, &config)?;
        let mut command = process::Command::new(&self.program);
        command.args(&self.args);
        let status = crate::command::run(&mut command)?;
        unmount_all(&self.target, &config)?;
        log_done();
        process::exit(status.code().unwrap_or_default())
    }
}

impl CliCommand for cli::ArgsCommandConfig {
    fn run(&self, config_path: &Path) -> Result<()> {
        let config = load_config(config_path)?;
        let stdout = io::stdout();
        config.dump(stdout)?;
        Ok(())
    }
}

impl CliCommand for cli::ArgsCommandCompletion {
    fn run(&self, _config_path: &Path) -> Result<()> {
        cli::Args::clap().gen_completions_to(crate_name!(), self.shell, &mut io::stdout());
        Ok(())
    }
}

impl CliCommand for cli::ArgsCommand {
    fn run(&self, config_path: &Path) -> Result<()> {
        match self {
            cli::ArgsCommand::Mount(cmd) => cmd.run(config_path),
            cli::ArgsCommand::Unmount(cmd) => cmd.run(config_path),
            cli::ArgsCommand::Run(cmd) => cmd.run(config_path),
            cli::ArgsCommand::Config(cmd) => cmd.run(config_path),
            cli::ArgsCommand::Completion(cmd) => cmd.run(config_path),
        }
    }
}

pub fn main(args: &cli::Args) -> Result<()> {
    env_logger::Builder::new()
        .format_module_path(false)
        .format_timestamp(None)
        .filter_level(args.log_level)
        .init();
    args.command.run(&args.config_path)
}
