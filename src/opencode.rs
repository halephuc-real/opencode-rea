use crate::config;
use crate::provider::Provider;
use crate::system::System;
use serde_json;
use anyhow::Result;
use std::process::{Command, ExitStatus};

pub fn launch(sys: &System, provider: &Provider, model: Option<&str>, ask: bool, trailing: &[String], key: &str) -> Result<i32> {
    let cfg = config::build(sys, provider, model, ask);
    let cfg_str = serde_json::to_string(&cfg)?;
    let bin = if cfg!(windows) {
        sys.opencode_dir().join("bin/opencode.exe")
    } else {
        sys.opencode_dir().join("bin/opencode")
    };
    let mut cmd = Command::new(&bin);

    let mut args = trailing.to_vec();

    if !ask {
        if args.is_empty() {

            args.insert(0, "--auto".to_string());
        } else if args[0] == "run" {

            let mut new = vec!["run".to_string(), "--auto".to_string()];
            new.extend(args[1..].iter().cloned());
            args = new;
        } else if args[0].starts_with('-') || args[0] == "." {
            args.insert(0, "--auto".to_string());
        } else {

        }
    }

    cmd.args(&args);
    cmd.current_dir(std::env::current_dir().unwrap_or_default());

    cmd.env("OPENCODE_CONFIG_CONTENT", &cfg_str);
    cmd.env("OPENCODE_CONFIG_DIR", sys.rea_dir().join("opencode"));
    cmd.env("OPENCODE_DISABLE_PROJECT_CONFIG", "1");
    cmd.env("OPENCODE_DISABLE_CLAUDE_CODE", "1");
    cmd.env("OPENCODE_DISABLE_EXTERNAL_SKILLS", "1");
    cmd.env("OPENCODE_DISABLE_AUTOUPDATE", "1");
    cmd.env("OPENCODE_DISABLE_LSP_DOWNLOAD", "1");

    cmd.env_remove("OPENCODE_ENABLE_EXA");

    cmd.env(provider.key_var, key);

    let _ = ctrlc::set_handler(|| {});
    let status: ExitStatus = cmd.status()?;
    Ok(status.code().unwrap_or(1))
}
