use std::{ffi::OsString, path::PathBuf};

use clap::{ArgEnum, Parser};
use clap_complete::shells::Shell;

#[derive(Parser)]
pub struct ArgsCommandMount {
    /// Unmounts and removes existing backup snapshots first
    #[clap(short = 'u', long = "unmount-before", alias = "umount-before")]
    pub unmount_before: bool,
    /// Toplevel mount directory
    #[clap(parse(from_os_str))]
    pub target: PathBuf,
}

#[derive(Parser)]
pub struct ArgsCommandUnmount {
    /// Toplevel mount directory
    #[clap(parse(from_os_str))]
    pub target: PathBuf,
}

#[derive(Parser)]
pub struct ArgsCommandRun {
    /// Unmounts and removes existing backup snapshots first
    #[clap(short = 'u', long = "unmount-before", alias = "umount-before")]
    pub unmount_before: bool,
    /// Toplevel mount directory
    #[clap(parse(from_os_str))]
    pub target: PathBuf,
    /// Program to be lauched
    #[clap(parse(from_os_str))]
    pub program: OsString,
    #[clap(parse(from_os_str))]
    /// Arguments to pass to the program
    pub args: Vec<OsString>,
}

#[derive(Parser)]
pub struct ArgsCommandConfig {}

#[derive(Parser)]
pub struct ArgsCommandCompletion {
    /// Shell to produce a completion file for
    #[clap(possible_values = Shell::value_variants().iter().filter_map(ArgEnum::to_possible_value))]
    pub shell: Shell,
}

#[derive(Parser)]
pub enum ArgsCommand {
    /// Creates and mounts backup snapshots
    Mount(ArgsCommandMount),
    /// Unmounts and removes backup snapshots
    #[clap(alias = "umount")]
    Unmount(ArgsCommandUnmount),
    /// Run an arbitrary command
    Run(ArgsCommandRun),
    /// Dumps the configuration
    Config(ArgsCommandConfig),
    /// Generates a completion script
    Completion(ArgsCommandCompletion),
}

/// Create and mount backup snapshots
#[derive(Parser)]
pub struct Args {
    /// Logging level.
    #[clap(
        short = 'l',
        long = "log-level",
        env = "SNAPMOUNT_LOG_LEVEL",
        default_value = "info",
        possible_values = &["off", "error", "warn", "info", "debug", "trace"]
    )]
    pub log_level: log::LevelFilter,
    /// Path to the configuration file.
    #[clap(
        short = 'c',
        long = "config",
        env = "SNAPMOUNT_CONFIG",
        default_value = "/etc/snapmount/config.toml"
    )]
    pub config_path: PathBuf,
    /// Does not actually run commands
    #[clap(short = 'n', long = "dry-run")]
    pub dry_run: bool,
    #[clap(subcommand)]
    pub command: ArgsCommand,
}
