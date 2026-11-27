use crate::system::System;
use anyhow::{anyhow, bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

pub const BRAND_NAME: &str = "PHX LAUNCHER";

const TITLE_OLD: &[u8] = b"setTerminalTitle(\"OpenCode\")";
const TITLE_NEW: &[u8] = b"setTerminalTitle(\"PHX LAUNCHER\")";
const PREFIX_OLD: &[u8] = b"`OC | ${";
const PREFIX_NEW: &[u8] = b"`PHX| ${";
const DISABLE_OLD: &[u8] = b"Disable terminal title";
const DISABLE_NEW: &[u8] = b"Disable term title";
const ENABLE_OLD: &[u8] = b"Enable terminal title";
const ENABLE_NEW: &[u8] = b"Enable term title";

const LOGO_MARKER: &[u8] = b"var iy={left:";
const LOGO_EXPORT: &[u8] = b"export{iy,zh};";
const LOGO_DECODER: &[u8] = b"var f=a=>a.map(s=>s.replace(/[0-3]/g";
const TRAILER: &[u8] = b"\n---- Bun! ----\n";

// 4x4 block font encoded as digits: 0 = space, 1 = full block,
// 2 = upper half block, 3 = lower half block.  The module decoder
// expands these to real Unicode blocks at load time, keeping the
// source ASCII and the Bun module length unchanged.
const ART_LEFT: [&str; 4] = [
    "12210100101001",
    "10010100100110",
    "12220122100110",
    "20000100101001",
];
const ART_RIGHT: [&str; 4] = [
    "100003223010010100103222010010122201221",
    "100001001010010130101000010010122001001",
    "100001221010010102101000012210100001222",
    "122201001022220100102222010010222202002",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrandStatus {
    NotInstalled,
    Original,
    Branded,
    Patched,
    Unknown,
}

pub fn bin_path(sys: &System) -> PathBuf {
    sys.opencode_dir().join("bin").join(if cfg!(windows) {
        "opencode.exe"
    } else {
        "opencode"
    })
}

fn backup_path(bin: &Path) -> PathBuf {
    append_suffix(bin, ".orig")
}

fn append_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut s = path.as_os_str().to_os_string();
    s.push(suffix);
    PathBuf::from(s)
}

pub fn status(sys: &System) -> Result<BrandStatus> {
    let bin = bin_path(sys);
    if !bin.exists() {
        return Ok(BrandStatus::NotInstalled);
    }
    let data = fs::read(&bin)?;
    if contains(&data, TITLE_NEW) && contains(&data, LOGO_DECODER) {
        return Ok(BrandStatus::Branded);
    }
    if contains(&data, TITLE_OLD) || contains(&data, LOGO_MARKER) {
        return Ok(BrandStatus::Original);
    }
    Ok(BrandStatus::Unknown)
}

pub fn ensure(sys: &System) -> Result<BrandStatus> {
    let bin = bin_path(sys);
    if !bin.exists() {
        return Ok(BrandStatus::NotInstalled);
    }
    let data = fs::read(&bin)?;
    if contains(&data, TITLE_NEW) && contains(&data, LOGO_DECODER) {
        return Ok(BrandStatus::Branded);
    }
    let patched = patch_bytes(&data)?;
    replace_binary(&bin, &patched)?;
    Ok(BrandStatus::Patched)
}

pub fn restore(sys: &System) -> Result<()> {
    let bin = bin_path(sys);
    if !bin.exists() {
        bail!("OpenCode is not installed: {}", bin.display());
    }
    let backup = backup_path(&bin);
    if !backup.exists() {
        bail!("backup not found: {}", backup.display());
    }
    let data = fs::read(&backup)?;
    replace_binary(&bin, &data)?;
    Ok(())
}

fn replace_binary(bin: &Path, bytes: &[u8]) -> Result<()> {
    let parent = bin
        .parent()
        .ok_or_else(|| anyhow!("no parent directory for {}", bin.display()))?;
    fs::create_dir_all(parent)?;

    let tmp = parent.join(format!(
        ".tmp-{}-{}",
        bin.file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "opencode".to_string()),
        rand::random::<u32>()
    ));
    fs::write(&tmp, bytes)?;

    if !bin.exists() {
        fs::rename(&tmp, bin)?;
        return Ok(());
    }

    let backup = backup_path(bin);
    if !backup.exists() {
        fs::copy(bin, &backup)?;
    }

    #[cfg(windows)]
    {
        let old = append_suffix(
            bin,
            &format!(".old-{}-{}", std::process::id(), rand::random::<u32>()),
        );
        match fs::rename(bin, &old) {
            Ok(()) => {
                if let Err(e) = fs::rename(&tmp, bin) {
                    let _ = fs::rename(&old, bin);
                    let _ = fs::remove_file(&tmp);
                    return Err(e).with_context(|| {
                        format!("could not install the patched binary at {}", bin.display())
                    });
                }
                // The old binary may still be running; removing it is best-effort.
                let _ = fs::remove_file(&old);
            }
            Err(e) => {
                let _ = fs::remove_file(&tmp);
                return Err(anyhow!(
                    "could not replace {}: {} (close all OpenCode windows and try again)",
                    bin.display(),
                    e
                ));
            }
        }
    }

    #[cfg(not(windows))]
    {
        fs::rename(&tmp, bin)?;
    }

    Ok(())
}

