use anyhow::{anyhow, Context, Result};
use flate2::read::GzDecoder;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use tar::Archive as TarArchive;
use zip::ZipArchive;

pub fn extract_zip(path: &Path, dest: &Path, strip_components: usize) -> Result<()> {
    let file = fs::File::open(path)?;
    let mut zip = ZipArchive::new(file)?;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i)?;
        let name = entry.name().to_string();
        let parts: Vec<&str> = name.split('/').collect();
        if parts.len() <= strip_components { continue; }
        let rel: PathBuf = parts[strip_components..].iter().collect();
        if rel.as_os_str().is_empty() { continue; }
        let out_path = dest.join(&rel);
        if entry.is_dir() {
            fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() { fs::create_dir_all(parent)?; }
            let mut out = fs::File::create(&out_path)?;
            io::copy(&mut entry, &mut out)?;
        }
    }
    Ok(())
}

pub fn extract_tar_gz(path: &Path, dest: &Path, strip_components: usize) -> Result<()> {
    let file = fs::File::open(path)?;
    let gz = GzDecoder::new(file);
    let mut archive = TarArchive::new(gz);
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path_name = entry.path()?.to_path_buf();
        let parts: Vec<_> = path_name.components().collect();
        if parts.len() <= strip_components { continue; }
        let rel: PathBuf = parts[strip_components..].iter().collect();
        if rel.as_os_str().is_empty() { continue; }
        let out_path = dest.join(&rel);
        if entry.header().entry_type().is_dir() {
            fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() { fs::create_dir_all(parent)?; }
            let mut out = fs::File::create(&out_path)?;
            io::copy(&mut entry, &mut out)?;
        }
    }
    Ok(())
}

pub fn download(url: &str, dest: &Path) -> Result<()> {
    let resp = reqwest::blocking::get(url)?;
    if !resp.status().is_success() {
        return Err(anyhow::anyhow!("download failed: {}", resp.status()));
    }
    let bytes = resp.bytes()?;
    if let Some(parent) = dest.parent() { fs::create_dir_all(parent)?; }
    fs::write(dest, &bytes)?;
    Ok(())
}
