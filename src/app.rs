use std::{
    io,
    path::{self, Path, PathBuf},
    process,
};

use fs_err as fs;
use structopt::{clap::crate_name, StructOpt};

use crate::{
    cli,
    command::{FakeRunner, ProcessRunner, Runner},
    config::{Config, ConfigMount, ConfigSnapshot},
    error::Result,
    lvm,
    mount::{mount, unmount},
};

fn load_config<P>(config_path: P) -> Result<Config>
where
    P: AsRef<Path>,
{
    log::debug!(
        "Loading configuration file {}",
        config_path.as_ref().display()
    );
    let config_file = fs::File::open(config_path.as_ref())?;
    Config::load(config_file)
}

fn create_snapshot<R>(runner: &R, config_snap: &ConfigSnapshot) -> Result<()>
where
    R: Runner,
{
    match config_snap {
        ConfigSnapshot::Lvm { source, name, size } => {
            let source_lv = lvm::LogicalVolume::from_path(source);
            source_lv.snapshot(runner, name, size)?;
        }
    }
    Ok(())
}

fn remove_snapshot<R>(runner: &R, config_snap: &ConfigSnapshot) -> Result<()>
where
    R: Runner,
{
    match config_snap {
        ConfigSnapshot::Lvm { source, name, .. } => {
            let target_lv = lvm::LogicalVolume::from_path(source).with_name(name);
            target_lv.remove(runner)?;
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
            .join(target.strip_prefix(root_dir).unwrap_or(target))
    }
}

fn create_mount<P, R>(runner: &R, toplevel: P, config_mount: &ConfigMount) -> Result<()>
where
    P: AsRef<Path>,
    R: Runner,
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
        runner,
        &config_mount.source,
        &target,
        config_mount.type_.as_deref(),
        &config_mount.options,
    )
}

fn remove_mount<P, R>(runner: &R, toplevel: P, config_mount: &ConfigMount) -> Result<()>
where
    P: AsRef<Path>,
    R: Runner,
{
    let target = get_mount_target(toplevel, config_mount);
    unmount(runner, target)
}

fn mount_all<R>(runner: &R, toplevel: &Path, config: &Config) -> Result<()>
where
    R: Runner,
{
    for config_snap in config.snapshots.iter() {
        create_snapshot(runner, config_snap)?;
    }
    for config_mount in config.mounts.iter() {
        create_mount(runner, toplevel, config_mount)?;
    }
    Ok(())
}

fn unmount_all<R>(runner: &R, toplevel: &Path, config: &Config) -> Result<()>
where
    R: Runner,
{
    for config_mount in config.mounts.iter().rev() {
        remove_mount(runner, toplevel, config_mount)?;
    }
    for config_snap in config.snapshots.iter().rev() {
        remove_snapshot(runner, config_snap)?;
    }
    Ok(())
}

fn unmount_mount_all<R>(
    runner: &R,
    unmount_before: bool,
    toplevel: &Path,
    config: &Config,
) -> Result<()>
where
    R: Runner,
{
    if unmount_before {
        unmount_all(runner, toplevel, config)?;
    }
    mount_all(runner, toplevel, config)?;
    Ok(())
}

#[inline]
fn log_done() {
    log::info!("All done");
}

trait CliCommand {
    fn run<R>(&self, runner: &R, config_path: &Path) -> Result<()>
    where
        R: Runner;
}

impl CliCommand for cli::ArgsCommandMount {
    fn run<R>(&self, runner: &R, config_path: &Path) -> Result<()>
    where
        R: Runner,
    {
        let config = load_config(config_path)?;
        unmount_mount_all(runner, self.unmount_before, &self.target, &config)?;
        log_done();
        Ok(())
    }
}

impl CliCommand for cli::ArgsCommandUnmount {
    fn run<R>(&self, runner: &R, config_path: &Path) -> Result<()>
    where
        R: Runner,
    {
        let config = load_config(config_path)?;
        unmount_all(runner, &self.target, &config)?;
        log_done();
        Ok(())
    }
}

impl CliCommand for cli::ArgsCommandRun {
    fn run<R>(&self, runner: &R, config_path: &Path) -> Result<()>
    where
        R: Runner,
    {
        let config = load_config(config_path)?;
        unmount_mount_all(runner, self.unmount_before, &self.target, &config)?;
        let mut command = process::Command::new(&self.program);
        command.args(&self.args);
        let status = runner.run(&mut command)?;
        unmount_all(runner, &self.target, &config)?;
        log_done();
        process::exit(status.code().unwrap_or_default())
    }
}

impl CliCommand for cli::ArgsCommandConfig {
    fn run<R>(&self, _runner: &R, config_path: &Path) -> Result<()>
    where
        R: Runner,
    {
        let config = load_config(config_path)?;
        let stdout = io::stdout();
        config.dump(stdout)?;
        Ok(())
    }
}

impl CliCommand for cli::ArgsCommandCompletion {
    fn run<R>(&self, _runner: &R, _config_path: &Path) -> Result<()>
    where
        R: Runner,
    {
        cli::Args::clap().gen_completions_to(crate_name!(), self.shell, &mut io::stdout());
        Ok(())
    }
}

impl CliCommand for cli::ArgsCommand {
    fn run<R>(&self, runner: &R, config_path: &Path) -> Result<()>
    where
        R: Runner,
    {
        match self {
            cli::ArgsCommand::Mount(cmd) => cmd.run(runner, config_path),
            cli::ArgsCommand::Unmount(cmd) => cmd.run(runner, config_path),
            cli::ArgsCommand::Run(cmd) => cmd.run(runner, config_path),
            cli::ArgsCommand::Config(cmd) => cmd.run(runner, config_path),
            cli::ArgsCommand::Completion(cmd) => cmd.run(runner, config_path),
        }
    }
}

pub fn main(args: &cli::Args) {
    env_logger::Builder::new()
        .format_module_path(false)
        .format_timestamp(None)
        .filter_level(args.log_level)
        .init();
    if let Err(e) = if args.dry_run {
        args.command.run(&FakeRunner, &args.config_path)
    } else {
        args.command.run(&ProcessRunner, &args.config_path)
    } {
        log::error!("{}: {:?}", e, e);
        process::exit(1);
    }
}
