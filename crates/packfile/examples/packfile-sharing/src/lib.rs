use std::fs::{self, File};
use std::io::{self, BufReader, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::str; // Import str for string conversion

pub fn read_packfile(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);

    let mut num_entries_bytes = [0u8; 4];
    reader.read_exact(&mut num_entries_bytes)?;
    let num_entries = u32::from_le_bytes(num_entries_bytes);

    println!("Number of commits: {}", num_entries);

    for _ in 0..num_entries {
        let mut offset_bytes = [0u8; 4];
        reader.read_exact(&mut offset_bytes)?;
        let offset = u32::from_le_bytes(offset_bytes);

        let mut size_bytes = [0u8; 4];
        reader.read_exact(&mut size_bytes)?;
        let size = u32::from_le_bytes(size_bytes);

        println!("Commit: Offset={}, Size={}", offset, size);

        reader.seek(SeekFrom::Start(offset as u64))?;
        let mut data = vec![0u8; size as usize];
        reader.read_exact(&mut data)?;

        // Attempt to convert the data to a UTF-8 string
        match str::from_utf8(&data) {
            Ok(commit_str) => println!("read_packfile: {}", commit_str),
            Err(_) => println!("read_packfile: {:?} (Non-UTF-8)", &data[..10.min(data.len())]), //print the first 10 bytes if not utf8.
        }
    }

    Ok(())
}

pub fn pack_repository(repo_path: &Path, packfile_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let mut packfile = File::create(packfile_path)?;
    let mut entries = Vec::new();
    let mut data_offset = 4;

    let mut file_paths = Vec::new();
    let mut file_data = Vec::new();

    for entry in fs::read_dir(repo_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            file_paths.push(path.clone());
            file_data.push(fs::read(&path)?);
        }
    }

    packfile.write_all(&(file_paths.len() as u32).to_le_bytes())?;

    for (path, data) in file_paths.iter().zip(file_data.iter()) {
        let filename_cow = path.file_name().unwrap().to_string_lossy(); // Named variable
        let filename = filename_cow.as_bytes();
        let filename_len = filename.len() as u32 + 1;

        let offset = data_offset as u32;
        let size = data.len() as u32;

        entries.push((offset, size, filename.to_vec()));
        data_offset += 8 + filename_len as usize + data.len();
    }

    for (offset, size, filename) in entries {
        packfile.write_all(&offset.to_le_bytes())?;
        packfile.write_all(&size.to_le_bytes())?;
        packfile.write_all(&filename)?;
        packfile.write_all(&[0])?;
    }

    for data in file_data {
        packfile.write_all(&data)?;
    }

    Ok(())
}

pub fn try_read_packfile(packfile_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
//fn try_read_packfile(packfile_path: &Path) -> io::Result<()> {
    let file = File::open(packfile_path)?;
    let mut reader = BufReader::new(file);

    let mut num_files_bytes = [0u8; 4];
    reader.read_exact(&mut num_files_bytes)?;
    let num_files = u32::from_le_bytes(num_files_bytes);

    println!("Number of files: {}", num_files);

    for _ in 0..num_files {
        let mut offset_bytes = [0u8; 4];
        reader.read_exact(&mut offset_bytes)?;
        let offset = u32::from_le_bytes(offset_bytes);

        let mut size_bytes = [0u8; 4];
        reader.read_exact(&mut size_bytes)?;
        let size = u32::from_le_bytes(size_bytes);

        let mut filename_bytes = Vec::new();
        let mut byte = [0u8; 1];

        loop {
            reader.read_exact(&mut byte)?;
            if byte[0] == 0 {
                break;
            }
            filename_bytes.push(byte[0]);
        }

        let filename = String::from_utf8_lossy(&filename_bytes).to_string();

        println!("File: Offset={}, Size={}, Filename={}", offset, size, filename);

        reader.seek(SeekFrom::Start(offset as u64))?;
        let mut data = vec![0u8; size as usize];
        reader.read_exact(&mut data)?;

        println!("File content (first 20 bytes): {:?}", &data[..20.min(data.len())]);
        println!("File content (first 1000 bytes): {:?}", &data[..1000.min(data.len())]);
    }

    Ok(())
}

//fn main() -> io::Result<()> {
//    let repo_path = Path::new("./example_repo");
//    let packfile_path = Path::new("repo.pack");
//
//    if repo_path.exists() {
//        fs::remove_dir_all(repo_path)?;
//        println!("./example_repo removed.");
//    } else {
//        println!("./example_repo did not exist.");
//    }
//
//    fs::create_dir_all(repo_path)?;
//    fs::write(repo_path.join("file2.txt"), &"File 2 content")?;
//
//    pack_repository(repo_path, packfile_path).expect("");
//    read_packfile(packfile_path).expect("");
//    try_read_packfile(packfile_path).expect("");
//
//    fs::write(repo_path.join("file1.txt"), b"File 1 content")?;
//    fs::write(repo_path.join("file2.bin"), &[1, 2, 3, 4, 5])?;
//
//    read_packfile(packfile_path).expect("");
//    try_read_packfile(packfile_path).expect("");
//
//    Ok(())
//}