pub fn patch_bytes(data: &[u8]) -> Result<Vec<u8>> {
    let payload = find_bun_payload(data)?;
    let marker = data
        .windows(TRAILER.len())
        .rposition(|w| w == TRAILER)
        .ok_or_else(|| anyhow!("Bun trailer not found"))?;
    if marker < 32 {
        bail!("invalid Bun trailer offset");
    }
    let header = marker - 32;
    let mod_ptr_off = read_u32(data, header + 8)? as usize;
    let mod_ptr_len = read_u32(data, header + 12)? as usize;
    if mod_ptr_len % 52 != 0 {
        bail!("invalid Bun module table length: {}", mod_ptr_len);
    }
    let table_abs = payload
        .checked_add(mod_ptr_off)
        .ok_or_else(|| anyhow!("module table offset overflow"))?;
    let table_end = table_abs
        .checked_add(mod_ptr_len)
        .ok_or_else(|| anyhow!("module table length overflow"))?;
    if table_end > data.len() {
        bail!("Bun module table is out of range");
    }

    let module_count = mod_ptr_len / 52;
    let mut title_module = None;
    let mut logo_module = None;

    for i in 0..module_count {
        let entry = table_abs + i * 52;
        let content_off = read_u32(data, entry + 8)? as usize;
        let content_len = read_u32(data, entry + 12)? as usize;
        let start = payload
            .checked_add(content_off)
            .ok_or_else(|| anyhow!("module content offset overflow"))?;
        let end = start
            .checked_add(content_len)
            .ok_or_else(|| anyhow!("module content length overflow"))?;
        if end > data.len() {
            bail!("module content is out of range");
        }
        let content = &data[start..end];

        if title_module.is_none() && contains(content, TITLE_OLD) {
            title_module = Some((start, content_len));
        }
        if logo_module.is_none() && contains(content, LOGO_MARKER) && contains(content, LOGO_EXPORT)
        {
            logo_module = Some((start, content_len));
        }
        if title_module.is_some() && logo_module.is_some() {
            break;
        }
    }

    let (title_start, title_len) =
        title_module.ok_or_else(|| anyhow!("terminal title module not found"))?;
    let (logo_start, logo_len) = logo_module.ok_or_else(|| anyhow!("logo module not found"))?;

    let mut out = data.to_vec();

    let mut title = out[title_start..title_start + title_len].to_vec();
    let original_title_len = title.len();
    let title_count = replace_all(&mut title, TITLE_OLD, TITLE_NEW)?;
    let prefix_count = replace_all(&mut title, PREFIX_OLD, PREFIX_NEW)?;
    let disable_count = replace_all(&mut title, DISABLE_OLD, DISABLE_NEW)?;
    let enable_count = replace_all(&mut title, ENABLE_OLD, ENABLE_NEW)?;

    if title_count != 2 {
        bail!("expected 2 terminal-title strings, found {}", title_count);
    }
    if prefix_count != 2 {
        bail!("expected 2 terminal-title prefixes, found {}", prefix_count);
    }
    if disable_count != 1 || enable_count != 1 {
        bail!(
            "expected one disable/enable title string, found {}/{}",
            disable_count,
            enable_count
        );
    }
    if title.len() != original_title_len {
        bail!(
            "title patch changed module length: {} -> {}",
            original_title_len,
            title.len()
        );
    }
    out[title_start..title_start + title_len].copy_from_slice(&title);

    let new_logo = build_logo_module(&data[logo_start..logo_start + logo_len], logo_len)?;
    out[logo_start..logo_start + logo_len].copy_from_slice(&new_logo);

    Ok(out)
}

fn build_logo_module(original: &[u8], target_len: usize) -> Result<Vec<u8>> {
    let zh_start = find_subslice(original, b",zh=")
        .ok_or_else(|| anyhow!("zh marker not found in logo module"))?
        + 1;
    let export_pos = find_subslice(original, LOGO_EXPORT)
        .ok_or_else(|| anyhow!("logo export not found in logo module"))?;
    let zh_end = original[..export_pos]
        .iter()
        .rposition(|&b| b == b';')
        .ok_or_else(|| anyhow!("zh terminator not found in logo module"))?;
    if zh_start >= zh_end {
        bail!("invalid zh block in logo module");
    }
    let zh =
        std::str::from_utf8(&original[zh_start..zh_end]).context("zh block is not valid UTF-8")?;

    let mut s = String::new();
    s.push_str("// @bun\n");
    s.push_str("var f=a=>a.map(s=>s.replace(/[0-3]/g,c=>\" \\u2588\\u2580\\u2584\"[c]));\n");
    s.push_str("var iy={left:f([");
    push_array(&mut s, &ART_LEFT);
    s.push_str("]),right:f([");
    push_array(&mut s, &ART_RIGHT);
    s.push_str("])},");
    s.push_str(zh);
    s.push_str(";\nexport{iy,zh};\n");

    let mut bytes = s.into_bytes();
    if bytes.len() > target_len {
        bail!(
            "new logo module is {} bytes but the original module is {} bytes",
            bytes.len(),
            target_len
        );
    }
    bytes.resize(target_len, b' ');
    Ok(bytes)
}

