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

use inline_c::assert_c;
//use gnostr_bins;

use std::process::{Command, Stdio};

fn gen_keys(){

use k256::schnorr::SigningKey;
use rand_core::OsRng;

    let signing_key = SigningKey::random(&mut OsRng);
    let verifying_key = signing_key.verifying_key();
    println!("PUBLIC: {:x}", verifying_key.to_bytes());
    println!("PRIVATE: {:x}", signing_key.to_bytes());

}

//COMMAND:
//CONTEXT
//gnostr --sec e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 | gnostr-post-event --relay wss://relay.damus.io
#[allow(dead_code)]
fn command_example() {
    let gnostr_sec = Command::new("gnostr")
        .arg("--sec")
        .arg("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
        .arg("-t")
        .arg("gnostr")
        .stdout(Stdio::piped())
        .output()
        .expect("url failed");
    let mut gnostr_event = String::from_utf8(gnostr_sec.stdout).unwrap();
    gnostr_event.pop();
    println!("{:?}", gnostr_event);

    let gnostr_post_event = Command::new("gnostr-post-event")
        .arg("--relay")
        .arg("wss://relay.damus.io")
        .arg(gnostr_event)
        .stdout(Stdio::piped())
        .output()
        .expect("picture failed");
    let post = String::from_utf8(gnostr_post_event.stdout).unwrap();
    println!("{}", post);
}


fn main() {

    let args_vector: Vec<String> = env::args().collect();
    //println!("args_vector = {:?}", args_vector);
    //println!("args_vector.len() = {:?}", args_vector.len());

    if args_vector.len() == 1 {

    }

    if args_vector.len() != 1 {
    //catch help
    if args_vector[1] == "-h" {
        println!("-h HELP!");
        process::exit(0);
    }
    if args_vector[1] == "--help" {
        println!("--help HELP!");
        process::exit(0);
    }
    //catch version
    if args_vector[1] == "-v" {
        println!("-v VERSION!");
        process::exit(0);
    }
    if args_vector[1] == "--version" {
        println!("--version VERSION!");
        process::exit(0);
    }
    //catch sec
    if args_vector[1] == "--sec" {
        println!("--sec CALLED!");
        process::exit(0);
    }
    //catch gen
    if args_vector[1] == "--gen" {
        //println!("--gen CALLED!");
        gen_keys();
        process::exit(0);
    }
    //catch genkey
    if args_vector[1] == "--genkey" {
        //println!("--genkey CALLED!");
        gen_keys();
        process::exit(0);
    }


    //catch genkeys
    if args_vector[1] == "--genkeys" {
        //println!("--genkeys CALLED!");
        gen_keys();
        process::exit(0);
    }

    //command_example();

    } else { // end if args_vector.len() == 1
      println!("default HELP!"); }

    (assert_c! {
        #include <stdio.h>
        int main() { printf("1:Hello, Gnostr!"); return 0; }
    }
    )
    .success()
    .stdout("1:Hello, Gnostr!");
    //rust
    println!("3:Hello, Gnostr!");

}
