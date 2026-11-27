use crate::engine;
use crate::system::System;
use std::collections::BTreeMap;
use std::env;

pub fn rea_env(sys: &System) -> BTreeMap<String, String> {
    let mut env = BTreeMap::new();

    for (k, v) in env::vars() {
        if k == "JAVA_HOME" || k.starts_with("GHIDRA_") || k.starts_with("REA_") {
            env.insert(k, v);
        }
    }

    let settings = engine::load(sys);
    if let Some(v) = settings.ghidra { env.insert("GHIDRA_INSTALL_DIR".to_string(), v); }
    if let Some(v) = settings.java_home { env.insert("JAVA_HOME".to_string(), v); }

    env.insert("NANO_GPT_API_KEY".to_string(), String::new());
    env.insert("NEARAI_API_KEY".to_string(), String::new());
    env
}

pub fn skills_dir(sys: &System) -> std::path::PathBuf {
    sys.rea_dir().join("opencode/skills")
}
