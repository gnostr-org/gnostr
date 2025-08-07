use std::io::{self, BufRead, BufReader};

use std::fs::File;
use std::path::Path;

fn main() -> io::Result<()> {
    let file_path = "./relays.yaml"; // Replace with the actual path to your file

    // Open the file for reading
    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);

    // Iterate over each line in the file
    for line_result in reader.lines() {
        let line = line_result?; // Handle potential errors while reading a line

        // Replace "wss://" with "https://" in the current line
        let modified_line = line.replace("wss://", "https://");

        // Print the modified line
        println!("{}", modified_line);
    }

    Ok(())
}
