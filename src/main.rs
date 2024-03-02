/// https://docs.rs/inline-c/latest/inline_c/#
///
/// Hello, Gnostr!
///
/// # Example
///
/// ```rust
/// # use inline_c::assert_c;
/// #
/// # fn main() {
/// #     (assert_c! {
/// #include <stdio.h>
///
/// int main() {
///     printf("Hello, Gnostr!");
///
///     return 0;
/// }
/// #    })
/// #    .success()
/// #    .stdout("Hello, Gnostr!");
/// # }
/// ```
//use nostr_types::Event;
use std::env;
//use std::io::Read;
use std::process;
use std::str::FromStr;

use inline_c::assert_c;
//use gnostr_bins;

use std::process::{Command, Stdio};

use getopts::Options;
use gnostr_bins::get_relays;

extern "C" {
    fn double_input(input: libc::c_int) -> libc::c_int;
}

extern "C" {
    ///static void gnostr_sha256(int argc, const char* argv[], struct args *args)
    fn gnostr_sha256(input: libc::c_int) -> libc::c_int;
}
extern "C" {
    ///static int copyx(unsigned char *output, const unsigned char *x32, const unsigned char *y32, void *data)
    fn copyx(input: libc::c_int) -> libc::c_int;
}
extern "C" {
    ///static void try_subcommand(int argc, const char* argv[])
    fn try_subcommand(input: libc::c_int) -> libc::c_int;
}
extern "C" {
    ///static void print_hex(unsigned char* data, size_t size)
    fn print_hex(input: libc::c_int) -> libc::c_int;
}

fn gen_keys() {
    use k256::schnorr::SigningKey;
    use rand_core::OsRng;

    let signing_key = SigningKey::random(&mut OsRng);
    let verifying_key = signing_key.verifying_key();
    println!("PUBLIC: {:x}", verifying_key.to_bytes());
    println!("PRIVATE: {:x}", signing_key.to_bytes());
}

fn print_type_of<T>(_: &T) -> String {
    format!("{}", std::any::type_name::<T>())
}

///gnostr-bins::get_relays()
pub fn relays(_program: &str, _opts: &Options) {
    let args: Vec<String> = env::args().collect();
    //let _program = args[0].clone();
    if args.len() >= 1 {
        let matches = match _opts.parse(&args[1..]) {
            Ok(m) => m,
            Err(f) => {
                println!("Error: {}", f.to_string());
                panic!("{}", f.to_string())
            }
        };
        if matches.opt_present("h") {
            print_relay_usage(&_program, &_opts);
            //process::exit(0);
        }
    }

    let relays = get_relays();
    println!("{}", format!("{  }", relays.unwrap()));
}

