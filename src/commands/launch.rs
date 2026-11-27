use crate::config;
use crate::key;
use crate::opencode;
use crate::pins;
use crate::provider;
use crate::system::System;
use crate::toolchain;
use anyhow::{anyhow, Result};
use clap::Parser;
use std::process::exit;

pub fn run(sys: &System, cli: crate::cli::Cli) -> Result<()> {
    // version flag? handled by clap --version maybe not. 
    if !pins::supported_platform() {
        return Err(anyhow!("unsupported platform"));
    }
    // ensure toolchain
    toolchain::ensure_all(sys)?;
    let provider = provider::resolve(&cli.provider).ok_or_else(|| {
        anyhow!("unknown provider {}; known: {}", cli.provider, provider::known().iter().map(|p| p.name).collect::<Vec<_>>().join(", "))
    })?;
    if cli.print_config {
        let cfg = config::build(sys, &provider, cli.model.as_deref(), cli.ask);
        println!("{}", serde_json::to_string_pretty(&cfg)?);
        return Ok(());
    }
    let key = key::resolve(sys, &provider)?;
    // print line
    let model_id = provider::model_id(&provider, cli.model.as_deref());
    eprintln!("{}: {}", provider.name, model_id);
    // launch
    let code = opencode::launch(sys, &provider, cli.model.as_deref(), cli.ask, &cli.args, &key)?;
    exit(code);
}
