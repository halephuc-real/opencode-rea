mod archive;
mod cli;
mod commands;
mod config;
mod engine;
mod key;
mod oauth;
mod opencode;
mod pins;
mod provider;
mod rea;
mod rea_tools;
mod system;
mod toolchain;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command};

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli) {
        eprintln!("opencode-rea: {e}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<()> {
    let sys = system::System::default_with_version(env!("CARGO_PKG_VERSION"));
    match cli.command {
        None => {

            crate::commands::launch::run(&sys, cli)
        }
        Some(Command::Setup(args)) => crate::commands::setup::run(&sys, args),
        Some(Command::Logout) => crate::commands::logout::run(&sys),
        Some(Command::Version) => crate::commands::version::run(&sys),
    }
}
