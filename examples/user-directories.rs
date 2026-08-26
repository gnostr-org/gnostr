use directories::UserDirs;

fn main() {
    println!("--- User Directories ---");
    if let Some(user_dirs) = UserDirs::new() {
        if let Some(desktop_dir) = user_dirs.desktop_dir() {
            println!("Desktop: {}", desktop_dir.display());
        }
        if let Some(document_dir) = user_dirs.document_dir() {
            println!("Documents: {}", document_dir.display());
        }
        if let Some(download_dir) = user_dirs.download_dir() {
            println!("Downloads: {}", download_dir.display());
        }
        if let Some(picture_dir) = user_dirs.picture_dir() {
            println!("Pictures: {}", picture_dir.display());
        }
        if let Some(video_dir) = user_dirs.video_dir() {
            println!("Videos: {}", video_dir.display());
        }
        if let Some(audio_dir) = user_dirs.audio_dir() {
            println!("Audio: {}", audio_dir.display());
        }
        //if let Some(home_dir) = user_dirs.home_dir() {
        //    println!("Home: {}", home_dir.display());
        //}
        if let Some(font_dir) = user_dirs.font_dir() {
            println!("Fonts: {}", font_dir.display());
        }
        if let Some(public_dir) = user_dirs.public_dir() {
            println!("Public: {}", public_dir.display());
        }
        if let Some(template_dir) = user_dirs.template_dir() {
            println!("Templates: {}", template_dir.display());
        }
    } else {
        println!("Could not determine user directories.");
    }
    println!();
}
