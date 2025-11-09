use time_0_3::{OffsetDateTime, format_description};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Resolve 'now_local' error by falling back to UTC (if 'local-offset' feature is missing).
    let now_with_offset = OffsetDateTime::now_utc();

    // 2. Define the format using the TUPLE VARIANT SYNTAX.
    // 💥 FIX: All FormatItem variants must be defined as TUPLES, e.g., FormatItem::Weekday(Weekday::Abbreviated)
    let format = format_description::parse("[weekday repr:short] [month repr:short] [day] [hour]:[minute]:[second] [year] [offset_hour sign:mandatory][offset_minute]")?;
    
    // 3. Format the time
    let formatted_time = now_with_offset.format(&format)?;

    println!("Formatted Git Commit Time:");
    println!("{}", formatted_time);

    Ok(())
}
