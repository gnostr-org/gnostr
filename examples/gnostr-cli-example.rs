use std::io::Read;

use reqwest::Url;
fn main() {
    let url = Url::parse("https://raw.githubusercontent.com/gnostr-org/gnostr-bins/master/src/bin/gnostr-cli-example.rs").unwrap();
    let mut res = reqwest::blocking::get(url).unwrap();

    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();

    println!("{}", body);
}
