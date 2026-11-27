use crate::provider::Provider;
use crate::rea;
use crate::system::System;
use serde_json::{json, Value};

pub fn build(sys: &System, provider: &Provider, model: Option<&str>, _ask: bool) -> Value {
    let model_id = crate::provider::model_id(provider, model);
    let node = if cfg!(windows) {
        sys.node_dir().join("node.exe")
    } else {
        sys.node_dir().join("bin/node")
    };
    let rea_script = sys.rea_dir().join("node_modules/rea-agents/scripts/rea.mjs");
    let rea_env = rea::rea_env(sys);
    let mut env_obj = serde_json::Map::new();
    for (k, v) in rea_env {
        env_obj.insert(k, json!(v));
    }
    let mut mcp_rea = serde_json::Map::new();
    mcp_rea.insert("type".into(), json!("local"));
    mcp_rea.insert("command".into(), json!([node.to_string_lossy(), rea_script.to_string_lossy(), "mcp"]));
    mcp_rea.insert("environment".into(), Value::Object(env_obj));
    mcp_rea.insert("enabled".into(), json!(true));
    mcp_rea.insert("timeout".into(), json!(30000));

    let mut bash = serde_json::Map::new();
    bash.insert("*".into(), json!("ask"));
    bash.insert("git status".into(), json!("allow"));
    bash.insert("git diff".into(), json!("allow"));
    bash.insert("git log".into(), json!("allow"));

    let mut read = serde_json::Map::new();
    read.insert("*".into(), json!("allow"));
    read.insert("*.env".into(), json!("deny"));
    read.insert("*.env.*".into(), json!("deny"));
    read.insert("*.env.example".into(), json!("allow"));
    read.insert("*.pem".into(), json!("deny"));
    read.insert("*.key".into(), json!("deny"));
    read.insert("*.npmrc".into(), json!("deny"));

    let mut ext = serde_json::Map::new();
    ext.insert("*".into(), json!("deny"));
    ext.insert(format!("{}/rea-6.10.1/opencode/skills/*", sys.cache_dir.display()), json!("allow"));

    let mut perm = serde_json::Map::new();
    perm.insert("bash".into(), Value::Object(bash));
    perm.insert("edit".into(), json!("ask"));
    perm.insert("read".into(), Value::Object(read));
    perm.insert("webfetch".into(), json!("deny"));
    perm.insert("websearch".into(), json!("deny"));
    perm.insert("codesearch".into(), json!("deny"));
    perm.insert("external_directory".into(), Value::Object(ext));
    perm.insert("rea_*".into(), json!("ask"));
    for t in crate::rea_tools::REA_STATIC_TOOLS {
        perm.insert(format!("rea_{}", t), json!("allow"));
    }

    let mut mcp = serde_json::Map::new();
    mcp.insert("rea".into(), Value::Object(mcp_rea));

    let mut cfg = serde_json::Map::new();
    cfg.insert("$schema".into(), json!("https://opencode.ai/config.json"));
    cfg.insert("model".into(), json!(model_id));
    cfg.insert("small_model".into(), json!(model_id));
    cfg.insert("share".into(), json!("disabled"));
    cfg.insert("autoupdate".into(), json!(false));
    cfg.insert("instructions".into(), json!([sys.agents_path().to_string_lossy()]));
    cfg.insert("plugin".into(), json!([sys.cache_dir.join("plugin.js").to_string_lossy()]));
    cfg.insert("mcp".into(), Value::Object(mcp));
    cfg.insert("permission".into(), Value::Object(perm));
    Value::Object(cfg)
}
