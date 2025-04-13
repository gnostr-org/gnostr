use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::Path;
use std::io::Write;

// A basic example of reading a simple packfile format.
// Replace with your actual packfile logic.
fn read_packfile(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    // Example packfile format:
    // - 4 bytes: Number of entries (u32)
    // - For each entry:
    //   - 4 bytes: Offset (u32)
    //   - 4 bytes: Size (u32)
    //   - N bytes: Data

    let mut num_entries_bytes = [0u8; 4];
    reader.read_exact(&mut num_entries_bytes)?;
    let num_entries = u32::from_le_bytes(num_entries_bytes);

    println!("Number of entries: {}", num_entries);

    for _ in 0..num_entries {
        let mut offset_bytes = [0u8; 4];
        reader.read_exact(&mut offset_bytes)?;
        let offset = u32::from_le_bytes(offset_bytes);

        let mut size_bytes = [0u8; 4];
        reader.read_exact(&mut size_bytes)?;
        let size = u32::from_le_bytes(size_bytes);

        println!("Entry: Offset={}, Size={}", offset, size);

        // Read the data
        reader.seek(SeekFrom::Start(offset as u64))?;
        let mut data = vec![0u8; size as usize];
        reader.read_exact(&mut data)?;

        // Process the data (e.g., print it, save it to a file)
        println!("Data: {:?}", &data[..10.min(data.len())]); // print first 10 bytes or the whole content if smaller.
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let packfile_path = Path::new("example.pack"); // Replace with your packfile path

    // Create a dummy packfile for testing (replace with your actual data)
    let mut packfile = File::create(packfile_path)?;

    // Example data
    let data1 = b"Hello, packfile!";
    let data2 = b"Another entry.";

    // Write the number of entries
    packfile.write_all(&(2u32).to_le_bytes())?;

    // Write entry 1 metadata
    let offset1 = 8; // Offset after the header and entry 1 metadata
    packfile.write_all(&(offset1 as u32).to_le_bytes())?;
    packfile.write_all(&(data1.len() as u32).to_le_bytes())?;

    // Write entry 2 metadata
    let offset2 = offset1 + data1.len() as u32;
    packfile.write_all(&(offset2).to_le_bytes())?;
    packfile.write_all(&(data2.len() as u32).to_le_bytes())?;

    // Write entry data
    packfile.write_all(data1)?;
    packfile.write_all(data2)?;

    read_packfile(packfile_path)?;

    Ok(())
}
