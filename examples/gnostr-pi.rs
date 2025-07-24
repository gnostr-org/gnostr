use std::str::FromStr;
use std::{env, process};

use num_bigint::BigInt;

// Helper function to get the help message string.
// This function returns a String, making it testable without relying on stdout capture.
fn get_help_message() -> String {
    format!(
        "gnostr_pi <depth> <offset>\nNote:<depth> is NOT the returned number of digits!\nUsage:\nENTROPY=$(gnostr-pi 100 0); gnostr-sha256 $ENTROPY\n806b4aba301c1702df94bdb398f579da7b8419455274cb2235d45cc244de749f"
    )
}

// Helper function to get the version message string.
// This function returns a String, making it testable.
fn get_version_message() -> String {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    format!("v{}", VERSION)
}

fn main() {
    //
    //
    //
    let args: Vec<String> = env::args().collect();
    //
    //
    //
    if (args.len() - 1) >= 1 {
        if &args[1] == "-h" || &args[1] == "--help" {
            println!("{}", get_help_message());
            process::exit(0);
        }
    }
    if (args.len() - 1) >= 1 {
        if &args[1] == "-v" || &args[1] == "-V" || &args[1] == "--version" {
            println!("{}", get_version_message());
            process::exit(0);
        }
    }
    if (args.len() - 1) == 1 {
        let depth = u64::from_str(&args[1]).unwrap() * 5 + 1;
        calculate_pi_digits_impl(depth as u64);
        process::exit(0);
    }
    if (args.len() - 1) == 2 {
        let depth = u64::from_str(&args[1]).unwrap() * 5 + 1;
        //println!("depth={}\n", depth);
        let offset = u64::from_str(&args[2]).unwrap() * 5 + 1;
        //println!("offset={}\n", offset);

        if offset <= 1 {
            //TODO handle negative offset simular to gnostr-pi.c
            calculate_pi_digits_impl(depth as u64);
            process::exit(0);
        } else {
            calculate_pi_digits_with_offset_impl(depth as u64, offset as u64);
        }

        process::exit(0);
    }
}
fn calculate_pi_digits_with_offset_impl(depth: u64, offset: u64) -> String {
    //println!("depth={}", depth);
    //println!("offset={}", offset);
    let mut q = BigInt::from(1);
    let mut r = BigInt::from(0);
    let mut t = BigInt::from(1);
    let mut k = BigInt::from(1);
    let mut n = BigInt::from(3);
    let mut l = BigInt::from(3);
    let mut first = true;
    let mut count = 0u64;
    //println!("count={}", count);
    loop {
        if count == depth + offset {
            //println!("limit={}", limit);
            //println!("count={}", count);
            process::exit(0);
        }
        //println!("count={}\n", count);
        if &q * 4 + &r - &t < &n * &t {
            //print!("count={}\n", count);
            if first {
                //we only print pi mantissa
                //print!("3.");
                first = false;
            } else {
                //detect limit and offset
                //dont print for offset number of digits
                //augment depth to limit + offset

                if count >= offset && offset >= 5 {
                    //print!("\n{}\n", n);
                    //print!("count={}\n", count);
                    //print!("offset={}\n", offset);
                    //print!("n={}\n", n);
                    print!("{}", n);
                }
            }
            let nr = (&r - &n * &t) * 10;
            n = (&q * 3 + &r) * 10 / &t - &n * 10;
            q *= 10;
            r = nr;
        } else {
            let nr = (&q * 2 + &r) * &l;
            let nn = (&q * &k * 7 + 2 + &r * &l) / (&t * &l);
            q *= &k;
            t *= &l;
            l += 2;
            k += 1;
            n = nn;
            r = nr;
        }
        count = count + 1u64;
    }
}

fn calculate_pi_digits_impl(limit: u64) -> String {
    //println!("limit={}", limit);
    let mut q = BigInt::from(1);
    let mut r = BigInt::from(0);
    let mut t = BigInt::from(1);
    let mut k = BigInt::from(1);
    let mut n = BigInt::from(3);
    let mut l = BigInt::from(3);
    let mut first = true;
    let mut count = 0u64;
    //println!("count={}", count);
    loop {
        if count == limit {
            process::exit(0);
        }
        if &q * 4 + &r - &t < &n * &t {
            //print!("count={}\n", count);
            if first {
                //print!("3.");
                first = false;
            } else {
                print!("{}", n);
            }
            let nr = (&r - &n * &t) * 10;
            n = (&q * 3 + &r) * 10 / &t - &n * 10;
            q *= 10;
            r = nr;
        } else {
            let nr = (&q * 2 + &r) * &l;
            let nn = (&q * &k * 7 + 2 + &r * &l) / (&t * &l);
            q *= &k;
            t *= &l;
            l += 2;
            k += 1;
            n = nn;
            r = nr;
        }
        count = count + 1u64;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_help_message() {
        let expected_message = "gnostr_pi <depth> <offset>\nNote:<depth> is NOT the returned number of digits!\nUsage:\nENTROPY=$(gnostr-pi 100 0); gnostr-sha256 $ENTROPY\n806b4aba301c1702df94bdb398f579da7b8419455274cb2235d45cc244de749f";
        assert_eq!(get_help_message(), expected_message);
    }

    #[test]
    fn test_get_version_message() {
        let version_message = get_version_message();
        assert!(version_message.starts_with("v"));
        assert!(version_message.len() > 1); // "v" + at least one digit
    }

    #[test]
    fn test_calculate_pi_digits_impl_small_length() {
        assert_eq!(calculate_pi_digits_impl(6), "14159");
    }

    #[test]
    fn test_calculate_pi_digits_impl_more_digits() {
        assert_eq!(calculate_pi_digits_impl(11), "1415926535");
    }

    #[test]
    fn test_calculate_pi_digits_impl_no_digits() {
        assert_eq!(calculate_pi_digits_impl(0), "");
        assert_eq!(calculate_pi_digits_impl(1), "");
    }

    #[test]
    fn test_calculate_pi_digits_with_offset_impl_basic() {
        assert_eq!(calculate_pi_digits_with_offset_impl(12, 6), "26535");
    }

    #[test]
    fn test_calculate_pi_digits_with_offset_impl_higher_offset() {
        assert_eq!(calculate_pi_digits_with_offset_impl(22, 11), "8979323846");
    }

    #[test]
    fn test_calculate_pi_digits_with_offset_impl_offset_condition() {
        assert_eq!(calculate_pi_digits_with_offset_impl(10, 2), ""); // Should be empty due to offset < 5 condition
    }

    #[test]
    fn test_calculate_pi_digits_with_offset_impl_empty_range() {
        assert_eq!(calculate_pi_digits_with_offset_impl(5, 6), "");
    }
}
