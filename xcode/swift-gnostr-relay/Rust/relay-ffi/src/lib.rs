use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
};

use gnostr_relay::cli::RelayCli;
use serde::Serialize;

#[derive(Serialize)]
struct Envelope<T> {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl<T> Envelope<T> {
    fn ok(data: T) -> Self {
        Self { ok: true, data: Some(data), error: None }
    }

    fn err(error: impl Into<String>) -> Self {
        Self { ok: false, data: None, error: Some(error.into()) }
    }
}

#[derive(Serialize)]
struct RelayCliView {
    logging: String,
    config_file_path: String,
}

fn to_c_string(json: String) -> *mut c_char {
    CString::new(json)
        .unwrap_or_else(|_| CString::new(r#"{"ok":false,"error":"embedded nul"}"#).unwrap())
        .into_raw()
}

fn encode<T: Serialize>(envelope: &Envelope<T>) -> *mut c_char {
    let json = serde_json::to_string(envelope).unwrap_or_else(|error| {
        format!(r#"{{"ok":false,"error":"failed to serialize response: {error}"}}"#)
    });
    to_c_string(json)
}

unsafe fn read_c_string<'a>(ptr: *const c_char) -> Result<&'a str, String> {
    if ptr.is_null() {
        return Err("null pointer".to_string());
    }
    CStr::from_ptr(ptr).to_str().map_err(|error| error.to_string())
}

#[no_mangle]
pub unsafe extern "C" fn relay_string_free(ptr: *mut c_char) {
    if !ptr.is_null() {
        drop(CString::from_raw(ptr));
    }
}

#[no_mangle]
pub unsafe extern "C" fn relay_default_configuration_json(_unused: *const c_char) -> *mut c_char {
    let defaults = RelayCli::default();
    let view = RelayCliView {
        logging: defaults.logging,
        config_file_path: defaults.config_file_path,
    };
    encode(&Envelope::ok(view))
}

#[no_mangle]
pub unsafe extern "C" fn relay_listen_endpoint_json(host: *const c_char, port: u16) -> *mut c_char {
    let host = match read_c_string(host) {
        Ok(value) => value,
        Err(error) => return encode(&Envelope::<String>::err(error)),
    };

    let normalized_host = match host.parse::<std::net::IpAddr>() {
        Ok(addr) if addr.is_unspecified() => "127.0.0.1".to_string(),
        Ok(addr) => addr.to_string(),
        Err(_) if host.is_empty() => "127.0.0.1".to_string(),
        Err(_) => host.to_string(),
    };

    encode(&Envelope::ok(format!("ws://{normalized_host}:{port}")))
}
