extern crate core;
use core::ffi::c_char;
use core::ffi::c_int;
use core::ffi::CStr;
use std::ffi::CString;
use std::process;
use std::ptr;

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

#[no_mangle]
pub extern "C" fn my_function_free(s: *mut c_char) {
    unsafe {
        let print_c_str = CString::from_raw(s);
        //println!("{:?}", print_c_str);
    }
}

use structopt::StructOpt;

#[derive(StructOpt)]
struct Cli {
    #[structopt(name = "input1", short = "x", help = "The first input")]
    input1: String,
    #[structopt(name = "input2", short = "y", help = "The second input")]
    input2: String,
}

fn main() {
    println!("[Rust] Hello from Rust! 🦀");

    use std::ops::Deref;
    let mut value: i8 = 42;
    //println!("value: {}", value);
    //my_function_free(value);
    //println!("*value: {}", *value);
    //my_function_free(*value);
    //println!("value.deref(): {}", value.deref());
    //my_function_free(value.deref());

let null_ptr: *mut i8 = ptr::null_mut();
    println!("null_ptr: {:?}", null_ptr);
    println!("null_ptr: {:#?}", null_ptr);
    //process::exit(0);
    //my_function_free(null_ptr);
    //process::exit(0);


    let raw_ptr = &value as *const i8; // Unsafe pointer to value

    // **Unsafe! Check for null before dereferencing**
    let dereferenced_value_i8: i8 = unsafe { *raw_ptr };
    println!("dereferenced_value_i8: {}", dereferenced_value_i8);
    let dereferenced_value_i32: i32 = unsafe { (*raw_ptr).into() };
    println!("dereferenced_value_i32: {}", dereferenced_value_i32);


    //let ref_to_value = &value; // Reference to value
    //println!("Referenced value: {}", ref_to_value);
    //my_function_free(ref_to_value);

    // Correct (use deref method)
    //let dereferenced_value = *ref_to_value.deref();
    //println!("Dereferenced value: {}", dereferenced_value);
    //my_function_free(dereferenced_value);

    let args = Cli::from_args();

    // Process the parsed inputs
    println!("First input: {}", args.input1);
    println!("Second input: {}", args.input2);

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
        println!("ptr={:?}", ptr);
        println!("ptr={:?}", ptr);
        my_function_free(ptr as *mut i8);
        unsafe {
            let _ = CString::from_raw(ptr as _);
        }
    }
}
