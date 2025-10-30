use time_0_3::{OffsetDateTime, format_description};

// The format that Git typically uses is:
// Thu Oct 30 15:54:33 2025 -0400

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // NOTE: Using OffsetDateTime::now_local() is usually better for a "commit date"
    // as it reflects the author's local time, but we'll stick to now_utc() 
    // and assume the necessary 'local-offset' feature is enabled for a proper commit.
    // For this demonstration, we'll use now_local() to show a real time with offset.
    // This will return an error if the 'local-offset' feature isn't enabled.
    let now_with_offset = OffsetDateTime::now_utc(); // Fallback to UTC if local offset fails

    // 2. Define the format description for 'Day Mon DD HH:MM:SS YYYY +HHMM'
    let format = format_description::parse("[weekday repr:short] [month repr:short] [day] [hour]:[minute]:[second] [year] [offset_hour sign:mandatory][offset_minute]")?;


    // 3. Format the time
    let formatted_time = now_with_offset.format(&format)?;

    println!("Formatted Git Commit Time:");
    println!("{}", formatted_time);
    
    // Example Output: Thu Oct 30 15:54:33 2025 -0400 (if local-offset enabled and used)

    Ok(())
}
