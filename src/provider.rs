#[derive(Clone, Debug)]
pub struct Provider {
    pub name: &'static str,          // launcher flag value
    pub opencode: &'static str,
    pub key_var: &'static str,
    pub default_model: &'static str,
}

pub fn known() -> &'static [Provider] {
    &[
        Provider { name: "nanogpt", opencode: "nano-gpt", key_var: "NANO_GPT_API_KEY", default_model: "pqhaz/DeepSeek-V4.1-Flash-1017-heretic:thinking" },
        Provider { name: "nearai", opencode: "nearai", key_var: "NEARAI_API_KEY", default_model: "deepseek-ai/DeepSeek-V4.1-Flash-1017-abliterated" },
    ]
}

pub fn resolve(name: &str) -> Option<Provider> {
    known().iter().find(|p| p.name == name).cloned()
}

pub fn model_id(provider: &Provider, model: Option<&str>) -> String {
    let m = model.unwrap_or(provider.default_model);
    format!("{}/{}", provider.opencode, m)
}
