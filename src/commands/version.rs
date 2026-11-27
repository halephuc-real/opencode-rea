use crate::system::System;
use anyhow::Result;

pub fn run(_sys: &System) -> Result<()> {
    println!("rea-launcher 0.1.0 (rust)");
    println!("Node.js 24.22.3");
    println!("REA 6.10.1");
    println!("OpenCode 1.19.5");
    Ok(())
}
