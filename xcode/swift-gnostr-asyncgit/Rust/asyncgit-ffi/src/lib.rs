use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
};

use gnostr_asyncgit::{default_notes_ref, sync::{repo_state, RepoPath}};
use gnostr_types::nostr::{
    nip34::{generate_git_note_event, generate_git_note_event_with_pow, git_note_event_id, GitNote},
    PrivateKey,
};
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

fn repo_state_name(state: impl std::fmt::Debug) -> String {
    format!("{state:?}").to_lowercase()
}

#[no_mangle]
pub unsafe extern "C" fn asyncgit_string_free(ptr: *mut c_char) {
    if !ptr.is_null() {
        drop(CString::from_raw(ptr));
    }
}

#[no_mangle]
pub unsafe extern "C" fn asyncgit_repo_state_json(repo_path: *const c_char) -> *mut c_char {
    match read_c_string(repo_path) {
        Ok(path) => match repo_state(&RepoPath::from(path)) {
            Ok(state) => encode(&Envelope::ok(repo_state_name(state))),
            Err(error) => encode(&Envelope::<String>::err(error.to_string())),
        },
        Err(error) => encode(&Envelope::<String>::err(error)),
    }
}

#[no_mangle]
pub unsafe extern "C" fn asyncgit_default_notes_ref_json(repo_path: *const c_char) -> *mut c_char {
    match read_c_string(repo_path) {
        Ok(path) => match default_notes_ref(&RepoPath::from(path)) {
            Ok(notes_ref) => encode(&Envelope::ok(notes_ref)),
            Err(error) => encode(&Envelope::<String>::err(error.to_string())),
        },
        Err(error) => encode(&Envelope::<String>::err(error)),
    }
}

#[no_mangle]
pub unsafe extern "C" fn asyncgit_git_note_event_id_json(commit_id: *const c_char) -> *mut c_char {
    match read_c_string(commit_id) {
        Ok(commit_id) => match git_note_event_id(commit_id) {
            Ok(event_id) => encode(&Envelope::ok(event_id.as_hex_string())),
            Err(error) => encode(&Envelope::<String>::err(error.to_string())),
        },
        Err(error) => encode(&Envelope::<String>::err(error)),
    }
}

#[no_mangle]
pub unsafe extern "C" fn asyncgit_generate_git_note_event_json(
    note_json: *const c_char,
    private_key_hex: *const c_char,
    pow_target_bits: u8,
) -> *mut c_char {
    let note_json = match read_c_string(note_json) {
        Ok(value) => value,
        Err(error) => return encode(&Envelope::<String>::err(error)),
    };
    let private_key_hex = match read_c_string(private_key_hex) {
        Ok(value) => value,
        Err(error) => return encode(&Envelope::<String>::err(error)),
    };

    let note: GitNote = match serde_json::from_str(note_json) {
        Ok(note) => note,
        Err(error) => return encode(&Envelope::<String>::err(error.to_string())),
    };

    let private_key = match PrivateKey::try_from_hex_string(private_key_hex) {
        Ok(key) => key,
        Err(error) => return encode(&Envelope::<String>::err(error.to_string())),
    };

    let event = if pow_target_bits > 0 {
        generate_git_note_event_with_pow(&note, &private_key, pow_target_bits)
    } else {
        generate_git_note_event(&note, &private_key)
    };

    match event {
        Ok(event) => match serde_json::to_string(&event) {
            Ok(event_json) => encode(&Envelope::ok(event_json)),
            Err(error) => encode(&Envelope::<String>::err(error.to_string())),
        },
        Err(error) => encode(&Envelope::<String>::err(error.to_string())),
    }
}
