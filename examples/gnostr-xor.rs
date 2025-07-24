use std::convert::TryInto;
use std::fmt;
use std::io::Read;
use std::ops::Deref;
use std::{env, process};

//use gnostr_types::Event;

static DEFAULT_SEC: &str = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

#[derive(Copy, Clone)]
struct FourByteString {
    inner: [u8; 4],
}

impl FourByteString {
    pub fn new(s: &str) -> Self {
        if s.len() != 4 {
            panic!("Invalid length");
        }
        let mut inner = [0; 4];
        inner.copy_from_slice(s.as_bytes());
        Self { inner }
    }

    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.inner).unwrap()
    }
}

impl Deref for FourByteString {
    type Target = str;
    fn deref(&self) -> &str {
        self.as_str()
    }
}
impl fmt::Display for FourByteString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}
impl fmt::Debug for FourByteString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

fn takes_str(a: &str) {
    println!("{}", a);
}

fn main() {
    let fbs = FourByteString::new("abcd");

    // This call works because of the Deref impl.
    takes_str(&fbs);

    // This call works because of the Display impl.
    println!("{}", fbs);

    let mut sec = DEFAULT_SEC;
    if sec == DEFAULT_SEC {}
    let args_vector: Vec<String> = env::args().collect();

    #[allow(unreachable_code)]
    for i in 0..args_vector.len() {
        if i == args_vector.len() {
            process::exit(i.try_into().unwrap());
        } else {
            //impossible case
            if args_vector.len() == 0 {
                print!("args_vector.len() = {}", 0);
            };
            //TODO send to self  as double encrypted --dm
            if args_vector.len() == 1 {
                //no args case
                let s: String = String::from(DEFAULT_SEC);
                //std::io::stdin().read_to_string(&mut s).unwrap();
                // //let event: Event = serde_json::from_str(&s).unwrap();
                // //relay_url = DEFAULT_RELAY_URL;
                // //always reprint s for further piping
                print!(
                    "LINE:30 args_vector.len()={}: gnostr-xor <key> {}\n",
                    args_vector.len(),
                    s
                );
                //gnostr_bins::post_event(&relay_url, event);
                process::exit(0);
            };
            if args_vector.len() == 2 {
                //catch help
                if args_vector[1] == "-h" {
                    print!("gnostr --sec <priv_key> | gnostr-post-event <input>");
                    process::exit(0);
                }
                if args_vector[1] == "--help" {
                    print!("gnostr --sec <priv_key> | gnostr-post-event <input>");
                    process::exit(0);
                }
                //catch version
                if args_vector[1] == "-v" {
                    const VERSION: &str = env!("CARGO_PKG_VERSION");
                    print!("v{}", VERSION);
                    process::exit(0);
                }
                if args_vector[1] == "--version" {
                    const VERSION: &str = env!("CARGO_PKG_VERSION");
                    print!("v{}", VERSION);
                    process::exit(0);
                }
                //because args_vector.len() == 2
                //with args_vector.len() == 2
                //we take the key from stdin
                //TODO set in git/signer config
                //
                //positional gnostr-xor --sec
                if args_vector[1] == "--sec" || args_vector[1] == "--privkey" {
                    const VERSION: &str = env!("CARGO_PKG_VERSION");
                    print!("TODO! we handle the secret from stdin!!! \n\n{}", VERSION);
                    process::exit(0);
                }
                //because args_vector.len() == 2
                if args_vector[1] == "--stdin" {
                    //TODO grab Signer and/or read from git config
                    //pipe event from command line
                    //gnostr-sha256 <priv_key> | gnostr-xor <input>//
                    let mut priv_key: String = String::new();
                    std::io::stdin().read_to_string(&mut priv_key).unwrap();
                    //let input: String = serde_json::from_str(&priv_key).unwrap();
                    let input: String = String::from(&priv_key).clone();
                    //always reprint s for further piping
                    print!("LINE:69:input={}, s={}\n", input, priv_key);
                    //gnostr_bins::post_event(relay_url, event);
                    process::exit(0);
                }
                //else assume the second arg is ...
                sec = &args_vector[1];
                //catch the stream
                //if arg len 4
                //gnostr --sec <privkey> | gnostr-post-event --key --stdin <note/file to xor>
                //if arg len 3? assume key via stdin?
                //gnostr --sec <privkey> | gnostr-post-event --stdin <note/file to xor>
                let mut s: String = String::new();
                //this captures the stream when no --relay flag
                std::io::stdin().read_to_string(&mut s).unwrap();
                //let event: Event = serde_json::from_str(&s).unwrap();
                //always reprint s for further piping
                print!("LINE:82 {} {}\n", sec, s);
                //gnostr_bins::post_event(relay_url, event);
                process::exit(0);
            };
            //this actually captures the stream when --relay flag
            if args_vector.len() == 3 {
                //and if
                if args_vector[1] == "--key" {
                    sec = &args_vector[2];
                    let mut s: String = String::new();
                    std::io::stdin().read_to_string(&mut s).unwrap();
                    //let event: Event = serde_json::from_str(&s).unwrap();
                    //always reprint s for further piping
                    print!("{} {}\n", sec, s);
                    //gnostr_bins::post_event(relay_url, event);
                    process::exit(0);
                }
                sec = &args_vector[3 - 1];
                let mut s: String = String::new();
                std::io::stdin().read_to_string(&mut s).unwrap();
                //always reprint s for further piping
                print!("{} {}\n", sec, s);
                //let event: Event = serde_json::from_str(&s).unwrap();
                //gnostr_bins::post_event(relay_url, event);
            };
        }
    }
}
