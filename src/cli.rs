use std::path::PathBuf;

use structopt::StructOpt;

#[derive(StructOpt)]
pub enum ArgsCommand {
    /// Creates and mounts backup snapshots
    Mount,
    /// Unmounts and removes backup snapshots
    Unmount,
    /// Dumps the configuration
    Config,
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
    #[structopt(subcommand)]
    pub command: ArgsCommand,
}
