use std::process::exit;
fn main() -> () {
    exit(0)
}
/// cargo +nightly t -- --nocapture
///
/// cargo    test -- --nocapture
///
#[cfg(test)]
mod tests {
    //use super::*;
    use gnostr_bins::*;
    use sha256::digest;
    #[test]
    fn test_get_relays_paid() {
        let paid = get_relays_paid();
        println!("paid:{}", paid.unwrap());
        println!("paid:{}", get_relays_paid().unwrap());
    }
    #[test]
    fn test_get_relays_public() {
        let public = get_relays_public();
        println!("public:{}", public.unwrap());
        println!("public:{}", get_relays_public().unwrap());
    }
    #[test]
    fn test_get_relays_online() {
        let online = get_relays_online();
        println!("{}", online.unwrap());
        println!("{}", get_relays_online().unwrap());
    }
    #[test]
    fn test_get_relays_by_nip() {
        let mut by_nip = get_relays_by_nip(&"0");
        println!("by_nip:0:{}\n\n\n", by_nip.unwrap());
        println!("{}\n\n\n", get_relays_by_nip(&"0").unwrap());
        by_nip = get_relays_by_nip(&"1");
        println!("{}\n\n\n", by_nip.unwrap());
        println!("{}\n\n\n", get_relays_by_nip(&"0").unwrap());
        by_nip = get_relays_by_nip(&"50");
        println!("{}\n\n\n", by_nip.unwrap());
        println!("{}\n\n\n", get_relays_by_nip(&"0").unwrap());
    }
    #[test]
    fn test_get_relays_offline() {
        let _ = get_relays_offline();
    }
    #[test]
    fn test_get_relays() {
        let _ = get_relays();
    }
    #[test]
    fn test_get_weeble() {
        let weeble = get_weeble();
        println!("{}", weeble.unwrap());
        println!("{}", get_weeble().unwrap());
    }
    #[test]
    fn test_get_wobble() {
        let wobble = get_wobble();
        println!("{}", wobble.unwrap());
        println!("{}", get_wobble().unwrap());
    }
    #[test]
    fn test_get_blockheight() {
        let blockheight = get_blockheight();
        println!("{}", blockheight.unwrap());
        println!("{}", get_blockheight().unwrap());
    }

    #[test]
    fn one_result() {
        let query = "duct";
        let contents = "\
Rust:
safe, fast, productive.
Pick three.";

        assert_eq!(vec!["safe, fast, productive."], search(query, contents));
    }
    #[test]
    fn one_query() {
        let query = digest("");
        let contents = "\
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

        assert_eq!(
            vec!["e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"],
            search(&query, contents)
        );
    }
    #[test]
    #[ignore]
    #[should_panic]
    fn panic_query() {
        let query = digest("");
        let contents = "\
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 ";

        assert_eq!(
            vec!["e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"],
            search(&query, contents)
        );
    }
}
