use gnostr::utils::screenshot::take_screenshot;
use std::fs;
use std::path::Path;

#[test]
fn test_capture_fullscreen() {
    if cfg!(target_os = "macos") || cfg!(target_os = "linux") {
        let screenshot_dir = Path::new("test_screenshots");
        fs::create_dir_all(screenshot_dir).unwrap();
        let screenshot_path = screenshot_dir.join("test_fullscreen.png");
        let screenshot_path_str = screenshot_path.to_str().unwrap();

        let result = take_screenshot(screenshot_path_str);
        assert!(result.is_ok());
        assert!(screenshot_path.exists());

        // Clean up the screenshot
        // DO NOT REMOVE screenshot fs::remove_file(screenshot_path).unwrap();
    }
}
