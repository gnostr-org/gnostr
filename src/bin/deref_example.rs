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
    #[structopt(name = "input1", short = "x", help = "The first input", default_value = "default_value")]
    input1: c_char,
    #[structopt(name = "input2", short = "y", help = "The second input", default_value = "default_value")]
    input2: String,
    #[structopt(name = "input3", short = "z", help = "The third input", default_value = "1")]
    input3: c_int,
}

fn main() {
    println!("[Rust] Hello from Rust! 🦀");

    let mut number: i32 = 42;
    let message: String = "Hello, world!".to_string();

    println!("number as i32: {}", number as i32); // Annotate with intended type
    println!("message as String: {}", message as String);

    let mut number: i8 = 42;
    println!("number as i8: {}", number as i8); // Annotate with intended type

    use std::ops::Deref;
    let mut value: i8 = 42;
    println!("value: {}", value);

    let null_ptr: *mut i8 = ptr::null_mut();
    println!("null_ptr: {:?}", null_ptr);
    println!("null_ptr: {:#?}", null_ptr);
    // **Unsafe! Check for null before dereferencing**
    let dereferenced_value_i8: i8 = unsafe { *null_ptr };
    if null_ptr == ptr::null_mut() {
    println!("null_ptr == ptr::null_mut(): {}",null_ptr == ptr::null_mut())
}
    //println!("dereferenced_value_i8: {}", dereferenced_value_i8);
    let dereferenced_value_i32: i32 = unsafe { (*null_ptr).into() };
    //println!("dereferenced_value_i32: {}", dereferenced_value_i32);
    //process::exit(0);
    //my_function_free(null_ptr);
    //process::exit(0);

    let raw_ptr = &value as *const i8; // Unsafe pointer to value

    // **Unsafe! Check for null before dereferencing**
    let dereferenced_value_i8: i8 = unsafe { *raw_ptr };
    println!("dereferenced_value_i8: {}", dereferenced_value_i8);

    //expected `*mut i8`, found `i8`
    //my_function_free(dereferenced_value_i8);
    //my_function_free(mut *dereferenced_value_i8);

    let dereferenced_value_i32: i32 = unsafe { (*raw_ptr).into() };
    println!("dereferenced_value_i32: {}", dereferenced_value_i32);
    //my_function_free(dereferenced_value_i32);

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
    println!("Third input: {}", args.input3);

    unsafe {
        println!("[Rust] Calling function in C..");

        let result = multiply(args.input3, args.input3);
        println!("[Rust] Result: {}", result);
        //pub extern "C" fn try_subcommand(argc: c_int, argv: *const *const c_char) -> *mut c_char
        //let args_input_1: *const *const c_char = args.input1.deref();
        //let result = try_subcommand(2, args_input_1);
        //println!("[Rust] Result: {:?}", result);
    }
    let strings = vec!["hello", "world", "!"];
    //let strings = vec![&args.input1, "world", "!"];
    let vector_cstring: Vec<*const u8> = strings
        .into_iter()
        .map(|s| CString::new(s).expect("Error creating CString").into_raw() as _)
        .collect();

    let stdout = std::io::stdout().as_raw_fd();
    for ptr in vector_cstring.into_iter() {
        unsafe {
        //println!("LINE:135");
            write(stdout, ptr as _, strlen(ptr as _));
        println!("");
        }
        //println!("LINE:137");
        //println!("LINE:138:ptr={:?}", ptr);
        //println!("LINE:139:ptr={:?}", ptr);
        my_function_free(ptr as *mut i8);
        unsafe {
            let _ = CString::from_raw(ptr as _);
        }
    }
}
