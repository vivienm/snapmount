use clap::Parser;

mod app;
mod cli;
mod command;
mod config;
mod error;
mod lvm;
mod mount;

fn main() {
    app::main(&cli::Args::parse());
}
