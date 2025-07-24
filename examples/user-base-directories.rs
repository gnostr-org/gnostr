use directories::BaseDirs;

fn main() {
    println!("--- Base Directories ---");
    if let Some(base_dirs) = BaseDirs::new() {
        // User-specific
        if let Some(config_dir) = Some(base_dirs.config_dir()) {
            println!("User Config: {}", config_dir.display());
        }
        if let Some(data_dir) = Some(base_dirs.data_dir()) {
            println!("User Data: {}", data_dir.display());
        }
        if let Some(data_local_dir) = Some(base_dirs.data_local_dir()) {
            println!("User Data (Local): {}", data_local_dir.display());
        }
        if let Some(cache_dir) = Some(base_dirs.cache_dir()) {
            println!("User Cache: {}", cache_dir.display());
        }
        if let Some(state_dir) = base_dirs.state_dir() {
            println!("User State: {}", state_dir.display());
        }
        if let Some(preference_dir) = Some(base_dirs.preference_dir()) {
            println!("User Preferences: {}", preference_dir.display());
        }
        if let Some(runtime_dir) = base_dirs.runtime_dir() {
            println!("User Runtime: {}", runtime_dir.display());
        }
        if let Some(executable_dir) = base_dirs.executable_dir() {
            println!("User Executables: {}", executable_dir.display());
        }
        //if let Some(font_dir) = Some(base_dirs.font_dir()) {
        //    println!("User Fonts: {}", font_dir.display());
        //}

    } else {
        println!("Could not determine base directories.");
    }
    println!();
}
