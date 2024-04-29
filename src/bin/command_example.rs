extern crate libc;
use libc::{strlen, write};
use std::ffi::CString;
use std::os::unix::prelude::AsRawFd;

fn main() {
    let strings = vec!["hello", "world", "!"];
    let vector_cstring: Vec<*const u8> = strings
        .into_iter()
        .map(|s| CString::new(s).expect("Error creating CString").into_raw() as _)
        .collect();

    let stdout = std::io::stdout().as_raw_fd();
    for ptr in vector_cstring.into_iter() {
        unsafe {
            write(stdout, ptr as _, strlen(ptr as _));
        }
        println!("");
        unsafe {
            let _ = CString::from_raw(ptr as _);
        }
    }
}
