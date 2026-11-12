use bitcoin_hashes::*;
pub fn hash(string: &str) -> Result<String, String> {
    let string = sha256::Hash::hash(string.as_bytes());
    //println!("lib:13:{}", string);
    Ok(string.to_string())
}

//pub fn blockhash() -> Result<String, ascii::AsciiChar> {
