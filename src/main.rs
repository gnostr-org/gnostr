extern crate core;
use core::ffi::c_char;
use core::ffi::c_int;
use core::ffi::CStr;
use std::ffi::CString;

extern crate libc;
use libc::{strlen, write};
use std::os::unix::prelude::AsRawFd;

extern "C" {
    fn multiply(a: c_int, b: c_int) -> c_int;
}
extern "C" {
    #[allow(dead_code)]
    fn copyx(a: &c_char, b: &c_char, c: &c_char, d: &c_char) -> c_int;
}

#[no_mangle]
pub extern "C" fn try_subcommand(argc: c_int, argv: *const *const c_char) -> *mut c_char {
    let _argv: Vec<_> = (0..argc)
        .map(|i| unsafe { CStr::from_ptr(*argv.add(i as usize)) })
        .collect();

    // Whatever processing you need to do.

    let output = CString::new("Test Output").expect("CString::new failed");

    return output.into_raw();
}

//#[no_mangle]
//pub extern "C" fn my_function_free(s: *mut c_char) {
//    unsafe {
//        CString::from_raw(s);
//    }
//}

fn main() {
    println!("[Rust] Hello from Rust! 🦀");

    unsafe {
        println!("[Rust] Calling function in C..");

        let result = multiply(5000, 5);
        println!("[Rust] Result: {}", result);
        //let result = try_subcommand(2, "char" as *const *const c_char);
        //println!("[Rust] Result: {}", result);
    }
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
