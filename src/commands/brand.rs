use crate::brand::{self, BrandStatus};
use crate::cli::BrandArgs;
use crate::system::System;
use crate::toolchain;
use anyhow::Result;

pub fn run(sys: &System, args: BrandArgs) -> Result<()> {
    if args.restore {
        brand::restore(sys)?;
        println!("Restored the original OpenCode binary.");
        return Ok(());
    }

    if args.status {
        let status = brand::status(sys)?;
        match status {
            BrandStatus::NotInstalled => println!("OpenCode is not installed yet."),
            BrandStatus::Original => println!("OpenCode is installed but not branded."),
            BrandStatus::Branded => println!("OpenCode is branded as {}.", brand::BRAND_NAME),
            BrandStatus::Patched => println!("OpenCode is branded as {}.", brand::BRAND_NAME),
            BrandStatus::Unknown => println!("OpenCode binary does not match a known build."),
        }
        return Ok(());
    }

    toolchain::ensure_all(sys)?;
    match brand::ensure(sys)? {
        BrandStatus::Patched => println!("Branded OpenCode as {}.", brand::BRAND_NAME),
        BrandStatus::Branded => println!("OpenCode is already branded as {}.", brand::BRAND_NAME),
        BrandStatus::NotInstalled => println!("OpenCode is not installed yet."),
        other => println!("No branding change applied: {:?}", other),
    }
    Ok(())
}
