use crate::archive::{download, extract_tar_gz, extract_zip};
use crate::pins;
use crate::system::System;
use anyhow::{anyhow, Result};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

pub fn ensure_all(sys: &System) -> Result<()> {
    if !pins::supported_platform() {
        return Err(anyhow!("unsupported platform: {} {}", std::env::consts::OS, std::env::consts::ARCH));
    }
    install_node(sys)?;
    install_rea(sys)?;
    install_opencode(sys)?;

    install_agents(sys)?;
    install_plugin(sys)?;
    Ok(())
}

fn tmp_dir(final_path: &PathBuf) -> PathBuf {
    let parent = final_path.parent().unwrap();
    let name = final_path.file_name().unwrap().to_string_lossy();
    parent.join(format!(".tmp-{}-{}", name, rand::random::<u32>()))
}

fn install_node(sys: &System) -> Result<()> {
    let final_path = sys.node_dir();
    if final_path.join(if cfg!(windows) { "node.exe" } else { "bin/node" }).exists() {
        return Ok(());
    }
    let pin = pins::node_pin();
    let downloads = sys.cache_dir.join("downloads");
    fs::create_dir_all(&downloads)?;
    let archive = downloads.join(pin.url.rsplit('/').next().unwrap());
    if !archive.exists() {
        download(&pin.url, &archive)?;
    }

    if !pins::verify_sha256(&archive, &pin.sha256) {
        let actual = pins::sha256_file_hex(&archive);
        fs::remove_file(&archive).ok();
        return Err(anyhow!("hash mismatch: expected {} got {}", pin.sha256, actual));
    }
    let tmp = tmp_dir(&final_path);
    fs::create_dir_all(&tmp)?;
    if pin.url.ends_with(".zip") {
        extract_zip(&archive, &tmp, 1)?;
    } else {
        extract_tar_gz(&archive, &tmp, 1)?;
    }

    if final_path.exists() { fs::remove_dir_all(&final_path).ok(); }
    fs::rename(&tmp, &final_path)?;
    Ok(())
}

fn install_rea(sys: &System) -> Result<()> {
    let final_path = sys.rea_dir();
    let marker = final_path.join("node_modules/rea-agents/scripts/rea.mjs");
    if marker.exists() {
        return Ok(());
    }

    let downloads = sys.cache_dir.join("downloads");
    fs::create_dir_all(&downloads)?;
    let archive = downloads.join("rea-agents-6.10.1.tgz");
    if !archive.exists() {
        download(pins::rea_tarball_url(), &archive)?;
    }

    if !pins::verify_integrity(&archive, pins::rea_integrity()) {
        let actual = pins::sha512_base64(&archive);
        fs::remove_file(&archive).ok();
        return Err(anyhow!("hash mismatch: expected {} got sha512-{}", pins::rea_integrity(), actual));
    }
    let tmp = tmp_dir(&final_path);
    fs::create_dir_all(&tmp)?;

    let node_bin = if cfg!(windows) {
        sys.node_dir().join("node.exe")
    } else {
        sys.node_dir().join("bin/node")
    };
    let npm_cli = sys.node_dir().join("lib/node_modules/npm/bin/npm-cli.js");

    let npm_cli = if npm_cli.exists() { npm_cli } else { sys.node_dir().join("node_modules/npm/bin/npm-cli.js") };
    let status = Command::new(&node_bin)
        .arg(&npm_cli)
        .arg("install")
        .arg("--prefix")
        .arg(&tmp)
        .arg("--omit=dev")
        .arg("--no-audit")
        .arg("--no-fund")
        .arg(archive.to_string_lossy().to_string())
        .env("PATH", path_with_node(sys))
        .status()?;
    if !status.success() {
        fs::remove_dir_all(&tmp).ok();
        return Err(anyhow!("npm install failed"));
    }
    if final_path.exists() { fs::remove_dir_all(&final_path).ok(); }
    fs::rename(&tmp, &final_path)?;

    copy_rea_skills(sys)?;
    Ok(())
}

fn copy_rea_skills(sys: &System) -> Result<()> {
    let src = sys.rea_dir().join("node_modules/rea-agents/opencode/skills");
    let dst = sys.rea_dir().join("opencode/skills");
    if src.exists() {
        copy_dir_all(&src, &dst)?;
    }
    Ok(())
}

fn copy_dir_all(src: &PathBuf, dst: &PathBuf) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in walkdir::WalkDir::new(src) {
        let entry = entry?;
        let path = entry.path();
        let rel = path.strip_prefix(src)?;
        let target = dst.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)?;
        } else {
            if let Some(p) = target.parent() { fs::create_dir_all(p)?; }
            fs::copy(path, &target)?;
        }
    }
    Ok(())
}

fn install_opencode(sys: &System) -> Result<()> {
    let final_path = sys.opencode_dir();
    let bin = final_path.join("bin").join(if cfg!(windows) { "opencode.exe" } else { "opencode" });
    if bin.exists() {
        return Ok(());
    }
    let (pkg, integrity) = pins::opencode_pin();
    let url = format!("https://registry.npmjs.org/{}/-/{}-{}.tgz", pkg.split('@').next().unwrap(), pkg.split('@').next().unwrap().trim_start_matches('@').replace('/', "-"), pins::OPENCODE_VERSION);

    let pkg_name = pkg.split('@').next().unwrap();
    let url = format!("https://registry.npmjs.org/{name}/-/{name}-{ver}.tgz", name=pkg.split('@').next().unwrap(), ver=pins::OPENCODE_VERSION);
    let downloads = sys.cache_dir.join("downloads");
    fs::create_dir_all(&downloads)?;
    let archive = downloads.join(format!("{}.tgz", pkg.replace('/', "-")));
    if !archive.exists() {
        download(&url, &archive)?;
    }
    if !pins::verify_integrity(&archive, integrity) {
        let actual = pins::sha512_base64(&archive);
        fs::remove_file(&archive).ok();
        return Err(anyhow!("hash mismatch: expected {} got sha512-{}", integrity, actual));
    }
    let tmp = tmp_dir(&final_path);
    fs::create_dir_all(&tmp)?;
    extract_tar_gz(&archive, &tmp, 1)?;
    if final_path.exists() { fs::remove_dir_all(&final_path).ok(); }
    fs::rename(&tmp, &final_path)?;
    Ok(())
}

fn install_agents(sys: &System) -> Result<()> {
    let path = sys.agents_path();
    let embedded = include_str!("agents.md");
    if !path.exists() || std::fs::read_to_string(&path).unwrap_or_default() != embedded {
        std::fs::write(&path, embedded)?;
    }
    Ok(())
}

fn install_plugin(sys: &System) -> Result<()> {
    let path = sys.cache_dir.join("plugin.js");
    let embedded = include_str!("plugin.js");
    if !path.exists() || std::fs::read_to_string(&path).unwrap_or_default() != embedded {
        std::fs::write(&path, embedded)?;
    }
    Ok(())
}

fn path_with_node(sys: &System) -> String {
    let node_bin = if cfg!(windows) { sys.node_dir() } else { sys.node_dir().join("bin") };
    let sep = if cfg!(windows) { ";" } else { ":" };
    format!("{}{}{}", node_bin.display(), sep, std::env::var("PATH").unwrap_or_default())
}
