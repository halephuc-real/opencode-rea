use crate::oauth;
use crate::provider::Provider;
use crate::system::System;
use anyhow::{anyhow, Result};

pub fn resolve(sys: &System, provider: &Provider) -> Result<String> {

    if let Ok(v) = std::env::var(provider.key_var) {
        if !v.trim().is_empty() {
            return Ok(v);
        }
    }

    if let Ok(entry) = keyring::Entry::new("rea-launcher", provider.name) {
        if let Ok(v) = entry.get_password() {
            if !v.trim().is_empty() {
                return Ok(v);
            }
        }
    }

    #[cfg(windows)]
    if let Some(v) = read_legacy_credential(provider.name) {
        if !v.trim().is_empty() {

            if let Ok(entry) = keyring::Entry::new("rea-launcher", provider.name) {
                let _ = entry.set_password(&v);
            }
            return Ok(v);
        }
    }

    let key = if provider.name == "nanogpt" {
        oauth::signin()?
    } else {

        if !atty_stdin() {
            return Err(anyhow!("{} not set and stdin is not a terminal", provider.key_var));
        }
        let k = rpassword::prompt_password(format!("{}: ", provider.key_var))?;
        if k.trim().is_empty() {
            return Err(anyhow!("empty key"));
        }
        k
    };

    match keyring::Entry::new("rea-launcher", provider.name) {
        Ok(entry) => {
            if let Err(e) = entry.set_password(&key) {
                eprintln!("warning: could not save key to credential store: {e}");
            }
        }
        Err(e) => eprintln!("warning: could not reach credential store: {e}"),
    }
    let _ = sys;
    Ok(key)
}

pub fn delete(provider_name: &str) {
    if let Ok(e) = keyring::Entry::new("rea-launcher", provider_name) {
        let _ = e.delete_credential();
    }
}

fn atty_stdin() -> bool {

    std::io::IsTerminal::is_terminal(&std::io::stdin())
}

#[cfg(windows)]
fn read_legacy_credential(account: &str) -> Option<String> {
    use std::ffi::c_void;
    use std::os::windows::ffi::OsStrExt;
    #[repr(C)]
    struct FileTime { _low: u32, _high: u32 }
    #[repr(C)]
    struct Credential {
        _flags: u32,
        _typ: u32,
        _target_name: *mut u16,
        _comment: *mut u16,
        _last_written: FileTime,
        cred_blob_size: u32,
        cred_blob: *mut u8,
        _persist: u32,
        _attribute_count: u32,
        _attributes: *mut c_void,
        _target_alias: *mut u16,
        _user_name: *mut u16,
    }
    #[link(name = "Advapi32")]
    extern "system" {
        fn CredReadW(target: *const u16, typ: u32, flags: u32, cred: *mut *mut Credential) -> i32;
        fn CredFree(buffer: *mut c_void);
    }
    let target = format!("rea-launcher:{}", account);
    let wide: Vec<u16> = std::ffi::OsStr::new(&target).encode_wide().chain(std::iter::once(0)).collect();
    let mut cred: *mut Credential = std::ptr::null_mut();
    let ok = unsafe { CredReadW(wide.as_ptr(), 1, 0, &mut cred) };
    if ok == 0 || cred.is_null() { return None; }
    let c = unsafe { &*cred };
    let bytes = unsafe { std::slice::from_raw_parts(c.cred_blob, c.cred_blob_size as usize) };
    let s = String::from_utf8(bytes.to_vec()).ok();
    unsafe { CredFree(cred as *mut c_void) };
    s
}
