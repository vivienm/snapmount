use std::path::PathBuf;

use structopt::clap::Shell;
use structopt::StructOpt;

#[derive(StructOpt)]
pub enum ArgsCommand {
    /// Creates and mounts backup snapshots
    Mount {
        /// Unmounts and removes existing backup snapshots first
        #[structopt(short = "u", long = "unmount-before", alias = "umount-before")]
        unmount_before: bool,
        #[structopt(parse(from_os_str))]
        target: PathBuf,
    },
    /// Unmounts and removes backup snapshots
    #[structopt(alias = "umount")]
    Unmount {
        #[structopt(parse(from_os_str))]
        target: PathBuf,
    },
    /// Dumps the configuration
    Config,
    /// Generates a completion script
    Completion {
        /// Shell to produce a completion file for
        #[structopt(possible_values = &Shell::variants())]
        shell: Shell,
    },
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
