use crate::system::System;
use anyhow::Result;

pub fn run(_sys: &System) -> Result<()> {
    for service in ["opencode-rea", "rea-launcher"] {
        for p in ["nanogpt", "nearai"] {
            if let Ok(e) = keyring::Entry::new(service, p) {
                let _ = e.delete_credential();
            }
        }
    }
    println!("logged out");
    Ok(())
}
