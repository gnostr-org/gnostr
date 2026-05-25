use tao::event_loop::{ControlFlow, EventLoop};
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    TrayIconBuilder,
};

fn main() {
    // 1. Initialize the Tao event loop for macOS
    let event_loop = EventLoop::new();

    // 2. Build the context menu that drops down when clicking the icon
    let tray_menu = Menu::new();

    // Create custom menu items
    let hello_item = MenuItem::new("Hello from Rust!", true, None);
    let separator = PredefinedMenuItem::separator();
    let quit_item = MenuItem::new("Quit App", true, None);

    // Append items to the menu
    tray_menu.append(&hello_item).unwrap();
    tray_menu.append(&separator).unwrap();
    tray_menu.append(&quit_item).unwrap();

    // 3. Load an icon
    // For a real app, use a proper 18x18 or 32x32 transparent PNG converted to RGBA pixels.
    // This example generates a simple solid 18x18 black square buffer dynamically.
    let width: u32 = 18;
    let height: u32 = 18;
    let mut rgba = vec![0u8; (width * height * 4) as usize];
    for pixel in rgba.chunks_mut(4) {
        pixel[3] = 255;
    }
    let icon = tray_icon::Icon::from_rgba(rgba, width, height).unwrap();

    // 4. Create the Tray Icon (Menu Bar Extra)
    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("Sovereign Menu Bar Utility")
        .with_icon(icon)
        .build()
        .unwrap();

    // 5. Connect the Menu Event receiver to intercept clicks
    let menu_channel = MenuEvent::receiver();

    // 6. Run the application event loop
    event_loop.run(move |_event, _, control_flow| {
        // Keep the loop running efficiently
        *control_flow = ControlFlow::Poll;

        // Check if a menu item was clicked
        if let Ok(event) = menu_channel.try_recv() {
            if event.id == quit_item.id() {
                println!("Exiting gracefully.");
                *control_flow = ControlFlow::Exit;
            } else if event.id == hello_item.id() {
                println!("Hello item clicked!");
            }
        }
    });
}
