use crate::engine;
use crate::key;
use crate::pins;
use crate::provider;
use crate::system::System;
use crate::toolchain;
use anyhow::{anyhow, Result};
use clap::Parser;

pub fn run(sys: &System, args: crate::cli::SetupArgs) -> Result<()> {
    let provider_name = args.provider.as_deref().unwrap_or("nanogpt");
    let provider = provider::resolve(provider_name).ok_or_else(|| anyhow!("unknown provider"))?;

    // toolchain
    println!("toolchain: installing...");
    toolchain::ensure_all(sys)?;
    println!("toolchain: ok");

    // engine settings
    let mut settings = engine::load(sys);
    // apply given over saved
    if let Some(g) = &args.ghidra { settings.ghidra = Some(g.clone()); }
    if let Some(j) = &args.java_home { settings.java_home = Some(j.clone()); }

    // compatibility check if both present (or saved)
    let g = settings.ghidra.clone();
    let j = settings.java_home.clone();
    if let (Some(g), Some(j)) = (g, j) {
        if let Err(e) = engine::compatible(&g, &j) {
            // report and save nothing
            eprintln!("engine incompatible: {e}");
            // don't save
            settings = engine::load(sys);
            // skip saving
            // but still obtain key etc? requirement: exit non-zero
            // We'll exit
            return Err(anyhow!("engine incompatible: {e}"));
        }
    }
    // save settings if provided
    if args.ghidra.is_some() || args.java_home.is_some() {
        engine::save(sys, &settings)?;
    }

    // key
    println!("key: obtaining...");
    let key = key::resolve(sys, &provider)?;
    let _ = key;
    println!("key: ok");

    // instructions already installed by toolchain
    println!("instructions: ok");

    // engine doctor
    let env = crate::rea::rea_env(sys);
    let (healthy, out) = engine::doctor(sys, &env);
    if healthy {
        println!("engine: ready");
    } else {
        println!("engine: not ready");
        // report failing scope_checks? just print output
        eprintln!("{}", out);
        println!("setup flags: --ghidra <dir> --java-home <dir>");
    }
    Ok(())
}
