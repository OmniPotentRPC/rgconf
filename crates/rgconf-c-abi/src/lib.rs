// Copyright 2023--present rgconf developers

use rgconf::{apply_edit, ConfigLayer, EditOrigin, ParamValue, Profile, Resolver};
use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::panic::{catch_unwind, AssertUnwindSafe};

thread_local! {
    static LAST_ERROR: RefCell<CString> = RefCell::new(match CString::new("") {
        Ok(value) => value,
        Err(_) => unreachable_empty(),
    });
}

fn unreachable_empty() -> CString {
    CString::new("").expect("empty C string")
}

fn set_error(text: &str) {
    let cleaned = text.replace('\0', " ");
    let value = match CString::new(cleaned) {
        Ok(value) => value,
        Err(_) => CString::new("rgconf error").expect("ascii error"),
    };
    LAST_ERROR.with(|slot| *slot.borrow_mut() = value);
}

fn clear_error() {
    set_error("");
}

fn c_str<'a>(ptr: *const c_char, what: &str) -> Result<&'a str, String> {
    if ptr.is_null() {
        return Err(format!("{what} is null"));
    }
    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .map_err(|_| format!("{what} is not utf-8"))
}

fn export_text(text: String) -> *mut c_char {
    match CString::new(text) {
        Ok(value) => value.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[cfg(test)]
fn pack_abi(major: u32, minor: u32, patch: u32) -> u32 {
    (major << 16) | (minor << 8) | patch
}

#[no_mangle]
pub extern "C" fn rgconf_abi_version() -> u32 {
    rgconf::ABI_VERSION
}

#[no_mangle]
pub extern "C" fn rgconf_last_error() -> *const c_char {
    LAST_ERROR.with(|slot| slot.borrow().as_ptr())
}

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn rgconf_free(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        drop(CString::from_raw(ptr));
    }
}

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn rgconf_resolve_ini(
    profile: *const c_char,
    text: *const c_char,
    path: *const c_char,
    out_text: *mut *mut c_char,
) -> c_int {
    let result = catch_unwind(AssertUnwindSafe(|| {
        if out_text.is_null() {
            return Err("out_text is null".to_string());
        }
        unsafe {
            *out_text = std::ptr::null_mut();
        }
        let profile = Profile::parse(c_str(profile, "profile")?).map_err(|err| err.to_string())?;
        let text = c_str(text, "text")?;
        let path = c_str(path, "path")?;
        let mut resolver = Resolver::new(profile);
        if !text.is_empty() {
            resolver
                .merge_ini(ConfigLayer::JobConfigFile, text, path)
                .map_err(|err| err.to_string())?;
        }
        let resolution = resolver.resolve().map_err(|err| err.to_string())?;
        Ok(resolution.to_string())
    }));
    match result {
        Ok(Ok(body)) => {
            clear_error();
            unsafe {
                *out_text = export_text(body);
            }
            if unsafe { *out_text }.is_null() {
                set_error("export failed");
                return 1;
            }
            0
        }
        Ok(Err(err)) => {
            set_error(&err);
            1
        }
        Err(_) => {
            set_error("rgconf panicked");
            1
        }
    }
}

#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn rgconf_apply_edit(
    profile: *const c_char,
    key: *const c_char,
    value: *const c_char,
    checkpoint: c_int,
    out_text: *mut *mut c_char,
) -> c_int {
    let result = catch_unwind(AssertUnwindSafe(|| {
        if out_text.is_null() {
            return Err("out_text is null".to_string());
        }
        unsafe {
            *out_text = std::ptr::null_mut();
        }
        let profile = Profile::parse(c_str(profile, "profile")?).map_err(|err| err.to_string())?;
        let key = c_str(key, "key")?;
        let value = c_str(value, "value")?;
        let mut resolution = Resolver::new(profile)
            .resolve()
            .map_err(|err| err.to_string())?;
        let parsed = if value == "true" || value == "false" {
            ParamValue::Bool(value == "true")
        } else {
            match value.parse::<i64>() {
                Ok(number) if number >= 0 && !value.contains('.') => {
                    ParamValue::Uint(number as u64)
                }
                Ok(number) => ParamValue::Int(number),
                Err(_) => match value.parse::<f64>() {
                    Ok(number) => ParamValue::Float(number),
                    Err(_) => ParamValue::Text(value.to_string()),
                },
            }
        };
        let origin = if checkpoint == 0 {
            EditOrigin::LiveCall
        } else {
            EditOrigin::CheckpointRestart
        };
        let edit =
            apply_edit(&mut resolution, key, parsed, origin).map_err(|err| err.to_string())?;
        let actions = edit
            .actions
            .iter()
            .map(|action| action.name())
            .collect::<Vec<_>>()
            .join(",");
        Ok(format!("{} {} {}", edit.key, edit.next, actions))
    }));
    match result {
        Ok(Ok(body)) => {
            clear_error();
            unsafe {
                *out_text = export_text(body);
            }
            0
        }
        Ok(Err(err)) => {
            set_error(&err);
            1
        }
        Err(_) => {
            set_error("rgconf panicked");
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    fn c(text: &str) -> CString {
        CString::new(text).expect("ascii")
    }

    #[test]
    fn abi_version_matches_the_header_contract() {
        assert_eq!(rgconf_abi_version(), pack_abi(0, 1, 0));
    }

    #[test]
    fn resolve_ini_round_trip() {
        let profile = c("eonLegacyIni");
        let text = c("[Optimizer]\nconverged_force = 0.03\n");
        let path = c("client.ini");
        let mut out = std::ptr::null_mut();
        let code = rgconf_resolve_ini(profile.as_ptr(), text.as_ptr(), path.as_ptr(), &mut out);
        assert_eq!(
            code,
            0,
            "{}",
            unsafe { CStr::from_ptr(rgconf_last_error()) }.to_string_lossy()
        );
        let body = unsafe { CStr::from_ptr(out) }
            .to_string_lossy()
            .into_owned();
        assert!(body.contains("optimizer.convergence.forceToleranceEvPerAngstrom=0.03"));
        rgconf_free(out);
    }

    #[test]
    fn live_edit_of_a_restart_only_field_fails() {
        let profile = c("rgconfCanonical");
        let key = c("runtime.job");
        let value = c("saddleSearch");
        let mut out = std::ptr::null_mut();
        let code = rgconf_apply_edit(profile.as_ptr(), key.as_ptr(), value.as_ptr(), 0, &mut out);
        assert_eq!(code, 1);
        let err = unsafe { CStr::from_ptr(rgconf_last_error()) }
            .to_string_lossy()
            .into_owned();
        assert!(err.contains("restartOnly"), "{err}");
    }
}
