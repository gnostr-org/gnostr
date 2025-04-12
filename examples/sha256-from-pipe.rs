use sha2::{Digest, Sha256};
use std::io::{self, Read};

fn main() -> io::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    let mut hasher = Sha256::new();
    hasher.update(input.trim().as_bytes()); // Trim to remove trailing newline
    let result = hasher.finalize();

    println!("{:x}", result);

    Ok(())
}
