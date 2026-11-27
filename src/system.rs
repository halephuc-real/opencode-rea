use directories::BaseDirs;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct System {
    pub cache_dir: PathBuf,
    pub config_dir: PathBuf,
    pub version: String,
}

impl System {
    pub fn default_with_version(version: &str) -> Self {
        let base = BaseDirs::new().expect("no home directory");
        let cache_dir = base.cache_dir().join("opencode-rea");
        let config_dir = base.config_dir().join("opencode-rea");
        // migrate from legacy rea-launcher directories if present
        let old_cache = base.cache_dir().join("rea-launcher");
        if !cache_dir.exists() && old_cache.exists() {
            let _ = std::fs::rename(&old_cache, &cache_dir);
        }
        let old_config = base.config_dir().join("rea-launcher");
        if !config_dir.exists() && old_config.exists() {
            let _ = std::fs::rename(&old_config, &config_dir);
        }
        Self {
            cache_dir,
            config_dir,
            version: version.to_string(),
        }
    }

    pub fn managed_dir(&self) -> PathBuf { self.cache_dir.clone() }
    pub fn node_dir(&self) -> PathBuf { self.cache_dir.join("node-24.22.3") }
    pub fn rea_dir(&self) -> PathBuf { self.cache_dir.join("rea-6.10.1") }
    pub fn opencode_dir(&self) -> PathBuf { self.cache_dir.join("opencode-1.19.5") }
    pub fn agents_path(&self) -> PathBuf { self.cache_dir.join("AGENTS.md") }
    pub fn engine_path(&self) -> PathBuf { self.config_dir.join("engine.json") }
}

impl Default for System {
    fn default() -> Self { Self::default_with_version("dev") }
}
