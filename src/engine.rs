use crate::system::System;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct EngineSettings {
    #[serde(rename = "GHIDRA_INSTALL_DIR", default)]
    pub ghidra: Option<String>,
    #[serde(rename = "JAVA_HOME", default)]
    pub java_home: Option<String>,
}

pub fn load(sys: &System) -> EngineSettings {
    let path = sys.engine_path();
    if let Ok(s) = fs::read_to_string(&path) {
        serde_json::from_str(&s).unwrap_or_default()
    } else {
        EngineSettings::default()
    }
}

pub fn save(sys: &System, settings: &EngineSettings) -> Result<()> {
    if let Some(p) = sys.engine_path().parent() { fs::create_dir_all(p)?; }
    fs::write(sys.engine_path(), serde_json::to_string_pretty(settings)?)?;
    Ok(())
}

pub fn compatible(ghidra: &str, java_home: &str) -> Result<(), String> {

    let props_path = std::path::Path::new(ghidra).join("Ghidra/application.properties");
    let release_path = std::path::Path::new(java_home).join("release");

    if !props_path.exists() || !release_path.exists() {
        return Ok(());
    }
    let props = fs::read_to_string(&props_path).unwrap_or_default();
    let mut app_version = String::new();
    let mut min_java = String::new();
    for line in props.lines() {
        if let Some(v) = line.strip_prefix("application.version=") { app_version = v.trim().to_string(); }
        if let Some(v) = line.strip_prefix("application.java.min=") { min_java = v.trim().to_string(); }
    }
    let release = fs::read_to_string(&release_path).unwrap_or_default();
    let mut java_version = String::new();
    for line in release.lines() {
        if let Some(v) = line.strip_prefix("JAVA_VERSION=") { java_version = v.trim().trim_matches('"').to_string(); }
    }

    let gv: Vec<&str> = app_version.split('.').collect();
    if gv.len() >= 2 && gv[0] == "12" {
        let minor: i32 = gv[1].parse().unwrap_or(0);
        if minor < 2 {
            return Err(format!("Ghidra {} not admitted (need 12.2+)", app_version));
        }
    } else if !app_version.is_empty() {
        return Err(format!("Ghidra {} not admitted (need 12.x)", app_version));
    }

    if !min_java.is_empty() && !java_version.is_empty() {

        let min_major: i32 = min_java.split('.').next().unwrap_or("0").parse().unwrap_or(0);
        let java_major: i32 = java_version.split('.').next().unwrap_or("0").parse().unwrap_or(0);
        if java_major < min_major {
            return Err(format!("JDK {} older than Ghidra minimum {}", java_version, min_java));
        }
    }
    Ok(())
}

pub fn doctor(sys: &System, env: &BTreeMap<String, String>) -> (bool, String) {

    let node = if cfg!(windows) { sys.node_dir().join("node.exe") } else { sys.node_dir().join("bin/node") };
    let rea_script = sys.rea_dir().join("node_modules/rea-agents/scripts/rea.mjs");
    let out = std::process::Command::new(&node)
        .arg(&rea_script)
        .arg("doctor")
        .arg("--provider")
        .arg("ghidra")
        .arg("--json")
        .envs(env)
        .output();
    match out {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);

            let healthy = stdout.contains("\"healthy\": true") || stdout.contains("\"healthy\":true");
            (healthy, stdout.to_string())
        }
        Err(e) => (false, e.to_string()),
    }
}
