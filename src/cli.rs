use std::ffi::OsString;
use std::path::PathBuf;

use structopt::clap::Shell;
use structopt::StructOpt;

#[derive(StructOpt)]
pub struct ArgsCommandMount {
    /// Unmounts and removes existing backup snapshots first
    #[structopt(short = "u", long = "unmount-before", alias = "umount-before")]
    pub unmount_before: bool,
    /// Toplevel mount directory
    #[structopt(parse(from_os_str))]
    pub target: PathBuf,
}

#[derive(StructOpt)]
pub struct ArgsCommandUnmount {
    /// Toplevel mount directory
    #[structopt(parse(from_os_str))]
    pub target: PathBuf,
}

#[derive(StructOpt)]
pub struct ArgsCommandRun {
    /// Unmounts and removes existing backup snapshots first
    #[structopt(short = "u", long = "unmount-before", alias = "umount-before")]
    pub unmount_before: bool,
    /// Toplevel mount directory
    #[structopt(parse(from_os_str))]
    pub target: PathBuf,
    /// Program to be lauched
    #[structopt(parse(from_os_str))]
    pub program: OsString,
    #[structopt(parse(from_os_str))]
    /// Arguments to pass to the program
    pub args: Vec<OsString>,
}

#[derive(StructOpt)]
pub struct ArgsCommandConfig {}

#[derive(StructOpt)]
pub struct ArgsCommandCompletion {
    /// Shell to produce a completion file for
    #[structopt(possible_values = &Shell::variants())]
    pub shell: Shell,
}

#[derive(StructOpt)]
pub enum ArgsCommand {
    /// Creates and mounts backup snapshots
    Mount(ArgsCommandMount),
    /// Unmounts and removes backup snapshots
    #[structopt(alias = "umount")]
    Unmount(ArgsCommandUnmount),
    /// Run an arbitrary command
    Run(ArgsCommandRun),
    /// Dumps the configuration
    Config(ArgsCommandConfig),
    /// Generates a completion script
    Completion(ArgsCommandCompletion),
}

/// Create and mount backup snapshots
#[derive(StructOpt)]
#[structopt(global_setting = structopt::clap::AppSettings::ColoredHelp)]
pub struct Args {
    /// Logging level.
    #[structopt(
        short = "l",
        long = "log-level",
        env = "SNAPMOUNT_LOG_LEVEL",
        default_value = "info",
        possible_values = &["off", "error", "warn", "info", "debug", "trace"]
    )]
    pub log_level: log::LevelFilter,
    /// Path to the configuration file.
    #[structopt(
        short = "c",
        long = "config",
        env = "SNAPMOUNT_CONFIG",
        default_value = "/etc/snapmount/config.toml"
    )]
    pub config_path: PathBuf,
    /// Do not actually run commands
    #[structopt(short = "n", long = "dry-run")]
    pub dry_run: bool,
    #[structopt(subcommand)]
    pub command: ArgsCommand,
}