pub fn print_usage(program: &str, opts: &Options) {
    let brief = format!("Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
    process::exit(0);
}

pub fn print_relay_usage(program: &str, opts: &Options) {
    let brief = format!("Relay Usage: {} FILE [options]", program);
    print!("{}", opts.usage(&brief));
    process::exit(0);
}

fn main() {
    let args_vector: Vec<String> = env::args().collect();
    //println!("args_vector = {:?}", args_vector);
    //println!("args_vector.len() = {:?}", args_vector.len());
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    //REF: https://docs.rs/getopts/latest/getopts/struct.Options.html
    let mut top_opts = Options::new();

    top_opts.optopt("o", "", "set output file name", "NAME");
    top_opts.optopt(
        "i",
        "input",
        "Specify the maximum number of commits to show (default: 10)",
        "NUMBER",
    );

    top_opts.optflag("h", "help", "print this help menu");
    top_opts.optflag("r", "relays", "print a json object of relays");

    if args.len() == 1 {
        println!("129:args.len()={}",args.len());
        print_usage(&program, &top_opts);
    }

    // -h --help -r --relays -o -i
    if args.len() == 2 {
        println!("135:args.len()={}",args.len());
        let matches = match top_opts.parse(&args[1..]) {
            Ok(m) => m,
            Err(f) => {
                println!("Error: {}", f.to_string());
                panic!("{}", f.to_string())
            }
        }; //end let matches

        if matches.opt_present("r") {
            println!("145:args.len()={}",args.len());
            //following -r --relays
            if args.len() == 2 {
                println!("148:args.len()={}",args.len());
                let mut relay_opts = Options::new();

                relay_opts.optflag("h", "help", "print relay help menu");
                relay_opts.optflag("r", "relay", "get-relays json list");

                let matches = match relay_opts.parse(&args[1..]) {
                    Ok(m) => m,
                    Err(f) => {
                        println!("Error: {}", f.to_string());
                        panic!("{}", f.to_string())
                    }
                }; //end let matches

                if matches.opt_present("h") {
                    println!("{}",matches.opt_present("h"));
                    print_relay_usage(&program, &relay_opts);
                } // end -h --help
            relays(&program, &relay_opts);

            } //end
        } //end

        ////////////////////////////////////////////////////////////////////////////////
        ////////////////////////////////////////////////////////////////////////////////
        ////////////////////////////////////////////////////////////////////////////////
        ////////////////////////////////////////////////////////////////////////////////

        if matches.opt_present("h") {
            print_usage(&program, &top_opts);
            process::exit(0);
        }

        ////////////////////////////////////////////////////////////////////////////////
        ////////////////////////////////////////////////////////////////////////////////
        ///////////////////////////////////////////////////////////////////////////////
        ///////////////////////////////////////////////////////////////////////////////

        let s = &"hello world".to_string();
        let cloned_s = s.clone();
        println!("{:?}", print_type_of(&s));
        println!("{:?}", print_type_of(&cloned_s));
        let _output = matches.opt_str("o");
        println!("{:?}", print_type_of(&_output));
        //leave input as &Option<String>
        //
        //
        let mut _input = matches.opt_str("i");

        //https://doc.rust-lang.org/std/primitive.str.html#method.parse
        let four = "4".parse::<u32>();
        println!("four {:?}", print_type_of(&four.as_ref().unwrap()));
        //assert_eq!(Ok(4), four);
        let mut mut_four = "4".parse::<u32>();
        println!("mut_four {:?}", print_type_of(&mut_four.as_mut().unwrap()));
        //assert_eq!(Ok(4), mut_four);

        let test = _input.as_mut().unwrap().parse::<u32>();
        println!("test {:?}", print_type_of(&test));
        println!("test {:?}", print_type_of(&test.as_ref().unwrap()));
        //assert_eq!(Ok(4), test);

        let input = _input.as_mut().unwrap().parse::<i32>();
        println!("input {:?}", print_type_of(&input));
        println!("input {:?}", print_type_of(&input.as_ref().unwrap()));
        //assert_eq!(Ok(4), input);

        let input32 = _input.as_mut().unwrap().parse::<i32>();
        println!("input32 {:?}", print_type_of(&input32));
        println!("input32 {:?}", print_type_of(&input32.as_ref().unwrap()));
        //assert_eq!(Ok(4), input32);

        let _input32 = _input.as_mut().unwrap().parse::<i32>();
        println!("&_input32 {:?}", print_type_of(&_input32));
        //let mut input: i32 = _input32.unwrap_or(i32::MAX);
        //let mut input: i32 = _input32.as_deref().unwrap_or(i32::MAX);
        let mut input: &i32 = _input32.as_ref().unwrap_or(&i32::MAX);
        println!("&input {:?}", print_type_of(&input));
        //deref &str
        //let mut _value: i32 = input32.as_deref().as_deref().unwrap_or("100");
        //let mut _value: i32 = &input32.as_deref().unwrap_or("100");
        //
        //REF: https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html
        let mut _value: i32 = input32.unwrap_or(i32::MAX);
        println!("179:&input={:?}", print_type_of(&_value));
        let result = unsafe {
            double_input(_value);
            println!("181:_value={:?}", _value);
        };
        println!("183:result={:?}", result);
        println!("184:_input={:?}", _input);

        ////////////////////////////////////////////////////////////////////////////////
        ////////////////////////////////////////////////////////////////////////////////
    } // end if args.len() >= 1

    ///////////////////////////////////////////////////////////////////////////////
    ///////////////////////////////////////////////////////////////////////////////
} //end main
