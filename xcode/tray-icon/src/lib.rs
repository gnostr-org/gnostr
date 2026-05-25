use tao::event_loop::{ControlFlow, EventLoop};
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    TrayIconBuilder,
};

pub const DEFAULT_TOOLTIP: &str = "Sovereign Menu Bar Utility";

pub struct TrayContext {
    pub menu: Menu,
    pub hello_item: MenuItem,
    pub quit_item: MenuItem,
}

pub fn load_icon_from_svg(svg: &[u8]) -> tray_icon::Icon {
    let mut options = usvg::Options::default();
    options.fontdb_mut().load_system_fonts();

    let tree = usvg::Tree::from_data(svg, &options).expect("load tray icon svg");
    let size = tree.size().to_int_size();
    let mut pixmap = tiny_skia::Pixmap::new(size.width(), size.height()).expect("create pixmap");
    resvg::render(
        &tree,
        tiny_skia::Transform::default(),
        &mut pixmap.as_mut(),
    );

    tray_icon::Icon::from_rgba(pixmap.take(), size.width(), size.height()).expect("build tray icon")
}

pub fn load_gnostr_icon() -> tray_icon::Icon {
    load_icon_from_svg(include_bytes!("../../icons/gnostr.svg"))
}

pub fn default_tray_context() -> TrayContext {
    let menu = Menu::new();
    let hello_item = MenuItem::new("Hello from Rust!", true, None);
    let separator = PredefinedMenuItem::separator();
    let quit_item = MenuItem::new("Quit App", true, None);

    menu.append(&hello_item).unwrap();
    menu.append(&separator).unwrap();
    menu.append(&quit_item).unwrap();

    TrayContext {
        menu,
        hello_item,
        quit_item,
    }
}

pub fn run_default_tray_app() -> ! {
    run_tray_app(DEFAULT_TOOLTIP, load_gnostr_icon(), default_tray_context())
}

pub fn run_tray_app(tooltip: &str, icon: tray_icon::Icon, context: TrayContext) -> ! {
    let event_loop = EventLoop::new();
    let tray_menu = context.menu;
    let hello_item = context.hello_item;
    let quit_item = context.quit_item;

    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip(tooltip)
        .with_icon(icon)
        .build()
        .unwrap();

    let menu_channel = MenuEvent::receiver();

    event_loop.run(move |_event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

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
