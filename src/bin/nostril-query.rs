extern crate core;
use core::ffi::c_char;
use core::ffi::c_int;
use core::ffi::CStr;
use std::ffi::CString;
use std::process;
use std::ptr;

use std::env::args;
use std::process::Command;
use std::{env, fs, io};

//use include_dir::{include_dir, Dir};
//use std::path::Path;
//use markdown::to_html;

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
//src/nostril.c
//static void try_subcommand(int argc, const char *argv[])
pub extern "C" fn try_subcommand(argc: c_int, argv: Vec<String>) -> Vec<String> {
    if argc > 1 {
        if argv.len() > 1 {
            let arg_zero = Some(argv.get(0));
            print!("arg_zero={:?}\n", arg_zero);
            if argv.len() > 2 {
                let arg_one = Some(argv.get(1));
                print!("argv.get(0)={:?}\n", arg_one);
            }
        }
    } else {
    }
    let output = Some(argv);
    return output.unwrap();
}

#[no_mangle]
pub extern "C" fn my_function_free(s: *mut c_char) {
    unsafe {
        let print_c_str = CString::from_raw(s);
        println!("{:?}", print_c_str);
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

//static void try_subcommand(int argc, const char *argv[])

fn empty_case() -> io::Result<()> {
    let event = Command::new("nostril")
        .output()
        .expect("failed to execute process");

    let nostril_event = String::from_utf8(event.stdout)
        .map_err(|non_utf8| String::from_utf8_lossy(non_utf8.as_bytes()).into_owned())
        .unwrap();

    println!("{}", nostril_event);
    Ok(())
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
    //println!("null_ptr: {:?}", null_ptr);
    //println!("null_ptr: {:#?}", null_ptr);
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

    //let args = Cli::from_args();

    let args_vec: Vec<String> = env::args().collect();
    if args_vec.len() == 1 {
        empty_case();
    }

    let mut app: &String = &("").to_string();
    let mut sec: &String = &("--sec").to_string();
    let mut private_key: &String = &("$(gnostr-sha256)").to_string();

    //capture git-nostril --sec <private_key>
    if args_vec.len() > 2 {
        app = &args_vec[0];
        sec = &args_vec[1];
    }
    //println!("app={}", &app);
    //println!("sec={}", &sec);
    if args_vec.len() >= 3 {
        private_key = &args_vec[2];
    }
    //println!("private_key={}", &private_key);

    //skip git-nostril --sec <private_key>
    //and capture everything else
    let args: Vec<String> = env::args().skip(3).collect();
    println!("args={:?}", &args);

    unsafe {
        //println!("[Rust] Calling function in C..");

        let result = multiply(5000, 5);
        println!("multiply:{}", result);
        //let result = try_subcommand(2, "char" as *const *const c_char);
        //println!("[Rust] Result: {}", result);

        //pub extern "C" fn try_subcommand(argc: c_int, argv: *const *const c_char) -> *mut c_char {
        let result = try_subcommand(args.len().try_into().unwrap(), args);
        println!("try_subcommand_result: {:?}", result.get(0));
        //println!("try_subcommand_result: {:}", *result);
        //println!("try_subcommand_result: {:?}", *result);
        //println!("try_subcommand_result: {:?}", *result);
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
        //println!("");
        //println!("ptr={:?}", ptr);
        println!("\nptr={:?}", ptr);
        my_function_free(ptr as *mut i8);
        unsafe {
            let _ = CString::from_raw(ptr as _);
        }
    }
}