fn push_array(out: &mut String, rows: &[&str; 4]) {
    for (i, row) in rows.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push('"');
        out.push_str(row);
        out.push('"');
    }
}

fn find_bun_payload(data: &[u8]) -> Result<usize> {
    if data.len() < 0x40 || &data[0..2] != b"MZ" {
        bail!("not a PE executable");
    }
    let pe_off = read_u32(data, 0x3C)? as usize;
    if pe_off + 24 > data.len() || &data[pe_off..pe_off + 4] != b"PE\0\0" {
        bail!("invalid PE header");
    }
    let coff = pe_off + 4;
    let section_count = read_u16(data, coff + 2)? as usize;
    let optional_size = read_u16(data, coff + 16)? as usize;
    let optional = coff + 20;
    if optional + optional_size > data.len() {
        bail!("truncated PE optional header");
    }
    let magic = read_u16(data, optional)?;
    if magic != 0x20B {
        bail!("only 64-bit PE executables are supported");
    }
    let sections = optional + optional_size;
    for i in 0..section_count {
        let section = sections + i * 40;
        if section + 40 > data.len() {
            bail!("truncated PE section table");
        }
        let name = &data[section..section + 8];
        if name.starts_with(b".bun\0") {
            let raw_size = read_u32(data, section + 16)? as usize;
            let raw_ptr = read_u32(data, section + 20)? as usize;
            let payload = raw_ptr
                .checked_add(8)
                .ok_or_else(|| anyhow!("Bun payload offset overflow"))?;
            let end = payload
                .checked_add(raw_size)
                .ok_or_else(|| anyhow!("Bun payload length overflow"))?;
            if end > data.len() {
                bail!("Bun payload is out of range");
            }
            return Ok(payload);
        }
    }
    bail!(".bun section not found")
}

fn replace_all(buf: &mut Vec<u8>, from: &[u8], to: &[u8]) -> Result<usize> {
    if from.is_empty() {
        bail!("empty search pattern");
    }
    let mut count = 0;
    let mut i = 0;
    while i + from.len() <= buf.len() {
        if &buf[i..i + from.len()] == from {
            buf.splice(i..i + from.len(), to.iter().copied());
            count += 1;
            i += to.len();
        } else {
            i += 1;
        }
    }
    Ok(count)
}

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    !needle.is_empty() && haystack.windows(needle.len()).any(|w| w == needle)
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return None;
    }
    haystack.windows(needle.len()).position(|w| w == needle)
}

fn read_u16(data: &[u8], off: usize) -> Result<u16> {
    let end = off
        .checked_add(2)
        .ok_or_else(|| anyhow!("offset overflow"))?;
    let bytes = data
        .get(off..end)
        .ok_or_else(|| anyhow!("offset out of range"))?;
    Ok(u16::from_le_bytes(bytes.try_into().unwrap()))
}

fn read_u32(data: &[u8], off: usize) -> Result<u32> {
    let end = off
        .checked_add(4)
        .ok_or_else(|| anyhow!("offset overflow"))?;
    let bytes = data
        .get(off..end)
        .ok_or_else(|| anyhow!("offset out of range"))?;
    Ok(u32::from_le_bytes(bytes.try_into().unwrap()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    #[test]
    #[ignore]
    fn patch_external_binary() {
        let input = std::env::var("BRAND_TEST_INPUT").expect("BRAND_TEST_INPUT");
        let output = std::env::var("BRAND_TEST_OUTPUT").expect("BRAND_TEST_OUTPUT");
        let data = std::fs::read(&input).expect("read input");
        let patched = patch_bytes(&data).expect("patch");
        std::fs::write(&output, &patched).expect("write output");
        let mut h = Sha256::new();
        h.update(&patched);
        println!("patched sha256: {:x}", h.finalize());
    }

    #[test]
    fn replace_binary_roundtrip() {
        let dir = tempfile::tempdir().expect("tempdir");
        let bin = dir.path().join("opencode.exe");
        std::fs::write(&bin, b"original").unwrap();
        replace_binary(&bin, b"patched").unwrap();
        assert_eq!(std::fs::read(&bin).unwrap(), b"patched");
        assert_eq!(std::fs::read(backup_path(&bin)).unwrap(), b"original");
    }
}
