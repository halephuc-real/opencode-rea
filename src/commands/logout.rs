use crate::system::System;
use anyhow::Result;

pub fn run(_sys: &System) -> Result<()> {
    for p in ["nanogpt", "nearai"] {
        if let Ok(e) = keyring::Entry::new("rea-launcher", p) {
            let _ = e.delete_credential();
        }
    }
    println!("logged out");
    Ok(())
}
