use std::io::{Read};
use reqwest::Url;
fn main() {
    let url = Url::parse("https://www.rust-lang.org/").unwrap();
    let mut res = reqwest::blocking::get(url).unwrap();

    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();

    println!("{}", body);
}
