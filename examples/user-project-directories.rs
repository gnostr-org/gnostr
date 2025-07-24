use directories::ProjectDirs;

fn main() {
    println!("--- Project Directories ---");

    // For a typical application:
    // qualifier: usually a reverse domain name (e.g., "com.mycompany")
    // organization: Your company or organization name
    // application: Your application name
    if let Some(proj_dirs) = ProjectDirs::from("com", "MyCompany", "MyRustApp") {
        //println!("Application Name: {}", proj_dirs.application_name());

        if let Some(config_dir) = proj_dirs.config_dir().to_str() {
            println!("Config Dir: {}", config_dir);
        }
        if let Some(data_dir) = proj_dirs.data_dir().to_str() {
            println!("Data Dir: {}", data_dir);
        }
        if let Some(data_local_dir) = proj_dirs.data_local_dir().to_str() {
            println!("Data Local Dir: {}", data_local_dir);
        }
        if let Some(cache_dir) = proj_dirs.cache_dir().to_str() {
            println!("Cache Dir: {}", cache_dir);
        }
        //if let Some(state_dir) = proj_dirs.state_dir().to_str() {
        //    println!("State Dir: {}", state_dir);
        //}
        //if let Some(log_dir) = proj_dirs.log_dir().to_str() {
        //    println!("Log Dir: {}", log_dir);
        //}
        //if let Some(executable_dir) = proj_dirs.executable_dir().to_str() {
        //    println!("Executable Dir: {}", executable_dir);
        //}
    } else {
        println!("Could not determine project directories.");
    }

    println!("\n--- Another Project Example (different qualifier) ---");
    // Example for a project without an organization (e.g., an open-source tool)
    // You might use just the application name as the qualifier and organization
    //if let Some(proj_dirs) = ProjectDirs::from(None, "MyOpenSourceTool", "MyOpenSourceTool") {
    //    //println!("Application Name: {}", proj_dirs.application_name());
    //    if let Some(config_dir) = proj_dirs.config_dir().to_str() {
    //        println!("Config Dir: {}", config_dir);
    //    }
    //    if let Some(data_dir) = proj_dirs.data_dir().to_str() {
    //        println!("Data Dir: {}", data_dir);
    //    }
    //} else {
    //    println!("Could not determine project directories for open source tool.");
    //}
}
