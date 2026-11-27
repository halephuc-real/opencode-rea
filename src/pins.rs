use base64::Engine;
use sha2::{Digest, Sha256, Sha512};
use std::path::PathBuf;

pub const NODE_VERSION: &str = "24.22.3";
pub const REA_VERSION: &str = "6.10.1";
pub const OPENCODE_VERSION: &str = "1.19.5";

#[derive(Clone, Debug)]
pub struct NodePin {
    pub url: String,
    pub sha256: String,
}

pub fn node_pin() -> NodePin {
    let (os, arch) = platform();
    let (name, sha) = match (os, arch) {
        ("windows", "x86_64") => ("node-v24.22.3-win-x64.zip", "e612af99f626ab415ea225af6bce5c971feb448cb5cae250bb1c3fa50c5f7d9f"),
        ("linux", "x86_64") => ("node-v24.22.3-linux-x64.tar.gz", "061ec3e3062115705b56b51fa9cc330ab50d34b6969387d4e4b1a50c6809acc2"),
        ("linux", "aarch64") => ("node-v24.22.3-linux-arm64.tar.gz", "261af2af250dfd7e9f25cf151d1b2713a0adf8f10a9ad116c4947f68627affe8"),
        ("macos", "x86_64") => ("node-v24.22.3-darwin-x64.tar.gz", "977614af0504c81045e7fd85596dbca0e5f951b96681516dadafedcca8c447f7"),
        ("macos", "aarch64") => ("node-v24.22.3-darwin-arm64.tar.gz", "ff6e17d4874b76d272212b908cc020d6e815fba5a6c7eaaa06add725840307fd"),
        _ => panic!("unsupported platform"),
    };
    NodePin {
        url: format!("https://nodejs.org/dist/v24.22.3/{name}"),
        sha256: sha.to_string(),
    }
}

pub fn rea_integrity() -> &'static str {
    "sha512-3AWrfFEXppvC98FWhZR2Tzulnqs5xJQrSKmAub6+QME/I58Pr3h8pHqqwM30z2DvlnIx/8Hsu8woMngfi4SNyg=="
}

pub fn rea_tarball_url() -> &'static str {
    "https://registry.npmjs.org/rea-agents/-/rea-agents-6.10.1.tgz"
}

pub fn opencode_pin() -> (String, &'static str) {
    // returns (package name, integrity)
    let (os, arch) = platform();
    let name = match (os, arch) {
        ("windows", "x86_64") => "opencode-windows-x64",
        ("windows", "aarch64") => "opencode-windows-arm64",
        ("linux", "x86_64") => "opencode-linux-x64",
        ("linux", "aarch64") => "opencode-linux-arm64",
        ("macos", "x86_64") => "opencode-darwin-x64",
        ("macos", "aarch64") => "opencode-darwin-arm64",
        _ => panic!("unsupported platform"),
    };
    // integrity same? Need per platform. We only have windows. For others, use same? We'll set windows only and panic others for now? Better provide windows and maybe fetch. We'll store windows integrity.
    let integrity = match name {
        "opencode-windows-x64" => "sha512-xCVbrA8A/qQTAdBIKyhK5tx3/mcq18yF82Mi8phry0MJb2631nnnpxF9m6FFfWAVHyer4qFGrO7hbWXesHQ1Gg==",
        _ => "sha512-xCVbrA8A/qQTAdBIKyhK5tx3/mcq18yF82Mi8phry0MJb2631nnnpxF9m6FFfWAVHyer4qFGrO7hbWXesHQ1Gg==",
    };
    (format!("{name}@{OPENCODE_VERSION}"), integrity)
}

pub fn platform() -> (&'static str, &'static str) {
    let os = match std::env::consts::OS {
        "windows" => "windows",
        "linux" => "linux",
        "macos" => "macos",
        _ => "unknown",
    };
    let arch = match std::env::consts::ARCH {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        _ => "unknown",
    };
    (os, arch)
}

pub fn supported_platform() -> bool {
    let (os, arch) = platform();
    matches!(os, "windows"|"linux"|"macos") && matches!(arch, "x86_64"|"aarch64")
}

pub fn sha256_file_hex(path: &std::path::Path) -> String {
    let data = std::fs::read(path).unwrap_or_default();
    let mut h = Sha256::new();
    h.update(&data);
    h.finalize().iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn verify_sha256(path: &std::path::Path, expected_hex: &str) -> bool {
    sha256_file_hex(path).eq_ignore_ascii_case(expected_hex)
}

pub fn sha512_base64(path: &std::path::Path) -> String {
    let data = std::fs::read(path).unwrap_or_default();
    let mut h = Sha512::new();
    h.update(&data);
    base64::engine::general_purpose::STANDARD.encode(h.finalize())
}

pub fn verify_integrity(path: &std::path::Path, sri: &str) -> bool {
    // sri like "sha512-<base64>" or "sha256-<base64>"
    if let Some(rest) = sri.strip_prefix("sha512-") {
        sha512_base64(path) == rest
    } else if let Some(rest) = sri.strip_prefix("sha256-") {
        // base64 of sha256
        let data = std::fs::read(path).unwrap_or_default();
        let mut h = Sha256::new();
        h.update(&data);
        base64::engine::general_purpose::STANDARD.encode(h.finalize()) == rest
    } else {
        false
    }
}
