use gnostr_types::PrivateKey;
use std::env;
use zeroize::Zeroize;

// The zeroize in here is really silly because we print it.
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() >= 2 {
        //println!("At least one argument (besides the program name) was provided.");
        //println!("The arguments are:");
        //// Iterate and print all arguments, skipping the program name (index 0)
        //for (i, arg) in args.iter().enumerate() {
        //    if i == 0 {
        //        println!("  [Program Name]: {}", arg);
        //    } else {
        //        println!("  [Argument {}]: {}", i, arg);
        //    }
        //}
        let mut private_key = PrivateKey::try_from_hex_string(&args[1]).unwrap();
        let mut bech32 = private_key.as_bech32_string();
        print!("{}", bech32);
        bech32.zeroize();
    } else {
        let mut hex = rpassword::prompt_password("Private key hex: ").unwrap();
        let mut private_key = PrivateKey::try_from_hex_string(&hex).unwrap();
        hex.zeroize();
        let mut bech32 = private_key.as_bech32_string();
        print!("{}", bech32);
        bech32.zeroize();
    }
}
