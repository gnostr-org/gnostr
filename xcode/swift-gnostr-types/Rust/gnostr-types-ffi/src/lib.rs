use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
};

use gnostr_types::nostr::{
    nip34::{generate_git_note_event, generate_git_note_event_with_pow, git_note_event_id, git_note_tags, GitNote},
    EventKindOrRange, EventV3, Filter, ImageDimensions, MetadataV1, NAddr, NEvent, PayRequestData, PreEventV3,
    PrivateKey, Profile, RelayInformationDocumentV2, SubscriptionId, TagV3, ClientMessageV3, RelayMessageV5,
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

#[no_mangle]
pub unsafe extern "C" fn gnostr_types_string_free(ptr: *mut c_char) {
    if !ptr.is_null() {
        drop(CString::from_raw(ptr));
    }
}

#[no_mangle]
pub unsafe extern "C" fn gnostr_types_git_note_event_id_json(commit_id: *const c_char) -> *mut c_char {
    match read_c_string(commit_id) {
        Ok(commit_id) => match git_note_event_id(commit_id) {
            Ok(event_id) => encode(&Envelope::ok(event_id.as_hex_string())),
            Err(error) => encode(&Envelope::<String>::err(error.to_string())),
        },
        Err(error) => encode(&Envelope::<String>::err(error)),
    }
}

#[no_mangle]
pub unsafe extern "C" fn gnostr_types_git_note_tags_json(note_json: *const c_char) -> *mut c_char {
    let note_json = match read_c_string(note_json) {
        Ok(value) => value,
        Err(error) => return encode(&Envelope::<String>::err(error)),
    };

    let note: GitNote = match serde_json::from_str(note_json) {
        Ok(note) => note,
        Err(error) => return encode(&Envelope::<String>::err(error.to_string())),
    };

    match git_note_tags(&note) {
        Ok(tags) => match serde_json::to_string(&tags) {
            Ok(tags_json) => encode(&Envelope::ok(tags_json)),
            Err(error) => encode(&Envelope::<String>::err(error.to_string())),
        },
        Err(error) => encode(&Envelope::<String>::err(error.to_string())),
    }
}

#[no_mangle]
pub unsafe extern "C" fn gnostr_types_generate_git_note_event_json(
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

#[no_mangle]
pub unsafe extern "C" fn gnostr_types_roundtrip_event_json(event_json: *const c_char) -> *mut c_char {
    roundtrip_json::<EventV3>(event_json)
}

#[no_mangle]
pub unsafe extern "C" fn gnostr_types_roundtrip_pre_event_json(pre_event_json: *const c_char) -> *mut c_char {
    roundtrip_json::<PreEventV3>(pre_event_json)
}

#[no_mangle]
pub unsafe extern "C" fn gnostr_types_roundtrip_tag_json(tag_json: *const c_char) -> *mut c_char {
    roundtrip_json::<TagV3>(tag_json)
}

#[no_mangle]
pub unsafe extern "C" fn gnostr_types_roundtrip_naddr_json(naddr_json: *const c_char) -> *mut c_char {
    roundtrip_json::<NAddr>(naddr_json)
}

#[no_mangle]
pub unsafe extern "C" fn gnostr_types_roundtrip_nevent_json(nevent_json: *const c_char) -> *mut c_char {
    roundtrip_json::<NEvent>(nevent_json)
}

#[no_mangle]
pub unsafe extern "C" fn gnostr_types_roundtrip_nprofile_json(nprofile_json: *const c_char) -> *mut c_char {
    roundtrip_json::<Profile>(nprofile_json)
}

#[no_mangle]
pub unsafe extern "C" fn gnostr_types_roundtrip_filter_json(filter_json: *const c_char) -> *mut c_char {
    roundtrip_json::<Filter>(filter_json)
}

#[no_mangle]
pub unsafe extern "C" fn gnostr_types_roundtrip_metadata_json(metadata_json: *const c_char) -> *mut c_char {
    roundtrip_json::<MetadataV1>(metadata_json)
}

#[no_mangle]
pub unsafe extern "C" fn gnostr_types_roundtrip_profile_json(profile_json: *const c_char) -> *mut c_char {
    roundtrip_json::<Profile>(profile_json)
}

#[no_mangle]
pub unsafe extern "C" fn gnostr_types_roundtrip_relay_information_document_json(
    relay_information_document_json: *const c_char,
) -> *mut c_char {
    roundtrip_json::<RelayInformationDocumentV2>(relay_information_document_json)
}

#[no_mangle]
pub unsafe extern "C" fn gnostr_types_roundtrip_pay_request_data_json(pay_request_data_json: *const c_char) -> *mut c_char {
    roundtrip_json::<PayRequestData>(pay_request_data_json)
}

#[no_mangle]
pub unsafe extern "C" fn gnostr_types_roundtrip_client_message_json(client_message_json: *const c_char) -> *mut c_char {
    roundtrip_json::<ClientMessageV3>(client_message_json)
}

#[no_mangle]
pub unsafe extern "C" fn gnostr_types_roundtrip_relay_message_json(relay_message_json: *const c_char) -> *mut c_char {
    roundtrip_json::<RelayMessageV5>(relay_message_json)
}

#[no_mangle]
pub unsafe extern "C" fn gnostr_types_roundtrip_subscription_id_json(subscription_id_json: *const c_char) -> *mut c_char {
    roundtrip_json::<SubscriptionId>(subscription_id_json)
}

#[no_mangle]
pub unsafe extern "C" fn gnostr_types_roundtrip_image_dimensions_json(image_dimensions_json: *const c_char) -> *mut c_char {
    roundtrip_json::<ImageDimensions>(image_dimensions_json)
}

#[no_mangle]
pub unsafe extern "C" fn gnostr_types_roundtrip_event_kind_or_range_json(event_kind_or_range_json: *const c_char) -> *mut c_char {
    roundtrip_json::<EventKindOrRange>(event_kind_or_range_json)
}

fn roundtrip_json<T>(json_ptr: *const c_char) -> *mut c_char
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    let json = match unsafe { read_c_string(json_ptr) } {
        Ok(value) => value,
        Err(error) => return encode(&Envelope::<String>::err(error)),
    };

    let value: T = match serde_json::from_str(json) {
        Ok(value) => value,
        Err(error) => return encode(&Envelope::<String>::err(error.to_string())),
    };

    match serde_json::to_string(&value) {
        Ok(serialized) => encode(&Envelope::ok(serialized)),
        Err(error) => encode(&Envelope::<String>::err(error.to_string())),
    }
}
