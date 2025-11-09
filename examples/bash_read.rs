use expectrl::{session::Session, Expect, Regex};
use std::process::Command;
use std::io::BufRead;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut p = Session::spawn(Command::new("bash"))?;
    p.expect(Regex(".*"))?;

    // case 1: wait until program is done
    p.send_line("hostname")?;
    let mut hostname = String::new();
    p.read_line(&mut hostname)?;
    p.expect(Regex(".*"))?; // go sure `hostname` is really done
    println!("Current hostname: {}", hostname);

    // case 2: wait until done, only extract a few infos
    p.send_line("wc /etc/passwd")?;
    // `expect` returns the matched string
    let lines = p.expect(Regex("[0-9]+"))?;
    let words = p.expect(Regex("[0-9]+"))?;
    let bytes = p.expect(Regex("[0-9]+"))?;
    p.expect(Regex(".*"))?; // go sure `wc` is really done
    println!(
        "/etc/passwd has {} lines, {} words, {} chars",
        std::str::from_utf8(lines.get(0).unwrap())?,
        std::str::from_utf8(words.get(0).unwrap())?,
        std::str::from_utf8(bytes.get(0).unwrap())?
    );

    // case 3: read while program is still executing
    p.send_line("ping 8.8.8.8")?;
    p.expect("bytes of data")?; // returns when it sees "bytes of data" in output
    for _ in 0..5 {
        // times out if one ping takes longer than 2s
        let duration = p.expect(Regex("[0-9. ]+ ms"))?;
        println!("Roundtrip time: {}", std::str::from_utf8(duration.get(0).unwrap())?);
    }
    p.send_line("\x03")?;
    Ok(())
}
